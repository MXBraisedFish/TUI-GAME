use std::cell::Cell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use mlua::chunk::ChunkMode;
use mlua::{
  Function, HookTriggers, IntoLuaMulti, Lua, LuaOptions, MultiValue, RegistryKey, StdLib, Table,
  Value, VmState,
};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

use crate::host_engine::services::Size;

use super::events::{LuaEventCallbackId, LuaEventDelivery, LuaEventRoute, LuaRuntimeEvent};
use super::policy::{LuaBudgetKind, LuaExecutionBudget, LuaPolicy};

const REQUIRED_CALLBACKS: &[&str] = &["Init", "HandleEvent", "Update", "UpdateFrame", "Render"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LuaSessionKind {
  Game,
  Screensaver,
}

impl LuaSessionKind {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Game => "game",
      Self::Screensaver => "screensaver",
    }
  }
}

impl fmt::Display for LuaSessionKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaSessionState {
  Loading,
  Running,
  Faulted,
  Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaErrorStage {
  ValidatePolicy,
  ReadSource,
  CreateVm,
  BuildSandbox,
  ExecuteEntry,
  DiscoverCallbacks,
  Callback,
  ExecutionLimit,
  MemoryLimit,
  ContinueDataValidation,
  SaveValidation,
  EventCallback,
  EventQueue,
}

#[derive(Clone, Debug)]
pub struct LuaSessionSpec {
  pub package_id: String,
  pub session_kind: LuaSessionKind,
  pub entry_path: PathBuf,
  pub fixed_delta: Duration,
  pub terminal_size: Size,
  pub continue_data: Option<JsonValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaSessionError {
  pub package_id: String,
  pub session_kind: LuaSessionKind,
  pub stage: LuaErrorStage,
  pub callback: Option<&'static str>,
  pub message: String,
}

impl fmt::Display for LuaSessionError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Lua {} session '{}' failed during {:?}",
      self.session_kind, self.package_id, self.stage
    )?;
    if let Some(callback) = self.callback {
      write!(f, " ({callback})")?;
    }
    write!(f, ": {}", self.message)
  }
}

impl std::error::Error for LuaSessionError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LuaExecutionStats {
  pub instructions: u64,
  pub elapsed: Duration,
  pub memory_bytes: usize,
}

struct LuaCallbacks {
  init: RegistryKey,
  handle_event: RegistryKey,
  update: RegistryKey,
  update_frame: RegistryKey,
  render: RegistryKey,
  save_game: Option<RegistryKey>,
  save_best: Option<RegistryKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaCallbackLifetime {
  Once,
  UntilTerminal,
}

struct LuaRegisteredCallback {
  key: RegistryKey,
  lifetime: LuaCallbackLifetime,
}

pub struct LuaSession {
  callbacks: LuaCallbacks,
  environment: RegistryKey,
  lua: Lua,
  policy: LuaPolicy,
  package_id: String,
  session_kind: LuaSessionKind,
  entry_path: PathBuf,
  fixed_delta: Duration,
  terminal_size: Size,
  state: LuaSessionState,
  last_stats: LuaExecutionStats,
  event_callbacks: HashMap<LuaEventCallbackId, LuaRegisteredCallback>,
  next_event_callback_id: u64,
}

impl LuaSession {
  pub(super) fn load(spec: LuaSessionSpec, policy: LuaPolicy) -> Result<Self, LuaSessionError> {
    policy
      .validate()
      .map_err(|error| session_error(&spec, LuaErrorStage::ValidatePolicy, None, error))?;
    validate_continue_data(&spec, &policy)?;
    let source = read_source(&spec, &policy)?;
    let lua = Lua::new_with(
      StdLib::MATH | StdLib::STRING | StdLib::UTF8 | StdLib::TABLE,
      LuaOptions::default(),
    )
    .map_err(|error| session_error(&spec, LuaErrorStage::CreateVm, None, error))?;
    lua
      .set_memory_limit(policy.memory_limit_bytes)
      .map_err(|error| session_error(&spec, LuaErrorStage::MemoryLimit, None, error))?;

    let environment = build_sandbox(&lua)
      .map_err(|error| session_error(&spec, LuaErrorStage::BuildSandbox, None, error))?;
    let entry_function = lua
      .load(source)
      .set_name(spec.entry_path.to_string_lossy())
      .set_mode(ChunkMode::Text)
      .set_environment(environment.clone())
      .into_function()
      .map_err(|error| session_error(&spec, LuaErrorStage::ExecuteEntry, None, error))?;
    run_with_budget(
      &lua,
      entry_function,
      (),
      policy.budget(LuaBudgetKind::Load),
      policy.hook_interval,
    )
    .map_err(|failure| execution_error(&spec, LuaErrorStage::ExecuteEntry, None, failure))?;

    let callbacks = discover_callbacks(&lua, &environment, &spec)?;
    let environment_key = lua
      .create_registry_value(environment)
      .map_err(|error| session_error(&spec, LuaErrorStage::BuildSandbox, None, error))?;

    let mut session = Self {
      callbacks,
      environment: environment_key,
      lua,
      policy,
      package_id: spec.package_id,
      session_kind: spec.session_kind,
      entry_path: spec.entry_path,
      fixed_delta: spec.fixed_delta,
      terminal_size: spec.terminal_size,
      state: LuaSessionState::Loading,
      last_stats: LuaExecutionStats::default(),
      event_callbacks: HashMap::new(),
      next_event_callback_id: 1,
    };
    let context = session.context_table(spec.continue_data.as_ref())?;
    session.invoke_required(
      Callback::Init,
      context,
      LuaBudgetKind::Init,
      LuaErrorStage::Callback,
    )?;
    session.state = LuaSessionState::Running;
    Ok(session)
  }

  pub fn package_id(&self) -> &str {
    &self.package_id
  }

  pub fn session_kind(&self) -> LuaSessionKind {
    self.session_kind
  }

  pub fn state(&self) -> LuaSessionState {
    self.state
  }

  pub fn entry_path(&self) -> &std::path::Path {
    &self.entry_path
  }

  pub fn terminal_size(&self) -> Size {
    self.terminal_size
  }

  pub fn set_terminal_size(&mut self, size: Size) {
    self.terminal_size = size;
  }

