use super::*;

mod pattern;

use pattern::{LuaCapture, LuaCaptures, LuaPattern};

pub(super) fn string_lib(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("AUTO", "auto"),
    ("PLAIN_TEXT", "plain_text"),
    ("RICH_TEXT", "rich_text"),
  ] {
    source.raw_set(name, value)?;
  }
  for (name, operation) in [
    ("lower", StringUnary::Lower),
    ("upper", StringUnary::Upper),
    ("reverse", StringUnary::Reverse),
    ("regex_escape", StringUnary::RegexEscape),
  ] {
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = format!("string.{name}");
        let text = args::string(args::one(&method, "text", values)?, &method, "text")?;
        Ok(operation.apply(&text))
      })?,
    )?;
  }
  source.raw_set(
    "sub",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("string.sub", values, &["text", "start", "finish"])?;
      let text = args::string(
        args::required(&table, "string.sub", "text")?,
        "string.sub",
        "text",
      )?;
      let chars = text.chars().collect::<Vec<_>>();
      let start = args::integer(
        args::required(&table, "string.sub", "start")?,
        "string.sub",
        "start",
      )?;
      let finish =
        args::optional_integer(&table, "string.sub", "finish", Some(chars.len() as i64))?.unwrap();
      let Some(start) = resolve_index(start, chars.len(), true) else {
        return Ok(String::new());
      };
      let Some(finish) = resolve_index(finish, chars.len(), false) else {
        return Ok(String::new());
      };
      if start > finish {
        return Ok(String::new());
      }
      Ok(chars[start..=finish].iter().collect())
    })?,
  )?;
  source.raw_set(
    "rep",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("string.rep", values, &["text", "times", "sep"])?;
      let text = args::string(
        args::required(&table, "string.rep", "text")?,
        "string.rep",
        "text",
      )?;
      let times = args::integer(
        args::required(&table, "string.rep", "times")?,
        "string.rep",
        "times",
      )?;
      let sep = args::optional_string(&table, "string.rep", "sep", Some(""))?.unwrap();
      if times < 0 {
        return Err(args::message("string.rep", "times must be non-negative"));
      }
      let size = text
        .len()
        .checked_mul(times as usize)
        .and_then(|v| v.checked_add(sep.len().saturating_mul(times.saturating_sub(1) as usize)))
        .ok_or_else(|| args::message("string.rep", "output size overflow"))?;
      if size > args::MAX_API_STRING_BYTES {
        return Err(args::message("string.rep", "output exceeds 1 MiB"));
      }
      Ok(
        std::iter::repeat_n(text, times as usize)
          .collect::<Vec<_>>()
          .join(&sep),
      )
    })?,
  )?;
  source.raw_set("find", string_find(lua, false)?)?;
  source.raw_set("match", string_match(lua, false)?)?;
  source.raw_set("gmatch", string_gmatch(lua, false)?)?;
  source.raw_set("gsub", string_gsub(lua, false)?)?;
  source.raw_set("regex_find", string_find(lua, true)?)?;
  source.raw_set("regex_match", string_match(lua, true)?)?;
  source.raw_set("regex_gmatch", string_gmatch(lua, true)?)?;
  source.raw_set("regex_gsub", string_gsub(lua, true)?)?;
  source.raw_set(
    "regex_test",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named("string.regex_test", values, &["text", "pattern"])?;
      let text = text_parameter(&parameters, "string.regex_test")?;
      let pattern = pattern_parameter(&parameters, "string.regex_test", true)?;
      Ok(
        pattern
          .captures(&text, 0)
          .map_err(|message| args::message("string.regex_test", message))?
          .is_some(),
      )
    })?,
  )?;
  source.raw_set(
    "regex_split",
    lua.create_function(|lua, values: MultiValue| {
      let parameters = args::named("string.regex_split", values, &["text", "pattern"])?;
      let text = text_parameter(&parameters, "string.regex_split")?;
      let pattern = pattern_parameter(&parameters, "string.regex_split", true)?;
      let output = lua.create_table()?;
      let mut total = 0_usize;
      for (index, part) in pattern
        .split(&text)
        .map_err(|message| args::message("string.regex_split", message))?
        .into_iter()
        .take(10_001)
        .enumerate()
      {
        if index == 10_000 {
          return Err(args::message(
            "string.regex_split",
            "result exceeds 10000 items",
          ));
        }
        total = total.saturating_add(part.len());
        if total > args::MAX_API_STRING_BYTES {
          return Err(args::message("string.regex_split", "output exceeds 1 MiB"));
        }
        output.raw_set(index + 1, part)?;
      }
      Ok(output)
    })?,
  )?;
  source.raw_set(
    "format",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named("string.format", values, &["format_string", "values"])?;
      let format_string = args::string(
        args::required(&parameters, "string.format", "format_string")?,
        "string.format",
        "format_string",
      )?;
      let values = match parameters.get::<Value>("values")? {
        Value::Nil => Vec::new(),
        _ => args::values(&parameters, "string.format")?,
      };
      safe_format(&format_string, &values)
    })?,
  )?;
  source.raw_set(
    "rich_text_to_plain_text",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named(
        "string.rich_text_to_plain_text",
        values,
        &["text", "rich_params", "strip_header"],
      )?;
      let text = text_parameter(&parameters, "string.rich_text_to_plain_text")?;
      let strip_header = args::optional_bool(
        &parameters,
        "string.rich_text_to_plain_text",
        "strip_header",
        true,
      )?;
      let params = rich_text_params(
        parameters.get::<Value>("rich_params")?,
        "string.rich_text_to_plain_text",
      )?;
      let input = if strip_header {
        text.strip_prefix("f%").unwrap_or(&text)
      } else {
        &text
      };
      Ok(
        crate::host_engine::services::RichTextService::new()
          .visible_text(&format!("f%{input}"), params.as_ref()),
      )
    })?,
  )?;
  readonly::proxy(lua, source)
}

