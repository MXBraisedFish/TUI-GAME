use crate::host_engine::core::{ExitState, RuntimeWorld, set_crash_phase};
use crate::host_engine::services::EngineServices;

use super::services::LogSource;

/// 执行引擎关闭流程：记录日志并退出终端
pub fn close(services: &mut EngineServices, mut world: RuntimeWorld, _exit_state: ExitState) {
  let write_barrier = services.async_runtime.write_barrier();
  write_barrier.stop_new_writes();
  services.async_runtime.stop_all_managed_threads();
  services.audio.shutdown();
  write_barrier.wait();
  services.async_runtime.shutdown();
  for (path, error) in write_barrier.snapshot().failed {
    services.log.error(
      LogSource::Shutdown,
      format!("Asynchronous write failed for {}: {error}", path.display()),
    );
  }

  if let Some(id) = services.screensaver.stop() {
    services.log.close_session(id);
  }
  let stop_data = services.game.stop(true);
  if let Some(package) = stop_data.package.as_ref() {
    let best = stop_data.best.clone().and_then(|value| {
      match crate::host_engine::services::BestGameSave::try_from(value) {
        Ok(best) => Some(best),
        Err(error) => {
          services.log.error_package(package, LogSource::Lua, error);
          None
        }
      }
    });
    let _ =
      services
        .storage
        .write_game_results(package, stop_data.game.clone(), best, &mut services.log);
  }
  for error in &stop_data.save_errors {
    if let Some(id) = stop_data.log_session {
      services
        .log
        .error_session(id, LogSource::Lua, error.to_string());
    } else if let Some(package) = &stop_data.package {
      services
        .log
        .error_package(package, LogSource::Lua, error.to_string());
    } else {
      services.log.error(LogSource::Lua, error.to_string());
    }
  }
  if let Some(id) = stop_data.log_session {
    services.log.close_session(id);
  }
  let _ = services.input_method.release_input_method();

  services
    .log
    .info(LogSource::Shutdown, "[Shutdown] Engine closed.");

  services.terminal.exit();
  world.state.enter_stopped();
  set_crash_phase(world.state.crash_phase());
}