  pub fn last_stats(&self) -> LuaExecutionStats {
    self.last_stats
  }

  pub fn memory_used(&self) -> usize {
    self.lua.used_memory()
  }

  pub fn handle_event(&mut self, event: &LuaRuntimeEvent) -> Result<(), LuaSessionError> {
    let event = match self.event_table(event, LuaErrorStage::Callback, "HandleEvent") {
      Ok(event) => event,
      Err(error) => {
        self.state = LuaSessionState::Faulted;
        return Err(error);
      }
    };
    self.invoke_hot(Callback::HandleEvent, event, LuaBudgetKind::HandleEvent)
  }

  pub fn dispatch_event(&mut self, delivery: &LuaEventDelivery) -> Result<(), LuaSessionError> {
    match delivery.route {
      LuaEventRoute::HandleEvent => self.handle_event(&delivery.event),
      LuaEventRoute::Callback(callback) => self.invoke_event_callback(callback, &delivery.event),
    }
  }

  pub(crate) fn register_event_callback(
    &mut self,
    function: Function,
    lifetime: LuaCallbackLifetime,
  ) -> Result<LuaEventCallbackId, LuaSessionError> {
    let id = LuaEventCallbackId(self.next_event_callback_id);
    self.next_event_callback_id = self.next_event_callback_id.saturating_add(1);
    let key = self
      .lua
      .create_registry_value(function)
      .map_err(|error| self.error(LuaErrorStage::EventCallback, Some("EventCallback"), error))?;
    self
      .event_callbacks
      .insert(id, LuaRegisteredCallback { key, lifetime });
    Ok(id)
  }

  pub(crate) fn unregister_event_callback(&mut self, id: LuaEventCallbackId) -> bool {
    let Some(callback) = self.event_callbacks.remove(&id) else {
      return false;
    };
    self.lua.remove_registry_value(callback.key).is_ok()
  }

  pub fn update(&mut self) -> Result<(), LuaSessionError> {
    self.invoke_hot(
      Callback::Update,
      self.fixed_delta.as_secs_f64(),
      LuaBudgetKind::Update,
    )
  }

  pub fn update_frame(&mut self, real_delta: Duration, alpha: f64) -> Result<(), LuaSessionError> {
    self.invoke_hot(
      Callback::UpdateFrame,
      (real_delta.as_secs_f64(), alpha.clamp(0.0, 1.0)),
      LuaBudgetKind::UpdateFrame,
    )
  }

  pub fn render(&mut self, size: Size) -> Result<(), LuaSessionError> {
    let draw = self
      .lua
      .create_table()
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Render"), error))?;
    draw
      .set("width", size.width)
      .and_then(|_| draw.set("height", size.height))
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Render"), error))?;
    self.invoke_hot(Callback::Render, draw, LuaBudgetKind::Render)
  }

  pub fn save_game(&mut self) -> Result<Option<JsonValue>, LuaSessionError> {
    self.invoke_save(Callback::SaveGame)
  }

  pub fn save_best(&mut self) -> Result<Option<JsonValue>, LuaSessionError> {
    self.invoke_save(Callback::SaveBest)
  }

  pub fn stop(&mut self) {
    if self.state == LuaSessionState::Stopped {
      return;
    }
    for (_, callback) in self.event_callbacks.drain() {
      let _ = self.lua.remove_registry_value(callback.key);
    }
    let _ = self.lua.gc_collect();
    self.state = LuaSessionState::Stopped;
  }

  fn context_table(&self, continue_data: Option<&JsonValue>) -> Result<Table, LuaSessionError> {
    let context = self
      .lua
      .create_table()
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
    let terminal = self
      .lua
      .create_table()
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
    terminal
      .set("width", self.terminal_size.width)
      .and_then(|_| terminal.set("height", self.terminal_size.height))
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;

    context
      .set("package_id", self.package_id.as_str())
      .and_then(|_| context.set("package_type", self.session_kind.as_str()))
      .and_then(|_| context.set("fixed_delta", self.fixed_delta.as_secs_f64()))
      .and_then(|_| context.set("terminal", terminal))
      .and_then(|_| context.set("api_version", 1_u32))
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
    let continue_value = match continue_data {
      Some(value) => json_to_lua(&self.lua, value)
        .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?,
      None => Value::Nil,
    };
    context
      .set("continue_data", continue_value)
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
    Ok(context)
  }

  fn event_table(
    &self,
    event: &LuaRuntimeEvent,
    stage: LuaErrorStage,
    callback: &'static str,
  ) -> Result<Table, LuaSessionError> {
    let table = self
      .lua
      .create_table()
      .map_err(|error| self.error(stage, Some(callback), error))?;
    let data = event
      .data
      .to_lua_table(&self.lua)
      .map_err(|error| self.error(stage, Some(callback), error))?;

    table
      .set("type", event.data.event_type())
      .and_then(|_| table.set("sequence", event.sequence))
      .and_then(|_| table.set("frame", event.frame))
      .and_then(|_| table.set("data", data))
      .map_err(|error| self.error(stage, Some(callback), error))?;
    Ok(table)
  }

  fn invoke_event_callback(
    &mut self,
    callback_id: LuaEventCallbackId,
    event: &LuaRuntimeEvent,
  ) -> Result<(), LuaSessionError> {
    if self.state != LuaSessionState::Running {
      return Err(self.error(
        LuaErrorStage::EventCallback,
        Some("EventCallback"),
        format!("session is {:?}", self.state),
      ));
    }
    let Some(registered) = self.event_callbacks.get(&callback_id) else {
      return Ok(());
    };
    let remove_after =
      registered.lifetime == LuaCallbackLifetime::Once || event.data.callback_is_terminal();
    let function = self
      .lua
      .registry_value::<Function>(&registered.key)
      .map_err(|error| self.error(LuaErrorStage::EventCallback, Some("EventCallback"), error))?;
    let event_table = match self.event_table(event, LuaErrorStage::EventCallback, "EventCallback") {
      Ok(event) => event,
      Err(error) => {
        self.state = LuaSessionState::Faulted;
        return Err(error);
      }
    };
    let outcome = run_with_budget(
      &self.lua,
      function,
      event_table,
      self.policy.budget(LuaBudgetKind::HandleEvent),
      self.policy.hook_interval,
    );
    if remove_after {
      self.unregister_event_callback(callback_id);
    }
    match outcome {
      Ok((_, stats)) => {
        self.last_stats = LuaExecutionStats {
          memory_bytes: self.lua.used_memory(),
          ..stats
        };
        if let Err(error) = self.lua.gc_step() {
          self.state = LuaSessionState::Faulted;
          return Err(self.error(LuaErrorStage::EventCallback, Some("EventCallback"), error));
        }
        Ok(())
      }
      Err(failure) => {
        self.state = LuaSessionState::Faulted;
        Err(self.execution_error(LuaErrorStage::EventCallback, "EventCallback", failure))
      }
    }
  }

