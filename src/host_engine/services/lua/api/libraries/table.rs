use super::*;

pub(super) fn table_lib(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  source.raw_set(
    "concat",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named(
        "table.concat",
        values,
        &["table", "separator", "start", "finish"],
      )?;
      let input = mutable_or_readonly_table(
        args::required(&parameters, "table.concat", "table")?,
        "table.concat",
        "table",
      )?;
      let separator =
        args::optional_string(&parameters, "table.concat", "separator", Some(""))?.unwrap();
      let start = args::optional_integer(&parameters, "table.concat", "start", Some(1))?.unwrap();
      let finish = args::optional_integer(
        &parameters,
        "table.concat",
        "finish",
        Some(input.raw_len() as i64),
      )?
      .unwrap();
      if start < 1 || finish < start {
        return Ok(String::new());
      }
      let mut parts = Vec::new();
      let mut output_len = 0_usize;
      for index in start..=finish {
        let value = input.raw_get::<Value>(index)?;
        let text = match value {
          Value::String(value) => value.to_str()?.to_string(),
          Value::Integer(value) => value.to_string(),
          Value::Number(value) => value.to_string(),
          value => {
            return Err(args::invalid(
              "table.concat",
              "table",
              "array containing only strings or numbers",
              &value,
            ));
          }
        };
        output_len = output_len
          .checked_add(text.len())
          .and_then(|size| size.checked_add(if parts.is_empty() { 0 } else { separator.len() }))
          .ok_or_else(|| args::message("table.concat", "output size overflow"))?;
        if output_len > args::MAX_API_STRING_BYTES {
          return Err(args::message("table.concat", "output exceeds 1 MiB"));
        }
        parts.push(text);
      }
      Ok(parts.join(&separator))
    })?,
  )?;
  source.raw_set(
    "insert",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named("table.insert", values, &["table", "position", "value"])?;
      let input = writable_table(
        args::required(&parameters, "table.insert", "table")?,
        "table.insert",
        "table",
      )?;
      let len = input.raw_len();
      let position = args::optional_integer(
        &parameters,
        "table.insert",
        "position",
        Some(len as i64 + 1),
      )?
      .unwrap();
      if position < 1 || position > len as i64 + 1 {
        return Err(args::message("table.insert", "position is out of range"));
      }
      let value = args::required(&parameters, "table.insert", "value")?;
      for index in (position as usize..=len).rev() {
        input.raw_set(index + 1, input.raw_get::<Value>(index)?)?;
      }
      input.raw_set(position, value)
    })?,
  )?;
  source.raw_set(
    "move",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named(
        "table.move",
        values,
        &["source", "start", "finish", "target_index", "target"],
      )?;
      let source = mutable_or_readonly_table(
        args::required(&parameters, "table.move", "source")?,
        "table.move",
        "source",
      )?;
      let target = match parameters.get::<Value>("target")? {
        Value::Nil => writable_table(parameters.get::<Value>("source")?, "table.move", "source")?,
        value => writable_table(value, "table.move", "target")?,
      };
      let start = args::integer(
        args::required(&parameters, "table.move", "start")?,
        "table.move",
        "start",
      )?;
      let finish = args::integer(
        args::required(&parameters, "table.move", "finish")?,
        "table.move",
        "finish",
      )?;
      let target_index = args::integer(
        args::required(&parameters, "table.move", "target_index")?,
        "table.move",
        "target_index",
      )?;
      if finish >= start {
        let count = usize::try_from(finish - start + 1)
          .map_err(|_| args::message("table.move", "range is too large"))?;
        if count > args::MAX_API_TABLE_ENTRIES {
          return Err(args::message("table.move", "range exceeds 16384 entries"));
        }
        let copied = (0..count)
          .map(|offset| source.raw_get::<Value>(start + offset as i64))
          .collect::<mlua::Result<Vec<_>>>()?;
        for (offset, value) in copied.into_iter().enumerate() {
          target.raw_set(target_index + offset as i64, value)?;
        }
      }
      Ok(target)
    })?,
  )?;
  source.raw_set(
    "pack",
    lua.create_function(|lua, values: MultiValue| {
      let table = args::named("table.pack", values, &["values"])?;
      let values = args::values(&table, "table.pack")?;
      let output = lua.create_table()?;
      for (index, value) in values.iter().cloned().enumerate() {
        output.raw_set(index + 1, value)?;
      }
      output.raw_set("n", values.len())?;
      Ok(output)
    })?,
  )?;
  source.raw_set(
    "remove",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named("table.remove", values, &["table", "position"])?;
      let input = writable_table(
        args::required(&parameters, "table.remove", "table")?,
        "table.remove",
        "table",
      )?;
      let len = input.raw_len();
      if len == 0 {
        return Ok(Value::Nil);
      }
      let position =
        args::optional_integer(&parameters, "table.remove", "position", Some(len as i64))?.unwrap();
      if position < 1 || position > len as i64 {
        return Err(args::message("table.remove", "position is out of range"));
      }
      let removed = input.raw_get::<Value>(position)?;
      for index in position as usize..len {
        input.raw_set(index, input.raw_get::<Value>(index + 1)?)?;
      }
      input.raw_set(len, Value::Nil)?;
      Ok(removed)
    })?,
  )?;
  source.raw_set(
    "sort",
    lua.create_function(|_, values: MultiValue| {
      let parameters = args::named("table.sort", values, &["table", "comparator"])?;
      let input = writable_table(
        args::required(&parameters, "table.sort", "table")?,
        "table.sort",
        "table",
      )?;
      let len = input.raw_len();
      if len > 4096 {
        return Err(args::message("table.sort", "array exceeds 4096 items"));
      }
      let comparator = match parameters.get::<Value>("comparator")? {
        Value::Nil => None,
        Value::Function(function) => Some(function),
        value => {
          return Err(args::invalid(
            "table.sort",
            "comparator",
            "function or nil",
            &value,
          ));
        }
      };
      let mut items = (1..=len)
        .map(|index| input.raw_get::<Value>(index))
        .collect::<mlua::Result<Vec<_>>>()?;
      let mut comparison_error = None;
      items.sort_by(|left, right| {
        if comparison_error.is_some() {
          return Ordering::Equal;
        }
        let result = if let Some(comparator) = &comparator {
          comparator.call::<bool>((left.clone(), right.clone()))
        } else {
          default_less(left, right)
        };
        match result {
          Ok(true) => Ordering::Less,
          Ok(false) => match if let Some(comparator) = &comparator {
            comparator.call::<bool>((right.clone(), left.clone()))
          } else {
            default_less(right, left)
          } {
            Ok(true) => Ordering::Greater,
            Ok(false) => Ordering::Equal,
            Err(error) => {
              comparison_error = Some(error);
              Ordering::Equal
            }
          },
          Err(error) => {
            comparison_error = Some(error);
            Ordering::Equal
          }
        }
      });
      if let Some(error) = comparison_error {
        return Err(error);
      }
      for (index, value) in items.into_iter().enumerate() {
        input.raw_set(index + 1, value)?;
      }
      Ok(())
    })?,
  )?;
  source.raw_set(
    "unpack",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("table.unpack", values, &["table", "start", "finish"])?;
      let value = args::required(&table, "table.unpack", "table")?;
      let Value::Table(input) = value else {
        return Err(args::invalid("table.unpack", "table", "table", &value));
      };
      let input = readonly::backing(&input)?;
      let start = args::optional_integer(&table, "table.unpack", "start", Some(1))?.unwrap();
      let finish = args::optional_integer(
        &table,
        "table.unpack",
        "finish",
        Some(input.raw_len() as i64),
      )?
      .unwrap();
      if start < 1 || finish < start {
        return Ok(MultiValue::new());
      }
      let mut output = Vec::new();
      for index in start..=finish {
        output.push(input.raw_get(index)?)
      }
      Ok(MultiValue::from_vec(output))
    })?,
  )?;
  readonly::proxy(lua, source)
}

fn mutable_or_readonly_table(value: Value, method: &str, name: &str) -> mlua::Result<Table> {
  let Value::Table(table) = value else {
    return Err(args::invalid(method, name, "table", &value));
  };
  readonly::backing(&table)
}

fn writable_table(value: Value, method: &str, name: &str) -> mlua::Result<Table> {
  let Value::Table(table) = value else {
    return Err(args::invalid(method, name, "table", &value));
  };
  if readonly::is_proxy(&table)? {
    return Err(args::message(
      method,
      format!("parameter '{name}' is read-only"),
    ));
  }
  Ok(table)
}

fn default_less(left: &Value, right: &Value) -> mlua::Result<bool> {
  match (left, right) {
    (Value::Integer(left), Value::Integer(right)) => Ok(left < right),
    (Value::Integer(left), Value::Number(right)) => Ok((*left as f64) < *right),
    (Value::Number(left), Value::Integer(right)) => Ok(*left < *right as f64),
    (Value::Number(left), Value::Number(right)) => Ok(left < right),
    (Value::String(left), Value::String(right)) => Ok(left.as_bytes() < right.as_bytes()),
    _ => Err(args::message(
      "table.sort",
      "values are not mutually comparable numbers or strings",
    )),
  }
}
