use super::*;

pub(super) fn file(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("AUTO", "auto"),
    ("ALL", "all"),
    ("CR", "cr"),
    ("LF", "lf"),
    ("CRLF", "crlf"),
    ("UTF_8", "utf-8"),
    ("UTF_16LE", "utf-16le"),
    ("UTF_16BE", "utf-16be"),
    ("GBK", "gbk"),
    ("GB18030", "gb18030"),
    ("BIG5", "big5"),
    ("SHIFT_JIS", "shift_jis"),
    ("EUC_JP", "euc-jp"),
    ("ISO_2022_JP", "iso-2022-jp"),
    ("EUC_KR", "euc-kr"),
    ("WINDOWS_874", "windows-874"),
    ("WINDOWS_1250", "windows-1250"),
    ("WINDOWS_1251", "windows-1251"),
    ("WINDOWS_1252", "windows-1252"),
    ("WINDOWS_1253", "windows-1253"),
    ("WINDOWS_1254", "windows-1254"),
    ("WINDOWS_1255", "windows-1255"),
    ("WINDOWS_1256", "windows-1256"),
    ("WINDOWS_1257", "windows-1257"),
    ("WINDOWS_1258", "windows-1258"),
    ("ISO_8859_2", "iso-8859-2"),
    ("ISO_8859_3", "iso-8859-3"),
    ("ISO_8859_4", "iso-8859-4"),
    ("ISO_8859_5", "iso-8859-5"),
    ("ISO_8859_6", "iso-8859-6"),
    ("ISO_8859_7", "iso-8859-7"),
    ("ISO_8859_8", "iso-8859-8"),
    ("ISO_8859_8_I", "iso-8859-8-i"),
    ("ISO_8859_10", "iso-8859-10"),
    ("ISO_8859_13", "iso-8859-13"),
    ("ISO_8859_14", "iso-8859-14"),
    ("ISO_8859_15", "iso-8859-15"),
    ("ISO_8859_16", "iso-8859-16"),
    ("KOI8_R", "koi8-r"),
    ("KOI8_U", "koi8-u"),
    ("IBM866", "ibm866"),
    ("MACINTOSH", "macintosh"),
    ("X_MAC_CYRILLIC", "x-mac-cyrillic"),
  ] {
    source.raw_set(name, value)?;
  }
  let read_state = state.clone();
  source.raw_set(
    "read",
    lua.create_function(move |_, values: MultiValue| {
      let method = "file.read";
      let table = args::named(
        method,
        values,
        &["path", "encoding", "end_of_line", "event_tip"],
      )?;
      let virtual_path = file_path(&table, method)?;
      validate_file_eol(&table, method)?;
      let encoding = file_encoding(&table, method)?;
      let event_tip = file_tip(&table, method)?;
      let path = resolve_asset_path(
        &read_state.borrow().context.assets_root,
        &virtual_path,
        AssetPathKind::File,
        method,
      )?;
      enqueue_file_request(
        &read_state,
        FileTask::LuaReadText { path, encoding },
        LuaFileOperation::ReadText,
        virtual_path,
        event_tip,
      );
      Ok(())
    })?,
  )?;
  let write_state = state.clone();
  source.raw_set(
    "write",
    lua.create_function(move |_, values: MultiValue| {
      let method = "file.write";
      if !file_permission(&write_state, method) {
        return Ok(());
      }
      let table = args::named(
        method,
        values,
        &["path", "text", "encoding", "end_of_line", "event_tip"],
      )?;
      let virtual_path = file_path(&table, method)?;
      let text = args::string(args::required(&table, method, "text")?, method, "text")?;
      if text.len() > args::MAX_API_STRING_BYTES || text.contains('\0') {
        return Err(args::message(
          method,
          "text must be at most 1 MiB and contain no NUL",
        ));
      }
      let encoding = file_encoding(&table, method)?;
      let end_of_line = validate_file_eol(&table, method)?;
      let event_tip = file_tip(&table, method)?;
      let path = resolve_asset_path(
        &write_state.borrow().context.assets_root,
        &virtual_path,
        AssetPathKind::WritableFile,
        method,
      )?;
      enqueue_file_request(
        &write_state,
        FileTask::LuaWriteText {
          path,
          text,
          encoding,
          end_of_line,
        },
        LuaFileOperation::WriteText,
        virtual_path,
        event_tip,
      );
      Ok(())
    })?,
  )?;
  let list_state = state.clone();
  source.raw_set(
    "list_dir",
    lua.create_function(move |_, values: MultiValue| {
      let method = "file.list_dir";
      if !file_permission(&list_state, method) {
        return Ok(());
      }
      let table = args::named(
        method,
        values,
        &["path", "recursive", "file_type", "event_tip"],
      )?;
      let virtual_path = file_path(&table, method)?;
      let recursive = match table.get::<Value>("recursive")? {
        Value::Nil => false,
        value => args::boolean(value, method, "recursive")?,
      };
      let file_type = match table.get::<Value>("file_type")? {
        Value::Nil => None,
        value => {
          let value = args::string(value, method, "file_type")?;
          if value.eq_ignore_ascii_case("all") {
            None
          } else if value.is_empty()
            || value.starts_with('.')
            || !value
              .chars()
              .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '+'))
          {
            return Err(args::message(
              method,
              "file_type must be an extension such as 'rs'",
            ));
          } else {
            Some(value.to_ascii_lowercase())
          }
        }
      };
      let event_tip = file_tip(&table, method)?;
      let path = resolve_asset_path(
        &list_state.borrow().context.assets_root,
        &virtual_path,
        AssetPathKind::Directory,
        method,
      )?;
      enqueue_file_request(
        &list_state,
        FileTask::LuaListDir {
          path,
          recursive,
          file_type,
        },
        LuaFileOperation::ListDir,
        virtual_path,
        event_tip,
      );
      Ok(())
    })?,
  )?;
  readonly::proxy(lua, source)
}