  fn invoke_hot<A>(
    &mut self,
    callback: Callback,
    args: A,
    budget_kind: LuaBudgetKind,
  ) -> Result<(), LuaSessionError>
  where
    A: IntoLuaMulti,
  {
    if self.state != LuaSessionState::Running {
      return Err(self.error(
        LuaErrorStage::Callback,
        Some(callback.name()),
        format!("session is {:?}", self.state),
      ));
    }
    let result = self.invoke_required(callback, args, budget_kind, LuaErrorStage::Callback);
    if result.is_err() {
      self.state = LuaSessionState::Faulted;
    }
    result
  }

  fn invoke_required<A>(
    &mut self,
    callback: Callback,
    args: A,
    budget_kind: LuaBudgetKind,
    stage: LuaErrorStage,
  ) -> Result<(), LuaSessionError>
  where
    A: IntoLuaMulti,
  {
    let function = self
      .lua
      .registry_value::<Function>(self.callback_key(callback).expect("required callback"))
      .map_err(|error| self.error(stage, Some(callback.name()), error))?;
    let outcome = run_with_budget(
      &self.lua,
      function,
      args,
      self.policy.budget(budget_kind),
      self.policy.hook_interval,
    );
    match outcome {
      Ok((_, stats)) => {
        self.last_stats = LuaExecutionStats {
          memory_bytes: self.lua.used_memory(),
          ..stats
        };
        self
          .lua
          .gc_step()
          .map_err(|error| self.error(stage, Some(callback.name()), error))?;
        Ok(())
      }
      Err(failure) => Err(self.execution_error(stage, callback.name(), failure)),
    }
  }

  fn invoke_save(&mut self, callback: Callback) -> Result<Option<JsonValue>, LuaSessionError> {
    if self.session_kind != LuaSessionKind::Game {
      return Ok(None);
    }
    let Some(key) = self.callback_key(callback) else {
      return Ok(None);
    };
    let function = self
      .lua
      .registry_value::<Function>(key)
      .map_err(|error| self.error(LuaErrorStage::Callback, Some(callback.name()), error))?;
    let (values, stats) = run_with_budget(
      &self.lua,
      function,
      (),
      self.policy.budget(LuaBudgetKind::Save),
      self.policy.hook_interval,
    )
    .map_err(|failure| self.execution_error(LuaErrorStage::Callback, callback.name(), failure))?;
    self.last_stats = LuaExecutionStats {
      memory_bytes: self.lua.used_memory(),
      ..stats
    };
    self
      .lua
      .gc_step()
      .map_err(|error| self.error(LuaErrorStage::Callback, Some(callback.name()), error))?;

    let value = values.into_iter().next().unwrap_or(Value::Nil);
    let mut seen = HashSet::new();
    let json = lua_to_json(value, 0, self.policy.save_max_depth, &mut seen).map_err(|message| {
      self.error(
        LuaErrorStage::SaveValidation,
        Some(callback.name()),
        message,
      )
    })?;
    let encoded = serde_json::to_vec(&json)
      .map_err(|error| self.error(LuaErrorStage::SaveValidation, Some(callback.name()), error))?;
    if encoded.len() > self.policy.save_limit_bytes {
      return Err(self.error(
        LuaErrorStage::SaveValidation,
        Some(callback.name()),
        format!(
          "serialized save value is {} bytes; limit is {} bytes",
          encoded.len(),
          self.policy.save_limit_bytes
        ),
      ));
    }
    Ok(Some(json))
  }

  fn callback_key(&self, callback: Callback) -> Option<&RegistryKey> {
    match callback {
      Callback::Init => Some(&self.callbacks.init),
      Callback::HandleEvent => Some(&self.callbacks.handle_event),
      Callback::Update => Some(&self.callbacks.update),
      Callback::UpdateFrame => Some(&self.callbacks.update_frame),
      Callback::Render => Some(&self.callbacks.render),
      Callback::SaveGame => self.callbacks.save_game.as_ref(),
      Callback::SaveBest => self.callbacks.save_best.as_ref(),
    }
  }

  fn execution_error(
    &self,
    stage: LuaErrorStage,
    callback: &'static str,
    failure: LuaExecutionFailure,
  ) -> LuaSessionError {
    let (actual_stage, message) = match failure {
      LuaExecutionFailure::Lua(error) => {
        let error_stage = if matches!(error, mlua::Error::MemoryError(_)) {
          LuaErrorStage::MemoryLimit
        } else {
          stage
        };
        (error_stage, error.to_string())
      }
      LuaExecutionFailure::Limit {
        instructions,
        elapsed,
      } => (
        LuaErrorStage::ExecutionLimit,
        format!(
          "execution budget exceeded after {instructions} instructions and {} ms",
          elapsed.as_millis()
        ),
      ),
    };
    self.error(actual_stage, Some(callback), message)
  }

  fn error(
    &self,
    stage: LuaErrorStage,
    callback: Option<&'static str>,
    message: impl ToString,
  ) -> LuaSessionError {
    LuaSessionError {
      package_id: self.package_id.clone(),
      session_kind: self.session_kind,
      stage,
      callback,
      message: message.to_string(),
    }
  }

  #[cfg(test)]
  fn environment_value(&self, name: &str) -> Value {
    let environment: Table = self.lua.registry_value(&self.environment).unwrap();
    environment.get(name).unwrap()
  }

