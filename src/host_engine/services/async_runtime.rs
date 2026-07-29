use std::{
  collections::{HashMap, HashSet},
  fs,
  path::PathBuf,
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
  },
  thread::{self, JoinHandle},
  time::Duration,
};

use crossbeam_channel::{Receiver, Sender, unbounded};

use super::{
  WriteBarrier,
  export::{self, ExportAsyncEvent, ExportTask},
  image::{ImageConvertParams, ImageService},
  input::{KeyEvent, SystemEvent},
  log::LogSource,
  network::{NetworkEvent, NetworkTask},
  package::{self, PackageAsyncEvent, PackageTask},
  recording::{self, RecordingAsyncEvent, RecordingTask},
  screenshot::{self, ScreenshotAsyncEvent, ScreenshotTask},
  storage::atomic_write,
  video::{self, VideoAsyncEvent, VideoExportTask},
  widget::runtime_object::time::TimeCallbackId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ManagedThreadId(pub u64);

#[derive(Clone)]
pub(crate) struct TaskCancellation {
  task_id: TaskId,
  cancelled: Arc<Mutex<HashSet<TaskId>>>,
}

impl TaskCancellation {
  #[cfg(test)]
  pub(crate) fn new(task_id: TaskId) -> Self {
    Self {
      task_id,
      cancelled: Arc::new(Mutex::new(HashSet::new())),
    }
  }

  pub(crate) fn is_cancelled(&self) -> bool {
    is_cancelled(&self.cancelled, self.task_id)
  }

  #[cfg(test)]
  pub(crate) fn cancel(&self) {
    self
      .cancelled
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
      .insert(self.task_id);
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
  Pending,
  Running,
  Finished,
  Failed,
  Cancelled,
}

#[derive(Clone, Debug)]
pub enum FileTask {
  ReadText { path: PathBuf },
  WriteText { path: PathBuf, text: String },
  ReadBytes { path: PathBuf },
  WriteBytes { path: PathBuf, bytes: Vec<u8> },
}

#[derive(Clone, Debug)]
pub enum FileEvent {
  ReadTextFinished {
    task_id: TaskId,
    path: PathBuf,
    text: String,
  },
  WriteTextFinished {
    task_id: TaskId,
    path: PathBuf,
  },
  ReadBytesFinished {
    task_id: TaskId,
    path: PathBuf,
    bytes: Vec<u8>,
  },
  WriteBytesFinished {
    task_id: TaskId,
    path: PathBuf,
  },
  Failed {
    task_id: TaskId,
    path: PathBuf,
    error: String,
  },
}

#[derive(Clone, Debug)]
pub enum ImageTask {
  Convert {
    params: ImageConvertParams,
    cache_dir: Option<PathBuf>,
  },
}

#[derive(Clone, Debug)]
pub enum ImageEvent {
  ConvertFinished { task_id: TaskId, output: String },
  Failed { task_id: TaskId, error: String },
}

#[derive(Clone, Debug)]
pub struct SleepTask {
  pub duration: Duration,
  pub callback: Option<TimeCallbackId>,
}

#[derive(Clone, Debug)]
pub enum TimeAsyncEvent {
  SleepFinished {
    task_id: TaskId,
    callback: Option<TimeCallbackId>,
  },
}

#[derive(Clone, Debug)]
pub enum EngineTask {
  Package(PackageTask),
  Export(ExportTask),
  Screenshot(ScreenshotTask),
  Recording(RecordingTask),
  Video(VideoExportTask),
  File(FileTask),
  Image(ImageTask),
  Network(NetworkTask),
  Sleep(SleepTask),
}

#[derive(Clone, Debug)]
pub enum EngineEvent {
  InputKey(KeyEvent),
  System(SystemEvent),
  Package(PackageAsyncEvent),
  Export(ExportAsyncEvent),
  Screenshot(ScreenshotAsyncEvent),
  Recording(RecordingAsyncEvent),
  Video(VideoAsyncEvent),
  File(FileEvent),
  Image(ImageEvent),
  Network(NetworkEvent),
  Time(TimeAsyncEvent),
  TaskFinished { id: TaskId },
  TaskFailed { id: TaskId, error: String },
  Log { source: LogSource, message: String },
}

enum WorkerMessage {
  Run(TaskId, EngineTask),
  Shutdown,
}

struct ManagedThread {
  stop: Arc<AtomicBool>,
  joinable: bool,
  handle: Option<JoinHandle<()>>,
}

pub struct AsyncRuntime {
  task_tx: Sender<WorkerMessage>,
  event_tx: Sender<EngineEvent>,
  event_rx: Receiver<EngineEvent>,
  workers: Vec<JoinHandle<()>>,
  task_states: Arc<Mutex<HashMap<TaskId, TaskState>>>,
  cancelled_tasks: Arc<Mutex<HashSet<TaskId>>>,
  write_barrier: WriteBarrier,
  managed_threads: HashMap<ManagedThreadId, ManagedThread>,
  next_task_id: AtomicU64,
  next_thread_id: u64,
}

impl AsyncRuntime {
  pub fn new() -> Self {
    Self::with_worker_count(4)
  }

  pub fn with_worker_count(worker_count: usize) -> Self {
    let (task_tx, task_rx) = unbounded();
    let (event_tx, event_rx) = unbounded();
    let task_states = Arc::new(Mutex::new(HashMap::new()));
    let cancelled_tasks = Arc::new(Mutex::new(HashSet::new()));
    let write_barrier = WriteBarrier::new();
    let mut workers = Vec::new();

    for _ in 0..worker_count.max(1) {
      let task_rx = task_rx.clone();
      let event_tx = event_tx.clone();
      let task_states = task_states.clone();
      let cancelled_tasks = cancelled_tasks.clone();
      let write_barrier = write_barrier.clone();
      // Note: thread::spawn failure (e.g. OOM) is a process-level abort in std;
      // no recoverable error to log here.
      workers.push(thread::spawn(move || {
        worker_loop(
          task_rx,
          event_tx,
          task_states,
          cancelled_tasks,
          write_barrier,
        );
      }));
    }

    Self {
      task_tx,
      event_tx,
      event_rx,
      workers,
      task_states,
      cancelled_tasks,
      write_barrier,
      managed_threads: HashMap::new(),
      next_task_id: AtomicU64::new(1),
      next_thread_id: 1,
    }
  }

  pub fn submit(&self, task: EngineTask) -> TaskId {
    let id = TaskId(self.next_task_id.fetch_add(1, Ordering::SeqCst));
    if let Some(target) = write_target(&task) {
      let temporary = temporary_target(&task, &target, id);
      if !self.write_barrier.register(id, target, Some(temporary)) {
        set_task_state(&self.task_states, id, TaskState::Cancelled);
        return id;
      }
    }
    set_task_state(&self.task_states, id, TaskState::Pending);
    if self.task_tx.send(WorkerMessage::Run(id, task)).is_err() {
      let error = "asynchronous worker queue is closed".to_string();
      set_task_state(&self.task_states, id, TaskState::Failed);
      self.write_barrier.fail(id, error.clone());
      let _ = self.event_tx.send(EngineEvent::TaskFailed { id, error });
    }
    id
  }

  pub fn write_barrier(&self) -> WriteBarrier {
    self.write_barrier.clone()
  }

  pub fn task_state(&self, id: TaskId) -> Option<TaskState> {
    let states = self.task_states.lock().unwrap_or_else(|poison| {
      // Mutex poisoned — a previous task panicked. Recover the guard.
      poison.into_inner()
    });
    states.get(&id).copied()
  }

  /// 请求取消任务。尚未开始的任务不会执行；运行中的任务会在任务边界停止提交结果。
  pub fn cancel_task(&self, id: TaskId) {
    self
      .cancelled_tasks
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
      .insert(id);
    if self.task_state(id) == Some(TaskState::Pending) {
      set_task_state(&self.task_states, id, TaskState::Cancelled);
    }
  }

  pub fn cancel_tasks(&self, ids: impl IntoIterator<Item = TaskId>) {
    for id in ids {
      self.cancel_task(id);
    }
  }

  pub fn poll_events(&self) -> Vec<EngineEvent> {
    self.event_rx.try_iter().collect()
  }

  pub fn event_sender(&self) -> Sender<EngineEvent> {
    self.event_tx.clone()
  }

  pub fn spawn_managed_listener<F>(&mut self, joinable: bool, start: F) -> ManagedThreadId
  where
    F: FnOnce(Sender<EngineEvent>, Arc<AtomicBool>) -> JoinHandle<()> + Send + 'static,
  {
    let id = ManagedThreadId(self.next_thread_id);
    self.next_thread_id += 1;

    let stop = Arc::new(AtomicBool::new(false));
    let handle = start(self.event_tx.clone(), stop.clone());
    let handle = joinable.then_some(handle);

    self.managed_threads.insert(
      id,
      ManagedThread {
        stop,
        joinable,
        handle,
      },
    );

    id
  }

  pub fn stop_managed_thread(&mut self, id: ManagedThreadId) -> bool {
    let Some(mut thread) = self.managed_threads.remove(&id) else {
      return false;
    };

    thread.stop.store(true, Ordering::SeqCst);
    if thread.joinable {
      if let Some(handle) = thread.handle.take() {
        let _ = handle.join();
      }
    }
    true
  }

  pub fn stop_all_managed_threads(&mut self) {
    let ids = self.managed_threads.keys().copied().collect::<Vec<_>>();
    for id in ids {
      let _ = self.stop_managed_thread(id);
    }
  }

  /// 停止任务执行器并等待所有工作线程结束。
  ///
  /// Shutdown 在销毁 Lua 与其它宿主服务前显式调用，Drop 仅作为异常路径兜底。
  pub fn shutdown(&mut self) {
    self.stop_all_managed_threads();
    for _ in &self.workers {
      if self.task_tx.send(WorkerMessage::Shutdown).is_err() {
        break;
      }
    }
    while let Some(worker) = self.workers.pop() {
      let _ = worker.join();
    }
  }
}

impl Default for AsyncRuntime {
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for AsyncRuntime {
  fn drop(&mut self) {
    self.shutdown();
  }
}

fn worker_loop(
  task_rx: Receiver<WorkerMessage>,
  event_tx: Sender<EngineEvent>,
  task_states: Arc<Mutex<HashMap<TaskId, TaskState>>>,
  cancelled_tasks: Arc<Mutex<HashSet<TaskId>>>,
  write_barrier: WriteBarrier,
) {
  while let Ok(message) = task_rx.recv() {
    match message {
      WorkerMessage::Run(id, task) => {
        if is_cancelled(&cancelled_tasks, id) {
          if let EngineTask::Network(task) = &task {
            super::network::emit_cancelled(id, task, &event_tx);
          }
          set_task_state(&task_states, id, TaskState::Cancelled);
          clear_cancelled(&cancelled_tasks, id);
          write_barrier.finish(id);
          continue;
        }
        set_task_state(&task_states, id, TaskState::Running);
        write_barrier.start(id);
        let cancellation = TaskCancellation {
          task_id: id,
          cancelled: cancelled_tasks.clone(),
        };
        let is_network = matches!(&task, EngineTask::Network(_));
        let result = run_task(id, task, &event_tx, &cancellation);
        let was_cancelled = is_cancelled(&cancelled_tasks, id);
        clear_cancelled(&cancelled_tasks, id);
        match result {
          Ok(()) => {
            if !is_network && was_cancelled {
              set_task_state(&task_states, id, TaskState::Cancelled);
            } else {
              set_task_state(&task_states, id, TaskState::Finished);
              let _ = event_tx.send(EngineEvent::TaskFinished { id });
            }
            write_barrier.finish(id);
          }
          Err(error) => {
            if was_cancelled {
              set_task_state(&task_states, id, TaskState::Cancelled);
              write_barrier.finish(id);
            } else {
              set_task_state(&task_states, id, TaskState::Failed);
              write_barrier.fail(id, error.clone());
              let _ = event_tx.send(EngineEvent::TaskFailed { id, error });
            }
          }
        }
      }
      WorkerMessage::Shutdown => break,
    }
  }
}

fn write_target(task: &EngineTask) -> Option<PathBuf> {
  match task {
    EngineTask::Package(_)
    | EngineTask::Image(_)
    | EngineTask::Network(_)
    | EngineTask::Sleep(_) => None,
    EngineTask::Export(task) => Some(task.output_dir.join(format!(
      "{}.{}",
      task.file_stem,
      task.format.extension()
    ))),
    EngineTask::Screenshot(task) => Some(task.png_path.clone()),
    EngineTask::Recording(task) => Some(task.path().to_path_buf()),
    EngineTask::Video(task) => Some(task.output_path.clone()),
    EngineTask::File(FileTask::WriteText { path, .. })
    | EngineTask::File(FileTask::WriteBytes { path, .. }) => Some(path.clone()),
    EngineTask::File(FileTask::ReadText { .. } | FileTask::ReadBytes { .. }) => None,
  }
}

fn temporary_target(task: &EngineTask, target: &std::path::Path, task_id: TaskId) -> PathBuf {
  if matches!(task, EngineTask::Video(_)) {
    return target.with_extension(format!("mp4.task-{}.part", task_id.0));
  }
  let extension = target
    .extension()
    .and_then(|value| value.to_str())
    .map(|value| format!("{value}.tmp"))
    .unwrap_or_else(|| "tmp".to_string());
  target.with_extension(extension)
}

fn is_cancelled(cancelled_tasks: &Arc<Mutex<HashSet<TaskId>>>, id: TaskId) -> bool {
  cancelled_tasks
    .lock()
    .unwrap_or_else(|poison| poison.into_inner())
    .contains(&id)
}

fn clear_cancelled(cancelled_tasks: &Arc<Mutex<HashSet<TaskId>>>, id: TaskId) {
  cancelled_tasks
    .lock()
    .unwrap_or_else(|poison| poison.into_inner())
    .remove(&id);
}

fn run_task(
  id: TaskId,
  task: EngineTask,
  event_tx: &Sender<EngineEvent>,
  cancellation: &TaskCancellation,
) -> Result<(), String> {
  match task {
    EngineTask::Package(task) => package::run_package_task(id, task, event_tx),
    EngineTask::Export(task) => export::run_export_task(id, task, event_tx, cancellation),
    EngineTask::Screenshot(task) => {
      screenshot::run_screenshot_task(id, task, event_tx, cancellation)
    }
    EngineTask::Recording(task) => recording::run_recording_task(id, task, event_tx),
    EngineTask::Video(task) => video::run_video_task(id, task, event_tx, cancellation),
    EngineTask::File(task) => run_file_task(id, task, event_tx),
    EngineTask::Image(task) => run_image_task(id, task, event_tx),
    EngineTask::Network(task) => super::network::run_network_task(id, task, event_tx, cancellation),
    EngineTask::Sleep(task) => {
      thread::sleep(task.duration);
      let _ = event_tx.send(EngineEvent::Time(TimeAsyncEvent::SleepFinished {
        task_id: id,
        callback: task.callback,
      }));
      Ok(())
    }
  }
}

fn run_file_task(
  task_id: TaskId,
  task: FileTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  match task {
    FileTask::ReadText { path } => match fs::read_to_string(&path) {
      Ok(text) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::ReadTextFinished {
          task_id,
          path,
          text,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.to_string());
        Err(error.to_string())
      }
    },
    FileTask::WriteText { path, text } => match atomic_write(&path, text.as_bytes(), true) {
      Ok(()) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::WriteTextFinished {
          task_id,
          path,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.to_string());
        Err(error.to_string())
      }
    },
    FileTask::ReadBytes { path } => match fs::read(&path) {
      Ok(bytes) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::ReadBytesFinished {
          task_id,
          path,
          bytes,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.to_string());
        Err(error.to_string())
      }
    },
    FileTask::WriteBytes { path, bytes } => match atomic_write(&path, &bytes, true) {
      Ok(()) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::WriteBytesFinished {
          task_id,
          path,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.to_string());
        Err(error.to_string())
      }
    },
  }
}

