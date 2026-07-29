use crate::host_engine::core::{ExitState, RuntimeWorld, set_crash_phase};
use crate::host_engine::services::EngineServices;

use super::services::LogSource;

/// 执行引擎关闭流程：记录日志并退出终端
pub fn close(services: &mut EngineServices, mut world: RuntimeWorld, _exit_state: ExitState) {
  let write_barrier = services.async_runtime.write_barrier();
  write_barrier.stop_new_writes();
  services.async_runtime.stop_all_managed_threads();
  write_barrier.wait();
  services.async_runtime.shutdown();
  for (path, error) in write_barrier.snapshot().failed {
    services.log.error(
      LogSource::Shutdown,
      format!("Asynchronous write failed for {}: {error}", path.display()),
    );
  }

  services.screensaver.stop();
  let stop_data = services.game.stop(true);
  for error in stop_data.save_errors {
    services.log.error(LogSource::Lua, error.to_string());
  }
  let _ = services.input_method.release_input_method();

  services
    .log
    .info(LogSource::Shutdown, "[Shutdown] Engine closed.");

  services.terminal.exit();
  world.state.enter_stopped();
  set_crash_phase(world.state.crash_phase());
}
