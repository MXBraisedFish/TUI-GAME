use super::*;

pub(super) fn math(lua: &Lua) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("PI", PI),
    ("E", E),
    ("POSITIVE_INFINITE", f64::INFINITY),
    ("INFINITE", f64::INFINITY),
    ("NEGATIVE_INFINITE", f64::NEG_INFINITY),
    ("DEG", 180.0 / PI),
    ("RAD", PI / 180.0),
  ] {
    source.raw_set(name, value)?;
  }
  source.raw_set("MAX_INTEGER", i64::MAX)?;
  source.raw_set("MIN_INTEGER", i64::MIN)?;
  for (name, operation) in [
    ("abs", UnaryMath::Abs),
    ("acos", UnaryMath::Acos),
    ("asin", UnaryMath::Asin),
    ("atan", UnaryMath::Atan),
    ("ceil", UnaryMath::Ceil),
    ("cos", UnaryMath::Cos),
    ("deg", UnaryMath::Deg),
    ("exp", UnaryMath::Exp),
    ("floor", UnaryMath::Floor),
    ("log10", UnaryMath::Log10),
    ("rad", UnaryMath::Rad),
    ("sin", UnaryMath::Sin),
    ("sqrt", UnaryMath::Sqrt),
    ("tan", UnaryMath::Tan),
    ("round", UnaryMath::Round),
    ("normalize_angle", UnaryMath::NormalizeAngle),
  ] {
    let function = lua.create_function(move |_, args: MultiValue| {
      let method = format!("math.{name}");
      let value = finite_number(
        args::one(&format!("math.{name}"), "value", args)?,
        &method,
        "value",
      )?;
      operation.apply(&method, value)
    })?;
    source.raw_set(name, function)?;
  }
  for (name, operation) in [
    ("atan2", BinaryMath::Atan2),
    ("fmod", BinaryMath::Fmod),
    ("ldexp", BinaryMath::Ldexp),
    ("pow", BinaryMath::Pow),
    ("round_to", BinaryMath::RoundTo),
  ] {
    let function = lua.create_function(move |_, values: MultiValue| {
      let method = format!("math.{name}");
      let table = args::named(&method, values, &["left", "right"])?;
      let left = finite_number(args::required(&table, &method, "left")?, &method, "left")?;
      let right = finite_number(args::required(&table, &method, "right")?, &method, "right")?;
      operation.apply(&method, left, right)
    })?;
    source.raw_set(name, function)?;
  }
  source.raw_set(
    "log",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("math.log", values, &["value", "base"])?;
      let value = finite_number(
        args::required(&table, "math.log", "value")?,
        "math.log",
        "value",
      )?;
      if value <= 0.0 {
        return Err(args::message("math.log", "value must be greater than zero"));
      }
      let base = table.get::<Value>("base")?;
      let result = if matches!(base, Value::Nil) {
        value.ln()
      } else {
        let base = finite_number(base, "math.log", "base")?;
        if base <= 0.0 || base == 1.0 {
          return Err(args::message(
            "math.log",
            "base must be greater than zero and not equal to one",
          ));
        }
        value.log(base)
      };
      finite_result("math.log", result)
    })?,
  )?;
  source.raw_set("max", extremum(lua, true)?)?;
  source.raw_set("min", extremum(lua, false)?)?;
  source.raw_set(
    "frexp",
    lua.create_function(|_, args: MultiValue| {
      let value = finite_number(
        args::one("math.frexp", "value", args)?,
        "math.frexp",
        "value",
      )?;
      if value == 0.0 {
        return Ok((0.0, 0_i64));
      }
      let exponent = value.abs().log2().floor() as i32 + 1;
      Ok((value / 2_f64.powi(exponent), exponent as i64))
    })?,
  )?;
  source.raw_set(
    "modf",
    lua.create_function(|_, args: MultiValue| {
      let value = finite_number(args::one("math.modf", "value", args)?, "math.modf", "value")?;
      let integer = value.trunc();
      Ok((integer, value - integer))
    })?,
  )?;
  source.raw_set(
    "tointeger",
    lua.create_function(|_, args: MultiValue| {
      let value = args::one("math.tointeger", "value", args)?;
      Ok(match value {
        Value::Integer(v) => Value::Integer(v),
        Value::Number(v)
          if v.is_finite() && v.fract() == 0.0 && v >= i64::MIN as f64 && v <= i64::MAX as f64 =>
        {
          Value::Integer(v as i64)
        }
        _ => Value::Nil,
      })
    })?,
  )?;
  source.raw_set(
    "type",
    lua.create_function(|lua, args: MultiValue| {
      let value = args::one("math.type", "value", args)?;
      match value {
        Value::Integer(_) => Ok(Value::String(lua.create_string("integer")?)),
        Value::Number(_) => Ok(Value::String(lua.create_string("float")?)),
        _ => Ok(Value::Nil),
      }
    })?,
  )?;
  source.raw_set(
    "ult",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("math.ult", values, &["left", "right"])?;
      let left = args::integer(
        args::required(&table, "math.ult", "left")?,
        "math.ult",
        "left",
      )? as u64;
      let right = args::integer(
        args::required(&table, "math.ult", "right")?,
        "math.ult",
        "right",
      )? as u64;
      Ok(left < right)
    })?,
  )?;
  source.raw_set(
    "percent",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("math.percent", values, &["value", "total"])?;
      let value = finite_number(
        args::required(&table, "math.percent", "value")?,
        "math.percent",
        "value",
      )?;
      let total = finite_number(
        args::required(&table, "math.percent", "total")?,
        "math.percent",
        "total",
      )?;
      if total == 0.0 {
        return Err(args::message("math.percent", "total must not be zero"));
      }
      finite_result("math.percent", value / total * 100.0)
    })?,
  )?;
  source.raw_set(
    "factorial",
    lua.create_function(|_, values: MultiValue| {
      let n = args::integer(
        args::one("math.factorial", "value", values)?,
        "math.factorial",
        "value",
      )?;
      if !(0..=170).contains(&n) {
        return Err(args::message("math.factorial", "value must be in 0..=170"));
      }
      Ok((1..=n).fold(1.0, |value, item| value * item as f64))
    })?,
  )?;
  source.raw_set(
    "combination",
    lua.create_function(|_, values: MultiValue| {
      let table = args::named("math.combination", values, &["n", "k"])?;
      let n = args::integer(
        args::required(&table, "math.combination", "n")?,
        "math.combination",
        "n",
      )?;
      let mut k = args::integer(
        args::required(&table, "math.combination", "k")?,
        "math.combination",
        "k",
      )?;
      if n < 0 || k < 0 || k > n || n > 1_000_000 {
        return Err(args::message(
          "math.combination",
          "requires 0 <= k <= n <= 1000000",
        ));
      }
      k = k.min(n - k);
      let mut result = 1.0;
      for i in 1..=k {
        result = result * (n - k + i) as f64 / i as f64;
        if !result.is_finite() {
          return Err(args::message(
            "math.combination",
            "result exceeds finite number range",
          ));
        }
      }
      Ok(result)
    })?,
  )?;
  readonly::proxy(lua, source)
}