pub(super) fn text_parameter(parameters: &Table, method: &str) -> mlua::Result<String> {
  args::string(args::required(parameters, method, "text")?, method, "text")
}

enum CompiledPattern {
  Lua(LuaPattern),
  Regex(Regex),
}

#[derive(Clone)]
enum PatternCapture {
  Text(Range<usize>),
  Position(usize),
}

#[derive(Clone)]
struct PatternCaptures {
  full: Range<usize>,
  captures: Vec<Option<PatternCapture>>,
}

impl PatternCaptures {
  fn len(&self) -> usize {
    self.captures.len() + 1
  }

  fn value(&self, index: usize) -> Option<PatternCapture> {
    if index == 0 {
      Some(PatternCapture::Text(self.full.clone()))
    } else {
      self.captures.get(index - 1).cloned().flatten()
    }
  }
}

impl From<LuaCaptures> for PatternCaptures {
  fn from(value: LuaCaptures) -> Self {
    Self {
      full: value.full,
      captures: value
        .captures
        .into_iter()
        .map(|capture| {
          capture.map(|capture| match capture {
            LuaCapture::Text(range) => PatternCapture::Text(range),
            LuaCapture::Position(position) => PatternCapture::Position(position),
          })
        })
        .collect(),
    }
  }
}

impl CompiledPattern {
  fn captures(&self, text: &str, offset: usize) -> Result<Option<PatternCaptures>, String> {
    match self {
      Self::Lua(pattern) => pattern
        .captures(text, text[..offset].chars().count())
        .map(|capture| capture.map(Into::into)),
      Self::Regex(pattern) => Ok(pattern.captures(&text[offset..]).map(|captures| {
        let full = captures.get(0).unwrap();
        PatternCaptures {
          full: offset + full.start()..offset + full.end(),
          captures: (1..captures.len())
            .map(|index| {
              captures.get(index).map(|capture| {
                PatternCapture::Text(offset + capture.start()..offset + capture.end())
              })
            })
            .collect(),
        }
      })),
    }
  }

  fn captures_iter(&self, text: &str) -> Result<Vec<PatternCaptures>, String> {
    match self {
      Self::Lua(pattern) => pattern
        .captures_iter(text)
        .map(|captures| captures.into_iter().map(Into::into).collect()),
      Self::Regex(pattern) => Ok(
        pattern
          .captures_iter(text)
          .take(10_001)
          .map(|captures| {
            let full = captures.get(0).unwrap();
            PatternCaptures {
              full: full.start()..full.end(),
              captures: (1..captures.len())
                .map(|index| {
                  captures
                    .get(index)
                    .map(|capture| PatternCapture::Text(capture.start()..capture.end()))
                })
                .collect(),
            }
          })
          .collect(),
      ),
    }
  }

