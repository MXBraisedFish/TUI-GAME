use mlua::{Lua, MultiValue, Table, Value};

use super::*;
use crate::host_engine::services::{
  LuaObjectPool, SliceId, SliceLength, SliceOptions, SliceRect, SliceService,
};

const MAX_SLICES: usize = 1024;

pub(super) fn slice(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for percent in [10_u8, 25, 33, 50, 66, 75, 100] {
    source.raw_set(format!("{percent}P"), format!("{percent}%"))?;
  }

  install_lifecycle(lua, &source, state.clone())?;
  install_mutations(lua, &source, state.clone())?;
  install_queries(lua, &source, state)?;
  readonly::proxy(lua, source)
}

fn install_lifecycle(lua: &Lua, source: &Table, state: SharedApiState) -> mlua::Result<()> {
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "slice.create";
      let table = args::named(method, values, &["width", "height", "layer"])?;
      let width = length(args::required(&table, method, "width")?, method, "width")?;
      let height = length(args::required(&table, method, "height")?, method, "height")?;
      let layer = args::optional_integer(&table, method, "layer", None)?
        .map(|value| checked_i32(value, method, "layer"))
        .transpose()?;
      with_pool_mut(&create_state, method, |objects| {
        let service = SliceService::new();
        if service.ids(objects.ui()).len() >= MAX_SLICES {
          return Err(args::message(method, "slice limit of 1024 was reached"));
        }
        let id = service
          .create(
            objects.ui_mut(),
            SliceOptions {
              rect: SliceRect {
                x: 0,
                y: 0,
                width,
                height,
              },
              layer,
              ..Default::default()
            },
          )
          .ok_or_else(|| args::message(method, "invalid slice dimensions"))?;
        Ok(Value::String(lua.create_string(format_id(id))?))
      })
    })?,
  )?;

  let delete_state = state.clone();
  source.raw_set(
    "delete",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.delete";
      let id = id_argument(values, method)?;
      with_pool_mut(&delete_state, method, |objects| {
        Ok(SliceService::new().remove(objects.ui_mut(), id))
      })
    })?,
  )?;

  source.raw_set(
    "clear",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.clear";
      args::no_args(method, values)?;
      with_pool_mut(&state, method, |objects| {
        let service = SliceService::new();
        for id in service.ids(objects.ui()) {
          service.remove(objects.ui_mut(), id);
        }
        Ok(())
      })
    })?,
  )
}

fn install_mutations(lua: &Lua, source: &Table, state: SharedApiState) -> mlua::Result<()> {
  for (name, field) in [("set_size", 0_u8), ("set_width", 1), ("set_height", 2)] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = match field {
          0 => "slice.set_size",
          1 => "slice.set_width",
          _ => "slice.set_height",
        };
        let allowed: &[&str] = match field {
          0 => &["id", "width", "height"],
          1 => &["id", "width"],
          _ => &["id", "height"],
        };
        let table = args::named(method, values, allowed)?;
        let id = table_id(&table, method)?;
        let width = if field != 2 {
          Some(length(
            args::required(&table, method, "width")?,
            method,
            "width",
          )?)
        } else {
          None
        };
        let height = if field != 1 {
          Some(length(
            args::required(&table, method, "height")?,
            method,
            "height",
          )?)
        } else {
          None
        };
        with_pool_mut(&state, method, |objects| {
          let service = SliceService::new();
          let Some(mut rect) = service.configured_rect(objects.ui(), id) else {
            return Err(args::message(method, "unknown slice id"));
          };
          if let Some(width) = width {
            rect.width = width;
          }
          if let Some(height) = height {
            rect.height = height;
          }
          if !service.set_rect(objects.ui_mut(), id, rect) {
            return Err(args::message(method, "invalid slice dimensions"));
          }
          Ok(())
        })
      })?,
    )?;
  }

  let layer_state = state.clone();
  source.raw_set(
    "set_layer",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.set_layer";
      let table = args::named(method, values, &["id", "layer"])?;
      let id = table_id(&table, method)?;
      let layer = checked_i32(
        args::integer(args::required(&table, method, "layer")?, method, "layer")?,
        method,
        "layer",
      )?;
      with_pool_mut(&layer_state, method, |objects| {
        if SliceService::new().set_layer(objects.ui_mut(), id, layer) {
          Ok(())
        } else {
          Err(args::message(method, "unknown slice id"))
        }
      })
    })?,
  )?;

  source.raw_set(
    "draw",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.draw";
      let table = args::named(method, values, &["id", "x", "y"])?;
      let id = table_id(&table, method)?;
      let x = checked_i32(
        args::integer(args::required(&table, method, "x")?, method, "x")?,
        method,
        "x",
      )?;
      let y = checked_i32(
        args::integer(args::required(&table, method, "y")?, method, "y")?,
        method,
        "y",
      )?;
      with_pool_mut(&state, method, |objects| {
        if SliceService::new().set_position(objects.ui_mut(), id, x, y) {
          Ok(())
        } else {
          Err(args::message(method, "unknown slice id"))
        }
      })
    })?,
  )
}