#[derive(Clone, Copy)]
enum UnaryMath {
  Abs,
  Acos,
  Asin,
  Atan,
  Ceil,
  Cos,
  Deg,
  Exp,
  Floor,
  Log10,
  Rad,
  Sin,
  Sqrt,
  Tan,
  Round,
  NormalizeAngle,
}
impl UnaryMath {
  fn apply(self, method: &str, v: f64) -> mlua::Result<f64> {
    match self {
      Self::Acos | Self::Asin if !(-1.0..=1.0).contains(&v) => {
        return Err(args::message(method, "value must be in -1..=1"));
      }
      Self::Log10 if v <= 0.0 => {
        return Err(args::message(method, "value must be greater than zero"));
      }
      Self::Sqrt if v < 0.0 => {
        return Err(args::message(method, "value must not be negative"));
      }
      _ => {}
    }
    let result = match self {
      Self::Abs => v.abs(),
      Self::Acos => v.acos(),
      Self::Asin => v.asin(),
      Self::Atan => v.atan(),
      Self::Ceil => v.ceil(),
      Self::Cos => v.cos(),
      Self::Deg => v.to_degrees(),
      Self::Exp => v.exp(),
      Self::Floor => v.floor(),
      Self::Log10 => v.log10(),
      Self::Rad => v.to_radians(),
      Self::Sin => v.sin(),
      Self::Sqrt => v.sqrt(),
      Self::Tan => v.tan(),
      Self::Round => v.round(),
      Self::NormalizeAngle => v.rem_euclid(360.0),
    };
    finite_result(method, result)
  }
}
#[derive(Clone, Copy)]
enum BinaryMath {
  Atan2,
  Fmod,
  Ldexp,
  Pow,
  RoundTo,
}
impl BinaryMath {
  fn apply(self, method: &str, a: f64, b: f64) -> mlua::Result<f64> {
    if matches!(self, Self::Fmod) && b == 0.0 {
      return Err(args::message(method, "right must not be zero"));
    }
    if matches!(self, Self::RoundTo) && (b.fract() != 0.0 || !(-308.0..=308.0).contains(&b)) {
      return Err(args::message(
        method,
        "right must be an integer in -308..=308",
      ));
    }
    let result = match self {
      Self::Atan2 => a.atan2(b),
      Self::Fmod => a % b,
      Self::Ldexp => a * 2_f64.powf(b),
      Self::Pow => a.powf(b),
      Self::RoundTo => {
        let p = 10_f64.powf(b);
        (a * p).round() / p
      }
    };
    finite_result(method, result)
  }
}

fn finite_number(value: Value, method: &str, parameter: &str) -> mlua::Result<f64> {
  let value = args::number(value, method, parameter)?;
  if value.is_finite() {
    Ok(value)
  } else {
    Err(args::message(
      method,
      format!("invalid parameter '{parameter}': expected finite number"),
    ))
  }
}

fn finite_result(method: &str, value: f64) -> mlua::Result<f64> {
  if value.is_finite() {
    Ok(value)
  } else {
    Err(args::message(
      method,
      "result exceeds finite number range or is undefined",
    ))
  }
}

fn extremum(lua: &Lua, maximum: bool) -> mlua::Result<Function> {
  lua.create_function(move |_, values: MultiValue| {
    let method = if maximum { "math.max" } else { "math.min" };
    let table = args::named(method, values, &["values"])?;
    let values = args::values(&table, method)?;
    if values.is_empty() {
      return Err(args::message(method, "values must not be empty"));
    }
    let mut result = finite_number(values[0].clone(), method, "values")?;
    for value in values.into_iter().skip(1) {
      let value = finite_number(value, method, "values")?;
      result = if maximum {
        result.max(value)
      } else {
        result.min(value)
      };
    }
    Ok(result)
  })
}