  fn split(&self, text: &str) -> Result<Vec<String>, String> {
    let captures = self.captures_iter(text)?;
    let mut output = Vec::with_capacity(captures.len() + 1);
    let mut last = 0;
    for capture in captures {
      output.push(text[last..capture.full.start].to_string());
      last = capture.full.end;
    }
    output.push(text[last..].to_string());
    Ok(output)
  }
}

fn pattern_parameter(
  parameters: &Table,
  method: &str,
  regex: bool,
) -> mlua::Result<CompiledPattern> {
  let pattern = args::string(
    args::required(parameters, method, "pattern")?,
    method,
    "pattern",
  )?;
  if pattern.len() > 8 * 1024 {
    return Err(args::message(method, "pattern exceeds 8 KiB"));
  }
  if regex {
    RegexBuilder::new(&pattern)
      .size_limit(1024 * 1024)
      .build()
      .map(CompiledPattern::Regex)
      .map_err(|error| args::message(method, format!("invalid pattern: {error}")))
  } else {
    LuaPattern::compile(&pattern)
      .map(CompiledPattern::Lua)
      .map_err(|message| args::message(method, format!("invalid pattern: {message}")))
  }
}

fn search_start(text: &str, index: i64, method: &str) -> mlua::Result<usize> {
  let chars = text
    .char_indices()
    .map(|(index, _)| index)
    .collect::<Vec<_>>();
  if index == 0 {
    return Err(args::message(method, "init must not be zero"));
  }
  let resolved = if index > 0 {
    (index - 1) as i128
  } else {
    chars.len() as i128 + index as i128
  };
  if resolved <= 0 {
    return Ok(0);
  }
  if resolved as usize >= chars.len() {
    return Ok(text.len());
  }
  Ok(chars[resolved as usize])
}

fn char_position(text: &str, byte: usize) -> i64 {
  text[..byte].chars().count() as i64 + 1
}

fn string_find(lua: &Lua, regex_mode: bool) -> mlua::Result<Function> {
  lua.create_function(move |lua, values: MultiValue| {
    let method = if regex_mode {
      "string.regex_find"
    } else {
      "string.find"
    };
    let allowed = if regex_mode {
      &["text", "pattern", "init"][..]
    } else {
      &["text", "pattern", "init", "plain"][..]
    };
    let parameters = args::named(method, values, allowed)?;
    let text = text_parameter(&parameters, method)?;
    let init = args::optional_integer(&parameters, method, "init", Some(1))?.unwrap();
    let offset = search_start(&text, init, method)?;
    let plain = !regex_mode && args::optional_bool(&parameters, method, "plain", false)?;
    if plain {
      let needle = args::string(
        args::required(&parameters, method, "pattern")?,
        method,
        "pattern",
      )?;
      let Some(found) = text[offset..].find(&needle) else {
        return Ok(MultiValue::from_vec(vec![Value::Nil]));
      };
      let start = offset + found;
      let finish = start + needle.len();
      return Ok(MultiValue::from_vec(vec![
        Value::Integer(char_position(&text, start)),
        Value::Integer(text[..finish].chars().count() as i64),
      ]));
    }
    let pattern = pattern_parameter(&parameters, method, regex_mode)?;
    let Some(captures) = pattern
      .captures(&text, offset)
      .map_err(|message| args::message(method, message))?
    else {
      return Ok(MultiValue::from_vec(vec![Value::Nil]));
    };
    let start = captures.full.start;
    let finish = captures.full.end;
    let mut result = vec![
      Value::Integer(char_position(&text, start)),
      Value::Integer(text[..finish].chars().count() as i64),
    ];
    if regex_mode {
      let capture_table = lua.create_table()?;
      for index in 1..captures.len() {
        if let Some(capture) = captures.value(index) {
          capture_table.raw_set(index, pattern_capture_value(lua, &text, capture)?)?;
        }
      }
      result.push(Value::Table(capture_table));
    } else {
      for index in 1..captures.len() {
        result.push(match captures.value(index) {
          Some(capture) => pattern_capture_value(lua, &text, capture)?,
          None => Value::Nil,
        });
      }
    }
    Ok(MultiValue::from_vec(result))
  })
}

