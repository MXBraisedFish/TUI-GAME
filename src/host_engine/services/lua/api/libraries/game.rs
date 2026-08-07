use super::*;

pub(super) fn game(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, command) in [
    ("exit_game", LuaHostCommand::ExitGame),
    ("save_game", LuaHostCommand::SaveGame),
    ("save_best", LuaHostCommand::SaveBest),
  ] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = match command {
          LuaHostCommand::ExitGame => "game.exit_game",
          LuaHostCommand::SaveGame => "game.save_game",
          _ => "game.save_best",
        };
        let mut api = state.borrow_mut();
        if api.context.session_kind != LuaSessionKind::Game {
          ignore_once(&mut api, method, "game API is unavailable to screensavers");
          return Ok(());
        }
        drop(api);
        args::no_args(method, values)?;
        let mut api = state.borrow_mut();
        if api.phase == LuaCallPhase::Save
          && matches!(command, LuaHostCommand::SaveGame | LuaHostCommand::SaveBest)
        {
          ignore_once(&mut api, method, "recursive save request was ignored")
        } else {
          push_host_command(&mut api, command.clone())
        }
        Ok(())
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}