  #[cfg(test)]
  fn register_environment_event_callback(
    &mut self,
    name: &str,
    lifetime: LuaCallbackLifetime,
  ) -> LuaEventCallbackId {
    let environment: Table = self.lua.registry_value(&self.environment).unwrap();
    let function: Function = environment.get(name).unwrap();
    self.register_event_callback(function, lifetime).unwrap()
  }
}

impl Drop for LuaSession {
  fn drop(&mut self) {
    self.stop();
  }
}

#[derive(Clone, Copy)]
enum Callback {
  Init,
  HandleEvent,
  Update,
  UpdateFrame,
  Render,
  SaveGame,
  SaveBest,
}

impl Callback {
  fn name(self) -> &'static str {
    match self {
      Self::Init => "Init",
      Self::HandleEvent => "HandleEvent",
      Self::Update => "Update",
      Self::UpdateFrame => "UpdateFrame",
      Self::Render => "Render",
      Self::SaveGame => "SaveGame",
      Self::SaveBest => "SaveBest",
    }
  }
}

fn read_source(spec: &LuaSessionSpec, policy: &LuaPolicy) -> Result<String, LuaSessionError> {
  if !has_lua_extension(&spec.entry_path) {
    return Err(session_error(
      spec,
      LuaErrorStage::ReadSource,
      None,
      "entry source must use the .lua extension",
    ));
  }

  let file = File::open(&spec.entry_path)
    .map_err(|error| session_error(spec, LuaErrorStage::ReadSource, None, error))?;
  let metadata = file
    .metadata()
    .map_err(|error| session_error(spec, LuaErrorStage::ReadSource, None, error))?;
  if !metadata.is_file() {
    return Err(session_error(
      spec,
      LuaErrorStage::ReadSource,
      None,
      "entry source is not a regular file",
    ));
  }
  if metadata.len() > policy.source_limit_bytes as u64 {
    return Err(session_error(
      spec,
      LuaErrorStage::ReadSource,
      None,
      format!(
        "entry source is {} bytes; limit is {} bytes",
        metadata.len(),
        policy.source_limit_bytes
      ),
    ));
  }

  let read_limit = policy.source_limit_bytes.saturating_add(1) as u64;
  let mut bytes = Vec::with_capacity(metadata.len() as usize);
  file
    .take(read_limit)
    .read_to_end(&mut bytes)
    .map_err(|error| session_error(spec, LuaErrorStage::ReadSource, None, error))?;
  if bytes.len() > policy.source_limit_bytes {
    return Err(session_error(
      spec,
      LuaErrorStage::ReadSource,
      None,
      format!(
        "entry source exceeds the {} byte limit while being read",
        policy.source_limit_bytes
      ),
    ));
  }
  String::from_utf8(bytes)
    .map_err(|error| session_error(spec, LuaErrorStage::ReadSource, None, error))
}

fn has_lua_extension(path: &Path) -> bool {
  path
    .extension()
    .and_then(|extension| extension.to_str())
    .is_some_and(|extension| extension.eq_ignore_ascii_case("lua"))
}

fn validate_continue_data(
  spec: &LuaSessionSpec,
  policy: &LuaPolicy,
) -> Result<(), LuaSessionError> {
  let Some(value) = spec.continue_data.as_ref() else {
    return Ok(());
  };
  validate_json_depth(value, 0, policy.save_max_depth).map_err(|message| {
    session_error(
      spec,
      LuaErrorStage::ContinueDataValidation,
      Some("Init"),
      message,
    )
  })?;
  let encoded = serde_json::to_vec(value).map_err(|error| {
    session_error(
      spec,
      LuaErrorStage::ContinueDataValidation,
      Some("Init"),
      error,
    )
  })?;
  if encoded.len() > policy.save_limit_bytes {
    return Err(session_error(
      spec,
      LuaErrorStage::ContinueDataValidation,
      Some("Init"),
      format!(
        "continue data is {} bytes; limit is {} bytes",
        encoded.len(),
        policy.save_limit_bytes
      ),
    ));
  }
  Ok(())
}

fn validate_json_depth(value: &JsonValue, depth: usize, max_depth: usize) -> Result<(), String> {
  if depth > max_depth {
    return Err(format!("continue data exceeds maximum depth {max_depth}"));
  }
  match value {
    JsonValue::Array(values) => {
      for value in values {
        validate_json_depth(value, depth + 1, max_depth)?;
      }
    }
    JsonValue::Object(values) => {
      for value in values.values() {
        validate_json_depth(value, depth + 1, max_depth)?;
      }
    }
    _ => {}
  }
  Ok(())
}

fn build_sandbox(lua: &Lua) -> mlua::Result<Table> {
  let globals = lua.globals();
  let environment = lua.create_table()?;
  for name in [
    "assert", "error", "pcall", "xpcall", "ipairs", "pairs", "next", "select", "tonumber",
    "tostring", "type", "rawequal", "rawlen", "_VERSION",
  ] {
    environment.set(name, globals.get::<Value>(name)?)?;
  }

  for library in ["string", "utf8", "table"] {
    environment.set(library, copy_table(lua, globals.get::<Table>(library)?)?)?;
  }
  let math = copy_table(lua, globals.get::<Table>("math")?)?;
  for name in ["random", "randomseed"] {
    math.set(name, unsupported_function(lua, &format!("math.{name}"))?)?;
  }
  environment.set("math", math)?;

  for name in [
    "print",
    "warn",
    "collectgarbage",
    "getmetatable",
    "setmetatable",
  ] {
    environment.set(name, unsupported_function(lua, name)?)?;
  }
  environment.set("_G", environment.clone())?;
  Ok(environment)
}

fn copy_table(lua: &Lua, source: Table) -> mlua::Result<Table> {
  let target = lua.create_table()?;
  for pair in source.pairs::<Value, Value>() {
    let (key, value) = pair?;
    target.raw_set(key, value)?;
  }
  Ok(target)
}

fn unsupported_function(lua: &Lua, name: &str) -> mlua::Result<Function> {
  let name = name.to_string();
  lua.create_function(move |_, _: MultiValue| -> mlua::Result<()> {
    Err(mlua::Error::RuntimeError(format!(
      "'{name}' is not supported by the TUI GAME Lua runtime"
    )))
  })
}