fn install_queries(lua: &Lua, source: &Table, state: SharedApiState) -> mlua::Result<()> {
  let exists_state = state.clone();
  source.raw_set(
    "exists",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.exists";
      let id = id_argument(values, method)?;
      with_pool(&exists_state, method, |objects| {
        Ok(SliceService::new().exists(objects.ui(), id))
      })
    })?,
  )?;

  for (name, field) in [("get_width", 0_u8), ("get_height", 1), ("get_layer", 2)] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |_, values: MultiValue| {
        let method = match field {
          0 => "slice.get_width",
          1 => "slice.get_height",
          _ => "slice.get_layer",
        };
        let id = id_argument(values, method)?;
        with_pool(&state, method, |objects| {
          let service = SliceService::new();
          let value = match field {
            0 => service.configured_rect(objects.ui(), id).map(|rect| {
              i64::from(resolve_length(
                rect.width,
                state.borrow().context.terminal_size.width,
              ))
            }),
            1 => service.configured_rect(objects.ui(), id).map(|rect| {
              i64::from(resolve_length(
                rect.height,
                state.borrow().context.terminal_size.height,
              ))
            }),
            _ => service.layer(objects.ui(), id).map(i64::from),
          };
          Ok(value.map(Value::Integer).unwrap_or(Value::Nil))
        })
      })?,
    )?;
  }

  let info_state = state.clone();
  source.raw_set(
    "get_info",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "slice.get_info";
      let id = id_argument(values, method)?;
      info_value(lua, &info_state, method, id)
    })?,
  )?;

  let list_state = state.clone();
  source.raw_set(
    "list",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "slice.list";
      args::no_args(method, values)?;
      with_pool(&list_state, method, |objects| {
        let result = lua.create_table()?;
        for (index, id) in SliceService::new()
          .ids(objects.ui())
          .into_iter()
          .enumerate()
        {
          result.raw_set(index + 1, format_id(id))?;
        }
        Ok(result)
      })
    })?,
  )?;

  let ordered_state = state.clone();
  source.raw_set(
    "list_by_layer",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "slice.list_by_layer";
      args::no_args(method, values)?;
      let ids = with_pool(&ordered_state, method, |objects| {
        Ok(SliceService::new().ids_by_layer(objects.ui()))
      })?;
      let result = lua.create_table()?;
      for (index, id) in ids.into_iter().enumerate() {
        result.raw_set(index + 1, info_value(lua, &ordered_state, method, id)?)?;
      }
      Ok(result)
    })?,
  )?;

  source.raw_set(
    "count",
    lua.create_function(move |_, values: MultiValue| {
      let method = "slice.count";
      args::no_args(method, values)?;
      with_pool(&state, method, |objects| {
        Ok(SliceService::new().ids(objects.ui()).len())
      })
    })?,
  )
}

