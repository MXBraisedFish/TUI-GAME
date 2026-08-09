use super::HostState;
use crate::host_engine::core::PackageId;

/// 游戏状态仅保存宿主状态机需要的身份与返回点；Lua VM 由 GameService 持有。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
  pub package: PackageId,
  pub min_width: u32,
  pub min_height: u32,
  pub target_fps: u32,
  pub game_loop: GameLoopState,
  pub return_host: Box<HostState>,
}

/// 游戏循环状态
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameLoopState;

impl GameState {
  pub fn new(
    package: PackageId,
    min_width: u32,
    min_height: u32,
    target_fps: u32,
    return_host: HostState,
  ) -> Self {
    Self {
      package,
      min_width,
      min_height,
      target_fps,
      game_loop: GameLoopState,
      return_host: Box::new(return_host),
    }
  }

  pub fn game_loop(&self) -> &GameLoopState {
    &self.game_loop
  }

  pub fn game_loop_mut(&mut self) -> &mut GameLoopState {
    &mut self.game_loop
  }
}