fn send_file_error(event_tx: &Sender<EngineEvent>, task_id: TaskId, path: PathBuf, error: String) {
  let _ = event_tx.send(EngineEvent::File(FileEvent::Failed {
    task_id,
    path,
    error,
  }));
}

fn run_image_task(
  task_id: TaskId,
  task: ImageTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  match task {
    ImageTask::Convert { params, cache_dir } => {
      match ImageService::new(cache_dir).convert(params) {
        Ok(output) => {
          let _ = event_tx.send(EngineEvent::Image(ImageEvent::ConvertFinished {
            task_id,
            output,
          }));
          Ok(())
        }
        Err(error) => {
          let _ = event_tx.send(EngineEvent::Image(ImageEvent::Failed {
            task_id,
            error: error.clone(),
          }));
          Err(error)
        }
      }
    }
  }
}

fn set_task_state(
  task_states: &Arc<Mutex<HashMap<TaskId, TaskState>>>,
  id: TaskId,
  state: TaskState,
) {
  let mut states = task_states.lock().unwrap_or_else(|poison| {
    // Mutex poisoned — a previous task panicked. Recover the guard.
    poison.into_inner()
  });
  states.insert(id, state);
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn async_runtime_assigns_unique_task_ids() {
    let runtime = AsyncRuntime::with_worker_count(1);
    let first = runtime.submit(EngineTask::Sleep(SleepTask {
      duration: Duration::ZERO,
      callback: None,
    }));
    let second = runtime.submit(EngineTask::Sleep(SleepTask {
      duration: Duration::ZERO,
      callback: None,
    }));

    assert_ne!(first, second);
  }

  #[test]
  fn sleep_task_returns_time_event() {
    let runtime = AsyncRuntime::with_worker_count(1);
    let task_id = runtime.submit(EngineTask::Sleep(SleepTask {
      duration: Duration::from_millis(1),
      callback: None,
    }));

    let mut found = false;
    for _ in 0..50 {
      if runtime.poll_events().into_iter().any(|event| {
        matches!(
            event,
            EngineEvent::Time(TimeAsyncEvent::SleepFinished { task_id: id, .. })
                if id == task_id
        )
      }) {
        found = true;
        break;
      }
      thread::sleep(Duration::from_millis(2));
    }

    assert!(found);
  }

  #[test]
  fn failed_file_task_emits_task_failed() {
    let runtime = AsyncRuntime::with_worker_count(1);
    let task_id = runtime.submit(EngineTask::File(FileTask::ReadText {
      path: PathBuf::from("missing-file-for-async-runtime-test.txt"),
    }));

    let mut found = false;
    for _ in 0..50 {
      if runtime
        .poll_events()
        .into_iter()
        .any(|event| matches!(event, EngineEvent::TaskFailed { id, .. } if id == task_id))
      {
        found = true;
        break;
      }
      thread::sleep(Duration::from_millis(2));
    }

    assert!(found);
  }
}