fn discover_callbacks(
  lua: &Lua,
  environment: &Table,
  spec: &LuaSessionSpec,
) -> Result<LuaCallbacks, LuaSessionError> {
  let required = |name: &'static str| -> Result<RegistryKey, LuaSessionError> {
    let value = environment
      .get::<Value>(name)
      .map_err(|error| session_error(spec, LuaErrorStage::DiscoverCallbacks, Some(name), error))?;
    let Value::Function(function) = value else {
      return Err(session_error(
        spec,
        LuaErrorStage::DiscoverCallbacks,
        Some(name),
        format!("required callback '{name}' is missing or is not a function"),
      ));
    };
    lua
      .create_registry_value(function)
      .map_err(|error| session_error(spec, LuaErrorStage::DiscoverCallbacks, Some(name), error))
  };
  let optional = |name: &'static str| -> Result<Option<RegistryKey>, LuaSessionError> {
    match environment
      .get::<Value>(name)
      .map_err(|error| session_error(spec, LuaErrorStage::DiscoverCallbacks, Some(name), error))?
    {
      Value::Nil => Ok(None),
      Value::Function(function) => lua
        .create_registry_value(function)
        .map(Some)
        .map_err(|error| session_error(spec, LuaErrorStage::DiscoverCallbacks, Some(name), error)),
      _ => Err(session_error(
        spec,
        LuaErrorStage::DiscoverCallbacks,
        Some(name),
        format!("optional callback '{name}' exists but is not a function"),
      )),
    }
  };

  let mut keys = Vec::with_capacity(REQUIRED_CALLBACKS.len());
  for name in REQUIRED_CALLBACKS {
    keys.push(required(name)?);
  }
  let mut keys = keys.into_iter();
  Ok(LuaCallbacks {
    init: keys.next().unwrap(),
    handle_event: keys.next().unwrap(),
    update: keys.next().unwrap(),
    update_frame: keys.next().unwrap(),
    render: keys.next().unwrap(),
    save_game: (spec.session_kind == LuaSessionKind::Game)
      .then(|| optional("SaveGame"))
      .transpose()?
      .flatten(),
    save_best: (spec.session_kind == LuaSessionKind::Game)
      .then(|| optional("SaveBest"))
      .transpose()?
      .flatten(),
  })
}

enum LuaExecutionFailure {
  Lua(mlua::Error),
  Limit {
    instructions: u64,
    elapsed: Duration,
  },
}

fn run_with_budget<A>(
  lua: &Lua,
  function: Function,
  args: A,
  budget: LuaExecutionBudget,
  hook_interval: u32,
) -> Result<(MultiValue, LuaExecutionStats), LuaExecutionFailure>
where
  A: IntoLuaMulti,
{
  let thread = lua
    .create_thread(function)
    .map_err(LuaExecutionFailure::Lua)?;
  let started = Instant::now();
  let instructions = Rc::new(Cell::new(0_u64));
  let exceeded = Rc::new(Cell::new(false));
  let hook_instructions = Rc::clone(&instructions);
  let hook_exceeded = Rc::clone(&exceeded);
  thread
    .set_hook(
      HookTriggers::new().every_nth_instruction(hook_interval),
      move |_, _| {
        let current = hook_instructions
          .get()
          .saturating_add(u64::from(hook_interval));
        hook_instructions.set(current);
        if current > budget.max_instructions || started.elapsed() > budget.max_duration {
          hook_exceeded.set(true);
          Ok(VmState::Yield)
        } else {
          Ok(VmState::Continue)
        }
      },
    )
    .map_err(LuaExecutionFailure::Lua)?;

  let result = thread.resume::<MultiValue>(args);
  thread.remove_hook();
  let elapsed = started.elapsed();
  let values = result.map_err(LuaExecutionFailure::Lua)?;
  if exceeded.get()
    || !thread.is_finished()
    || elapsed > budget.max_duration
    || instructions.get() > budget.max_instructions
  {
    return Err(LuaExecutionFailure::Limit {
      instructions: instructions.get(),
      elapsed,
    });
  }
  Ok((
    values,
    LuaExecutionStats {
      instructions: instructions.get(),
      elapsed,
      memory_bytes: lua.used_memory(),
    },
  ))
}

fn session_error(
  spec: &LuaSessionSpec,
  stage: LuaErrorStage,
  callback: Option<&'static str>,
  message: impl ToString,
) -> LuaSessionError {
  LuaSessionError {
    package_id: spec.package_id.clone(),
    session_kind: spec.session_kind,
    stage,
    callback,
    message: message.to_string(),
  }
}

fn execution_error(
  spec: &LuaSessionSpec,
  stage: LuaErrorStage,
  callback: Option<&'static str>,
  failure: LuaExecutionFailure,
) -> LuaSessionError {
  match failure {
    LuaExecutionFailure::Lua(error) => {
      let stage = if matches!(error, mlua::Error::MemoryError(_)) {
        LuaErrorStage::MemoryLimit
      } else {
        stage
      };
      session_error(spec, stage, callback, error)
    }
    LuaExecutionFailure::Limit {
      instructions,
      elapsed,
    } => session_error(
      spec,
      LuaErrorStage::ExecutionLimit,
      callback,
      format!(
        "execution budget exceeded after {instructions} instructions and {} ms",
        elapsed.as_millis()
      ),
    ),
  }
}

fn json_to_lua(lua: &Lua, value: &JsonValue) -> mlua::Result<Value> {
  match value {
    JsonValue::Null => Ok(Value::Nil),
    JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
    JsonValue::Number(value) => {
      if let Some(integer) = value.as_i64() {
        Ok(Value::Integer(integer))
      } else {
        Ok(Value::Number(value.as_f64().unwrap_or_default()))
      }
    }
    JsonValue::String(value) => lua.create_string(value).map(Value::String),
    JsonValue::Array(values) => {
      let table = lua.create_table_with_capacity(values.len(), 0)?;
      for (index, value) in values.iter().enumerate() {
        table.raw_set(index + 1, json_to_lua(lua, value)?)?;
      }
      Ok(Value::Table(table))
    }
    JsonValue::Object(values) => {
      let table = lua.create_table_with_capacity(0, values.len())?;
      for (key, value) in values {
        table.raw_set(key.as_str(), json_to_lua(lua, value)?)?;
      }
      Ok(Value::Table(table))
    }
  }
}