#[derive(Clone, Copy)]
enum AssetPathKind {
  File,
  WritableFile,
  Directory,
}

pub(super) fn file_permission(state: &SharedApiState, method: &'static str) -> bool {
  let mut state = state.borrow_mut();
  if state.context.session_kind == LuaSessionKind::Game && !state.context.safe_mode_enabled {
    true
  } else {
    ignore_once(
      &mut state,
      method,
      "method requires a game with safe mode disabled",
    );
    false
  }
}

pub(super) fn file_path(table: &Table, method: &str) -> mlua::Result<String> {
  let path = args::string(args::required(table, method, "path")?, method, "path")?;
  if path.is_empty() || path.len() > 8192 || path.contains('\0') {
    return Err(args::message(method, "invalid asset path"));
  }
  Ok(path.replace('\\', "/"))
}

pub(super) fn file_encoding(table: &Table, method: &str) -> mlua::Result<String> {
  let encoding = match table.get::<Value>("encoding")? {
    Value::Nil => "auto".to_string(),
    value => args::string(value, method, "encoding")?.to_ascii_lowercase(),
  };
  let valid = encoding == "auto"
    || encoding == "utf-16le"
    || encoding == "utf-16be"
    || encoding_rs::Encoding::for_label(encoding.as_bytes()).is_some();
  if !valid {
    return Err(args::message(method, "unsupported text encoding"));
  }
  Ok(encoding)
}

fn validate_file_eol(table: &Table, method: &str) -> mlua::Result<String> {
  let value = match table.get::<Value>("end_of_line")? {
    Value::Nil => "auto".to_string(),
    value => args::string(value, method, "end_of_line")?.to_ascii_lowercase(),
  };
  if !matches!(value.as_str(), "auto" | "cr" | "lf" | "crlf") {
    return Err(args::message(method, "unsupported end_of_line value"));
  }
  Ok(value)
}

pub(super) fn file_tip(table: &Table, method: &str) -> mlua::Result<Option<String>> {
  match table.get::<Value>("event_tip")? {
    Value::Nil => Ok(None),
    value => {
      let value = args::string(value, method, "event_tip")?;
      if value.len() > 4096 {
        Err(args::message(method, "event_tip exceeds 4 KiB"))
      } else {
        Ok(Some(value))
      }
    }
  }
}

fn resolve_asset_path(
  root: &Path,
  input: &str,
  kind: AssetPathKind,
  method: &str,
) -> mlua::Result<PathBuf> {
  let mut relative = PathBuf::new();
  for component in Path::new(input).components() {
    match component {
      Component::Normal(value) => relative.push(value),
      Component::CurDir => {}
      _ => return Err(args::message(method, "asset path must be relative")),
    }
  }
  let root = root
    .canonicalize()
    .map_err(|_| args::message(method, "assets root is unavailable"))?;
  let candidate = root.join(relative);
  let resolved = if candidate.exists() {
    candidate
      .canonicalize()
      .map_err(|_| args::message(method, "asset path could not be resolved"))?
  } else if matches!(kind, AssetPathKind::WritableFile) {
    let parent = candidate
      .parent()
      .ok_or_else(|| args::message(method, "asset path has no parent"))?
      .canonicalize()
      .map_err(|_| args::message(method, "asset parent does not exist"))?;
    if !parent.starts_with(&root) {
      return Err(args::message(method, "asset path escapes assets root"));
    }
    candidate
  } else {
    return Err(args::message(method, "asset path was not found"));
  };
  if !resolved.starts_with(&root) {
    return Err(args::message(method, "asset path escapes assets root"));
  }
  match kind {
    AssetPathKind::File if !resolved.is_file() => {
      Err(args::message(method, "asset path is not a file"))
    }
    AssetPathKind::Directory if !resolved.is_dir() => {
      Err(args::message(method, "asset path is not a directory"))
    }
    _ => Ok(resolved),
  }
}

fn enqueue_file_request(
  state: &SharedApiState,
  task: FileTask,
  operation: LuaFileOperation,
  virtual_path: String,
  event_tip: Option<String>,
) {
  let mut state = state.borrow_mut();
  let request_id = state.next_file_request_id;
  state.next_file_request_id = state.next_file_request_id.wrapping_add(1).max(1);
  push_host_command(
    &mut state,
    LuaHostCommand::FileRequest {
      request_id,
      task,
      operation,
      virtual_path,
      event_tip,
    },
  );
}
