use super::*;

const LUA_API_LIBRARY_NAMES: [&str; 15] = [
  "base",
  "math",
  "string",
  "utf8",
  "table",
  "align",
  "char",
  "color",
  "measurement",
  "draw",
  "debug",
  "game",
  "event",
  "loader",
  "file",
];

pub(super) fn loader(lua: &Lua, environment: &Table, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, return_instance) in [("load_execute", false), ("load", true)] {
    let method = if return_instance {
      "loader.load"
    } else {
      "loader.load_execute"
    };
    let environment = environment.clone();
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        execute_loaded_module(lua, &environment, &state, method, values, return_instance)
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}

fn execute_loaded_module(
  lua: &Lua,
  environment: &Table,
  state: &SharedApiState,
  method: &str,
  values: MultiValue,
  return_instance: bool,
) -> mlua::Result<MultiValue> {
  let value = args::one(method, "path", values)?;
  let virtual_path = args::string(value, method, "path")?;
  if virtual_path.is_empty() || virtual_path.len() > 8192 || virtual_path.contains('\0') {
    return Err(args::message(method, "invalid module path"));
  }
  let scripts_root = state.borrow().context.scripts_root.clone();
  let path = resolve_loader_path(&scripts_root, &virtual_path, method)?;
  {
    let mut api = state.borrow_mut();
    if api.loader_stack.len() >= 16 {
      return Err(args::message(method, "module nesting exceeds 16 levels"));
    }
    if api.loader_stack.contains(&path) {
      return Err(args::message(method, "cyclic module load detected"));
    }
    if api.loader_stack.is_empty() {
      api.loader_source_bytes = 0;
    }
    api.loader_stack.push(path.clone());
  }

  let result = (|| {
    let bytes = fs::read(&path).map_err(|_| args::message(method, "module could not be read"))?;
    if bytes.len() > 1024 * 1024 {
      return Err(args::message(method, "module source exceeds 1 MiB"));
    }
    if bytes.first() == Some(&0x1b) {
      return Err(args::message(method, "Lua bytecode is not allowed"));
    }
    {
      let mut api = state.borrow_mut();
      api.loader_source_bytes = api.loader_source_bytes.saturating_add(bytes.len());
      if api.loader_source_bytes > 4 * 1024 * 1024 {
        return Err(args::message(method, "module chain source exceeds 4 MiB"));
      }
    }
    let source = String::from_utf8(bytes)
      .map_err(|_| args::message(method, "module source must be UTF-8 text"))?;
    let module = lua.create_table()?;
    for name in LUA_API_LIBRARY_NAMES {
      module.raw_set(name, environment.get::<Value>(name)?)?;
    }
    let display_name = format!("@scripts/{}", virtual_path.replace('\\', "/"));
    let returned = lua
      .load(&source)
      .set_name(display_name)
      .set_mode(mlua::chunk::ChunkMode::Text)
      .set_environment(module.clone())
      .call::<MultiValue>(())?;
    if return_instance {
      if let Some(Value::Table(table)) = returned.front() {
        Ok(MultiValue::from_vec(vec![Value::Table(table.clone())]))
      } else {
        Ok(MultiValue::from_vec(vec![Value::Table(module)]))
      }
    } else {
      Ok(returned)
    }
  })();

  let mut api = state.borrow_mut();
  api.loader_stack.pop();
  if api.loader_stack.is_empty() {
    api.loader_source_bytes = 0;
  }
  result
}

fn resolve_loader_path(root: &Path, input: &str, method: &str) -> mlua::Result<PathBuf> {
  let mut relative = PathBuf::new();
  for component in Path::new(input).components() {
    match component {
      Component::Normal(value) => relative.push(value),
      _ => return Err(args::message(method, "module path must be relative")),
    }
  }
  if relative.extension().is_none() {
    relative.set_extension("lua");
  }
  if !relative
    .extension()
    .and_then(|value| value.to_str())
    .is_some_and(|value| value.eq_ignore_ascii_case("lua"))
  {
    return Err(args::message(method, "module path must use .lua"));
  }
  let root = root
    .canonicalize()
    .map_err(|_| args::message(method, "scripts root is unavailable"))?;
  let candidate = root
    .join(relative)
    .canonicalize()
    .map_err(|_| args::message(method, "module was not found"))?;
  if !candidate.starts_with(&root) || !candidate.is_file() {
    return Err(args::message(method, "module path escapes scripts root"));
  }
  Ok(candidate)
}
