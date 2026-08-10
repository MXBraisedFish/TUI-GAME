pub(crate) mod animation;
mod async_runtime;
mod audio;
mod canvas;
mod clipboard;
mod code_highlight;
mod event;
pub(crate) mod export;
mod ffmpeg;
mod file;
mod host_object;
mod i18n;
mod image;
mod input;
mod input_method;
mod layout;
mod log;
mod lua;
mod network;
mod package;
mod popup;
mod random;
mod recording;
mod render;
mod render_pipeline;
mod rich_text;
mod screenshot;
mod storage;
mod terminal;
mod terminal_capabilities;
pub mod text_layout;
mod time;
mod ui;
mod unicode;
mod version;
mod video;
pub(crate) mod widget;
mod write_barrier;

pub use animation::{
  AnimationBinding, AnimationCallbackId, AnimationCallbackRequest, AnimationClip, AnimationClock,
  AnimationColor, AnimationEasing, AnimationEndMode, AnimationError, AnimationEvent,
  AnimationEventKind, AnimationHandle, AnimationId, AnimationInterpolation, AnimationKeyframe,
  AnimationMarker, AnimationOwner, AnimationPlaybackOptions, AnimationProperty,
  AnimationRepeatCount, AnimationRepeatMode, AnimationRepeatOptions, AnimationService,
  AnimationSource, AnimationTarget, AnimationTargetRouter, AnimationTrack, AnimationUpdate,
  AnimationValue, AnimationValueId, AnimationValueKind, AnimationWrite, AnimationWriteOperation,
  CellEffectId, CharacterEffectService, CharacterFrame, EffectParameterId, GameInstanceId,
  GameObjectRef, PlaybackDirection, PlaybackState, TweenDefinition, UiObjectKind, UiObjectRef,
  UiPoolId,
};
pub use async_runtime::{
  AsyncRuntime, EngineEvent, EngineTask, FileEvent, FileTask, ImageEvent, ImageTask,
  ManagedThreadId, SleepTask, TaskId, TaskState, TimeAsyncEvent,
};
pub use audio::{
  AudioAsyncEvent, AudioCaptureId, AudioError, AudioErrorCode, AudioId, AudioObject,
  AudioObjectPool, AudioPoolId, AudioService, AudioSource, AudioState, AudioType, AudioTypeId,
  ResolvedAudioFile,
};
pub use canvas::{CanvasCell, CanvasService};
pub use clipboard::ClipboardService;
pub use code_highlight::{
  CodeHighlightService, CodeHighlightTheme, CodeHighlightToken, CodeLanguage, CodeTokenKind,
};
pub use event::EngineEventQueue;
pub use export::{ExportAsyncEvent, ExportService, ExportTask};
pub use ffmpeg::{FfmpegInstallation, FfmpegService};
pub use file::FileService;
pub use host_object::{HostArea, HostAreaId, HostAreaKind, HostObjectPool};
pub use i18n::{I18nService, LanguageRegistryEntry};
pub use image::{ImageConvertParams, ImageService};
pub use input::{
  ActionMapEntry, InputActionEvent, InputEventType, InputService, Key, KeyEvent, KeyEventKind,
  KeyState, MouseButton, MouseEvent, MouseEventKind, RawKeyEvent, ScrollDirection, SystemEvent,
  TerminalKeyCode, TerminalKeyEvent, canonical_key_token, format_key_display, key_token,
  translate_action_map,
};
pub use input_method::{ImPolicy, InputMethodService};
pub use layout::{LayoutService, Rect, Size};
pub use log::{LogService, LogSessionId, LogSessionKind, LogSource};
pub use lua::{
  GameService, GameStopData, LuaActionState, LuaAnimationEvent, LuaAnimationEventKind,
  LuaApiConfig, LuaApiContext, LuaAudioEvent, LuaAudioEventKind, LuaBudgetKind,
  LuaCallbackLifetime, LuaDrawCommand, LuaDrawTarget, LuaEnqueueError, LuaErrorStage,
  LuaEventBroker, LuaEventCallbackId, LuaEventData, LuaEventDelivery, LuaEventError,
  LuaEventErrorCode, LuaEventRoute, LuaExecutionBudget, LuaExecutionStats, LuaFileEntry,
  LuaFileEvent, LuaFileOperation, LuaFileOutcome, LuaHitAreaEvent, LuaHostCommand,
  LuaHyperlinkEvent, LuaImageEvent, LuaImageOutcome, LuaMarkdownEvent, LuaNetworkBody,
  LuaNetworkEvent, LuaNetworkOutcome, LuaObjectPool, LuaPolicy, LuaRuntimeEvent, LuaScrollBoxEvent,
  LuaService, LuaSession, LuaSessionDiagnostics, LuaSessionError, LuaSessionKind, LuaSessionSpec,
  LuaSessionState, LuaSessionToken, LuaTaskOperation, LuaTextInputEvent, LuaTimerEvent,
  LuaTimerEventKind, LuaTimerKind, MAX_LUA_EVENTS_PER_FRAME, MAX_LUA_FILE_TASKS_PER_SESSION,
  MAX_LUA_NETWORK_TASKS_PER_SESSION, MAX_LUA_PENDING_EVENTS, ScreensaverService,
};
pub use network::{
  NetworkError, NetworkErrorCode, NetworkEvent, NetworkHeader, NetworkMethod, NetworkRequest,
  NetworkRequestBody, NetworkRequestStatus, NetworkResponse, NetworkResponseBody,
  NetworkResponseMode, NetworkService, NetworkSubmitError, NetworkTask,
};
pub use package::{
  PackageAsset, PackageEvent, PackageId, PackageInfo, PackageListEntry, PackageService,
  PackageSource, PackageType,
};
pub use popup::{PopupDismissEvent, PopupRequest, PopupService, PopupView};
pub use random::RandomService;
pub use recording::{
  RecordingAsyncEvent, RecordingAudioMetadata, RecordingPlayback, RecordingPlaybackMetadata,
  RecordingService, RecordingSnapshot, RecordingState, RecordingTask, load_recording_playback,
  load_recording_playback_metadata,
};
pub use render::{BorderCharacter, BorderStyle, CustomBorder, RenderService};
pub use render_pipeline::{ComposedCell, ComposedFrame, FrameCompositor, FramePresenter};
pub use rich_text::{
  RichText, RichTextParams, RichTextSegment, RichTextService, TerminalColor, TextColor, TextStyle,
  parse_text_color,
};
pub use screenshot::{ScreenshotAsyncEvent, ScreenshotRect, ScreenshotService, ScreenshotTask};
pub use storage::{
  ActionKeyMap, AutoRecordingMode, AutoSplitDuration, BestGameSave, ContinueGameSave,
  DisplayFpsLimit, DisplayLogoMode, DisplayOrderMode, DisplaySettingsProfile, DisplaySourceMode,
  GamePackageState, GameSaveProfile, KeyBindingsProfile, PackageDefaultState, PackageStateProfile,
  RecordingExportFrameRate, RecordingExportQuality, RecordingFrameRate, RecordingGpuAcceleration,
  RecordingPixelScale, RecordingPopupMode, RecordingProfile, SafeModeDefault,
  ScreensaverPackageState, ScreenshotDoubleAction, ScreenshotProfile, StorageService,
};
pub use terminal::TerminalService;
pub use text_layout::{DrawTextParams, TextAlign, TextMode, TextWrapMode};
pub use time::TimeService;
pub use ui::{UiEvent, UiObjectPool, UiObjectPoolOwner, UiService};
pub use unicode::UnicodeService;
pub use version::{HOST_API_VERSION, HOST_VERSION, PACKAGE_MANIFEST_VERSION};
pub use video::{
  VideoAsyncEvent, VideoExportError, VideoExportProgress, VideoExportStage, VideoExportStatus,
  VideoExportTask, VideoService,
};
pub use widget::{
  DelayTimerEvent, DelayTimerId, DelayTimerOptions, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, HyperlinkEvent, HyperlinkId, HyperlinkOptions, HyperlinkService, MarkdownEvent,
  MarkdownRenderParams, MarkdownService, MarkdownTheme, MarkdownViewId, MarkdownViewOptions,
  Overflow, ProgressBarFillOrigin, ProgressBarId, ProgressBarOptions, ProgressBarSegmentStyle,
  ProgressBarService, RandomAlgorithm, RandomConfiguration, RandomConfiguredRange,
  RandomGeneratedValue, RandomGeneratorId, RandomSeed, RandomSnapshot, RepeatMode,
  RepeatTimerEvent, RepeatTimerId, RepeatTimerOptions, RuntimeObjectPool, RuntimeObjectPoolOwner,
  ScrollBoxEvent, ScrollBoxId, ScrollBoxOptions, ScrollBoxService, ScrollbarLayout,
  ScrollbarPolicy, ScrollbarSide, ScrollbarStyle, ScrollbarVisibility, SliceId, SliceLength,
  SliceOptions, SliceRect, SliceService, SurfaceId, TableAlign, TableBorderMode, TableBorderStyle,
  TableCell, TableColumn, TableDrawParams, TableId, TableOptions, TableOverflow, TableRow,
  TableService, TableStyle, TextInputCursorShape, TextInputEvent, TextInputId, TextInputMode,
  TextInputOptions, TextInputRenderParams, TextInputService, TimeCallbackId, TimeCallbackRequest,
  TimerEvent, TimerId, TimerMode, TimerOptions, TimerState, VerticalAlign,
};
pub use write_barrier::{WriteBarrier, WriteBarrierSnapshot};