fn lua_to_json(
  value: Value,
  depth: usize,
  max_depth: usize,
  seen: &mut HashSet<usize>,
) -> Result<JsonValue, String> {
  if depth > max_depth {
    return Err(format!("save value exceeds maximum depth {max_depth}"));
  }
  match value {
    Value::Nil => Ok(JsonValue::Null),
    Value::Boolean(value) => Ok(JsonValue::Bool(value)),
    Value::Integer(value) => Ok(JsonValue::Number(JsonNumber::from(value))),
    Value::Number(value) => {
      if !value.is_finite() {
        return Err("save value contains a non-finite number".to_string());
      }
      JsonNumber::from_f64(value)
        .map(JsonValue::Number)
        .ok_or_else(|| "save value contains an invalid number".to_string())
    }
    Value::String(value) => value
      .to_str()
      .map(|value| JsonValue::String(value.to_string()))
      .map_err(|_| "save value contains a non-UTF-8 string".to_string()),
    Value::Table(table) => {
      let pointer = table.to_pointer() as usize;
      if !seen.insert(pointer) {
        return Err("save value contains a table cycle".to_string());
      }
      let result = table_to_json(table, depth, max_depth, seen);
      seen.remove(&pointer);
      result
    }
    Value::Function(_) | Value::Thread(_) | Value::UserData(_) | Value::LightUserData(_) => {
      Err(format!(
        "save value contains unsupported Lua type '{}'",
        value.type_name()
      ))
    }
    Value::Error(error) => Err(format!("save value contains Lua error: {error}")),
    Value::Other(_) => Err("save value contains an unsupported Lua value".to_string()),
  }
}

