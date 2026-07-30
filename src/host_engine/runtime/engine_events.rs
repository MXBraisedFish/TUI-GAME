use super::*;
use crate::host_engine::services::{
  AudioAsyncEvent, EngineEvent, ExportAsyncEvent, NetworkEvent, ScreenshotAsyncEvent,
  VideoAsyncEvent,
};

pub(super) struct RuntimeEngineEvents {
  pub package: Vec<PackageEvent>,
  pub export: Vec<ExportAsyncEvent>,
  pub screenshot: Vec<ScreenshotAsyncEvent>,
  pub video: Vec<VideoAsyncEvent>,
  pub network: Vec<NetworkEvent>,
}

pub(super) fn drain_engine_events(
  services: &mut EngineServices,
  lua_events: &mut LuaEventBroker,
  frame: u64,
) -> RuntimeEngineEvents {
  let mut package_events = Vec::new();
  let mut export_events = Vec::new();
  let mut screenshot_events = Vec::new();
  let mut video_events = Vec::new();
  let mut network_events = Vec::new();

  for event in services.engine_events.drain() {
    if let Err(error) = lua_events.route_engine_event(frame, &event) {
      match error {
        LuaEnqueueError::StaleTaskCompletion(task_id) => services.log.debug(
          LogSource::Lua,
          format!("Discarded stale Lua task completion: {task_id:?}"),
        ),
        LuaEnqueueError::StaleAudioEvent(audio_id) => services.log.debug(
          LogSource::Lua,
          format!("Discarded stale Lua audio event: {audio_id:?}"),
        ),
        error => services.log.warn(
          LogSource::Lua,
          format!("Lua async event routing rejected: {error:?}"),
        ),
      }
    }
    match event {
      EngineEvent::InputKey(event) => services.input.queue_key_event(event, &mut services.log),
      EngineEvent::System(event) => services.input.queue_system_event(event, &mut services.log),
      EngineEvent::Package(event) => {
        let event = services
          .package
          .handle_async_event(event, &mut services.log);
        if matches!(event, PackageEvent::ScanFinished { .. }) {
          synchronize_key_bindings_profile(services);
        }
        if matches!(event, PackageEvent::WatchChanged { .. }) {
          let _ = services.package.request_rescan(&services.async_runtime);
        }
        package_events.push(event);
      }
      EngineEvent::Export(event) => export_events.push(event),
      EngineEvent::Screenshot(event) => {
        services.screenshot.handle_engine_event(&event);
        match event {
          ScreenshotAsyncEvent::Progress {
            task_id,
            completed_rows,
            total_rows,
          } => screenshot_events.push(ScreenshotAsyncEvent::Progress {
            task_id,
            completed_rows,
            total_rows,
          }),
          ScreenshotAsyncEvent::Saved { task_id, png_path } => {
            services.log.info(
              LogSource::Storage,
              format!(
                "Screenshot task {task_id:?} saved PNG: {}",
                png_path.display()
              ),
            );
            screenshot_events.push(ScreenshotAsyncEvent::Saved { task_id, png_path });
          }
          ScreenshotAsyncEvent::Failed { task_id, error } => {
            services.log.warn(
              LogSource::Storage,
              format!("Screenshot task {task_id:?} failed: {error}"),
            );
            screenshot_events.push(ScreenshotAsyncEvent::Failed { task_id, error });
          }
        }
      }
      EngineEvent::Recording(event) => {
        services.recording.handle_engine_event(&event);
        match event {
          crate::host_engine::services::RecordingAsyncEvent::Saved { task_id, path } => {
            services.log.info(
              LogSource::Storage,
              format!("Recording task {task_id:?} saved: {}", path.display()),
            );
          }
          crate::host_engine::services::RecordingAsyncEvent::Failed { task_id, error } => {
            services.log.warn(
              LogSource::Storage,
              format!("Recording task {task_id:?} failed: {error}"),
            );
          }
        }
      }
      EngineEvent::Video(event) => {
        services.video.handle_engine_event(&event);
        match &event {
          VideoAsyncEvent::Saved {
            task_id,
            source_path,
            mp4_path,
          } => services.log.info(
            LogSource::Storage,
            format!(
              "Video export task {task_id:?} saved {} from {}",
              mp4_path.display(),
              source_path.display()
            ),
          ),
          VideoAsyncEvent::Failed {
            task_id,
            source_path,
            output_path,
            stage,
            error,
          } => services.log.warn(
            LogSource::Storage,
            format!(
              "Video export task {task_id:?} failed during {stage}: source={}, output={}, error={error}",
              source_path.display(),
              output_path.display()
            ),
          ),
          VideoAsyncEvent::Preparing { .. }
          | VideoAsyncEvent::Progress { .. }
          | VideoAsyncEvent::Finalizing { .. } => {}
        }
        video_events.push(event);
      }
      EngineEvent::Network(event) => {
        services.network.handle_engine_event(&event);
        match &event {
          NetworkEvent::Started {
            task_id,
            method,
            url,
          } => services.log.debug(
            LogSource::Engine,
            format!(
              "Network task {task_id:?} started: {} {}",
              method.as_str(),
              redact_network_url(url)
            ),
          ),
          NetworkEvent::Finished {
            task_id,
            method,
            response,
          } => services.log.debug(
            LogSource::Engine,
            format!(
              "Network task {task_id:?} finished: {} {} -> {}",
              method.as_str(),
              redact_network_url(&response.final_url),
              response.status
            ),
          ),
          NetworkEvent::Failed {
            task_id,
            method,
            url,
            error,
          } => services.log.warn(
            LogSource::Engine,
            format!(
              "Network task {task_id:?} failed during {}: {} {} ({})",
              error.stage(),
              method.as_str(),
              redact_network_url(url),
              error.code.as_str()
            ),
          ),
          NetworkEvent::Cancelled {
            task_id,
            method,
            url,
          } => services.log.debug(
            LogSource::Engine,
            format!(
              "Network task {task_id:?} cancelled: {} {}",
              method.as_str(),
              redact_network_url(url)
            ),
          ),
        }
        network_events.push(event);
      }
      EngineEvent::Audio(event) => {
        services.audio.handle_engine_event(&event);
        services
          .recording
          .handle_audio_event(&event, &services.async_runtime);
        match &event {
          AudioAsyncEvent::Failed {
            pool_id,
            audio_id,
            error,
          } => services.log.warn(
            LogSource::Audio,
            format!(
              "Audio object {audio_id:?} in pool {pool_id:?} failed: {}",
              error.code.as_str()
            ),
          ),
          AudioAsyncEvent::BackendFailed { error } => services.log.warn(
            LogSource::Audio,
            format!("Audio backend unavailable: {}", error.code.as_str()),
          ),
          AudioAsyncEvent::CaptureFailed {
            capture_id,
            path,
            error,
          } => services.log.warn(
            LogSource::Audio,
            format!(
              "Audio capture {capture_id:?} failed for {}: {}",
              path.display(),
              error.code.as_str()
            ),
          ),
          AudioAsyncEvent::Ready { .. }
          | AudioAsyncEvent::Started { .. }
          | AudioAsyncEvent::Paused { .. }
          | AudioAsyncEvent::Resumed { .. }
          | AudioAsyncEvent::Stopped { .. }
          | AudioAsyncEvent::Finished { .. }
          | AudioAsyncEvent::CaptureSaved { .. } => {}
        }
      }
      EngineEvent::File(_)
      | EngineEvent::Image(_)
      | EngineEvent::Time(_)
      | EngineEvent::TaskFinished { .. } => {}
      EngineEvent::TaskFailed { id, error } => {
        services.log.warn(
          LogSource::Engine,
          format!("Async task {id:?} failed: {error}"),
        );
      }
      EngineEvent::Log { source, message } => {
        services.log.warn(source, message);
      }
    }
  }

  RuntimeEngineEvents {
    package: package_events,
    export: export_events,
    screenshot: screenshot_events,
    video: video_events,
    network: network_events,
  }
}

fn redact_network_url(url: &str) -> String {
  let Ok(mut url) = reqwest::Url::parse(url) else {
    return "<invalid-url>".to_string();
  };
  url.set_query(None);
  url.set_fragment(None);
  url.to_string()
}
