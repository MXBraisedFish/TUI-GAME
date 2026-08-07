use super::*;

pub(super) fn utf8(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  source.raw_set(
    "len",
    single_text(
      lua,
      "utf8.len",
      |s| Value::Integer(s.chars().count() as i64),
    )?,
  )?;
  source.raw_set(
    "byte_len",
    single_text(lua, "utf8.byte_len", |s| Value::Integer(s.len() as i64))?,
  )?;
  source.raw_set(
    "is_ascii",
    single_text(lua, "utf8.is_ascii", |s| Value::Boolean(s.is_ascii()))?,
  )?;
  source.raw_set(
    "codepoint_to_char",
    lua.create_function(|lua, values: MultiValue| {
      let table = args::named("utf8.codepoint_to_char", values, &["values"])?;
      let values = args::values(&table, "utf8.codepoint_to_char")?;
      let mut output = String::new();
      for value in values {
        let cp = args::integer(value, "utf8.codepoint_to_char", "values")?;
        let Some(ch) = char::from_u32(cp.try_into().unwrap_or(u32::MAX)) else {
          return Err(args::message(
            "utf8.codepoint_to_char",
            "invalid Unicode codepoint",
          ));
        };
        output.push(ch);
        if output.len() > args::MAX_API_STRING_BYTES {
          return Err(args::message(
            "utf8.codepoint_to_char",
            "output exceeds 1 MiB",
          ));
        }
      }
      lua.create_string(output)
    })?,
  )?;
  source.raw_set(
    "ascii_to_char",
    lua.create_function(|lua, values: MultiValue| {
      let table = args::named("utf8.ascii_to_char", values, &["values"])?;
      let values = args::values(&table, "utf8.ascii_to_char")?;
      let mut output = Vec::with_capacity(values.len());
      for value in values {
        let value = args::integer(value, "utf8.ascii_to_char", "values")?;
        if !(0..=127).contains(&value) {
          return Err(args::message(
            "utf8.ascii_to_char",
            "ASCII values must be in 0..=127",
          ));
        }
        output.push(value as u8);
      }
      lua.create_string(output)
    })?,
  )?;
  source.raw_set(
    "char_to_codepoint",
    lua.create_function(|_, values: MultiValue| utf8_codes(values, false))?,
  )?;
  source.raw_set(
    "char_to_ascii",
    lua.create_function(|_, values: MultiValue| utf8_codes(values, true))?,
  )?;
  source.raw_set(
    "char_position",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("utf8.char_position", values, &["text", "index", "start"])?;
      let text = args::string(
        args::required(&table, "utf8.char_position", "text")?,
        "utf8.char_position",
        "text",
      )?;
      let index = args::integer(
        args::required(&table, "utf8.char_position", "index")?,
        "utf8.char_position",
        "index",
      )?;
      let start = args::optional_integer(&table, "utf8.char_position", "start", Some(1))?.unwrap();
      let positions = text.char_indices().map(|(i, _)| i + 1).collect::<Vec<_>>();
      let start = resolve_index(start, positions.len(), true);
      let target = start.and_then(|start| {
        let value = start as i128 + index as i128 - 1;
        (value >= 0 && value <= usize::MAX as i128).then_some(value as usize)
      });
      Ok(
        target
          .and_then(|i| positions.get(i).copied())
          .map(|v| Value::Integer(v as i64))
          .unwrap_or(Value::Nil),
      )
    })?,
  )?;
  source.raw_set(
    "codepoints",
    lua.create_function(|lua, values: MultiValue| {
      let text = args::string(
        args::one("utf8.codepoints", "text", values)?,
        "utf8.codepoints",
        "text",
      )?;
      let items = text
        .char_indices()
        .map(|(i, ch)| (i as i64 + 1, ch as i64))
        .collect::<Vec<_>>();
      let index = std::rc::Rc::new(std::cell::Cell::new(0usize));
      lua.create_function(move |_, (): ()| {
        let i = index.get();
        if let Some((pos, cp)) = items.get(i) {
          index.set(i + 1);
          Ok((Value::Integer(*pos), Value::Integer(*cp)))
        } else {
          Ok((Value::Nil, Value::Nil))
        }
      })
    })?,
  )?;
  source.raw_set(
    "next",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("utf8.next", values, &["text", "pos"])?;
      let text = args::string(
        args::required(&table, "utf8.next", "text")?,
        "utf8.next",
        "text",
      )?;
      let pos = args::integer(
        args::required(&table, "utf8.next", "pos")?,
        "utf8.next",
        "pos",
      )?;
      let found = text
        .char_indices()
        .find(|(i, _)| *i + 1 > pos.max(0) as usize);
      Ok(
        found
          .map(|(i, ch)| (Value::Integer(ch as i64), Value::Integer(i as i64 + 1)))
          .unwrap_or((Value::Nil, Value::Nil)),
      )
    })?,
  )?;
  readonly::proxy(lua, source)
}

pub(super) fn single_text(
  lua: &Lua,
  method: &'static str,
  operation: fn(&str) -> Value,
) -> mlua::Result<Function> {
  lua.create_function(move |_, values: MultiValue| {
    let text = args::string(args::one(method, "text", values)?, method, "text")?;
    Ok(operation(&text))
  })
}
pub(super) fn resolve_index(index: i64, len: usize, allow_end: bool) -> Option<usize> {
  let value = if index < 0 {
    len as i64 + index
  } else {
    index - 1
  };
  let max = if allow_end {
    len
  } else {
    len.saturating_sub(1)
  };
  (value >= 0 && (value as usize) <= max).then_some(value as usize)
}
pub(super) fn utf8_codes(values: MultiValue, ascii_only: bool) -> mlua::Result<MultiValue> {
  let method = if ascii_only {
    "utf8.char_to_ascii"
  } else {
    "utf8.char_to_codepoint"
  };
  let table = args::named(method, values, &["text", "start", "finish"])?;
  let text = args::string(args::required(&table, method, "text")?, method, "text")?;
  let chars = text.chars().collect::<Vec<_>>();
  let start = args::optional_integer(&table, method, "start", Some(1))?.unwrap();
  let finish = args::optional_integer(&table, method, "finish", Some(start))?.unwrap();
  let Some(start) = resolve_index(start, chars.len(), false) else {
    return Ok(MultiValue::new());
  };
  let Some(finish) = resolve_index(finish, chars.len(), false) else {
    return Ok(MultiValue::new());
  };
  let mut result = Vec::new();
  for ch in chars[start..=finish].iter().copied() {
    if ascii_only && !ch.is_ascii() {
      result.push(Value::Nil)
    } else {
      result.push(Value::Integer(ch as i64))
    }
  }
  Ok(MultiValue::from_vec(result))
}
