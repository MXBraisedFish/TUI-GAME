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

use super::api::{
  self, LuaApiConfig, LuaApiContext, LuaCallPhase, LuaDrawCommand, LuaHostCommand, SharedApiState,
};
use super::events::{LuaEventCallbackId, LuaEventDelivery, LuaEventRoute, LuaRuntimeEvent};
use super::object_pool::{SharedLuaObjectPool, shared_lua_object_pool};
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
  BestDataValidation,
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
  pub best_data: Option<JsonValue>,
  pub save_game_enabled: bool,
  pub save_best_enabled: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaExecutionLimitKind {
  Time,
  Instructions,
}

impl LuaExecutionLimitKind {
  fn as_str(self) -> &'static str {
    match self {
      Self::Time => "time",
      Self::Instructions => "instructions",
    }
  }
}

#[derive(Clone, Copy, Debug)]
struct SlowCallbackWarning {
  last_logged: Instant,
  suppressed: u64,
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
  objects: SharedLuaObjectPool,
  event_callbacks: HashMap<LuaEventCallbackId, LuaRegisteredCallback>,
  next_event_callback_id: u64,
  api_state: SharedApiState,
  slow_callback_warnings: HashMap<&'static str, SlowCallbackWarning>,
}

impl LuaSession {
  pub(super) fn load(spec: LuaSessionSpec, policy: LuaPolicy) -> Result<Self, LuaSessionError> {
    Self::load_with_api(spec, policy, LuaApiConfig::default())
  }

  pub(super) fn load_with_api(
    spec: LuaSessionSpec,
    policy: LuaPolicy,
    api_config: LuaApiConfig,
  ) -> Result<Self, LuaSessionError> {
    policy
      .validate()
      .map_err(|error| session_error(&spec, LuaErrorStage::ValidatePolicy, None, error))?;
    validate_continue_data(&spec, &policy)?;
    validate_best_data(&spec, &policy)?;
    let source = read_source(&spec, &policy)?;
    let lua = Lua::new_with(StdLib::NONE, LuaOptions::default())
      .map_err(|error| session_error(&spec, LuaErrorStage::CreateVm, None, error))?;
    lua
      .set_memory_limit(policy.memory_limit_bytes)
      .map_err(|error| session_error(&spec, LuaErrorStage::MemoryLimit, None, error))?;

    let scripts_root = spec
      .entry_path
      .ancestors()
      .find(|path| {
        path
          .file_name()
          .is_some_and(|name| name.eq_ignore_ascii_case("scripts"))
      })
      .unwrap_or_else(|| spec.entry_path.parent().unwrap_or(Path::new(".")))
      .to_path_buf();
    let assets_root = scripts_root
      .parent()
      .unwrap_or_else(|| Path::new("."))
      .join("assets");
    let objects = shared_lua_object_pool();
    let (environment, api_state) = api::build_environment(
      &lua,
      LuaApiContext {
        package_id: spec.package_id.clone(),
        session_kind: spec.session_kind,
        scripts_root,
        assets_root,
        debug_enabled: api_config.debug_enabled,
        safe_mode_enabled: spec.session_kind == LuaSessionKind::Screensaver
          || api_config.safe_mode_enabled,
        terminal_size: spec.terminal_size,
        key_actions: api_config.key_actions,
        key_default_actions: api_config.key_default_actions,
      },
      Rc::downgrade(&objects),
    )
    .map_err(|error| session_error(&spec, LuaErrorStage::BuildSandbox, None, error))?;
    let entry_function = lua
      .load(source)
      .set_name(spec.entry_path.to_string_lossy())
      .set_mode(ChunkMode::Text)
      .set_environment(environment.clone())
      .into_function()
      .map_err(|error| session_error(&spec, LuaErrorStage::ExecuteEntry, None, error))?;
    let (_, load_stats) = run_with_budget(
      &lua,
      entry_function,
      (),
      policy.budget(LuaBudgetKind::Load),
      policy.hook_interval,
      &api_state,
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
      objects,
      event_callbacks: HashMap::new(),
      next_event_callback_id: 1,
      api_state,
      slow_callback_warnings: HashMap::new(),
    };
    let load_budget = session.policy.budget(LuaBudgetKind::Load);
    session.record_slow_callback("Load", load_budget, load_stats);
    let context = session.context_table(spec.continue_data.as_ref(), spec.best_data.as_ref())?;
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
    self.api_state.borrow_mut().context.terminal_size = size;
  }

  pub fn configure_api(
    &mut self,
    debug_enabled: bool,
    safe_mode_enabled: bool,
    key_actions: HashMap<String, Vec<Vec<String>>>,
    key_default_actions: HashMap<String, Vec<Vec<String>>>,
  ) {
    let mut state = self.api_state.borrow_mut();
    state.context.debug_enabled = debug_enabled;
    state.context.safe_mode_enabled =
      self.session_kind == LuaSessionKind::Screensaver || safe_mode_enabled;
    state.context.key_actions = key_actions;
    state.context.key_default_actions = key_default_actions;
  }

  pub fn last_stats(&self) -> LuaExecutionStats {
    self.last_stats
  }

  pub fn memory_used(&self) -> usize {
    self.lua.used_memory()
  }

  pub fn has_objects(&self) -> bool {
    self.objects.borrow().is_some()
  }

  pub fn with_objects<R>(&self, operation: impl FnOnce(&super::LuaObjectPool) -> R) -> Option<R> {
    let objects = self.objects.borrow();
    Some(operation(objects.as_ref()?))
  }

  pub fn with_objects_mut<R>(
    &self,
    operation: impl FnOnce(&mut super::LuaObjectPool) -> R,
  ) -> Option<R> {
    let mut objects = self.objects.borrow_mut();
    Some(operation(objects.as_mut()?))
  }