fn info_value(lua: &Lua, state: &SharedApiState, method: &str, id: SliceId) -> mlua::Result<Value> {
  let size = state.borrow().context.terminal_size;
  with_pool(state, method, |objects| {
    let service = SliceService::new();
    let Some(rect) = service.configured_rect(objects.ui(), id) else {
      return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.raw_set("id", format_id(id))?;
    info.raw_set("width", resolve_length(rect.width, size.width))?;
    info.raw_set("height", resolve_length(rect.height, size.height))?;
    info.raw_set("layer", service.layer(objects.ui(), id).unwrap_or_default())?;
    Ok(Value::Table(info))
  })
}

fn length(value: Value, method: &str, name: &str) -> mlua::Result<SliceLength> {
  match value {
    Value::Integer(value) => u16::try_from(value)
      .ok()
      .filter(|value| *value > 0)
      .map(SliceLength::Fixed)
      .ok_or_else(|| args::message(method, format!("{name} must be in 1..=65535"))),
    Value::Number(value) if value.is_finite() && value.fract() == 0.0 => {
      length(Value::Integer(value as i64), method, name)
    }
    Value::String(value) => {
      let value = value.to_str()?;
      let Some(percent) = value.strip_suffix('%') else {
        return Err(args::message(
          method,
          format!("invalid {name} percentage constant"),
        ));
      };
      let percent = percent
        .parse::<u8>()
        .ok()
        .filter(|percent| matches!(percent, 10 | 25 | 33 | 50 | 66 | 75 | 100))
        .ok_or_else(|| args::message(method, format!("invalid {name} percentage constant")))?;
      Ok(SliceLength::Percent(percent))
    }
    value => Err(args::invalid(
      method,
      name,
      "positive integer or slice percentage",
      &value,
    )),
  }
}

pub(super) fn resolve_length(length: SliceLength, total: u16) -> u16 {
  match length {
    SliceLength::Fixed(value) => value,
    SliceLength::Auto => total,
    SliceLength::Percent(value) => (u32::from(total) * u32::from(value) / 100) as u16,
  }
}

fn id_argument(values: MultiValue, method: &str) -> mlua::Result<SliceId> {
  let value = args::one(method, "id", values)?;
  let value = args::string(value, method, "id")?;
  parse_id(&value, method, "id")
}

fn table_id(table: &Table, method: &str) -> mlua::Result<SliceId> {
  let value = args::string(args::required(table, method, "id")?, method, "id")?;
  parse_id(&value, method, "id")
}

pub(super) fn parse_id(value: &str, method: &str, name: &str) -> mlua::Result<SliceId> {
  let raw = value
    .strip_prefix("slice_")
    .and_then(|value| value.parse::<u64>().ok())
    .filter(|value| *value > 0)
    .ok_or_else(|| args::message(method, format!("invalid {name} slice id")))?;
  Ok(SliceId(raw))
}

fn format_id(id: SliceId) -> String {
  format!("slice_{:03}", id.0)
}

fn checked_i32(value: i64, method: &str, name: &str) -> mlua::Result<i32> {
  i32::try_from(value).map_err(|_| args::message(method, format!("{name} is out of i32 range")))
}

fn with_pool<R>(
  state: &SharedApiState,
  method: &str,
  operation: impl FnOnce(&LuaObjectPool) -> mlua::Result<R>,
) -> mlua::Result<R> {
  let objects = state
    .borrow()
    .objects
    .upgrade()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  let objects = objects
    .try_borrow()
    .map_err(|_| args::message(method, "session object pool is busy"))?;
  operation(
    objects
      .as_ref()
      .ok_or_else(|| args::message(method, "session object pool is unavailable"))?,
  )
}

fn with_pool_mut<R>(
  state: &SharedApiState,
  method: &str,
  operation: impl FnOnce(&mut LuaObjectPool) -> mlua::Result<R>,
) -> mlua::Result<R> {
  let objects = state
    .borrow()
    .objects
    .upgrade()
    .ok_or_else(|| args::message(method, "session object pool is unavailable"))?;
  let mut objects = objects
    .try_borrow_mut()
    .map_err(|_| args::message(method, "session object pool is busy"))?;
  operation(
    objects
      .as_mut()
      .ok_or_else(|| args::message(method, "session object pool is unavailable"))?,
  )
}
