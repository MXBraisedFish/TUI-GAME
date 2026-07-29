use std::{
  collections::{HashMap, HashSet, VecDeque},
  path::{Component, Path},
};

use crate::host_engine::services::{
  EngineEvent, FileEvent, ImageEvent, NetworkError, NetworkErrorCode, NetworkEvent, NetworkMethod,
  NetworkResponseBody, NetworkResponseMode, TaskId, TimeAsyncEvent,
};

use super::super::LuaSessionKind;
use super::{
  LuaEventData, LuaEventError, LuaEventErrorCode, LuaFileEvent, LuaFileOperation, LuaFileOutcome,
  LuaImageEvent, LuaImageOutcome, LuaNetworkBody, LuaNetworkEvent, LuaNetworkOutcome,
  LuaRuntimeEvent, LuaTimerEvent, LuaTimerEventKind, LuaTimerKind, sanitize_io_error,
  sanitize_network_error,
};

pub const MAX_LUA_EVENTS_PER_FRAME: usize = 128;
pub const MAX_LUA_PENDING_EVENTS: usize = 1_024;
pub const MAX_LUA_NETWORK_TASKS_PER_SESSION: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LuaSessionToken {
  pub kind: LuaSessionKind,
  pub generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LuaEventCallbackId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaEventRoute {
  HandleEvent,
  Callback(LuaEventCallbackId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaEventDelivery {
  pub event: LuaRuntimeEvent,
  pub route: LuaEventRoute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaTaskOperation {
  File {
    request_id: u64,
    kind: LuaFileOperation,
    virtual_path: String,
  },
  ImageConvert {
    request_id: u64,
  },
  Network {
    request_id: u64,
    method: NetworkMethod,
    original_url: String,
    response_mode: NetworkResponseMode,
  },
  Sleep {
    id: u64,
  },
}

#[derive(Clone, Debug)]
struct LuaTaskRoute {
  token: LuaSessionToken,
  route: LuaEventRoute,
  operation: LuaTaskOperation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaEnqueueError {
  InactiveSession(LuaSessionKind),
  StaleSession(LuaSessionToken),
  EventNotAllowed {
    target: LuaSessionKind,
    event_type: &'static str,
  },
  QueueOverflow(LuaSessionToken),
  TaskAlreadyRegistered(TaskId),
  NetworkTaskLimit(LuaSessionToken),
  InvalidVirtualPath,
  StaleTaskCompletion(TaskId),
}

#[derive(Default)]
struct SessionQueue {
  token: Option<LuaSessionToken>,
  events: VecDeque<LuaEventDelivery>,
  overflowed: bool,
}

impl SessionQueue {
  fn synchronize(&mut self, token: Option<LuaSessionToken>) -> bool {
    if self.token == token {
      return false;
    }
    self.token = token;
    self.events.clear();
    self.overflowed = false;
    true
  }

  fn push(&mut self, delivery: LuaEventDelivery) -> Result<(), LuaSessionToken> {
    let token = self.token.expect("active queue without a token");
    if let Some(index) = self
      .events
      .iter()
      .rposition(|queued| queued.event.data.is_coalescible_with(&delivery.event.data))
    {
      self.events.remove(index);
      self.events.push_back(delivery);
      return Ok(());
    }
    if self.events.len() >= MAX_LUA_PENDING_EVENTS {
      self.overflowed = true;
      return Err(token);
    }
    self.events.push_back(delivery);
    Ok(())
  }

  fn drain_frame(&mut self) -> Vec<LuaEventDelivery> {
    let count = self.events.len().min(MAX_LUA_EVENTS_PER_FRAME);
    self.events.drain(..count).collect()
  }
}

/// Runtime 主线程上的 Lua 事件 Broker。
///
/// 宿主任务 ID 和 Session generation 只存在于 Rust 侧。Lua 只能观察到
/// Session 本地对象/请求 ID。
pub struct LuaEventBroker {
  game: SessionQueue,
  screensaver: SessionQueue,
  tasks: HashMap<TaskId, LuaTaskRoute>,
  discarded_tasks: HashSet<TaskId>,
  orphaned_tasks: Vec<TaskId>,
  next_sequence: u64,
}

impl LuaEventBroker {
  pub fn new() -> Self {
    Self {
      game: SessionQueue::default(),
      screensaver: SessionQueue::default(),
      tasks: HashMap::new(),
      discarded_tasks: HashSet::new(),
      orphaned_tasks: Vec::new(),
      next_sequence: 1,
    }
  }

  pub fn synchronize_sessions(
    &mut self,
    game: Option<LuaSessionToken>,
    screensaver: Option<LuaSessionToken>,
  ) {
    let game_changed = self.game.synchronize(game);
    let screensaver_changed = self.screensaver.synchronize(screensaver);
    if game_changed || screensaver_changed {
      let game_token = self.game.token;
      let screensaver_token = self.screensaver.token;
      let discarded: Vec<TaskId> = self
        .tasks
        .iter()
        .filter_map(|(task_id, route)| {
          let active = match route.token.kind {
            LuaSessionKind::Game => game_token,
            LuaSessionKind::Screensaver => screensaver_token,
          };
          (!active.is_some_and(|active| active == route.token)).then_some(*task_id)
        })
        .collect();
      for task_id in discarded {
        self.tasks.remove(&task_id);
        self.discarded_tasks.insert(task_id);
        self.orphaned_tasks.push(task_id);
      }
    }
  }

  pub fn push_system(
    &mut self,
    frame: u64,
    data: LuaEventData,
  ) -> Result<Option<u64>, LuaEnqueueError> {
    let targets: &[LuaSessionKind] = match &data {
      LuaEventData::Action { .. } | LuaEventData::Mouse { .. } => {
        if self.screensaver.token.is_some() {
          &[]
        } else {
          &[LuaSessionKind::Game]
        }
      }
      LuaEventData::Resize { .. } | LuaEventData::Focus { .. } => {
        &[LuaSessionKind::Game, LuaSessionKind::Screensaver]
      }
      LuaEventData::ScreensaverStarted | LuaEventData::ScreensaverStopped => {
        &[LuaSessionKind::Game]
      }
      _ => {
        return Err(LuaEnqueueError::EventNotAllowed {
          target: LuaSessionKind::Game,
          event_type: data.event_type(),
        });
      }
    };
    self.enqueue_targets(frame, data, LuaEventRoute::HandleEvent, targets)
  }

  pub fn push_owned(
    &mut self,
    token: LuaSessionToken,
    frame: u64,
    data: LuaEventData,
    route: LuaEventRoute,
  ) -> Result<Option<u64>, LuaEnqueueError> {
    let Some(active) = self.active_token(token.kind) else {
      return Err(LuaEnqueueError::InactiveSession(token.kind));
    };
    if active != token {
      return Err(LuaEnqueueError::StaleSession(token));
    }
    if token.kind == LuaSessionKind::Game
      && self.screensaver.token.is_some()
      && data.is_interactive()
    {
      return Err(LuaEnqueueError::EventNotAllowed {
        target: token.kind,
        event_type: data.event_type(),
      });
    }
    if !data.allowed_for(token.kind) {
      return Err(LuaEnqueueError::EventNotAllowed {
        target: token.kind,
        event_type: data.event_type(),
      });
    }
    self.enqueue_targets(frame, data, route, &[token.kind])
  }

  pub fn register_task(
    &mut self,
    task_id: TaskId,
    token: LuaSessionToken,
    operation: LuaTaskOperation,
    route: LuaEventRoute,
  ) -> Result<(), LuaEnqueueError> {
    let Some(active) = self.active_token(token.kind) else {
      return Err(LuaEnqueueError::InactiveSession(token.kind));
    };
    if active != token {
      return Err(LuaEnqueueError::StaleSession(token));
    }
    if let LuaTaskOperation::File {
      kind, virtual_path, ..
    } = &operation
    {
      if !valid_virtual_path(virtual_path) {
        return Err(LuaEnqueueError::InvalidVirtualPath);
      }
      if token.kind == LuaSessionKind::Screensaver && kind.is_write() {
        return Err(LuaEnqueueError::EventNotAllowed {
          target: token.kind,
          event_type: "file",
        });
      }
    }
    if matches!(operation, LuaTaskOperation::Network { .. })
      && self
        .tasks
        .values()
        .filter(|task| {
          task.token == token && matches!(task.operation, LuaTaskOperation::Network { .. })
        })
        .count()
        >= MAX_LUA_NETWORK_TASKS_PER_SESSION
    {
      return Err(LuaEnqueueError::NetworkTaskLimit(token));
    }
    if self.tasks.contains_key(&task_id) {
      return Err(LuaEnqueueError::TaskAlreadyRegistered(task_id));
    }
    self.tasks.insert(
      task_id,
      LuaTaskRoute {
        token,
        route,
        operation,
      },
    );
    Ok(())
  }

  pub fn unregister_task(&mut self, task_id: TaskId) -> bool {
    let removed = self.tasks.remove(&task_id).is_some();
    if removed {
      self.discarded_tasks.insert(task_id);
    }
    removed
  }

  pub fn take_orphaned_tasks(&mut self) -> Vec<TaskId> {
    std::mem::take(&mut self.orphaned_tasks)
  }

  /// 翻译已经登记所有权的异步服务终态事件。
  ///
  /// 包、导出、截图、录屏、视频、日志和通用 TaskFinished/TaskFailed
  /// 不会在这里产生 Lua 事件。
  pub fn route_engine_event(
    &mut self,
    frame: u64,
    event: &EngineEvent,
  ) -> Result<Option<u64>, LuaEnqueueError> {
    let Some(task_id) = service_event_task_id(event) else {
      return Ok(None);
    };
    let Some(task) = self.tasks.remove(&task_id) else {
      if self.discarded_tasks.remove(&task_id) {
        return Err(LuaEnqueueError::StaleTaskCompletion(task_id));
      }
      return Ok(None);
    };
    let data = translate_task_event(&task.operation, event);
    match data {
      Some(data) => self.push_owned(task.token, frame, data, task.route),
      None => Ok(None),
    }
  }

  pub fn drain_frame(&mut self, kind: LuaSessionKind) -> Vec<LuaEventDelivery> {
    self.queue_mut(kind).drain_frame()
  }

  pub fn pending_len(&self, kind: LuaSessionKind) -> usize {
    self.queue(kind).events.len()
  }

  pub fn take_overflowed_sessions(&mut self) -> Vec<LuaSessionToken> {
    let mut tokens = Vec::new();
    for queue in [&mut self.game, &mut self.screensaver] {
      if queue.overflowed {
        queue.overflowed = false;
        if let Some(token) = queue.token {
          tokens.push(token);
        }
      }
    }
    tokens
  }

  fn enqueue_targets(
    &mut self,
    frame: u64,
    data: LuaEventData,
    route: LuaEventRoute,
    targets: &[LuaSessionKind],
  ) -> Result<Option<u64>, LuaEnqueueError> {
    let active_targets: Vec<LuaSessionKind> = targets
      .iter()
      .copied()
      .filter(|kind| self.active_token(*kind).is_some())
      .collect();
    if active_targets.is_empty() {
      return Ok(None);
    }
    let sequence = self.next_sequence;
    self.next_sequence = self.next_sequence.saturating_add(1);
    let delivery = LuaEventDelivery {
      event: LuaRuntimeEvent {
        sequence,
        frame,
        data,
      },
      route,
    };
    let mut first_overflow = None;
    for target in active_targets {
      if let Err(token) = self.queue_mut(target).push(delivery.clone()) {
        first_overflow.get_or_insert(token);
      }
    }
    match first_overflow {
      Some(token) => Err(LuaEnqueueError::QueueOverflow(token)),
      None => Ok(Some(sequence)),
    }
  }

  fn active_token(&self, kind: LuaSessionKind) -> Option<LuaSessionToken> {
    self.queue(kind).token
  }

  fn queue(&self, kind: LuaSessionKind) -> &SessionQueue {
    match kind {
      LuaSessionKind::Game => &self.game,
      LuaSessionKind::Screensaver => &self.screensaver,
    }
  }

  fn queue_mut(&mut self, kind: LuaSessionKind) -> &mut SessionQueue {
    match kind {
      LuaSessionKind::Game => &mut self.game,
      LuaSessionKind::Screensaver => &mut self.screensaver,
    }
  }
}

impl Default for LuaEventBroker {
  fn default() -> Self {
    Self::new()
  }
}

fn valid_virtual_path(path: &str) -> bool {
  !path.is_empty()
    && !Path::new(path).is_absolute()
    && Path::new(path)
      .components()
      .all(|component| matches!(component, Component::Normal(_)))
}

fn service_event_task_id(event: &EngineEvent) -> Option<TaskId> {
  match event {
    EngineEvent::File(event) => Some(match event {
      FileEvent::ReadTextFinished { task_id, .. }
      | FileEvent::WriteTextFinished { task_id, .. }
      | FileEvent::ReadBytesFinished { task_id, .. }
      | FileEvent::WriteBytesFinished { task_id, .. }
      | FileEvent::Failed { task_id, .. } => *task_id,
    }),
    EngineEvent::Image(event) => Some(match event {
      ImageEvent::ConvertFinished { task_id, .. } | ImageEvent::Failed { task_id, .. } => *task_id,
    }),
    EngineEvent::Network(event) => match event {
      NetworkEvent::Started { .. } => None,
      NetworkEvent::Finished { task_id, .. }
      | NetworkEvent::Failed { task_id, .. }
      | NetworkEvent::Cancelled { task_id, .. } => Some(*task_id),
    },
    EngineEvent::Time(TimeAsyncEvent::SleepFinished { task_id, .. }) => Some(*task_id),
    _ => None,
  }
}

fn translate_task_event(operation: &LuaTaskOperation, event: &EngineEvent) -> Option<LuaEventData> {
  match (operation, event) {
    (
      LuaTaskOperation::File {
        request_id,
        kind,
        virtual_path,
      },
      EngineEvent::File(event),
    ) => {
      let outcome = match (kind, event) {
        (LuaFileOperation::ReadText, FileEvent::ReadTextFinished { text, .. }) => {
          LuaFileOutcome::Text(text.clone())
        }
        (LuaFileOperation::ReadBytes, FileEvent::ReadBytesFinished { bytes, .. }) => {
          LuaFileOutcome::Bytes(bytes.clone())
        }
        (LuaFileOperation::WriteText, FileEvent::WriteTextFinished { .. })
        | (LuaFileOperation::WriteBytes, FileEvent::WriteBytesFinished { .. }) => {
          LuaFileOutcome::Written
        }
        (_, FileEvent::Failed { error, .. }) => LuaFileOutcome::Failed(sanitize_io_error(error)),
        _ => LuaFileOutcome::Failed(LuaEventError::sanitized(LuaEventErrorCode::Internal)),
      };
      Some(LuaEventData::File(LuaFileEvent {
        request_id: *request_id,
        kind: *kind,
        path: virtual_path.clone(),
        outcome,
      }))
    }
    (
      LuaTaskOperation::ImageConvert { request_id },
      EngineEvent::Image(ImageEvent::ConvertFinished { output, .. }),
    ) => Some(LuaEventData::Image(LuaImageEvent {
      request_id: *request_id,
      outcome: LuaImageOutcome::Converted(output.clone()),
    })),
    (
      LuaTaskOperation::ImageConvert { request_id },
      EngineEvent::Image(ImageEvent::Failed { error, .. }),
    ) => Some(LuaEventData::Image(LuaImageEvent {
      request_id: *request_id,
      outcome: LuaImageOutcome::Failed(sanitize_io_error(error)),
    })),
    (
      LuaTaskOperation::Network {
        request_id,
        method,
        original_url,
        response_mode,
      },
      EngineEvent::Network(NetworkEvent::Finished { response, .. }),
    ) => Some(LuaEventData::Network(LuaNetworkEvent {
      request_id: *request_id,
      method: *method,
      url: original_url.clone(),
      outcome: LuaNetworkOutcome::Response {
        final_url: response.final_url.clone(),
        status: response.status,
        headers: response.headers.clone(),
        body: match (&response.body, response_mode) {
          (NetworkResponseBody::Text(text), NetworkResponseMode::Text) => {
            LuaNetworkBody::Text(text.clone())
          }
          (NetworkResponseBody::Bytes(bytes), NetworkResponseMode::Bytes) => {
            LuaNetworkBody::Bytes(bytes.clone())
          }
          _ => {
            return Some(LuaEventData::Network(LuaNetworkEvent {
              request_id: *request_id,
              method: *method,
              url: original_url.clone(),
              outcome: LuaNetworkOutcome::Failed(LuaEventError::sanitized(
                LuaEventErrorCode::Internal,
              )),
            }));
          }
        },
      },
    })),
    (
      LuaTaskOperation::Network {
        request_id,
        method,
        original_url,
        ..
      },
      EngineEvent::Network(NetworkEvent::Failed { error, .. }),
    ) => Some(LuaEventData::Network(LuaNetworkEvent {
      request_id: *request_id,
      method: *method,
      url: original_url.clone(),
      outcome: LuaNetworkOutcome::Failed(sanitize_network_error(error)),
    })),
    (
      LuaTaskOperation::Network {
        request_id,
        method,
        original_url,
        ..
      },
      EngineEvent::Network(NetworkEvent::Cancelled { .. }),
    ) => Some(LuaEventData::Network(LuaNetworkEvent {
      request_id: *request_id,
      method: *method,
      url: original_url.clone(),
      outcome: LuaNetworkOutcome::Failed(sanitize_network_error(&NetworkError::at(
        NetworkErrorCode::Cancelled,
        "cancel",
      ))),
    })),
    (LuaTaskOperation::Sleep { id }, EngineEvent::Time(TimeAsyncEvent::SleepFinished { .. })) => {
      Some(LuaEventData::Timer(LuaTimerEvent {
        id: *id,
        timer_kind: LuaTimerKind::Sleep,
        kind: LuaTimerEventKind::Finished,
        executed_count: None,
      }))
    }
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::*;
  use crate::host_engine::services::{
    KeyState, LuaActionState, LuaSessionKind, MouseEvent, MouseEventKind,
  };

  fn token(kind: LuaSessionKind, generation: u64) -> LuaSessionToken {
    LuaSessionToken { kind, generation }
  }

  #[test]
  fn system_routing_matches_game_and_screensaver_rules() {
    let game = token(LuaSessionKind::Game, 1);
    let screen = token(LuaSessionKind::Screensaver, 1);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), None);
    broker
      .push_system(
        1,
        LuaEventData::Action {
          action: "jump".to_string(),
          state: LuaActionState::from(KeyState::Pressed),
        },
      )
      .unwrap();
    assert_eq!(broker.pending_len(LuaSessionKind::Game), 1);

    broker.synchronize_sessions(Some(game), Some(screen));
    broker
      .push_system(
        2,
        LuaEventData::mouse(MouseEvent {
          kind: MouseEventKind::Move,
          button: None,
          scroll: None,
          x: 3,
          y: 4,
        }),
      )
      .unwrap();
    assert_eq!(broker.pending_len(LuaSessionKind::Game), 1);
    assert_eq!(broker.pending_len(LuaSessionKind::Screensaver), 0);

    broker
      .push_system(
        3,
        LuaEventData::Resize {
          width: 120,
          height: 40,
        },
      )
      .unwrap();
    assert_eq!(broker.pending_len(LuaSessionKind::Game), 2);
    assert_eq!(broker.pending_len(LuaSessionKind::Screensaver), 1);
  }

  #[test]
  fn screensaver_filters_interaction_and_write_events_by_owner() {
    let game = token(LuaSessionKind::Game, 1);
    let screen = token(LuaSessionKind::Screensaver, 1);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), Some(screen));

    let game_ui = LuaEventData::ScrollBox(super::super::LuaScrollBoxEvent { id: 1, x: 2, y: 3 });
    assert!(matches!(
      broker.push_owned(game, 1, game_ui, LuaEventRoute::HandleEvent),
      Err(LuaEnqueueError::EventNotAllowed {
        target: LuaSessionKind::Game,
        event_type: "scroll_box"
      })
    ));
    assert!(matches!(
      broker.push_owned(
        screen,
        1,
        LuaEventData::File(LuaFileEvent {
          request_id: 1,
          kind: LuaFileOperation::WriteText,
          path: "storage/save.txt".to_string(),
          outcome: LuaFileOutcome::Written,
        }),
        LuaEventRoute::HandleEvent,
      ),
      Err(LuaEnqueueError::EventNotAllowed {
        target: LuaSessionKind::Screensaver,
        event_type: "file"
      })
    ));

    broker
      .push_owned(
        screen,
        1,
        LuaEventData::File(LuaFileEvent {
          request_id: 2,
          kind: LuaFileOperation::ReadText,
          path: "assets/story.txt".to_string(),
          outcome: LuaFileOutcome::Text("safe".to_string()),
        }),
        LuaEventRoute::HandleEvent,
      )
      .unwrap();
    broker
      .push_owned(
        game,
        1,
        LuaEventData::Network(LuaNetworkEvent {
          request_id: 3,
          method: NetworkMethod::Get,
          url: "https://example.invalid".to_string(),
          outcome: LuaNetworkOutcome::Response {
            final_url: "https://example.invalid".to_string(),
            status: 200,
            headers: std::collections::BTreeMap::new(),
            body: LuaNetworkBody::Text("background".to_string()),
          },
        }),
        LuaEventRoute::HandleEvent,
      )
      .unwrap();
    assert_eq!(broker.pending_len(LuaSessionKind::Game), 1);
    assert_eq!(broker.pending_len(LuaSessionKind::Screensaver), 1);
  }

  #[test]
  fn frame_drain_is_limited_per_session_and_preserves_fifo() {
    let game = token(LuaSessionKind::Game, 1);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), None);
    for index in 0..(MAX_LUA_EVENTS_PER_FRAME + 5) {
      broker
        .push_system(
          index as u64,
          LuaEventData::Action {
            action: format!("action_{index}"),
            state: LuaActionState::Pressed,
          },
        )
        .unwrap();
    }

    let first = broker.drain_frame(LuaSessionKind::Game);
    assert_eq!(first.len(), MAX_LUA_EVENTS_PER_FRAME);
    assert!(
      first
        .windows(2)
        .all(|pair| pair[0].event.sequence < pair[1].event.sequence)
    );
    assert_eq!(broker.pending_len(LuaSessionKind::Game), 5);
  }

  #[test]
  fn queue_coalesces_high_frequency_events_and_faults_at_the_hard_limit() {
    let game = token(LuaSessionKind::Game, 1);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), None);

    for x in 0..100 {
      broker
        .push_system(
          1,
          LuaEventData::Mouse {
            kind: "moved",
            button: None,
            scroll: None,
            x,
            y: 0,
          },
        )
        .unwrap();
    }
    assert_eq!(broker.pending_len(LuaSessionKind::Game), 1);
    assert!(matches!(
      broker
        .drain_frame(LuaSessionKind::Game)
        .pop()
        .unwrap()
        .event
        .data,
      LuaEventData::Mouse { x: 99, .. }
    ));

    for index in 0..MAX_LUA_PENDING_EVENTS {
      broker
        .push_system(
          2,
          LuaEventData::Action {
            action: format!("action_{index}"),
            state: LuaActionState::Pressed,
          },
        )
        .unwrap();
    }
    assert!(matches!(
      broker.push_system(
        2,
        LuaEventData::Action {
          action: "overflow".to_string(),
          state: LuaActionState::Pressed,
        },
      ),
      Err(LuaEnqueueError::QueueOverflow(actual)) if actual == game
    ));
    assert_eq!(broker.take_overflowed_sessions(), vec![game]);
  }

  #[test]
  fn service_results_require_matching_session_ownership_and_hide_host_paths() {
    let game = token(LuaSessionKind::Game, 4);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), None);
    broker
      .register_task(
        TaskId(88),
        game,
        LuaTaskOperation::File {
          request_id: 3,
          kind: LuaFileOperation::ReadText,
          virtual_path: "assets/story.txt".to_string(),
        },
        LuaEventRoute::HandleEvent,
      )
      .unwrap();
    broker
      .route_engine_event(
        7,
        &EngineEvent::File(FileEvent::ReadTextFinished {
          task_id: TaskId(88),
          path: PathBuf::from(r"C:\private\story.txt"),
          text: "hello".to_string(),
        }),
      )
      .unwrap();
    let event = broker
      .drain_frame(LuaSessionKind::Game)
      .pop()
      .unwrap()
      .event;
    assert!(matches!(
      event.data,
      LuaEventData::File(LuaFileEvent {
        request_id: 3,
        path,
        outcome: LuaFileOutcome::Text(text),
        ..
      }) if path == "assets/story.txt" && text == "hello"
    ));
  }

  #[test]
  fn task_registration_rejects_unsafe_virtual_paths_and_screensaver_writes() {
    let game = token(LuaSessionKind::Game, 1);
    let screen = token(LuaSessionKind::Screensaver, 1);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), Some(screen));

    for path in ["", "../secret.txt", "/absolute.txt", r"C:\secret.txt"] {
      assert!(matches!(
        broker.register_task(
          TaskId(10),
          game,
          LuaTaskOperation::File {
            request_id: 1,
            kind: LuaFileOperation::ReadText,
            virtual_path: path.to_string(),
          },
          LuaEventRoute::HandleEvent,
        ),
        Err(LuaEnqueueError::InvalidVirtualPath)
      ));
    }
    assert!(matches!(
      broker.register_task(
        TaskId(11),
        screen,
        LuaTaskOperation::File {
          request_id: 2,
          kind: LuaFileOperation::WriteText,
          virtual_path: "storage/save.txt".to_string(),
        },
        LuaEventRoute::HandleEvent,
      ),
      Err(LuaEnqueueError::EventNotAllowed {
        target: LuaSessionKind::Screensaver,
        event_type: "file"
      })
    ));
  }

  #[test]
  fn async_result_preserves_callback_route_and_is_delivered_once() {
    let game = token(LuaSessionKind::Game, 2);
    let callback = LuaEventCallbackId(9);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), None);
    broker
      .register_task(
        TaskId(41),
        game,
        LuaTaskOperation::Sleep { id: 5 },
        LuaEventRoute::Callback(callback),
      )
      .unwrap();
    let finished = EngineEvent::Time(TimeAsyncEvent::SleepFinished {
      task_id: TaskId(41),
      callback: None,
    });

    assert!(broker.route_engine_event(3, &finished).unwrap().is_some());
    assert!(broker.route_engine_event(3, &finished).unwrap().is_none());
    let delivery = broker.drain_frame(LuaSessionKind::Game).pop().unwrap();
    assert_eq!(delivery.route, LuaEventRoute::Callback(callback));
    assert!(matches!(
      delivery.event.data,
      LuaEventData::Timer(LuaTimerEvent {
        id: 5,
        timer_kind: LuaTimerKind::Sleep,
        kind: LuaTimerEventKind::Finished,
        ..
      })
    ));
  }

  fn network_operation(request_id: u64, response_mode: NetworkResponseMode) -> LuaTaskOperation {
    LuaTaskOperation::Network {
      request_id,
      method: NetworkMethod::Get,
      original_url: format!("https://example.com/{request_id}"),
      response_mode,
    }
  }

  #[test]
  fn network_task_limit_is_scoped_to_each_session() {
    let game = token(LuaSessionKind::Game, 1);
    let screen = token(LuaSessionKind::Screensaver, 1);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), Some(screen));

    for index in 0..MAX_LUA_NETWORK_TASKS_PER_SESSION {
      broker
        .register_task(
          TaskId(index as u64),
          game,
          network_operation(index as u64, NetworkResponseMode::Text),
          LuaEventRoute::HandleEvent,
        )
        .unwrap();
      broker
        .register_task(
          TaskId(100 + index as u64),
          screen,
          network_operation(index as u64, NetworkResponseMode::Text),
          LuaEventRoute::HandleEvent,
        )
        .unwrap();
    }
    assert!(matches!(
      broker.register_task(
        TaskId(999),
        game,
        network_operation(999, NetworkResponseMode::Text),
        LuaEventRoute::HandleEvent,
      ),
      Err(LuaEnqueueError::NetworkTaskLimit(actual)) if actual == game
    ));
  }

  #[test]
  fn network_terminal_event_preserves_callback_and_is_delivered_once() {
    let game = token(LuaSessionKind::Game, 2);
    let callback = LuaEventCallbackId(17);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), None);
    broker
      .register_task(
        TaskId(61),
        game,
        network_operation(8, NetworkResponseMode::Text),
        LuaEventRoute::Callback(callback),
      )
      .unwrap();
    let finished = EngineEvent::Network(NetworkEvent::Finished {
      task_id: TaskId(61),
      method: NetworkMethod::Get,
      response: crate::host_engine::services::NetworkResponse {
        original_url: "https://example.com/8".to_string(),
        final_url: "https://example.com/final".to_string(),
        status: 404,
        headers: std::collections::BTreeMap::from([(
          "content-type".to_string(),
          "text/plain".to_string(),
        )]),
        body: NetworkResponseBody::Text("missing".to_string()),
      },
    });

    assert!(broker.route_engine_event(3, &finished).unwrap().is_some());
    assert!(broker.route_engine_event(3, &finished).unwrap().is_none());
    let delivery = broker.drain_frame(LuaSessionKind::Game).pop().unwrap();
    assert_eq!(delivery.route, LuaEventRoute::Callback(callback));
    assert!(matches!(
      delivery.event.data,
      LuaEventData::Network(LuaNetworkEvent {
        request_id: 8,
        outcome: LuaNetworkOutcome::Response {
          status: 404,
          body: LuaNetworkBody::Text(ref text),
          ..
        },
        ..
      }) if text == "missing"
    ));
  }

  #[test]
  fn network_failures_and_cancellation_are_sanitized_terminal_events() {
    let game = token(LuaSessionKind::Game, 3);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(game), None);
    broker
      .register_task(
        TaskId(71),
        game,
        network_operation(1, NetworkResponseMode::Text),
        LuaEventRoute::HandleEvent,
      )
      .unwrap();
    broker
      .register_task(
        TaskId(72),
        game,
        network_operation(2, NetworkResponseMode::Bytes),
        LuaEventRoute::HandleEvent,
      )
      .unwrap();

    broker
      .route_engine_event(
        4,
        &EngineEvent::Network(NetworkEvent::Failed {
          task_id: TaskId(71),
          method: NetworkMethod::Get,
          url: "https://example.com/1".to_string(),
          error: NetworkError::at(NetworkErrorCode::PermissionDenied, "address_validation"),
        }),
      )
      .unwrap();
    broker
      .route_engine_event(
        4,
        &EngineEvent::Network(NetworkEvent::Cancelled {
          task_id: TaskId(72),
          method: NetworkMethod::Get,
          url: "https://example.com/2".to_string(),
        }),
      )
      .unwrap();

    let deliveries = broker.drain_frame(LuaSessionKind::Game);
    assert!(matches!(
      deliveries[0].event.data,
      LuaEventData::Network(LuaNetworkEvent {
        outcome: LuaNetworkOutcome::Failed(LuaEventError {
          code: LuaEventErrorCode::PermissionDenied,
          ..
        }),
        ..
      })
    ));
    assert!(matches!(
      deliveries[1].event.data,
      LuaEventData::Network(LuaNetworkEvent {
        outcome: LuaNetworkOutcome::Failed(LuaEventError {
          code: LuaEventErrorCode::Cancelled,
          ..
        }),
        ..
      })
    ));
  }

  #[test]
  fn generation_change_drops_queued_events_and_old_task_routes() {
    let first = token(LuaSessionKind::Game, 1);
    let second = token(LuaSessionKind::Game, 2);
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(first), None);
    broker
      .push_system(1, LuaEventData::Focus { gained: false })
      .unwrap();
    broker
      .register_task(
        TaskId(1),
        first,
        LuaTaskOperation::ImageConvert { request_id: 1 },
        LuaEventRoute::HandleEvent,
      )
      .unwrap();

    broker.synchronize_sessions(Some(second), None);
    assert_eq!(broker.pending_len(LuaSessionKind::Game), 0);
    assert_eq!(broker.take_orphaned_tasks(), vec![TaskId(1)]);
    assert!(matches!(
      broker.route_engine_event(
        2,
        &EngineEvent::Image(ImageEvent::ConvertFinished {
          task_id: TaskId(1),
          output: "old".to_string(),
        }),
      ),
      Err(LuaEnqueueError::StaleTaskCompletion(TaskId(1)))
    ));
  }
}