/// 引擎核心服务集合，持有所有子服务的实例
pub struct EngineServices {
  pub async_runtime: AsyncRuntime,
  pub engine_events: EngineEventQueue,
  pub audio: AudioService,
  pub file: FileService,
  pub network: NetworkService,
  pub random: RandomService,
  pub animation: AnimationService,
  pub character_effect: CharacterEffectService,
  pub screenshot: ScreenshotService,
  pub recording: RecordingService,
  pub ffmpeg: FfmpegService,
  pub video: VideoService,
  pub package: PackageService,
  pub popup: PopupService,
  pub clipboard: ClipboardService,
  pub runtime_objects: RuntimeObjectPool,
  pub time: TimeService,
  pub host_objects: HostObjectPool,
  pub hit_area: HitAreaService,
  pub hyperlink: HyperlinkService,
  pub scroll_box: ScrollBoxService,
  pub markdown: MarkdownService,
  pub code_highlight: CodeHighlightService,
  pub progress_bar: ProgressBarService,
  pub table: TableService,
  pub slice: SliceService,
  pub input: InputService,
  pub input_method: InputMethodService,
  pub ui: UiService,
  pub game: GameService,
  pub image: ImageService,
  pub screensaver: ScreensaverService,
  pub storage: StorageService,
  pub export: ExportService,
  pub lua: LuaService,
  pub render: RenderService,
  pub terminal: TerminalService,
  pub text_input: TextInputService,
  pub log: LogService,
  pub i18n: I18nService,
  pub rich_text: RichTextService,
  pub unicode: UnicodeService,
  pub canvas: CanvasService,
  pub layout: LayoutService,
  pub compositor: FrameCompositor,
  pub presenter: FramePresenter,
}