fn table_to_json(
  table: Table,
  depth: usize,
  max_depth: usize,
  seen: &mut HashSet<usize>,
) -> Result<JsonValue, String> {
  let mut integer_values = BTreeMap::<i64, Value>::new();
  let mut string_values = BTreeMap::<String, Value>::new();

  for pair in table.pairs::<Value, Value>() {
    let (key, value) = pair.map_err(|error| error.to_string())?;
    match key {
      Value::Integer(index) if index > 0 => {
        integer_values.insert(index, value);
      }
      Value::String(key) => {
        let key = key
          .to_str()
          .map_err(|_| "save object contains a non-UTF-8 key".to_string())?
          .to_string();
        string_values.insert(key, value);
      }
      _ => return Err("save object keys must be strings or sequential integers".to_string()),
    }
  }

  if !integer_values.is_empty() && !string_values.is_empty() {
    return Err("save table cannot mix array and object keys".to_string());
  }
  if !integer_values.is_empty() {
    let expected_len = integer_values.len() as i64;
    if integer_values.keys().copied().ne(1..=expected_len) {
      return Err("save array keys must be contiguous and start at 1".to_string());
    }
    let values = integer_values
      .into_values()
      .map(|value| lua_to_json(value, depth + 1, max_depth, seen))
      .collect::<Result<Vec<_>, _>>()?;
    return Ok(JsonValue::Array(values));
  }

  let mut object = JsonMap::new();
  for (key, value) in string_values {
    object.insert(key, lua_to_json(value, depth + 1, max_depth, seen)?);
  }
  Ok(JsonValue::Object(object))
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::sync::atomic::{AtomicU64, Ordering};

  use super::super::LuaEventData;
  use super::*;

  static TEST_ID: AtomicU64 = AtomicU64::new(1);

  fn script_path(source: &str) -> PathBuf {
    let id = TEST_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
      "tui_game_lua_session_{}_{}",
      std::process::id(),
      id
    ));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("main.lua");
    fs::write(&path, source).unwrap();
    path
  }

  fn spec(source: &str, kind: LuaSessionKind) -> LuaSessionSpec {
    LuaSessionSpec {
      package_id: "test.package".to_string(),
      session_kind: kind,
      entry_path: script_path(source),
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      terminal_size: Size {
        width: 120,
        height: 40,
      },
      continue_data: None,
    }
  }

  fn valid_script(extra: &str) -> String {
    format!(
      r#"
        function Init(ctx) init_ctx = ctx end
        function HandleEvent(event) last_event = event end
        function Update(dt) last_update = dt end
        function UpdateFrame(dt, alpha) last_frame = {{ dt, alpha }} end
        function Render(draw) last_draw = draw end
        {extra}
      "#
    )
  }

  #[test]
  fn creates_isolated_game_and_screensaver_vms() {
    let first = LuaSession::load(
      spec(
        &valid_script("private_value = 'game'"),
        LuaSessionKind::Game,
      ),
      LuaPolicy::default(),
    )
    .unwrap();
    let second = LuaSession::load(
      spec(
        &valid_script("private_value = 'screensaver'"),
        LuaSessionKind::Screensaver,
      ),
      LuaPolicy::default(),
    )
    .unwrap();

    assert_eq!(
      first.environment_value("private_value"),
      Value::String(first.lua.create_string("game").unwrap())
    );
    assert_eq!(
      second.environment_value("private_value"),
      Value::String(second.lua.create_string("screensaver").unwrap())
    );
    assert_ne!(
      first.environment_value("_G").to_pointer(),
      second.environment_value("_G").to_pointer()
    );
  }

  #[test]
  fn sandbox_keeps_whitelist_and_hides_dangerous_apis() {
    let session = LuaSession::load(
      spec(&valid_script(""), LuaSessionKind::Game),
      LuaPolicy::default(),
    )
    .unwrap();
    for name in [
      "math", "string", "utf8", "table", "pcall", "rawequal", "rawlen",
    ] {
      assert_ne!(session.environment_value(name), Value::Nil, "{name}");
    }
    for name in [
      "load",
      "loadfile",
      "loadstring",
      "dofile",
      "require",
      "rawget",
      "rawset",
      "os",
      "io",
      "debug",
      "package",
      "coroutine",
    ] {
      assert_eq!(session.environment_value(name), Value::Nil, "{name}");
    }
  }

  #[test]
  fn rejects_missing_required_callback() {
    let error = match LuaSession::load(
      spec("function Init(ctx) end", LuaSessionKind::Game),
      LuaPolicy::default(),
    ) {
      Ok(_) => panic!("session unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(error.stage, LuaErrorStage::DiscoverCallbacks);
    assert_eq!(error.callback, Some("HandleEvent"));
  }

  #[test]
  fn save_validation_accepts_json_and_rejects_cycles() {
    let source = valid_script(
      r#"
        function SaveGame()
          return { name = "save", values = { 1, 2, 3 }, enabled = true }
        end
        function SaveBest()
          local value = {}
          value.self = value
          return value
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let save = session.save_game().unwrap().unwrap();
    assert_eq!(save["name"], "save");
    assert_eq!(save["values"][2], 3);
    assert_eq!(
      session.save_best().unwrap_err().stage,
      LuaErrorStage::SaveValidation
    );
  }

  #[test]
  fn instruction_budget_faults_an_infinite_update() {
    let source = valid_script("function Update(dt) while true do end end");
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert_eq!(session.state(), LuaSessionState::Faulted);
  }

  #[test]
  fn instruction_budget_cannot_be_hidden_by_pcall() {
    let source = valid_script("function Update(dt) pcall(function() while true do end end) end");
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert_eq!(session.state(), LuaSessionState::Faulted);
  }

  #[test]
  fn lifecycle_context_events_and_draw_are_stable() {
    let source = r#"
      local calls = {}
      local context = nil
      local event_value = nil
      local draw_value = nil

      function Init(ctx)
        context = ctx
        calls[#calls + 1] = "Init"
      end
      function HandleEvent(event)
        event_value = event
        calls[#calls + 1] = "HandleEvent"
      end
      function Update(dt)
        calls[#calls + 1] = "Update"
      end
      function UpdateFrame(dt, alpha)
        calls[#calls + 1] = "UpdateFrame"
      end
      function Render(draw)
        draw_value = draw
        calls[#calls + 1] = "Render"
      end
      function SaveGame()
        return {
          calls = calls,
          package_id = context.package_id,
          package_type = context.package_type,
          fixed_delta = context.fixed_delta,
          terminal_width = context.terminal.width,
          terminal_height = context.terminal.height,
          api_version = context.api_version,
          event_type = event_value.type,
          event_sequence = event_value.sequence,
          event_frame = event_value.frame,
          event_action = event_value.data.action,
          event_state = event_value.data.state,
          draw_width = draw_value.width,
          draw_height = draw_value.height
        }
      end
    "#;
    let mut session =
      LuaSession::load(spec(source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    session
      .handle_event(&LuaRuntimeEvent {
        sequence: 9,
        frame: 15,
        data: LuaEventData::Action {
          action: "jump".to_string(),
          state: super::super::LuaActionState::Pressed,
        },
      })
      .unwrap();
    session.update().unwrap();
    session
      .update_frame(Duration::from_millis(16), 0.5)
      .unwrap();
    session
      .render(Size {
        width: 100,
        height: 30,
      })
      .unwrap();

    let save = session.save_game().unwrap().unwrap();
    assert_eq!(
      save["calls"],
      serde_json::json!(["Init", "HandleEvent", "Update", "UpdateFrame", "Render"])
    );
    assert_eq!(save["package_id"], "test.package");
    assert_eq!(save["package_type"], "game");
    assert_eq!(save["terminal_width"], 120);
    assert_eq!(save["terminal_height"], 40);
    assert_eq!(save["api_version"], 1);
    assert_eq!(save["event_type"], "action");
    assert_eq!(save["event_sequence"], 9);
    assert_eq!(save["event_frame"], 15);
    assert_eq!(save["event_action"], "jump");
    assert_eq!(save["event_state"], "pressed");
    assert_eq!(save["draw_width"], 100);
    assert_eq!(save["draw_height"], 30);
  }

  #[test]
  fn registered_callback_receives_the_envelope_without_calling_handle_event() {
    let source = valid_script(
      r#"
        handle_count = 0
        callback_count = 0
        function HandleEvent(event)
          handle_count = handle_count + 1
        end
        function ServiceCallback(event)
          callback_count = callback_count + 1
          callback_type = event.type
          callback_sequence = event.sequence
          callback_frame = event.frame
          callback_gained = event.data.gained
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let callback =
      session.register_environment_event_callback("ServiceCallback", LuaCallbackLifetime::Once);
    let delivery = LuaEventDelivery {
      event: LuaRuntimeEvent {
        sequence: 17,
        frame: 29,
        data: LuaEventData::Focus { gained: false },
      },
      route: LuaEventRoute::Callback(callback),
    };

    session.dispatch_event(&delivery).unwrap();
    assert!(session.event_callbacks.is_empty());
    // 一次性回调已回收；重复完成事件不能转投 HandleEvent。
    session.dispatch_event(&delivery).unwrap();

    assert_eq!(session.environment_value("handle_count"), Value::Integer(0));
    assert_eq!(
      session.environment_value("callback_count"),
      Value::Integer(1)
    );
    assert_eq!(
      session.environment_value("callback_type"),
      Value::String(session.lua.create_string("focus").unwrap())
    );
    assert_eq!(
      session.environment_value("callback_sequence"),
      Value::Integer(17)
    );
    assert_eq!(
      session.environment_value("callback_frame"),
      Value::Integer(29)
    );
    assert_eq!(
      session.environment_value("callback_gained"),
      Value::Boolean(false)
    );
  }

  #[test]
  fn persistent_callback_is_removed_after_its_terminal_event() {
    let source = valid_script(
      r#"
        callback_count = 0
        function AnimationCallback(event)
          callback_count = callback_count + 1
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let callback = session
      .register_environment_event_callback("AnimationCallback", LuaCallbackLifetime::UntilTerminal);
    let delivery = |kind| LuaEventDelivery {
      event: LuaRuntimeEvent {
        sequence: 1,
        frame: 1,
        data: LuaEventData::Animation(super::super::LuaAnimationEvent { id: 4, kind }),
      },
      route: LuaEventRoute::Callback(callback),
    };

    session
      .dispatch_event(&delivery(super::super::LuaAnimationEventKind::Marker {
        name: "half".to_string(),
      }))
      .unwrap();
    session
      .dispatch_event(&delivery(super::super::LuaAnimationEventKind::Finished))
      .unwrap();
    assert!(session.event_callbacks.is_empty());
    session
      .dispatch_event(&delivery(super::super::LuaAnimationEventKind::Finished))
      .unwrap();

    assert_eq!(
      session.environment_value("callback_count"),
      Value::Integer(2)
    );
  }

  #[test]
  fn event_callback_uses_the_handle_event_execution_budget() {
    let source = valid_script(
      r#"
        function BusyCallback(event)
          while true do end
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let callback =
      session.register_environment_event_callback("BusyCallback", LuaCallbackLifetime::Once);
    let error = session
      .dispatch_event(&LuaEventDelivery {
        event: LuaRuntimeEvent {
          sequence: 1,
          frame: 1,
          data: LuaEventData::Focus { gained: true },
        },
        route: LuaEventRoute::Callback(callback),
      })
      .unwrap_err();

    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert_eq!(error.callback, Some("EventCallback"));
    assert_eq!(session.state(), LuaSessionState::Faulted);
  }

  #[test]
  fn test_package_entries_execute_the_basic_lifecycle() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for (directory, session_kind) in [
      ("test_package/game", LuaSessionKind::Game),
      ("test_package/screensaver", LuaSessionKind::Screensaver),
    ] {
      let package_root = manifest_dir.join(directory);
      let mut package_count = 0;

      for entry in fs::read_dir(&package_root).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
          continue;
        }
        package_count += 1;

        let package_id = entry.file_name().to_string_lossy().into_owned();
        let entry_path = entry.path().join("scripts").join("main.lua");
        let mut session = LuaSession::load(
          LuaSessionSpec {
            package_id: package_id.clone(),
            session_kind,
            entry_path,
            fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
            terminal_size: Size {
              width: 120,
              height: 40,
            },
            continue_data: None,
          },
          LuaPolicy::default(),
        )
        .unwrap_or_else(|error| panic!("{package_id} failed to load: {error}"));

        session
          .handle_event(&LuaRuntimeEvent {
            sequence: 1,
            frame: 1,
            data: LuaEventData::Resize {
              width: 100,
              height: 30,
            },
          })
          .unwrap_or_else(|error| panic!("{package_id} HandleEvent failed: {error}"));
        session
          .update()
          .unwrap_or_else(|error| panic!("{package_id} Update failed: {error}"));
        session
          .update_frame(Duration::from_millis(16), 0.5)
          .unwrap_or_else(|error| panic!("{package_id} UpdateFrame failed: {error}"));
        session
          .render(Size {
            width: 100,
            height: 30,
          })
          .unwrap_or_else(|error| panic!("{package_id} Render failed: {error}"));

        if session_kind == LuaSessionKind::Game {
          assert!(
            session.save_game().unwrap().is_some(),
            "{package_id} SaveGame returned no value"
          );
          assert!(
            session.save_best().unwrap().is_some(),
            "{package_id} SaveBest returned no value"
          );
        }
      }

      assert!(package_count > 0, "no test packages found in {directory}");
    }
  }

  #[test]
  fn source_limits_and_utf8_are_checked_before_vm_creation() {
    let mut policy = LuaPolicy::default();
    policy.source_limit_bytes = 16;
    let too_large = match LuaSession::load(spec(&valid_script(""), LuaSessionKind::Game), policy) {
      Ok(_) => panic!("oversized source unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(too_large.stage, LuaErrorStage::ReadSource);

    let path = script_path("");
    fs::write(&path, [0xff, 0xfe, 0xfd]).unwrap();
    let invalid_utf8 = match LuaSession::load(
      LuaSessionSpec {
        package_id: "invalid.utf8".to_string(),
        session_kind: LuaSessionKind::Game,
        entry_path: path,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        terminal_size: Size {
          width: 80,
          height: 24,
        },
        continue_data: None,
      },
      LuaPolicy::default(),
    ) {
      Ok(_) => panic!("non-UTF-8 source unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(invalid_utf8.stage, LuaErrorStage::ReadSource);
  }

  #[test]
  fn invalid_policy_and_non_lua_entries_are_rejected_before_vm_creation() {
    let mut policy = LuaPolicy::default();
    policy.hook_interval = 0;
    let invalid_policy =
      match LuaSession::load(spec(&valid_script(""), LuaSessionKind::Game), policy) {
        Ok(_) => panic!("invalid policy unexpectedly loaded"),
        Err(error) => error,
      };
    assert_eq!(invalid_policy.stage, LuaErrorStage::ValidatePolicy);

    let path = script_path(&valid_script(""));
    let text_path = path.with_extension("txt");
    fs::rename(path, &text_path).unwrap();
    let non_lua = match LuaSession::load(
      LuaSessionSpec {
        package_id: "invalid.extension".to_string(),
        session_kind: LuaSessionKind::Game,
        entry_path: text_path,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        terminal_size: Size {
          width: 80,
          height: 24,
        },
        continue_data: None,
      },
      LuaPolicy::default(),
    ) {
      Ok(_) => panic!("non-Lua entry unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(non_lua.stage, LuaErrorStage::ReadSource);
  }

  #[test]
  fn continue_data_obeys_save_size_and_depth_limits() {
    let mut oversized_spec = spec(&valid_script(""), LuaSessionKind::Game);
    oversized_spec.continue_data = Some(serde_json::json!({ "value": "too large" }));
    let mut policy = LuaPolicy::default();
    policy.save_limit_bytes = 4;
    let oversized = match LuaSession::load(oversized_spec, policy) {
      Ok(_) => panic!("oversized continue data unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(oversized.stage, LuaErrorStage::ContinueDataValidation);

    let mut deep_spec = spec(&valid_script(""), LuaSessionKind::Game);
    deep_spec.continue_data = Some(serde_json::json!({ "a": { "b": { "c": true } } }));
    let mut policy = LuaPolicy::default();
    policy.save_max_depth = 2;
    let too_deep = match LuaSession::load(deep_spec, policy) {
      Ok(_) => panic!("deep continue data unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(too_deep.stage, LuaErrorStage::ContinueDataValidation);
  }

  #[test]
  fn memory_limit_faults_only_the_current_session() {
    let source =
      valid_script("function Update(dt) local value = string.rep('x', 40 * 1024 * 1024) end");
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::MemoryLimit);
    assert_eq!(session.state(), LuaSessionState::Faulted);
  }
}
