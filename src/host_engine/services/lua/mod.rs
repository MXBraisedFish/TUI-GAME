mod api;
mod events;
mod game;
mod object_pool;
mod policy;
mod screensaver;
mod session;

pub use api::{LuaApiConfig, LuaApiContext, LuaCallPhase, LuaDrawCommand, LuaHostCommand};
pub use events::{
  LuaActionState, LuaAnimationEvent, LuaAnimationEventKind, LuaAudioEvent, LuaAudioEventKind,
  LuaEnqueueError, LuaEventBroker, LuaEventCallbackId, LuaEventData, LuaEventDelivery,
  LuaEventError, LuaEventErrorCode, LuaEventRoute, LuaFileEntry, LuaFileEvent, LuaFileOperation,
  LuaFileOutcome, LuaHitAreaEvent, LuaHyperlinkEvent, LuaImageEvent, LuaImageOutcome,
  LuaMarkdownEvent, LuaNetworkBody, LuaNetworkEvent, LuaNetworkOutcome, LuaRuntimeEvent,
  LuaScrollBoxEvent, LuaSessionToken, LuaTaskOperation, LuaTextInputEvent, LuaTimerEvent,
  LuaTimerEventKind, LuaTimerKind, MAX_LUA_EVENTS_PER_FRAME, MAX_LUA_FILE_TASKS_PER_SESSION,
  MAX_LUA_NETWORK_TASKS_PER_SESSION, MAX_LUA_PENDING_EVENTS, translate_animation_event,
  translate_delay_timer_event, translate_hit_area_event, translate_hyperlink_event,
  translate_markdown_event, translate_repeat_timer_event, translate_scroll_box_event,
  translate_text_input_event, translate_timer_event,
};
pub use game::{GameService, LuaSessionDiagnostics};
pub use object_pool::LuaObjectPool;
pub use policy::{LuaBudgetKind, LuaExecutionBudget, LuaPolicy};
pub use screensaver::ScreensaverService;
pub use session::{
  LuaCallbackLifetime, LuaErrorStage, LuaExecutionStats, LuaSession, LuaSessionError,
  LuaSessionKind, LuaSessionSpec, LuaSessionState,
};

/// 无状态 Lua Session 工厂。每次调用都会创建完全独立的 Lua VM。
pub struct LuaService {
  policy: LuaPolicy,
}

impl LuaService {
  pub fn new() -> Self {
    Self {
      policy: LuaPolicy::default(),
    }
  }

  pub fn with_policy(policy: LuaPolicy) -> Self {
    Self { policy }
  }

  pub fn policy(&self) -> &LuaPolicy {
    &self.policy
  }

  pub fn create_session(&self, spec: LuaSessionSpec) -> Result<LuaSession, LuaSessionError> {
    LuaSession::load(spec, self.policy.clone())
  }

  pub fn create_session_with_api(
    &self,
    spec: LuaSessionSpec,
    api: LuaApiConfig,
  ) -> Result<LuaSession, LuaSessionError> {
    LuaSession::load_with_api(spec, self.policy.clone(), api)
  }
}

impl Default for LuaService {
  fn default() -> Self {
    Self::new()
  }
}
