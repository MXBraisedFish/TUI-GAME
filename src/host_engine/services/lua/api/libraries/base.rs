use super::*;

pub(super) fn base(lua: &Lua) -> mlua::Result<Table> {
  let ipairs = lua.create_function(|lua, args: MultiValue| {
    let value = args::one("base.ipairs", "table", args)?;
    let Value::Table(table) = value else {
      return Err(args::invalid("base.ipairs", "table", "table", &value));
    };
    readonly::iterator(lua, readonly::backing(&table)?, true)
  })?;
  let pairs = lua.create_function(|lua, args: MultiValue| {
    let value = args::one("base.pairs", "table", args)?;
    let Value::Table(table) = value else {
      return Err(args::invalid("base.pairs", "table", "table", &value));
    };
    readonly::iterator(lua, readonly::backing(&table)?, false)
  })?;
  let next = lua.create_function(|_, args: MultiValue| {
    let table = args::named("base.next", args, &["table", "index"])?;
    let value = args::required(&table, "base.next", "table")?;
    let Value::Table(source) = value else {
      return Err(args::invalid("base.next", "table", "table", &value));
    };
    let source = readonly::backing(&source)?;
    let index = table.get::<Value>("index")?;
    let mut found = matches!(index, Value::Nil);
    for pair in source.pairs::<Value, Value>() {
      let (key, value) = pair?;
      if found {
        return Ok((key, value));
      }
      if raw_equal(&key, &index) {
        found = true;
      }
    }
    Ok((Value::Nil, Value::Nil))
  })?;
  let select = lua.create_function(|_, args: MultiValue| {
    let table = args::named("base.select", args, &["index", "values"])?;
    let index = args::required(&table, "base.select", "index")?;
    let values = args::values(&table, "base.select")?;
    if let Value::String(index) = &index
      && index.to_str()?.as_ref() == "#"
    {
      return Ok(MultiValue::from_vec(vec![Value::Integer(
        values.len() as i64
      )]));
    }
    let index = args::integer(index, "base.select", "index")?;
    let start = if index < 0 {
      values.len() as i64 + index + 1
    } else {
      index
    };
    if start < 1 || start > values.len() as i64 + 1 {
      return Err(args::message("base.select", "index out of range"));
    }
    Ok(MultiValue::from_vec(
      values.into_iter().skip((start - 1) as usize).collect(),
    ))
  })?;
  let rawequal = lua.create_function(|_, args: MultiValue| {
    let table = args::named("base.rawequal", args, &["left", "right"])?;
    Ok(raw_equal(
      &table.get::<Value>("left")?,
      &table.get::<Value>("right")?,
    ))
  })?;
  let rawlen = lua.create_function(|_, args: MultiValue| {
    let value = args::one("base.rawlen", "value", args)?;
    match value {
      Value::String(value) => Ok(value.as_bytes().len() as i64),
      Value::Table(table) => Ok(readonly::backing(&table)?.raw_len() as i64),
      value => Err(args::invalid(
        "base.rawlen",
        "value",
        "string or table",
        &value,
      )),
    }
  })?;
  let tonumber = lua.create_function(|_, args: MultiValue| {
    let table = args::named("base.tonumber", args, &["value", "base"])?;
    let value = args::required(&table, "base.tonumber", "value")?;
    let base = args::optional_integer(&table, "base.tonumber", "base", None)?;
    match (value, base) {
      (Value::Integer(value), None) => Ok(Value::Integer(value)),
      (Value::Number(value), None) => Ok(Value::Number(value)),
      (Value::String(value), None) => {
        let text = value.to_str()?;
        Ok(parse_number(text.as_ref()).unwrap_or(Value::Nil))
      }
      (Value::String(value), Some(base @ 2..=36)) => {
        let text = value.to_str()?;
        Ok(
          i64::from_str_radix(text.trim(), base as u32)
            .map(Value::Integer)
            .unwrap_or(Value::Nil),
        )
      }
      (value, Some(_)) => Err(args::invalid(
        "base.tonumber",
        "base",
        "integer 2..36",
        &value,
      )),
      _ => Ok(Value::Nil),
    }
  })?;
  let tostring = lua.create_function(|lua, args: MultiValue| {
    let value = args::one("base.tostring", "value", args)?;
    let text = match value {
      Value::Nil => "nil".to_string(),
      Value::Boolean(value) => value.to_string(),
      Value::Integer(value) => value.to_string(),
      Value::Number(value) => value.to_string(),
      Value::String(value) => value.to_str()?.to_string(),
      Value::Table(value) => format!("table: {:p}", value.to_pointer()),
      Value::Function(value) => format!("function: {:p}", value.to_pointer()),
      Value::Thread(value) => format!("thread: {:p}", value.to_pointer()),
      Value::UserData(value) => format!("userdata: {:p}", value.to_pointer()),
      value => args::type_name(&value).to_string(),
    };
    lua.create_string(text)
  })?;
  let type_fn = lua.create_function(|lua, args: MultiValue| {
    let value = args::one("base.type", "value", args)?;
    lua.create_string(args::type_name(&value))
  })?;
  readonly::library(
    lua,
    [
      ("ipairs", function_value(ipairs)),
      ("pairs", function_value(pairs)),
      ("next", function_value(next)),
      ("select", function_value(select)),
      ("rawequal", function_value(rawequal)),
      ("rawlen", function_value(rawlen)),
      ("tonumber", function_value(tonumber)),
      ("tostring", function_value(tostring)),
      ("type", function_value(type_fn)),
    ],
  )
}

fn raw_equal(left: &Value, right: &Value) -> bool {
  match (left, right) {
    (Value::Nil, Value::Nil) => true,
    (Value::Boolean(a), Value::Boolean(b)) => a == b,
    (Value::Integer(a), Value::Integer(b)) => a == b,
    (Value::Integer(a), Value::Number(b)) | (Value::Number(b), Value::Integer(a)) => {
      *a as f64 == *b
    }
    (Value::Number(a), Value::Number(b)) => a == b,
    (Value::String(a), Value::String(b)) => a.as_bytes() == b.as_bytes(),
    (Value::Table(a), Value::Table(b)) => a.to_pointer() == b.to_pointer(),
    (Value::Function(a), Value::Function(b)) => a.to_pointer() == b.to_pointer(),
    (Value::Thread(a), Value::Thread(b)) => a.to_pointer() == b.to_pointer(),
    (Value::UserData(a), Value::UserData(b)) => a.to_pointer() == b.to_pointer(),
    _ => false,
  }
}

fn parse_number(text: &str) -> Option<Value> {
  let text = text.trim();
  if let Ok(value) = text.parse::<i64>() {
    Some(Value::Integer(value))
  } else {
    text.parse::<f64>().ok().map(Value::Number)
  }
}
