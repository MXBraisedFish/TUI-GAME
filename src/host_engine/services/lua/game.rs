use std::time::Duration;

use serde_json::Value as JsonValue;

use crate::host_engine::services::{PackageSource, Size};

use super::{
  LuaDrawCommand, LuaEventDelivery, LuaExecutionStats, LuaHostCommand, LuaObjectPool, LuaSession,
  LuaSessionError, LuaSessionKind, LuaSessionState, LuaSessionToken,
};

const MAX_REAL_DELTA: Duration = Duration::from_millis(250);
const MAX_FIXED_UPDATES_PER_FRAME: usize = 8;

#[derive(Clone, Debug)]
pub struct LuaSessionDiagnostics {
  pub entry_path: std::path::PathBuf,
  pub stats: LuaExecutionStats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameSessionState {
  Inactive,
  Running,
  Faulted,
}

#[derive(Clone, Debug, Default)]
pub struct GameStopData {
  pub game: Option<JsonValue>,
  pub best: Option<JsonValue>,
  pub save_errors: Vec<LuaSessionError>,
}

/// 唯一游戏 Session 的宿主生命周期。
pub struct GameService {
  session: Option<LuaSession>,
  package_id: Option<String>,
  package_source: Option<PackageSource>,
  target_fps: Option<u32>,
  min_size: Size,
  accumulator: Duration,
  generation: u64,
}

impl GameService {
  pub fn new() -> Self {
    Self {
      session: None,
      package_id: None,
      package_source: None,
      target_fps: None,
      min_size: Size::default(),
      accumulator: Duration::ZERO,
      generation: 0,
    }
  }

  pub fn start(
    &mut self,
    session: LuaSession,
    source: PackageSource,
    target_fps: u32,
    min_size: Size,
  ) {
    self.stop(false);
    self.generation = self.generation.wrapping_add(1).max(1);
    self.package_id = Some(session.package_id().to_string());
    self.package_source = Some(source);
    self.target_fps = Some(target_fps);
    self.min_size = min_size;
    self.accumulator = Duration::ZERO;
    self.session = Some(session);
  }

  pub fn stop(&mut self, save: bool) -> GameStopData {
    let mut result = GameStopData::default();
    if let Some(mut session) = self.session.take() {
      if save && session.state() != LuaSessionState::Faulted {
        match session.save_game() {
          Ok(value) => result.game = value,
          Err(error) => result.save_errors.push(error),
        }
        match session.save_best() {
          Ok(value) => result.best = value,
          Err(error) => result.save_errors.push(error),
        }
      }
      session.stop();
    }
    self.package_id = None;
    self.package_source = None;
    self.target_fps = None;
    self.min_size = Size::default();
    self.accumulator = Duration::ZERO;
    result
  }

  pub fn state(&self) -> GameSessionState {
    match self.session.as_ref().map(LuaSession::state) {
      Some(LuaSessionState::Running) => GameSessionState::Running,
      Some(LuaSessionState::Faulted) => GameSessionState::Faulted,
      _ => GameSessionState::Inactive,
    }
  }

  pub fn is_active(&self) -> bool {
    self.session.is_some()
  }

  pub fn package_id(&self) -> Option<&str> {
    self.package_id.as_deref()
  }

  pub fn session_token(&self) -> Option<LuaSessionToken> {
    self.session.as_ref().map(|_| LuaSessionToken {
      kind: LuaSessionKind::Game,
      generation: self.generation,
    })
  }

  pub fn diagnostics(&self) -> Option<LuaSessionDiagnostics> {
    self.session.as_ref().map(|session| LuaSessionDiagnostics {
      entry_path: session.entry_path().to_path_buf(),
      stats: session.last_stats(),
    })
  }

  pub fn set_terminal_size(&mut self, size: Size) {
    if let Some(session) = self.session.as_mut() {
      session.set_terminal_size(size);
    }
  }

  pub fn package_source(&self) -> Option<&PackageSource> {
    self.package_source.as_ref()
  }

  pub fn objects(&self) -> Option<&LuaObjectPool> {
    self.session.as_ref().and_then(LuaSession::objects)
  }

  pub fn objects_mut(&mut self) -> Option<&mut LuaObjectPool> {
    self.session.as_mut().and_then(LuaSession::objects_mut)
  }

  pub fn target_fps(&self) -> Option<u32> {
    self.target_fps
  }

  pub fn min_size(&self) -> Size {
    self.min_size
  }

