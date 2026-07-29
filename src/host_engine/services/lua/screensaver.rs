use std::time::Duration;

use crate::host_engine::services::{PackageSource, Size};

use super::{
  LuaEventDelivery, LuaSession, LuaSessionDiagnostics, LuaSessionError, LuaSessionKind,
  LuaSessionState, LuaSessionToken,
};

const MAX_REAL_DELTA: Duration = Duration::from_millis(250);
const MAX_FIXED_UPDATES_PER_FRAME: usize = 8;

/// 唯一屏保 Session 的宿主生命周期。全终端显示层仍由 Runtime Overlay 管理。
pub struct ScreensaverService {
  session: Option<LuaSession>,
  package_id: Option<String>,
  package_source: Option<PackageSource>,
  accumulator: Duration,
  generation: u64,
}

impl ScreensaverService {
  pub fn new() -> Self {
    Self {
      session: None,
      package_id: None,
      package_source: None,
      accumulator: Duration::ZERO,
      generation: 0,
    }
  }

  pub fn start(&mut self, session: LuaSession, source: PackageSource) {
    self.stop();
    self.generation = self.generation.wrapping_add(1).max(1);
    self.package_id = Some(session.package_id().to_string());
    self.package_source = Some(source);
    self.accumulator = Duration::ZERO;
    self.session = Some(session);
  }

  pub fn stop(&mut self) {
    if let Some(mut session) = self.session.take() {
      session.stop();
    }
    self.package_id = None;
    self.package_source = None;
    self.accumulator = Duration::ZERO;
  }

  pub fn is_active(&self) -> bool {
    self.session.is_some()
  }

  pub fn is_faulted(&self) -> bool {
    self
      .session
      .as_ref()
      .is_some_and(|session| session.state() == LuaSessionState::Faulted)
  }

  pub fn package_id(&self) -> Option<&str> {
    self.package_id.as_deref()
  }

  pub fn session_token(&self) -> Option<LuaSessionToken> {
    self.session.as_ref().map(|_| LuaSessionToken {
      kind: LuaSessionKind::Screensaver,
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

  pub fn dispatch_event(&mut self, delivery: &LuaEventDelivery) -> Result<(), LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(());
    };
    session.dispatch_event(delivery)
  }

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
}

impl Default for ScreensaverService {
  fn default() -> Self {
    Self::new()
  }
}