fn string_match(lua: &Lua, regex_mode: bool) -> mlua::Result<Function> {
  lua.create_function(move |lua, values: MultiValue| {
    let method = if regex_mode {
      "string.regex_match"
    } else {
      "string.match"
    };
    let parameters = args::named(method, values, &["text", "pattern", "init"])?;
    let text = text_parameter(&parameters, method)?;
    let init = args::optional_integer(&parameters, method, "init", Some(1))?.unwrap();
    let offset = search_start(&text, init, method)?;
    let pattern = pattern_parameter(&parameters, method, regex_mode)?;
    let Some(captures) = pattern
      .captures(&text, offset)
      .map_err(|message| args::message(method, message))?
    else {
      return Ok(MultiValue::from_vec(vec![Value::Nil]));
    };
    let first = if captures.len() > 1 { 1 } else { 0 };
    let mut result = Vec::new();
    for index in first..captures.len() {
      result.push(match captures.value(index) {
        Some(capture) => pattern_capture_value(lua, &text, capture)?,
        None => Value::Nil,
      });
    }
    Ok(MultiValue::from_vec(result))
  })
}

fn string_gmatch(lua: &Lua, regex_mode: bool) -> mlua::Result<Function> {
  lua.create_function(move |lua, values: MultiValue| {
    let method = if regex_mode {
      "string.regex_gmatch"
    } else {
      "string.gmatch"
    };
    let parameters = args::named(method, values, &["text", "pattern"])?;
    let text = text_parameter(&parameters, method)?;
    let pattern = pattern_parameter(&parameters, method, regex_mode)?;
    let matches = pattern
      .captures_iter(&text)
      .map_err(|message| args::message(method, message))?
      .into_iter()
      .take(10_001)
      .map(|captures| {
        let first = if captures.len() > 1 { 1 } else { 0 };
        (first..captures.len())
          .map(|index| captures.value(index))
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    if matches.len() > 10_000 {
      return Err(args::message(method, "result exceeds 10000 items"));
    }
    let output_bytes = matches
      .iter()
      .flatten()
      .flatten()
      .try_fold(0_usize, |total, capture| {
        let bytes = match capture {
          PatternCapture::Text(range) => range.len(),
          PatternCapture::Position(position) => position.to_string().len(),
        };
        total.checked_add(bytes)
      })
      .ok_or_else(|| args::message(method, "result size overflow"))?;
    if output_bytes > args::MAX_API_STRING_BYTES {
      return Err(args::message(method, "result exceeds 1 MiB"));
    }
    let index = std::rc::Rc::new(std::cell::Cell::new(0_usize));
    lua.create_function(move |lua, (): ()| {
      let current = index.get();
      let Some(values) = matches.get(current) else {
        return Ok(MultiValue::new());
      };
      index.set(current + 1);
      Ok(MultiValue::from_vec(
        values
          .iter()
          .map(|value| match value {
            Some(value) => pattern_capture_value(lua, &text, value.clone()),
            None => Ok(Value::Nil),
          })
          .collect::<mlua::Result<Vec<_>>>()?,
      ))
    })
  })
}

fn string_gsub(lua: &Lua, regex_mode: bool) -> mlua::Result<Function> {
  lua.create_function(move |lua, values: MultiValue| {
    let method = if regex_mode {
      "string.regex_gsub"
    } else {
      "string.gsub"
    };
    let parameters = args::named(method, values, &["text", "pattern", "repl", "limit"])?;
    let text = text_parameter(&parameters, method)?;
    let pattern = pattern_parameter(&parameters, method, regex_mode)?;
    let replacement = args::required(&parameters, method, "repl")?;
    let limit = args::optional_integer(&parameters, method, "limit", None)?;
    if matches!(limit, Some(value) if value < 0) {
      return Err(args::message(method, "limit must be non-negative"));
    }
    let limit = limit.map_or(10_000, |value| value.min(10_000) as usize);
    let mut result = String::with_capacity(text.len());
    let mut last = 0_usize;
    let mut count = 0_usize;
    let captures_list = pattern
      .captures_iter(&text)
      .map_err(|message| args::message(method, message))?;
    for captures in captures_list {
      if count >= limit {
        break;
      }
      result.push_str(&text[last..captures.full.start]);
      let value = replacement_value(lua, &replacement, &captures, &text, regex_mode)?;
      result.push_str(value.as_deref().unwrap_or(&text[captures.full.clone()]));
      if result.len() > args::MAX_API_STRING_BYTES {
        return Err(args::message(method, "output exceeds 1 MiB"));
      }
      last = captures.full.end;
      count += 1;
    }
    result.push_str(&text[last..]);
    if result.len() > args::MAX_API_STRING_BYTES {
      return Err(args::message(method, "output exceeds 1 MiB"));
    }
    Ok((result, count as i64))
  })
}

fn replacement_value(
  lua: &Lua,
  replacement: &Value,
  captures: &PatternCaptures,
  text: &str,
  regex_mode: bool,
) -> mlua::Result<Option<String>> {
  let key = captures.value(1).or_else(|| captures.value(0)).unwrap();
  let key = pattern_capture_value(lua, text, key)?;
  let value = match replacement {
    Value::String(value) => {
      let source = value.to_str()?;
      let mut output = String::new();
      let mut chars = source.chars().peekable();
      while let Some(ch) = chars.next() {
        let marker = if regex_mode { '$' } else { '%' };
        if ch == marker {
          match chars.peek().copied() {
            Some(next @ '0'..='9') => {
              chars.next();
              let index = next.to_digit(10).unwrap() as usize;
              if let Some(value) = captures.value(index) {
                output.push_str(&pattern_capture_text(text, value));
              }
            }
            Some(next) if next == marker => {
              chars.next();
              output.push(marker);
            }
            _ => output.push(ch),
          }
        } else {
          output.push(ch);
        }
      }
      Some(output)
    }
    Value::Table(table) => lua_value_to_replacement(table.get::<Value>(key)?),
    Value::Function(function) => {
      let first = if captures.len() > 1 { 1 } else { 0 };
      let parameters = (first..captures.len())
        .map(|index| match captures.value(index) {
          Some(value) => pattern_capture_value(lua, text, value),
          None => Ok(Value::Nil),
        })
        .collect::<mlua::Result<Vec<_>>>()?;
      lua_value_to_replacement(function.call::<Value>(MultiValue::from_vec(parameters))?)
    }
    value => {
      return Err(args::invalid(
        "string.gsub",
        "repl",
        "string, table, or function",
        value,
      ));
    }
  };
  Ok(value)
}

fn pattern_capture_text(text: &str, capture: PatternCapture) -> String {
  match capture {
    PatternCapture::Text(range) => text[range].to_string(),
    PatternCapture::Position(position) => position.to_string(),
  }
}

fn pattern_capture_value(lua: &Lua, text: &str, capture: PatternCapture) -> mlua::Result<Value> {
  match capture {
    PatternCapture::Text(range) => lua.create_string(&text[range]).map(Value::String),
    PatternCapture::Position(position) => Ok(Value::Integer(position as i64)),
  }
}

fn lua_value_to_replacement(value: Value) -> Option<String> {
  match value {
    Value::String(value) => value.to_str().ok().map(|value| value.to_string()),
    Value::Integer(value) => Some(value.to_string()),
    Value::Number(value) => Some(value.to_string()),
    Value::Boolean(false) | Value::Nil => None,
    _ => None,
  }
}

fn safe_format(format_string: &str, values: &[Value]) -> mlua::Result<String> {
  let mut output = String::new();
  let mut chars = format_string.chars().peekable();
  let mut index = 0_usize;
  while let Some(ch) = chars.next() {
    if ch != '%' {
      output.push(ch);
      continue;
    }
    if chars.peek() == Some(&'%') {
      chars.next();
      output.push('%');
      continue;
    }
    let mut spec = String::new();
    while let Some(next) = chars.peek().copied() {
      if "-+ #0.123456789".contains(next) {
        spec.push(next);
        chars.next();
      } else {
        break;
      }
    }
    let Some(kind) = chars.next() else {
      return Err(args::message(
        "string.format",
        "incomplete format specifier",
      ));
    };
    let value = values
      .get(index)
      .ok_or_else(|| args::message("string.format", "not enough values"))?;
    index += 1;
    let formatted = match kind {
      's' => format_value(value),
      'q' => format!("{:?}", format_value(value)),
      'd' | 'i' => args::integer(value.clone(), "string.format", "values")?.to_string(),
      'u' => u64::try_from(args::integer(value.clone(), "string.format", "values")?)
        .map_err(|_| args::message("string.format", "unsigned value must be non-negative"))?
        .to_string(),
      'x' | 'X' => {
        let value = u64::try_from(args::integer(value.clone(), "string.format", "values")?)
          .map_err(|_| args::message("string.format", "hex value must be non-negative"))?;
        if kind == 'x' {
          format!("{value:x}")
        } else {
          format!("{value:X}")
        }
      }
      'o' => format!(
        "{:o}",
        args::integer(value.clone(), "string.format", "values")?
      ),
      'f' | 'e' | 'E' | 'g' | 'G' => {
        let value = args::number(value.clone(), "string.format", "values")?;
        let precision = spec
          .split_once('.')
          .and_then(|(_, value)| value.parse::<usize>().ok())
          .unwrap_or(6)
          .min(32);
        match kind {
          'f' => format!("{value:.precision$}"),
          'e' => format!("{value:.precision$e}"),
          'E' => format!("{value:.precision$E}"),
          'g' | 'G' => format!("{value:.precision$}"),
          _ => unreachable!(),
        }
      }
      'c' => char::from_u32(
        u32::try_from(args::integer(value.clone(), "string.format", "values")?)
          .map_err(|_| args::message("string.format", "character code is out of range"))?,
      )
      .ok_or_else(|| args::message("string.format", "invalid Unicode scalar value"))?
      .to_string(),
      _ => {
        return Err(args::message(
          "string.format",
          format!("unsupported format '%{kind}'"),
        ));
      }
    };
    output.push_str(&formatted);
    if output.len() > args::MAX_API_STRING_BYTES {
      return Err(args::message("string.format", "output exceeds 1 MiB"));
    }
  }
  Ok(output)
}

fn format_value(value: &Value) -> String {
  match value {
    Value::Nil => "nil".to_string(),
    Value::Boolean(value) => value.to_string(),
    Value::Integer(value) => value.to_string(),
    Value::Number(value) => value.to_string(),
    Value::String(value) => value
      .to_str()
      .map_or_else(|_| "<invalid utf-8>".to_string(), |v| v.to_string()),
    value => args::type_name(value).to_string(),
  }
}

pub(super) fn rich_text_params(
  value: Value,
  method: &str,
) -> mlua::Result<Option<crate::host_engine::services::RichTextParams>> {
  if matches!(value, Value::Nil) {
    return Ok(None);
  }
  let Value::Table(table) = value else {
    return Err(args::invalid(method, "rich_params", "table or nil", &value));
  };
  let mut output = crate::host_engine::services::RichTextParams::default();
  for pair in table.pairs::<String, Value>() {
    let (key, value) = pair?;
    let value = match value {
      Value::String(value) => value.to_str()?.to_string(),
      Value::Integer(value) => value.to_string(),
      Value::Number(value) if value.is_finite() => value.to_string(),
      Value::Boolean(value) => value.to_string(),
      value => {
        return Err(args::invalid(
          method,
          "rich_params",
          "string, number, or boolean values",
          &value,
        ));
      }
    };
    output.values.insert(key, value);
  }
  Ok(Some(output))
}
#[derive(Clone, Copy)]
enum StringUnary {
  Lower,
  Upper,
  Reverse,
  RegexEscape,
}
impl StringUnary {
  fn apply(self, text: &str) -> String {
    match self {
      Self::Lower => text.to_lowercase(),
      Self::Upper => text.to_uppercase(),
      Self::Reverse => text.chars().rev().collect(),
      Self::RegexEscape => regex::escape(text),
    }
  }
}