impl EngineServices {
  pub fn new() -> Self {
    let mut log = LogService::new();
    let storage = StorageService::new(&mut log);
    let _ = log.set_output_path(storage.tui_log_path());
    let image_cache_dir = storage.path("data/cache/images");
    let ffmpeg = FfmpegService::new(
      storage.root_dir().to_path_buf(),
      storage.cache_dir_path().join("ffmpeg"),
    );
    let async_runtime = AsyncRuntime::new();
    let audio = AudioService::new(async_runtime.event_sender());

    Self {
      async_runtime,
      engine_events: EngineEventQueue::new(),
      audio,
      file: FileService::new(),
      network: NetworkService::new(),
      random: RandomService::new(),
      animation: AnimationService::new(),
      character_effect: CharacterEffectService::new(),
      screenshot: ScreenshotService::new(),
      recording: RecordingService::new(),
      ffmpeg,
      video: VideoService::new(),
      terminal: TerminalService::new(),
      clipboard: ClipboardService::new(),
      runtime_objects: RuntimeObjectPool::new(),
      time: TimeService::new(),
      host_objects: HostObjectPool::new(),
      hit_area: HitAreaService::new(),
      hyperlink: HyperlinkService::new(),
      scroll_box: ScrollBoxService::new(),
      markdown: MarkdownService::new(),
      code_highlight: CodeHighlightService::new(),
      progress_bar: ProgressBarService::new(),
      table: TableService::new(),
      slice: SliceService::new(),
      text_input: TextInputService::new(),
      package: PackageService::new(),
      popup: PopupService::new(),
      input: InputService::new(),
      input_method: InputMethodService::new(),
      ui: UiService::new(),
      game: GameService::new(),
      image: ImageService::new(Some(image_cache_dir)),
      screensaver: ScreensaverService::new(),
      storage,
      export: ExportService::new(),
      lua: LuaService::new(),
      render: RenderService::new(),
      log,
      i18n: I18nService::new(),
      rich_text: RichTextService::new(),
      unicode: UnicodeService::new(),
      canvas: CanvasService::new(),
      layout: LayoutService::new(),
      compositor: FrameCompositor::new(),
      presenter: FramePresenter::new(),
    }
  }
}