  pub fn handle_event(&mut self, event: &LuaRuntimeEvent) -> Result<(), LuaSessionError> {
    let event = match self.event_table(event, LuaErrorStage::Callback, "HandleEvent") {
      Ok(event) => event,
      Err(error) => {
        self.mark_faulted();
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
    let draw_context = self
      .lua
      .create_table()
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Render"), error))?;
    draw_context
      .set("width", size.width)
      .and_then(|_| draw_context.set("height", size.height))
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Render"), error))?;
    let environment: Table = self
      .lua
      .registry_value(&self.environment)
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Render"), error))?;
    let draw_library: Table = environment
      .get("draw")
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Render"), error))?;
    for name in ["text", "fill_rect", "stroke_rect", "erase_rect", "render"] {
      let value = draw_library
        .get::<Value>(name)
        .map_err(|error| self.error(LuaErrorStage::Callback, Some("Render"), error))?;
      draw_context
        .set(name, value)
        .map_err(|error| self.error(LuaErrorStage::Callback, Some("Render"), error))?;
    }
    let draw = super::api::readonly::proxy(&self.lua, draw_context)
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Render"), error))?;
    self.invoke_hot(Callback::Render, draw, LuaBudgetKind::Render)
  }

  pub fn take_host_commands(&mut self) -> Vec<LuaHostCommand> {
    let mut state = self.api_state.borrow_mut();
    let mut host = Vec::new();
    state.commands.retain(|command| {
      if matches!(command, LuaHostCommand::Draw(_)) {
        true
      } else {
        host.push(command.clone());
        false
      }
    });
    host
  }

  pub fn take_draw_commands(&mut self) -> Vec<LuaDrawCommand> {
    let mut state = self.api_state.borrow_mut();
    let mut draw = Vec::new();
    state.commands.retain(|command| {
      if let LuaHostCommand::Draw(command) = command {
        draw.push(command.clone());
        false
      } else {
        true
      }
    });
    state.draw_command_count = 0;
    state.draw_text_bytes = 0;
    draw
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
    self.state = LuaSessionState::Stopped;
    for (_, callback) in self.event_callbacks.drain() {
      let _ = self.lua.remove_registry_value(callback.key);
    }
    self.objects.borrow_mut().take();
    let _ = self.lua.gc_collect();
  }

  fn context_table(
    &self,
    continue_data: Option<&JsonValue>,
    best_data: Option<&JsonValue>,
  ) -> Result<Table, LuaSessionError> {
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
    let best_value = match best_data {
      Some(value) => json_to_lua(&self.lua, value)
        .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?,
      None => Value::Nil,
    };
    context
      .set("best_data", best_value)
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
        self.mark_faulted();
        return Err(error);
      }
    };
    let budget = self.policy.budget(LuaBudgetKind::HandleEvent);
    let outcome = run_with_budget(
      &self.lua,
      function,
      event_table,
      budget,
      self.policy.hook_interval,
      &self.api_state,
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
        self.record_slow_callback("EventCallback", budget, stats);
        if let Err(error) = self.lua.gc_step() {
          self.mark_faulted();
          return Err(self.error(LuaErrorStage::EventCallback, Some("EventCallback"), error));
        }
        Ok(())
      }
      Err(failure) => {
        self.mark_faulted();
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
      self.mark_faulted();
    }
    result
  }