  pub fn dispatch_event(&mut self, delivery: &LuaEventDelivery) -> Result<(), LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(());
    };
    session.dispatch_event(delivery)
  }

  /// 执行固定 60 Hz 更新和一次帧更新，返回本帧固定更新次数。
  pub fn advance(&mut self, real_delta: Duration) -> Result<usize, LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(0);
    };
    let real_delta = real_delta.min(MAX_REAL_DELTA);
    let fixed_delta = Duration::from_secs_f64(1.0 / 60.0);
    self.accumulator = self.accumulator.saturating_add(real_delta);

    let mut updates = 0;
    while self.accumulator >= fixed_delta && updates < MAX_FIXED_UPDATES_PER_FRAME {
      session.update()?;
      self.accumulator = self.accumulator.saturating_sub(fixed_delta);
      updates += 1;
    }
    if updates == MAX_FIXED_UPDATES_PER_FRAME && self.accumulator >= fixed_delta {
      self.accumulator =
        Duration::from_secs_f64(self.accumulator.as_secs_f64() % fixed_delta.as_secs_f64());
    }
    let alpha = self.accumulator.as_secs_f64() / fixed_delta.as_secs_f64();
    session.update_frame(real_delta, alpha)?;
    Ok(updates)
  }

  pub fn render(&mut self, size: Size) -> Result<(), LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(());
    };
    session.render(size)
  }

  pub fn take_host_commands(&mut self) -> Vec<LuaHostCommand> {
    self
      .session
      .as_mut()
      .map(LuaSession::take_host_commands)
      .unwrap_or_default()
  }

  pub fn take_draw_commands(&mut self) -> Vec<LuaDrawCommand> {
    self
      .session
      .as_mut()
      .map(LuaSession::take_draw_commands)
      .unwrap_or_default()
  }

  pub fn save_game(&mut self) -> Result<Option<JsonValue>, LuaSessionError> {
    self
      .session
      .as_mut()
      .map_or(Ok(None), LuaSession::save_game)
  }

  pub fn save_best(&mut self) -> Result<Option<JsonValue>, LuaSessionError> {
    self
      .session
      .as_mut()
      .map_or(Ok(None), LuaSession::save_best)
  }
}

impl Default for GameService {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::sync::atomic::{AtomicU64, Ordering};

  use super::*;
  use crate::host_engine::services::{LuaPolicy, LuaSessionKind, LuaSessionSpec};

  static TEST_ID: AtomicU64 = AtomicU64::new(1);

  fn test_session() -> LuaSession {
    let directory = std::env::temp_dir().join(format!(
      "tui_game_service_{}_{}",
      std::process::id(),
      TEST_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();
    let entry_path = directory.join("main.lua");
    fs::write(
      &entry_path,
      r#"
        local updates = 0
        local frames = 0
        function Init(ctx) end
        function HandleEvent(event) end
        function Update(dt) updates = updates + 1 end
        function UpdateFrame(dt, alpha) frames = frames + 1 end
        function Render(draw) end
        function SaveGame() return { updates = updates, frames = frames } end
      "#,
    )
    .unwrap();
    LuaSession::load(
      LuaSessionSpec {
        package_id: "test.game.service".to_string(),
        session_kind: LuaSessionKind::Game,
        entry_path,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        terminal_size: Size {
          width: 80,
          height: 24,
        },
        continue_data: None,
      },
      LuaPolicy::default(),
    )
    .unwrap()
  }

  #[test]
  fn inactive_service_ignores_updates() {
    let mut service = GameService::new();
    assert_eq!(service.advance(Duration::from_secs(1)).unwrap(), 0);
    assert_eq!(service.state(), GameSessionState::Inactive);
  }

  #[test]
  fn fixed_update_clamps_delta_and_catches_up_at_most_eight_times() {
    let mut service = GameService::new();
    service.start(
      test_session(),
      PackageSource::Mod,
      120,
      Size {
        width: 40,
        height: 12,
      },
    );

    assert_eq!(service.advance(Duration::from_secs(2)).unwrap(), 8);
    assert_eq!(
      service
        .advance(Duration::from_secs_f64(1.0 / 60.0))
        .unwrap(),
      1
    );
    let stop = service.stop(true);
    assert_eq!(stop.game.as_ref().unwrap()["updates"], 9);
    assert_eq!(stop.game.as_ref().unwrap()["frames"], 2);
  }

  #[test]
  fn each_started_session_receives_a_new_generation() {
    let mut service = GameService::new();
    assert!(service.objects().is_none());
    service.start(
      test_session(),
      PackageSource::Mod,
      60,
      Size {
        width: 40,
        height: 12,
      },
    );
    let first = service.session_token().unwrap();
    assert!(service.objects().is_some());
    service.stop(false);
    assert!(service.session_token().is_none());
    assert!(service.objects().is_none());

    service.start(
      test_session(),
      PackageSource::Mod,
      60,
      Size {
        width: 40,
        height: 12,
      },
    );
    let second = service.session_token().unwrap();
    assert_eq!(first.kind, LuaSessionKind::Game);
    assert_eq!(second.kind, LuaSessionKind::Game);
    assert_ne!(first.generation, second.generation);
  }
}