  fn mark_faulted(&mut self) {
    self.state = LuaSessionState::Faulted;
    self.objects.take();
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
    {
      let mut api = self.api_state.borrow_mut();
      api.phase = callback.phase();
    }
    let budget = self.policy.budget(budget_kind);
    let outcome = run_with_budget(
      &self.lua,
      function,
      args,
      budget,
      self.policy.hook_interval,
      &self.api_state,
    );
    self.api_state.borrow_mut().phase = LuaCallPhase::Idle;
    match outcome {
      Ok((_, stats)) => {
        self.last_stats = LuaExecutionStats {
          memory_bytes: self.lua.used_memory(),
          ..stats
        };
        self.record_slow_callback(callback.name(), budget, stats);
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
    self.api_state.borrow_mut().phase = callback.phase();
    let budget = self.policy.budget(LuaBudgetKind::Save);
    let outcome = run_with_budget(
      &self.lua,
      function,
      (),
      budget,
      self.policy.hook_interval,
      &self.api_state,
    );
    self.api_state.borrow_mut().phase = LuaCallPhase::Idle;
    let (values, stats) = outcome
      .map_err(|failure| self.execution_error(LuaErrorStage::Callback, callback.name(), failure))?;
    self.last_stats = LuaExecutionStats {
      memory_bytes: self.lua.used_memory(),
      ..stats
    };
    self.record_slow_callback(callback.name(), budget, stats);
    self
      .lua
      .gc_step()
      .map_err(|error| self.error(LuaErrorStage::Callback, Some(callback.name()), error))?;

    let value = values.into_iter().next().unwrap_or(Value::Nil);
    if matches!(value, Value::Nil) {
      return Err(self.error(
        LuaErrorStage::SaveValidation,
        Some(callback.name()),
        "callback must return a serializable value",
      ));
    }
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
    if callback == Callback::SaveBest {
      let best_string = json
        .as_object()
        .and_then(|value| value.get("best_string"))
        .and_then(JsonValue::as_str);
      if best_string.is_none() {
        return Err(self.error(
          LuaErrorStage::SaveValidation,
          Some(callback.name()),
          "callback must return a table containing string field 'best_string'",
        ));
      }
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
        let error_stage = if is_memory_error(&error) {
          LuaErrorStage::MemoryLimit
        } else {
          stage
        };
        (error_stage, error.to_string())
      }
      LuaExecutionFailure::Limit {
        kind,
        instructions,
        instruction_limit,
        elapsed,
        duration_limit,
      } => (
        LuaErrorStage::ExecutionLimit,
        format_execution_limit(
          kind,
          instructions,
          instruction_limit,
          elapsed,
          duration_limit,
        ),
      ),
    };
    self.error(actual_stage, Some(callback), message)
  }

  fn record_slow_callback(
    &mut self,
    callback: &'static str,
    budget: LuaExecutionBudget,
    stats: LuaExecutionStats,
  ) {
    self.record_slow_callback_at(callback, budget, stats, Instant::now());
  }

  fn record_slow_callback_at(
    &mut self,
    callback: &'static str,
    budget: LuaExecutionBudget,
    stats: LuaExecutionStats,
    now: Instant,
  ) {
    const LOG_INTERVAL: Duration = Duration::from_secs(5);

    if stats.elapsed <= budget.warn_duration || !self.api_state.borrow().context.debug_enabled {
      return;
    }
    let warning = self
      .slow_callback_warnings
      .entry(callback)
      .or_insert(SlowCallbackWarning {
        last_logged: now.checked_sub(LOG_INTERVAL).unwrap_or(now),
        suppressed: 0,
      });
    if now.duration_since(warning.last_logged) < LOG_INTERVAL {
      warning.suppressed = warning.suppressed.saturating_add(1);
      return;
    }
    let suppressed = warning.suppressed;
    warning.last_logged = now;
    warning.suppressed = 0;
    let message = format!(
      "slow Lua callback: callback={callback}; elapsed_ms={:.3}; warn_ms={:.3}; hard_ms={:.3}; instructions={}; instruction_limit={}; suppressed={suppressed}",
      duration_millis(stats.elapsed),
      duration_millis(budget.warn_duration),
      duration_millis(budget.hard_duration),
      stats.instructions,
      budget.max_instructions,
    );
    self
      .api_state
      .borrow_mut()
      .commands
      .push(LuaHostCommand::Log {
        level: "warn".to_string(),
        message,
      });
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

#[derive(Clone, Copy, PartialEq, Eq)]
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

  fn phase(self) -> LuaCallPhase {
    match self {
      Self::Init => LuaCallPhase::Init,
      Self::HandleEvent => LuaCallPhase::Event,
      Self::Update => LuaCallPhase::Update,
      Self::UpdateFrame => LuaCallPhase::UpdateFrame,
      Self::Render => LuaCallPhase::Render,
      Self::SaveGame | Self::SaveBest => LuaCallPhase::Save,
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
  validate_json_depth(value, 0, policy.save_max_depth, "continue data").map_err(|message| {
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

fn validate_best_data(spec: &LuaSessionSpec, policy: &LuaPolicy) -> Result<(), LuaSessionError> {
  let Some(value) = spec.best_data.as_ref() else {
    return Ok(());
  };
  validate_json_depth(value, 0, policy.save_max_depth, "best data").map_err(|message| {
    session_error(
      spec,
      LuaErrorStage::BestDataValidation,
      Some("Init"),
      message,
    )
  })?;
  let encoded = serde_json::to_vec(value)
    .map_err(|error| session_error(spec, LuaErrorStage::BestDataValidation, Some("Init"), error))?;
  if encoded.len() > policy.save_limit_bytes {
    return Err(session_error(
      spec,
      LuaErrorStage::BestDataValidation,
      Some("Init"),
      format!(
        "best data is {} bytes; limit is {} bytes",
        encoded.len(),
        policy.save_limit_bytes
      ),
    ));
  }
  Ok(())
}

fn validate_json_depth(
  value: &JsonValue,
  depth: usize,
  max_depth: usize,
  label: &str,
) -> Result<(), String> {
  if depth > max_depth {
    return Err(format!("{label} exceeds maximum depth {max_depth}"));
  }
  match value {
    JsonValue::Array(values) => {
      for value in values {
        validate_json_depth(value, depth + 1, max_depth, label)?;
      }
    }
    JsonValue::Object(values) => {
      for value in values.values() {
        validate_json_depth(value, depth + 1, max_depth, label)?;
      }
    }
    _ => {}
  }
  Ok(())
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
    save_game: if spec.session_kind != LuaSessionKind::Game {
      None
    } else if spec.save_game_enabled {
      Some(required("SaveGame")?)
    } else {
      optional("SaveGame")?
    },
    save_best: if spec.session_kind != LuaSessionKind::Game {
      None
    } else if spec.save_best_enabled {
      Some(required("SaveBest")?)
    } else {
      optional("SaveBest")?
    },
  })
}

enum LuaExecutionFailure {
  Lua(mlua::Error),
  Limit {
    kind: LuaExecutionLimitKind,
    instructions: u64,
    instruction_limit: u64,
    elapsed: Duration,
    duration_limit: Duration,
  },
}

fn run_with_budget<A>(
  lua: &Lua,
  function: Function,
  args: A,
  budget: LuaExecutionBudget,
  hook_interval: u32,
  api_state: &SharedApiState,
) -> Result<(MultiValue, LuaExecutionStats), LuaExecutionFailure>
where
  A: IntoLuaMulti,
{
  {
    let mut api = api_state.borrow_mut();
    api.fatal_budget_exceeded = false;
    api.fatal_api_error = false;
  }
  let thread = lua
    .create_thread(function)
    .map_err(LuaExecutionFailure::Lua)?;
  let started = Instant::now();
  let instructions = Rc::new(Cell::new(0_u64));
  let exceeded = Rc::new(Cell::new(None));
  let hook_instructions = Rc::clone(&instructions);
  let hook_exceeded = Rc::clone(&exceeded);
  let hook_api_state = api_state.clone();
  thread
    .set_hook(
      HookTriggers::new().every_nth_instruction(hook_interval),
      move |_, _| {
        let current = hook_instructions
          .get()
          .saturating_add(u64::from(hook_interval));
        hook_instructions.set(current);
        let limit = if current > budget.max_instructions {
          Some(LuaExecutionLimitKind::Instructions)
        } else if started.elapsed() > budget.hard_duration {
          Some(LuaExecutionLimitKind::Time)
        } else {
          None
        };
        if let Some(limit) = limit {
          hook_exceeded.set(Some(limit));
          hook_api_state.borrow_mut().fatal_budget_exceeded = true;
          Err(mlua::Error::RuntimeError(format!(
            "Lua {} execution limit exceeded",
            limit.as_str()
          )))
        } else {
          Ok(VmState::Continue)
        }
      },
    )
    .map_err(LuaExecutionFailure::Lua)?;

  let result = thread.resume::<MultiValue>(args);
  thread.remove_hook();
  let elapsed = started.elapsed();
  let values = match result {
    Err(error) if is_memory_error(&error) => return Err(LuaExecutionFailure::Lua(error)),
    Err(_) if exceeded.get().is_some() || api_state.borrow().fatal_budget_exceeded => {
      return Err(LuaExecutionFailure::Limit {
        kind: exceeded
          .get()
          .unwrap_or(LuaExecutionLimitKind::Instructions),
        instructions: instructions.get(),
        instruction_limit: budget.max_instructions,
        elapsed,
        duration_limit: budget.hard_duration,
      });
    }
    Err(_) if elapsed > budget.hard_duration => {
      return Err(LuaExecutionFailure::Limit {
        kind: LuaExecutionLimitKind::Time,
        instructions: instructions.get(),
        instruction_limit: budget.max_instructions,
        elapsed,
        duration_limit: budget.hard_duration,
      });
    }
    Err(error) => return Err(LuaExecutionFailure::Lua(error)),
    Ok(values) => values,
  };
  if let Some(kind) = exceeded.get() {
    return Err(LuaExecutionFailure::Limit {
      kind,
      instructions: instructions.get(),
      instruction_limit: budget.max_instructions,
      elapsed,
      duration_limit: budget.hard_duration,
    });
  }
  if instructions.get() > budget.max_instructions {
    return Err(LuaExecutionFailure::Limit {
      kind: LuaExecutionLimitKind::Instructions,
      instructions: instructions.get(),
      instruction_limit: budget.max_instructions,
      elapsed,
      duration_limit: budget.hard_duration,
    });
  }
  if elapsed > budget.hard_duration {
    return Err(LuaExecutionFailure::Limit {
      kind: LuaExecutionLimitKind::Time,
      instructions: instructions.get(),
      instruction_limit: budget.max_instructions,
      elapsed,
      duration_limit: budget.hard_duration,
    });
  }
  if !thread.is_finished() {
    return Err(LuaExecutionFailure::Lua(mlua::Error::RuntimeError(
      "Lua callback yielded before completion".to_string(),
    )));
  }
  if api_state.borrow().fatal_api_error {
    return Err(LuaExecutionFailure::Lua(mlua::Error::RuntimeError(
      "fatal Lua API resource limit exceeded".to_string(),
    )));
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

fn is_memory_error(error: &mlua::Error) -> bool {
  match error {
    mlua::Error::MemoryError(_) => true,
    mlua::Error::BadArgument { cause, .. } | mlua::Error::CallbackError { cause, .. } => {
      is_memory_error(cause)
    }
    _ => false,
  }
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
      let stage = if is_memory_error(&error) {
        LuaErrorStage::MemoryLimit
      } else {
        stage
      };
      session_error(spec, stage, callback, error)
    }
    LuaExecutionFailure::Limit {
      kind,
      instructions,
      instruction_limit,
      elapsed,
      duration_limit,
    } => session_error(
      spec,
      LuaErrorStage::ExecutionLimit,
      callback,
      format_execution_limit(
        kind,
        instructions,
        instruction_limit,
        elapsed,
        duration_limit,
      ),
    ),
  }
}

fn duration_millis(duration: Duration) -> f64 {
  duration.as_secs_f64() * 1_000.0
}

fn format_execution_limit(
  kind: LuaExecutionLimitKind,
  instructions: u64,
  instruction_limit: u64,
  elapsed: Duration,
  duration_limit: Duration,
) -> String {
  format!(
    "{} execution limit exceeded: elapsed_ms={:.3}; time_limit_ms={:.3}; instructions={instructions}; instruction_limit={instruction_limit}",
    kind.as_str(),
    duration_millis(elapsed),
    duration_millis(duration_limit),
  )
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

  use crate::host_engine::services::{LuaFileOperation, SliceId};

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
      best_data: None,
      save_game_enabled: false,
      save_best_enabled: false,
    }
  }

  fn valid_script(extra: &str) -> String {
    format!(
      r##"
        function Init(ctx) init_ctx = ctx end
        function HandleEvent(event) last_event = event end
        function Update(dt) last_update = dt end
        function UpdateFrame(dt, alpha) last_frame = {{ dt, alpha }} end
        function Render(draw) last_draw = draw end
        {extra}
      "##
    )
  }

  #[test]
  fn creates_isolated_game_and_screensaver_vms() {
    let mut first = LuaSession::load(
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
    assert_eq!(first.environment_value("_G"), Value::Nil);
    assert_eq!(second.environment_value("_G"), Value::Nil);

    let first_pool_id = first.with_objects(|objects| objects.ui().id()).unwrap();
    let second_pool_id = second.with_objects(|objects| objects.ui().id()).unwrap();
    assert_ne!(first_pool_id, second_pool_id);

    let time = crate::host_engine::services::TimeService::new();
    let timer = first
      .with_objects_mut(|objects| time.create_count_up(objects.runtime_mut()))
      .unwrap();
    assert_eq!(
      first
        .with_objects(|objects| time.state(objects.runtime(), timer))
        .flatten(),
      Some(crate::host_engine::services::TimerState::Idle)
    );
    assert_eq!(
      second
        .with_objects(|objects| time.state(objects.runtime(), timer))
        .flatten(),
      None
    );

    first.stop();
    assert!(!first.has_objects());
  }

  #[test]
  fn sandbox_exposes_only_host_libraries_and_hides_native_globals() {
    let session = LuaSession::load(
      spec(&valid_script(""), LuaSessionKind::Game),
      LuaPolicy::default(),
    )
    .unwrap();
    for name in [
      "base",
      "math",
      "string",
      "utf8",
      "table",
      "align",
      "char",
      "color",
      "measurement",
      "draw",
      "debug",
      "game",
      "event",
      "loader",
      "file",
      "random",
      "slice",
      "serialization",
      "encoding",
      "ipairs",
      "pairs",
      "next",
      "select",
      "rawequal",
      "rawlen",
      "tonumber",
      "tostring",
      "type",
    ] {
      assert_ne!(session.environment_value(name), Value::Nil, "{name}");
    }
    for name in [
      "_G",
      "_VERSION",
      "assert",
      "error",
      "pcall",
      "xpcall",
      "load",
      "loadfile",
      "loadstring",
      "dofile",
      "require",
      "rawget",
      "rawset",
      "os",
      "io",
      "package",
      "coroutine",
    ] {
      assert_eq!(session.environment_value(name), Value::Nil, "{name}");
    }
  }

  #[test]
  fn api_tables_are_read_only_and_iterators_do_not_expose_backing_tables() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local ok = debug.pcall{ func = function() math.PI = 0 end }
          debug.assert{ value = not ok.ok, message = "math must be read-only" }
          local iterator = pairs(math)
          local item = iterator()
          debug.assert{ value = type{ value = item } == "table" }
          debug.assert{ value = item.index ~= nil and item.value ~= nil }
          local count = 0
          for pair in pairs(math) do
            debug.assert{ value = pair.index ~= nil and pair.value ~= nil }
            count = count + 1
          end
          debug.assert{ value = count > 0 }
          local first = next{ table = math, index = nil }
          debug.assert{ value = first.index ~= nil and first.value ~= nil }
          debug.assert{ value = math.PI > 3, message = "iterator leaked backing table" }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn align_resolve_rect_returns_a_named_coordinate_table() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local top_left, extra = align.resolve_rect{
            width = 10,
            height = 4,
            horizontal_align = align.LEFT,
            vertical_align = align.TOP,
          }
          local center = align.resolve_rect{
            width = 10,
            height = 4,
            horizontal_align = align.CENTER,
            vertical_align = align.CENTER,
          }
          local bottom_right = align.resolve_rect{
            width = 10,
            height = 4,
            horizontal_align = align.RIGHT,
            vertical_align = align.BOTTOM,
          }
          debug.assert{ value = type{ value = top_left } == "table" }
          debug.assert{ value = top_left.x == 0 and top_left.y == 0 }
          debug.assert{ value = center.x == 55 and center.y == 17 }
          debug.assert{ value = bottom_right.x == 110 and bottom_right.y == 34 }
          debug.assert{ value = extra == nil }
        end
      "#,
    );
    let mut session_spec = spec(&source, LuaSessionKind::Game);
    session_spec.terminal_size = Size {
      width: 120,
      height: 38,
    };
    LuaSession::load(session_spec, LuaPolicy::default()).unwrap();
  }

  #[test]
  fn lua_text_modes_share_the_expected_measurement_semantics() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local text = "f%<fg:green>Test"
          local plain = measurement.get_text_width{
            text = text,
            text_mode = string.PLAIN_TEXT,
          }
          local rich = measurement.get_text_width{
            text = text,
            text_mode = string.RICH_TEXT,
          }
          local auto = measurement.get_text_width{
            text = text,
            text_mode = string.AUTO,
          }
          debug.assert{ value = plain == 16, message = "plain mode must preserve all syntax" }
          debug.assert{ value = rich == 6, message = "rich mode must preserve the f% prefix" }
          debug.assert{ value = auto == 4, message = "auto mode must consume the f% prefix" }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn text_parsing_only_attaches_the_key_map_requested_by_rich_text() {
    let source = valid_script(
      r#"
        function Init(ctx)
          debug.assert{
            value = measurement.get_text_width{ text = "f%{key:jump}" } == 3,
          }
          debug.assert{
            value = measurement.get_text_width{ text = "f%{key_default:jump}" } == 4,
          }
        end
        function Render(draw_context)
          draw.text{ x = 1, y = 1, text = "ordinary" }
          draw.text{ x = 1, y = 2, text = "f%{key:jump}" }
          draw.text{ x = 1, y = 3, text = "f%{key_default:jump}" }
          draw.text{
            x = 1,
            y = 4,
            text = "f%{value:name}",
            rich_params = { name = "TUI" },
          }
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        key_actions: HashMap::from([("jump".to_string(), vec![vec!["1".to_string()]])]),
        key_default_actions: HashMap::from([("jump".to_string(), vec![vec!["f1".to_string()]])]),
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    session
      .render(Size {
        width: 120,
        height: 40,
      })
      .unwrap();
    let commands = session.take_draw_commands();
    let params = commands
      .iter()
      .filter_map(|command| match command {
        LuaDrawCommand::Text { params, .. } => Some(params),
        _ => None,
      })
      .collect::<Vec<_>>();
    assert_eq!(params.len(), 4);
    assert!(params[0].params.is_none());
    let user = params[1].params.as_ref().unwrap();
    assert!(user.key_actions.contains_key("jump"));
    assert!(user.key_default_actions.is_empty());
    let default = params[2].params.as_ref().unwrap();
    assert!(default.key_actions.is_empty());
    assert!(default.key_default_actions.contains_key("jump"));
    let values = params[3].params.as_ref().unwrap();
    assert_eq!(values.values.get("name").map(String::as_str), Some("TUI"));
    assert!(values.key_actions.is_empty());
    assert!(values.key_default_actions.is_empty());
  }

  #[test]
  fn random_slice_serialization_and_encoding_libraries_work_together() {
    let source = valid_script(
      r##"
        local slice_id

        function Init(ctx)
          local first = random.create{
            type = random.INT,
            min = -10,
            max = 10,
            seed = 2468,
          }
          local second = random.create{
            type = random.INT,
            min = -10,
            max = 10,
            seed = 2468,
          }
          debug.assert{ value = random.generate(first) == random.generate(second) }
          debug.assert{ value = random.generate(first) == random.generate(second) }

          slice_id = slice.create{ width = 20, height = slice["50P"], layer = 3 }
          slice.draw{ id = slice_id, x = -4, y = 2 }
          local info = slice.get_info(slice_id)
          debug.assert{
            value = info.width == 20 and info.height == 20 and info.layer == 3,
          }

          local json = serialization.json_encode{
            title = "TUI GAME",
            enabled = true,
            values = { 1, 2, 3 },
          }
          local decoded = serialization.json_decode(json)
          debug.assert{
            value = decoded.title == "TUI GAME" and decoded.enabled
              and decoded.values[3] == 3,
          }

          local packed = serialization.binary_pack{
            fmt = "<i2c3",
            values = { 513, "abc" },
          }
          local number, text, next_position = serialization.binary_unpack{
            fmt = "<i2c3",
            s = packed,
          }
          debug.assert{
            value = number == 513 and text == "abc" and next_position == 6,
          }

          local encoded = encoding.base64_encode("TUI GAME")
          debug.assert{ value = encoding.base64_decode(encoded) == "TUI GAME" }
          debug.assert{ value = encoding.url_decode(encoding.url_encode("a b/中")) == "a b/中" }
          debug.assert{ value = encoding.hex_decode(encoding.hex_encode("abc")) == "abc" }
        end

        function Render(draw)
          draw.fill_rect{
            x = -2,
            y = 1,
            width = 5,
            height = 2,
            char = "#",
            slice_layer = slice_id,
          }
        end
      "##,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    session
      .render(Size {
        width: 120,
        height: 40,
      })
      .unwrap();

    let commands = session.take_draw_commands();
    assert!(commands.iter().any(|command| matches!(
      command,
      LuaDrawCommand::FillRect {
        target: super::super::LuaDrawTarget::Slice(SliceId(1)),
        x: -2,
        y: 1,
        width: 5,
        height: 2,
        ..
      }
    )));
  }

  #[test]
  fn serialization_rejects_cycles_sparse_tables_entities_and_malformed_encoding() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local cyclic = {}
          cyclic.self = cyclic
          local cyclic_ok = debug.pcall{
            func = function() serialization.json_encode(cyclic) end,
          }
          local sparse_ok = debug.pcall{
            func = function() serialization.json_encode{ [1] = "a", [3] = "c" } end,
          }
          local entity_ok = debug.pcall{
            func = function()
              serialization.xml_decode(
                "<!DOCTYPE root [<!ENTITY secret 'hidden'>]><root>&secret;</root>"
              )
            end,
          }
          local hex_ok = debug.pcall{
            func = function() encoding.hex_decode("0xz1") end,
          }
          debug.assert{
            value = not cyclic_ok.ok and not sparse_ok.ok and not entity_ok.ok and not hex_ok.ok,
          }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn drawing_is_allowed_in_all_callbacks_but_render_requests_are_not_reentrant() {
    let source = valid_script(
      r#"
        function Init(ctx)
          draw.fill_rect{ x = 2, y = 1, width = 10, height = 4, bg = color.BLUE }
          draw.render()
        end
        function Update(dt)
          draw.erase_rect{ x = 3, y = 2, width = 8, height = 2 }
        end
        function Render(draw_context)
          local ok = debug.pcall{ func = function() draw.render() end }
          debug.assert{ value = not ok.ok }
          draw.text{ x = 1, y = 1, text = "render" }
        end
      "#,
    );
    let mut session = LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default())
      .expect("drawing during Init must be accepted");
    session
      .update()
      .expect("drawing during Update must be accepted");
    session
      .render(Size {
        width: 120,
        height: 40,
      })
      .expect("ordinary drawing during Render must remain valid");

    let commands = session.take_draw_commands();
    assert!(matches!(
      commands.first(),
      Some(LuaDrawCommand::FillRect { .. })
    ));
    assert!(matches!(
      commands.get(1),
      Some(LuaDrawCommand::EraseRect { .. })
    ));
    assert!(matches!(commands.get(2), Some(LuaDrawCommand::Text { .. })));
  }

  #[test]
  fn draw_limit_is_reset_when_the_host_finishes_each_frame() {
    let source = valid_script(
      r#"
        function Update(dt)
          local x = 0
          local y = 0
          for item in ipairs(char.ASCII_LETTER) do
            x = x + 2
            if x % 20 == 0 then
              x = 2
              y = y + 1
            end
            draw.text{ x = x, y = y, text = item.value }
          end
        end
      "#,
    );
    let mut session = LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default())
      .expect("the drawing fixture must load");

    for _ in 0..100 {
      session
        .update()
        .expect("one frame must stay below the limit");
      assert_eq!(session.take_draw_commands().len(), 52);
    }
  }

  #[test]
  fn debug_print_uses_header_free_defaults() {
    let source = valid_script(
      r#"
        function Init(ctx)
          debug.print{ message = "plain" }
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    assert!(session.take_host_commands().iter().any(|command| matches!(
      command,
      LuaHostCommand::Print {
        message,
        title: None,
        time: false,
        level: None,
        type_head: false,
      } if message == "plain"
    )));
  }

  #[test]
  fn debug_print_constants_and_convenience_methods_use_the_standard_options() {
    let source = valid_script(
      r#"
        function Init(ctx)
          debug.assert{
            value = debug.VERSION == "Lua 5.4 / TUI GAME API 1"
              and debug.TRACE == "trace"
              and debug.DEBUG == "debug"
              and debug.INFO == "info"
              and debug.WARN == "warn"
              and debug.ERROR == "error"
              and debug.FATAL == "fatal",
          }
          debug.print{
            message = "custom",
            title = "Title",
            level = debug.WARN,
            time = true,
            type_head = true,
          }
          debug.info("info")
          debug.warn{ message = "warn" }
          debug.error("error")
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    let commands = session.take_host_commands();
    assert!(commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::Print {
        message,
        title: Some(title),
        time: true,
        level: Some(level),
        type_head: true,
      } if message == "custom" && title == "Title" && level == "warn"
    )));
    for (expected_message, expected_level) in
      [("info", "info"), ("warn", "warn"), ("error", "error")]
    {
      assert!(commands.iter().any(|command| matches!(
        command,
        LuaHostCommand::Print {
          message,
          title: None,
          time: true,
          level: Some(level),
          type_head: true,
        } if message == expected_message && level == expected_level
      )));
    }
  }

  #[test]
  fn protected_calls_return_named_result_tables() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local success = debug.pcall{
            func = function(left, right)
              return left + right, nil, "tail"
            end,
            values = { 2, 3 },
          }
          debug.assert{
            value = success.ok
              and type(success.values) == "table"
              and success.values.n == 3
              and success.values[1] == 5
              and success.values[2] == nil
              and success.values[3] == "tail"
              and success.error == nil,
          }

          local failure = debug.pcall{
            func = function()
              debug.assert{ value = false, message = "failed" }
            end,
          }
          debug.assert{
            value = not failure.ok
              and type(failure.error) == "string"
              and failure.values == nil,
          }

          local obsolete_message = debug.pcall{
            func = function()
              debug.pcall{ func = function() end, message = "removed" }
            end,
          }
          debug.assert{
            value = not obsolete_message.ok and type(obsolete_message.error) == "string",
          }

          local handled = debug.xpcall{
            func = function()
              debug.assert{ value = false, message = "failed" }
            end,
            error_callback = function(message)
              return "handled"
            end,
          }
          debug.assert{
            value = not handled.ok and handled.error == "handled",
          }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn api_rejects_invalid_math_domains_and_excessively_deep_parameters() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local sqrt_ok = debug.pcall{ func = function() math.sqrt(-1) end }
          local pow_ok = debug.pcall{ func = function() math.pow{ left = -1, right = 0.5 } end }
          local value = {}
          local cursor = value
          for index = 1, 33 do
            cursor.child = {}
            cursor = cursor.child
          end
          local depth_ok = debug.pcall{ func = function() base.type(value) end }
          local cyclic = {}
          cyclic.self = cyclic
          local cycle_ok = debug.pcall{ func = function() base.type(cyclic) end }
          local unknown_ok = debug.pcall{
            func = function()
              measurement.get_text_width{ text = "value", unknown = true }
            end,
          }
          local packed = table.pack{ values = { [1] = "first", n = 3 } }
          debug.assert{
            value = not sqrt_ok.ok and not pow_ok.ok and not depth_ok.ok and not cycle_ok.ok
              and not unknown_ok.ok
              and packed.n == 3 and packed[1] == "first" and packed[3] == nil,
          }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn save_callbacks_cannot_enqueue_recursive_save_commands() {
    let source = valid_script(
      r#"
        function SaveGame()
          game.save_game()
          return { saved = true }
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    assert_eq!(session.save_game().unwrap().unwrap()["saved"], true);
    let commands = session.take_host_commands();
    assert!(commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::Ignored {
        method: "game.save_game",
        ..
      }
    )));
    assert!(
      !commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::SaveGame))
    );
  }

  #[test]
  fn restricted_calls_are_ignored_before_parameter_validation() {
    let source = valid_script(
      r#"
        function Init(ctx)
          game.exit_game("ignored")
          event.clear_action("ignored")
          file.write("ignored")
          file.list_dir("ignored")
          debug.info({ invalid = true })
        end
      "#,
    );
    let mut session = LuaSession::load(
      spec(&source, LuaSessionKind::Screensaver),
      LuaPolicy::default(),
    )
    .unwrap();
    let commands = session.take_host_commands();
    for method in [
      "game.exit_game",
      "event.clear_action",
      "file.write",
      "file.list_dir",
      "debug.info",
    ] {
      assert!(
        commands.iter().any(|command| matches!(
          command,
          LuaHostCommand::Ignored { method: found, .. } if *found == method
        )),
        "missing ignored command for {method}"
      );
    }
  }

  #[test]
  fn api_configuration_is_active_during_entry_and_init() {
    let source = valid_script(
      r#"
        debug.info("entry")
        function Init(ctx)
          debug.info("init")
          event.clear_action()
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        safe_mode_enabled: false,
        key_actions: HashMap::new(),
        key_default_actions: HashMap::new(),
      },
    )
    .unwrap();
    let commands = session.take_host_commands();
    assert_eq!(
      commands
        .iter()
        .filter(|command| matches!(command, LuaHostCommand::Print { .. }))
        .count(),
      2
    );
    assert!(
      commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::ClearActions))
    );
  }

  #[test]
  fn event_action_controls_require_an_unrestricted_game_session() {
    let source = valid_script(
      r#"
        function Init(ctx)
          event.skip_action()
          event.clear_action()
        end
      "#,
    );
    let load = |session_kind, safe_mode_enabled| {
      let mut session = LuaSession::load_with_api(
        spec(&source, session_kind),
        LuaPolicy::default(),
        LuaApiConfig {
          safe_mode_enabled,
          ..LuaApiConfig::default()
        },
      )
      .unwrap();
      session.take_host_commands()
    };

    let permitted = load(LuaSessionKind::Game, false);
    assert!(
      permitted
        .iter()
        .any(|command| matches!(command, LuaHostCommand::SkipActions))
    );
    assert!(
      permitted
        .iter()
        .any(|command| matches!(command, LuaHostCommand::ClearActions))
    );

    for commands in [
      load(LuaSessionKind::Game, true),
      load(LuaSessionKind::Screensaver, false),
      load(LuaSessionKind::Screensaver, true),
    ] {
      assert!(!commands.iter().any(|command| matches!(
        command,
        LuaHostCommand::SkipActions | LuaHostCommand::ClearActions
      )));
      for method in ["event.skip_action", "event.clear_action"] {
        assert!(commands.iter().any(|command| matches!(
          command,
          LuaHostCommand::Ignored { method: found, .. } if *found == method
        )));
      }
    }
  }

  #[test]
  fn file_apis_share_current_directory_and_parent_traversal_rules() {
    let id = TEST_ID.fetch_add(1, Ordering::Relaxed);
    let package_root = std::env::temp_dir().join(format!(
      "tui_game_lua_assets_root_{}_{}",
      std::process::id(),
      id
    ));
    let scripts_root = package_root.join("scripts");
    let assets_root = package_root.join("assets");
    fs::create_dir_all(&scripts_root).unwrap();
    fs::create_dir_all(&assets_root).unwrap();
    fs::write(assets_root.join("input.txt"), "input").unwrap();
    let entry_path = scripts_root.join("main.lua");
    fs::write(
      &entry_path,
      valid_script(
        r#"
          function Init(ctx)
            file.list_dir{ path = ".", recursive = true }
            file.read{ path = "./input.txt" }
            file.write{ path = "./output.txt", text = "output" }
            local empty = debug.pcall{
              func = function() file.list_dir{ path = "" } end,
            }
            local traversal = debug.pcall{
              func = function() file.list_dir{ path = "./folder/../" } end,
            }
            debug.assert{ value = not empty.ok and not traversal.ok }
          end
        "#,
      ),
    )
    .unwrap();
    let mut session = LuaSession::load_with_api(
      LuaSessionSpec {
        package_id: "test.assets_root".to_string(),
        session_kind: LuaSessionKind::Game,
        entry_path,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        terminal_size: Size {
          width: 120,
          height: 40,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
      },
      LuaPolicy::default(),
      LuaApiConfig {
        safe_mode_enabled: false,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();

    let expected_root = assets_root.canonicalize().unwrap();
    let requests = session.take_host_commands();
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaListDir { path, recursive: true, .. },
        virtual_path,
        ..
      } if path == &expected_root && virtual_path == "."
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaReadText { path, .. },
        virtual_path,
        ..
      } if path == &expected_root.join("input.txt") && virtual_path == "input.txt"
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaWriteText { path, .. },
        virtual_path,
        ..
      } if path == &expected_root.join("output.txt") && virtual_path == "output.txt"
    )));

    fs::remove_dir_all(package_root).unwrap();
  }

  #[test]
  fn safe_mode_lab_exercises_debug_logging_and_file_write_permissions() {
    let package_root =
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_package/game/safe_mode_lab");
    let entry_path = package_root.join("scripts/main.lua");
    let make_spec = || LuaSessionSpec {
      package_id: "test.safe_mode_lab".to_string(),
      session_kind: LuaSessionKind::Game,
      entry_path: entry_path.clone(),
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      terminal_size: Size {
        width: 120,
        height: 40,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: true,
      save_best_enabled: true,
    };
    let action = LuaRuntimeEvent {
      sequence: 1,
      frame: 1,
      data: LuaEventData::Action {
        action: "write_probe".to_string(),
        state: super::super::LuaActionState::Pressed,
      },
    };

    let mut restricted =
      LuaSession::load_with_api(make_spec(), LuaPolicy::default(), LuaApiConfig::default())
        .unwrap();
    restricted.handle_event(&action).unwrap();
    let restricted_commands = restricted.take_host_commands();
    assert!(restricted_commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::Ignored {
        method: "file.write",
        ..
      }
    )));
    assert!(
      !restricted_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::FileRequest { .. }))
    );

    let mut permitted = LuaSession::load_with_api(
      make_spec(),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        safe_mode_enabled: false,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    permitted.handle_event(&action).unwrap();
    let permitted_commands = permitted.take_host_commands();
    assert!(
      permitted_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::Print { .. }))
    );
    assert!(permitted_commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        operation: LuaFileOperation::WriteText,
        virtual_path,
        ..
      } if virtual_path == "state/probe.log"
    )));
  }

  #[test]
  fn loader_isolates_modules_and_rejects_unsafe_sources() {
    let source = valid_script(
      r#"
        private_state = "main-only"
        function Init(ctx)
          local first = loader.load("module")
          local second = loader.load{ path = "module.lua" }
          local third = loader.load("./module")
          debug.assert{ value = first.value == 1 and second.value == 1 and third.value == 1 }
          debug.assert{
            value = first.leaked == nil and second.leaked == nil and third.leaked == nil,
          }
          debug.assert{ value = loader.load_execute("value") == 42 }

          local traversal_ok = debug.pcall{
            func = function() loader.load("../outside") end,
          }
          local extension_ok = debug.pcall{
            func = function() loader.load("module.txt") end,
          }
          local bytecode_ok = debug.pcall{
            func = function() loader.load("bytecode.lua") end,
          }
          local cycle_ok = debug.pcall{
            func = function() loader.load("cycle") end,
          }
          debug.assert{
            value = not traversal_ok.ok and not extension_ok.ok and not bytecode_ok.ok
              and not cycle_ok.ok,
          }
        end
      "#,
    );
    let session_spec = spec(&source, LuaSessionKind::Game);
    let scripts_root = session_spec.entry_path.parent().unwrap();
    fs::write(
      scripts_root.join("module.lua"),
      "instance_count = (instance_count or 0) + 1\nreturn { value = instance_count, leaked = private_state }",
    )
    .unwrap();
    fs::write(scripts_root.join("value.lua"), "return 42").unwrap();
    fs::write(scripts_root.join("module.txt"), "return {}").unwrap();
    fs::write(scripts_root.join("bytecode.lua"), [0x1b, b'L', b'u', b'a']).unwrap();
    fs::write(
      scripts_root.join("cycle.lua"),
      "return loader.load('cycle')",
    )
    .unwrap();

    LuaSession::load(session_spec, LuaPolicy::default()).unwrap();
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
  fn enabled_save_callbacks_are_required_and_best_needs_best_string() {
    let source = valid_script("");
    let mut game_save_spec = spec(&source, LuaSessionKind::Game);
    game_save_spec.save_game_enabled = true;
    let error = LuaSession::load(game_save_spec, LuaPolicy::default())
      .err()
      .expect("SaveGame should be required");
    assert_eq!(error.stage, LuaErrorStage::DiscoverCallbacks);
    assert_eq!(error.callback, Some("SaveGame"));

    let source = valid_script("function SaveGame() return {} end");
    let mut best_spec = spec(&source, LuaSessionKind::Game);
    best_spec.save_game_enabled = true;
    best_spec.save_best_enabled = true;
    let error = LuaSession::load(best_spec, LuaPolicy::default())
      .err()
      .expect("SaveBest should be required");
    assert_eq!(error.stage, LuaErrorStage::DiscoverCallbacks);
    assert_eq!(error.callback, Some("SaveBest"));

    let source = valid_script(
      "function SaveGame() return {} end\nfunction SaveBest() return { score = 1 } end",
    );
    let mut invalid_best_spec = spec(&source, LuaSessionKind::Game);
    invalid_best_spec.save_game_enabled = true;
    invalid_best_spec.save_best_enabled = true;
    let mut session = LuaSession::load(invalid_best_spec, LuaPolicy::default()).unwrap();
    let error = session.save_best().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::SaveValidation);
    assert!(error.message.contains("best_string"));
  }

  #[test]
  fn instruction_budget_faults_an_infinite_update() {
    let source = valid_script("function Update(dt) while true do end end");
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert!(
      error
        .message
        .contains("instructions execution limit exceeded")
    );
    assert!(error.message.contains("instruction_limit=200000"));
    assert!(error.message.contains("time_limit_ms=75.000"));
    assert_eq!(session.state(), LuaSessionState::Faulted);
    assert!(!session.has_objects());
  }

  #[test]
  fn instruction_budget_cannot_be_hidden_by_pcall() {
    let source = valid_script(
      "function Update(dt) debug.pcall{ func = function() while true do end end } end",
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert_eq!(session.state(), LuaSessionState::Faulted);
  }

  fn install_test_sleep(session: &LuaSession, duration: Duration) {
    let sleep = session
      .lua
      .create_function(move |_, ()| {
        std::thread::sleep(duration);
        Ok(())
      })
      .unwrap();
    let environment: Table = session.lua.registry_value(&session.environment).unwrap();
    environment.set("sleep_for_test", sleep).unwrap();
  }

  #[test]
  fn rust_api_time_is_included_in_the_hard_callback_budget() {
    let source = valid_script("function Update(dt) sleep_for_test() end");
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    install_test_sleep(&session, Duration::from_millis(90));

    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert!(error.message.contains("time execution limit exceeded"));
    assert!(error.message.contains("time_limit_ms=75.000"));
    assert!(error.message.contains("instruction_limit=200000"));
  }

  #[test]
  fn slow_callback_warnings_require_debug_mode() {
    let source = valid_script("function Update(dt) sleep_for_test() end");
    let mut debug_session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    install_test_sleep(&debug_session, Duration::from_millis(25));
    debug_session.update().unwrap();
    assert!(
      debug_session
        .take_host_commands()
        .iter()
        .any(|command| matches!(
          command,
          LuaHostCommand::Log { level, message }
            if level == "warn" && message.contains("callback=Update")
              && message.contains("warn_ms=20.000") && message.contains("hard_ms=75.000")
        ))
    );

    let mut release_session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    install_test_sleep(&release_session, Duration::from_millis(25));
    release_session.update().unwrap();
    assert!(
      !release_session
        .take_host_commands()
        .iter()
        .any(|command| matches!(
          command,
          LuaHostCommand::Log { message, .. } if message.contains("slow Lua callback")
        ))
    );
  }

  #[test]
  fn slow_callback_warnings_are_rate_limited_per_callback() {
    let source = valid_script("");
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    let _ = session.take_host_commands();
    let budget = session.policy.budget(LuaBudgetKind::Render);
    let stats = LuaExecutionStats {
      instructions: 4_000,
      elapsed: Duration::from_millis(21),
      memory_bytes: session.memory_used(),
    };
    let now = Instant::now();
    session.record_slow_callback_at(
      "Update",
      budget,
      LuaExecutionStats {
        elapsed: Duration::from_millis(19),
        ..stats
      },
      now,
    );
    session.record_slow_callback_at("Render", budget, stats, now);
    session.record_slow_callback_at("Render", budget, stats, now + Duration::from_secs(1));
    session.record_slow_callback_at("Render", budget, stats, now + Duration::from_secs(5));

    let warnings = session
      .take_host_commands()
      .into_iter()
      .filter_map(|command| match command {
        LuaHostCommand::Log { message, .. } if message.contains("slow Lua callback") => {
          Some(message)
        }
        _ => None,
      })
      .collect::<Vec<_>>();
    assert_eq!(warnings.len(), 2);
    assert!(warnings[1].contains("suppressed=1"));
  }

  #[test]
  fn ascii_text_rendering_stays_within_the_callback_budget() {
    let source = valid_script(
      r#"
        function Render(draw_context)
          local x = 0
          local y = 0
          for item in ipairs(char.ASCII) do
            x = x + 1
            if x > 20 then
              x = 1
              y = y + 1
            end
            draw.text{ x = x, y = y, text = item.value }
          end
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    session
      .render(Size {
        width: 120,
        height: 40,
      })
      .unwrap();
    assert!(session.take_draw_commands().len() >= 90);
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
          continue_level = context.continue_data.level,
          best_score = context.best_data.score,
          best_string = context.best_data.best_string,
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
    let mut session_spec = spec(source, LuaSessionKind::Game);
    session_spec.continue_data = Some(serde_json::json!({"level": 4}));
    session_spec.best_data = Some(serde_json::json!({
      "best_string": "Best: 12",
      "score": 12
    }));
    let mut session = LuaSession::load(session_spec, LuaPolicy::default()).unwrap();
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
    assert_eq!(save["continue_level"], 4);
    assert_eq!(save["best_score"], 12);
    assert_eq!(save["best_string"], "Best: 12");
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
            best_data: None,
            save_game_enabled: session_kind == LuaSessionKind::Game,
            save_best_enabled: session_kind == LuaSessionKind::Game,
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

      assert_eq!(
        package_count, 3,
        "expected three test packages in {directory}"
      );
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
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
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
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
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
    let source = valid_script(
      "function Update(dt) local value = string.rep{ text = 'x', times = 1024 * 1024 } end",
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    session
      .lua
      .set_memory_limit(session.lua.used_memory() + 128 * 1024)
      .unwrap();
    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::MemoryLimit);
    assert_eq!(session.state(), LuaSessionState::Faulted);
  }
}
