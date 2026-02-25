use std::collections::BTreeMap;
use std::fs;
use std::io::{Stdout, Write, stdout};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::queue;
use crossterm::style::{
    Color as CColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
};
use mlua::{Lua, Table, Value};
use once_cell::sync::Lazy;
use serde_json::{Map, Number, Value as JsonValue};
use unicode_width::UnicodeWidthStr;

use crate::app::{i18n, stats};
use crate::utils::path_utils;

const EXIT_GAME_SENTINEL: &str = "__TUI_GAME_EXIT__";
static OUT: Lazy<Mutex<Stdout>> = Lazy::new(|| Mutex::new(stdout()));
static TERMINAL_DIRTY_FROM_LUA: AtomicBool = AtomicBool::new(false);
static RNG_STATE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchMode {
    New,
    Continue,
}

impl LaunchMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Continue => "continue",
        }
    }
}

/// Registers Lua APIs for game scripts.
pub fn register_api(lua: &Lua, mode: LaunchMode) -> mlua::Result<()> {
    let get_key = lua.create_function(|_, blocking: bool| {
        flush_output()?;

        if blocking {
            loop {
                if let Event::Key(key) = event::read().map_err(mlua::Error::external)? {
                    if key.kind == KeyEventKind::Press {
                        return decode_key_event(key);
                    }
                }
            }
        }

        if event::poll(Duration::from_millis(0)).map_err(mlua::Error::external)? {
            if let Event::Key(key) = event::read().map_err(mlua::Error::external)? {
                if key.kind == KeyEventKind::Press {
                    return decode_key_event(key);
                }
            }
        }
        Ok(String::new())
    })?;
    lua.globals().set("get_key", get_key)?;

    let clear = lua.create_function(|_, ()| {
        let mut out = lock_out()?;
        queue!(
            out,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )
        .map_err(mlua::Error::external)?;
        Ok(())
    })?;
    lua.globals().set("clear", clear)?;

    let draw_text = lua.create_function(
        |_, (x, y, text, fg, bg): (i64, i64, String, Option<String>, Option<String>)| {
            draw_text_impl(x, y, &text, fg.as_deref(), bg.as_deref())
        },
    )?;
    lua.globals().set("draw_text", draw_text)?;

    let draw_text_ex = lua.create_function(
        |_,
         (x, y, text, fg, bg, max_width, align): (
            i64,
            i64,
            String,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<String>,
        )| {
            let width = max_width.unwrap_or(text.len() as i64).max(0) as usize;
            let mut rendered = text.clone();
            if width > 0 {
                let w = UnicodeWidthStr::width(text.as_str());
                if w < width {
                    let pad = width - w;
                    match align.unwrap_or_else(|| "left".to_string()).as_str() {
                        "center" => {
                            let left = pad / 2;
                            let right = pad - left;
                            rendered = format!("{}{}{}", " ".repeat(left), text, " ".repeat(right));
                        }
                        "right" => rendered = format!("{}{}", " ".repeat(pad), text),
                        _ => {}
                    }
                }
            }
            draw_text_impl(x, y, &rendered, fg.as_deref(), bg.as_deref())
        },
    )?;
    lua.globals().set("draw_text_ex", draw_text_ex)?;

    let sleep = lua.create_function(|_, ms: i64| {
        flush_output()?;
        let ms = ms.max(0) as u64;
        std::thread::sleep(Duration::from_millis(ms));
        if ms >= 200 {
            drain_input_events();
        }
        Ok(())
    })?;
    lua.globals().set("sleep", sleep)?;

    let clear_input_buffer = lua.create_function(|_, ()| {
        drain_input_events();
        Ok(true)
    })?;
    lua.globals().set("clear_input_buffer", clear_input_buffer)?;

    let random = lua.create_function(|_, max: i64| {
        if max <= 0 {
            return Ok(0);
        }
        Ok((next_random_u64() % (max as u64)) as i64)
    })?;
    lua.globals().set("random", random)?;

    let exit_game = lua.create_function(|_, ()| -> mlua::Result<()> {
        Err(mlua::Error::RuntimeError(EXIT_GAME_SENTINEL.to_string()))
    })?;
    lua.globals().set("exit_game", exit_game)?;

    let translate = lua.create_function(|_, key: String| Ok(i18n::t(&key)))?;
    lua.globals().set("translate", translate)?;

    let get_terminal_size = lua.create_function(|_, ()| {
        let (w, h) = crossterm::terminal::size().map_err(mlua::Error::external)?;
        Ok((w, h))
    })?;
    lua.globals().set("get_terminal_size", get_terminal_size)?;

    let get_text_width =
        lua.create_function(|_, text: String| Ok(UnicodeWidthStr::width(text.as_str()) as i64))?;
    lua.globals().set("get_text_width", get_text_width)?;

    let get_launch_mode = lua.create_function(move |_, ()| Ok(mode.as_str().to_string()))?;
    lua.globals().set("get_launch_mode", get_launch_mode)?;

    let save_data = lua.create_function(|_, (key, value): (String, Value)| {
        save_lua_data(&key, &value)?;
        Ok(true)
    })?;
    lua.globals().set("save_data", save_data)?;

    let load_data = lua.create_function(|lua, key: String| load_lua_data(lua, &key))?;
    lua.globals().set("load_data", load_data)?;

    let save_game_slot = lua.create_function(|_, (game_id, value): (String, Value)| {
        save_game_slot_data(&game_id, &value)?;
        Ok(true)
    })?;
    lua.globals().set("save_game_slot", save_game_slot)?;

    let load_game_slot =
        lua.create_function(|lua, game_id: String| load_lua_data(lua, &game_slot_key(&game_id)))?;
    lua.globals().set("load_game_slot", load_game_slot)?;

    let update_game_stats = lua.create_function(
        |_, (game_id, score, duration_sec): (String, i64, i64)| {
            let score_u32 = score.max(0).min(u32::MAX as i64) as u32;
            let duration_u64 = duration_sec.max(0) as u64;
            stats::update_game_stats(&game_id, score_u32, duration_u64)
                .map_err(mlua::Error::external)?;
            Ok(true)
        },
    )?;
    lua.globals().set("update_game_stats", update_game_stats)?;

    Ok(())
}

/// Runs a Lua game script and returns when the script exits.
pub fn run_game_script(script_path: &Path, mode: LaunchMode) -> Result<()> {
    drain_input_events();
    let source = fs::read_to_string(script_path)?;
    let source = source.trim_start_matches('\u{feff}');
    let lua = Lua::new();
    register_api(&lua, mode).map_err(|e| anyhow!("Lua API registration error: {e}"))?;

    let result = match lua.load(source).set_name(script_path.to_string_lossy()).exec() {
        Ok(()) => Ok(()),
        Err(err) if err.to_string().contains(EXIT_GAME_SENTINEL) => Ok(()),
        Err(err) => Err(anyhow!("Lua runtime error: {err}")),
    };

    finalize_terminal_after_script();
    TERMINAL_DIRTY_FROM_LUA.store(true, Ordering::Release);
    result
}

/// Returns whether Lua wrote directly to terminal since last check.
pub fn take_terminal_dirty_from_lua() -> bool {
    TERMINAL_DIRTY_FROM_LUA.swap(false, Ordering::AcqRel)
}

/// Returns latest saved game id from shared Lua save store.
pub fn latest_saved_game_id() -> Option<String> {
    let store = load_json_store().ok()?;
    if let Some(JsonValue::String(id)) = store.get("__latest_save_game") {
        let normalized = id.trim().to_string();
        if !normalized.is_empty() {
            return Some(normalized);
        }
    }
    for key in store.keys() {
        if let Some(id) = key.strip_prefix("game:") {
            if !id.trim().is_empty() {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// Clears active game save slot metadata and all game slot payloads.
///
/// This does not remove per-game best records or other auxiliary data.
pub fn clear_active_game_save() -> Result<()> {
    let mut store =
        load_json_store().map_err(|e| anyhow!("failed to load lua save store for clearing: {e}"))?;
    clear_game_slots(&mut store);
    write_json_store(&store).map_err(|e| anyhow!("failed to write lua save store after clear: {e}"))
}

fn draw_text_impl(
    x: i64,
    y: i64,
    text: &str,
    fg: Option<&str>,
    bg: Option<&str>,
) -> mlua::Result<()> {
    let mut out = lock_out()?;
    if let Some(color) = parse_color(fg) {
        queue!(out, SetForegroundColor(color)).map_err(mlua::Error::external)?;
    }
    if let Some(color) = parse_color(bg) {
        queue!(out, SetBackgroundColor(color)).map_err(mlua::Error::external)?;
    }
    queue!(
        out,
        crossterm::cursor::MoveTo(coord_to_terminal(x), coord_to_terminal(y)),
        Print(text),
        ResetColor
    )
    .map_err(mlua::Error::external)?;
    Ok(())
}

fn lock_out() -> mlua::Result<MutexGuard<'static, Stdout>> {
    OUT.lock()
        .map_err(|_| mlua::Error::external("stdout lock poisoned"))
}

fn flush_output() -> mlua::Result<()> {
    let mut out = lock_out()?;
    out.flush().map_err(mlua::Error::external)
}

fn finalize_terminal_after_script() {
    if let Ok(mut out) = OUT.lock() {
        let _ = queue!(out, ResetColor, crossterm::cursor::MoveTo(0, 0));
        let _ = out.flush();
    }

    drain_input_events();
}

fn drain_input_events() {
    loop {
        match event::poll(Duration::from_millis(0)) {
            Ok(true) => {
                let _ = event::read();
            }
            _ => break,
        }
    }
}

fn keycode_to_string(code: KeyCode) -> String {
    match code {
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Delete => "delete".to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Char(' ') => "space".to_string(),
        KeyCode::Char(c) => c.to_ascii_lowercase().to_string(),
        _ => String::new(),
    }
}

fn decode_key_event(key: KeyEvent) -> mlua::Result<String> {
    if key.code != KeyCode::Esc {
        return Ok(keycode_to_string(key.code));
    }

    // Some terminals emit arrow keys as ESC [ A/B/C/D or ESC O A/B/C/D.
    if let Some(mapped) = try_read_escaped_arrow()? {
        return Ok(mapped);
    }

    Ok("esc".to_string())
}

fn try_read_escaped_arrow() -> mlua::Result<Option<String>> {
    if !event::poll(Duration::from_millis(2)).map_err(mlua::Error::external)? {
        return Ok(None);
    }
    let first = match event::read().map_err(mlua::Error::external)? {
        Event::Key(k) if k.kind == KeyEventKind::Press => k,
        _ => return Ok(None),
    };

    let prefix_ok = matches!(first.code, KeyCode::Char('[') | KeyCode::Char('O'));
    if !prefix_ok {
        return Ok(None);
    }

    if !event::poll(Duration::from_millis(2)).map_err(mlua::Error::external)? {
        return Ok(None);
    }
    let second = match event::read().map_err(mlua::Error::external)? {
        Event::Key(k) if k.kind == KeyEventKind::Press => k,
        _ => return Ok(None),
    };

    let mapped = match second.code {
        KeyCode::Char('A') | KeyCode::Char('a') => Some("up".to_string()),
        KeyCode::Char('B') | KeyCode::Char('b') => Some("down".to_string()),
        KeyCode::Char('C') | KeyCode::Char('c') => Some("right".to_string()),
        KeyCode::Char('D') | KeyCode::Char('d') => Some("left".to_string()),
        _ => None,
    };
    Ok(mapped)
}

fn coord_to_terminal(v: i64) -> u16 {
    if v <= 0 {
        0
    } else {
        (v - 1).min(u16::MAX as i64) as u16
    }
}

fn parse_color(name: Option<&str>) -> Option<CColor> {
    let raw = name.unwrap_or("").trim();
    if let Some(hex) = parse_hex_color(raw) {
        return Some(hex);
    }
    if let Some(rgb) = parse_rgb_color(raw) {
        return Some(rgb);
    }

    match raw.to_ascii_lowercase().as_str() {
        "black" => Some(CColor::Black),
        "white" => Some(CColor::White),
        "red" => Some(CColor::Red),
        "light_red" => Some(CColor::Red),
        "dark_red" => Some(CColor::DarkRed),
        "yellow" => Some(CColor::Yellow),
        "light_yellow" => Some(CColor::Yellow),
        "dark_yellow" => Some(CColor::DarkYellow),
        "orange" => Some(CColor::DarkYellow),
        "green" => Some(CColor::Green),
        "light_green" => Some(CColor::Green),
        "blue" => Some(CColor::Blue),
        "light_blue" => Some(CColor::Blue),
        "cyan" => Some(CColor::Cyan),
        "light_cyan" => Some(CColor::Cyan),
        "magenta" => Some(CColor::Magenta),
        "light_magenta" => Some(CColor::Magenta),
        "grey" | "gray" => Some(CColor::Grey),
        "dark_grey" | "dark_gray" => Some(CColor::DarkGrey),
        _ => None,
    }
}

fn parse_hex_color(raw: &str) -> Option<CColor> {
    if raw.len() != 7 || !raw.starts_with('#') {
        return None;
    }
    let r = u8::from_str_radix(&raw[1..3], 16).ok()?;
    let g = u8::from_str_radix(&raw[3..5], 16).ok()?;
    let b = u8::from_str_radix(&raw[5..7], 16).ok()?;
    Some(CColor::Rgb { r, g, b })
}

fn parse_rgb_color(raw: &str) -> Option<CColor> {
    let lower = raw.to_ascii_lowercase();
    if !lower.starts_with("rgb(") || !lower.ends_with(')') {
        return None;
    }
    let inner = &lower[4..lower.len() - 1];
    let mut parts = inner.split(',').map(|s| s.trim().parse::<u8>().ok());
    let r = parts.next()??;
    let g = parts.next()??;
    let b = parts.next()??;
    if parts.next().is_some() {
        return None;
    }
    Some(CColor::Rgb { r, g, b })
}

fn next_random_u64() -> u64 {
    let mut cur = RNG_STATE.load(Ordering::Relaxed);
    if cur == 0 {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        let seeded = if seed == 0 { 0xA409_3822_299F_31D0 } else { seed };
        let _ = RNG_STATE.compare_exchange(0, seeded, Ordering::SeqCst, Ordering::Relaxed);
        cur = RNG_STATE.load(Ordering::Relaxed);
    }

    loop {
        let mut x = cur;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        if x == 0 {
            x = 0x2545_F491_4F6C_DD1D;
        }
        match RNG_STATE.compare_exchange(cur, x, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(_) => return x,
            Err(actual) => cur = actual,
        }
    }
}

fn save_file_path() -> PathBuf {
    match path_utils::lua_saves_file() {
        Ok(path) => path,
        Err(_) => PathBuf::from("lua_saves.json"),
    }
}

fn load_json_store() -> mlua::Result<Map<String, JsonValue>> {
    let path = save_file_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(mlua::Error::external)?;
        }
        fs::write(&path, "{}").map_err(mlua::Error::external)?;
        return Ok(Map::new());
    }

    let raw = fs::read_to_string(path).map_err(mlua::Error::external)?;
    let parsed = serde_json::from_str::<JsonValue>(&raw).unwrap_or(JsonValue::Object(Map::new()));
    if let JsonValue::Object(map) = parsed {
        Ok(map)
    } else {
        Ok(Map::new())
    }
}

fn write_json_store(store: &Map<String, JsonValue>) -> mlua::Result<()> {
    let path = save_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(mlua::Error::external)?;
    }
    let payload = serde_json::to_string_pretty(store).map_err(mlua::Error::external)?;
    fs::write(path, payload).map_err(mlua::Error::external)?;
    Ok(())
}

fn save_lua_data(key: &str, value: &Value) -> mlua::Result<()> {
    let mut store = load_json_store()?;
    let json = lua_to_json(value)?;
    store.insert(key.to_string(), json);
    write_json_store(&store)
}

fn save_game_slot_data(game_id: &str, value: &Value) -> mlua::Result<()> {
    let mut store = load_json_store()?;
    clear_game_slots(&mut store);
    let json = lua_to_json(value)?;
    let game_id = game_id.trim().to_ascii_lowercase();
    store.insert(game_slot_key(&game_id), json);
    store.insert("__latest_save_game".to_string(), JsonValue::String(game_id));
    write_json_store(&store)
}

fn clear_game_slots(store: &mut Map<String, JsonValue>) {
    store.retain(|key, _| key != "__latest_save_game" && !key.starts_with("game:"));
}

fn game_slot_key(game_id: &str) -> String {
    format!("game:{}", game_id.trim().to_ascii_lowercase())
}

fn load_lua_data(lua: &Lua, key: &str) -> mlua::Result<Value> {
    let store = load_json_store()?;
    if let Some(v) = store.get(key) {
        json_to_lua(lua, v)
    } else {
        Ok(Value::Nil)
    }
}

fn lua_to_json(value: &Value) -> mlua::Result<JsonValue> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(v) => Ok(JsonValue::Bool(*v)),
        Value::Integer(v) => Ok(JsonValue::Number(Number::from(*v))),
        Value::Number(v) => Number::from_f64(*v)
            .map(JsonValue::Number)
            .ok_or_else(|| mlua::Error::external("invalid lua number")),
        Value::String(v) => Ok(JsonValue::String(v.to_str()?.to_string())),
        Value::Table(t) => table_to_json(t),
        _ => Err(mlua::Error::external("unsupported lua value type for save_data")),
    }
}

fn table_to_json(table: &Table) -> mlua::Result<JsonValue> {
    let mut as_array: BTreeMap<usize, JsonValue> = BTreeMap::new();
    let mut as_object = Map::new();
    let mut array_only = true;

    for pair in table.pairs::<Value, Value>() {
        let (k, v) = pair?;
        match k {
            Value::Integer(i) if i > 0 => as_array.insert(i as usize, lua_to_json(&v)?),
            Value::String(s) => {
                array_only = false;
                as_object.insert(s.to_str()?.to_string(), lua_to_json(&v)?);
                None
            }
            _ => {
                array_only = false;
                as_object.insert(format!("{k:?}"), lua_to_json(&v)?);
                None
            }
        };
    }

    if array_only && !as_array.is_empty() {
        let mut list = Vec::new();
        let max = *as_array.keys().max().unwrap_or(&0);
        for idx in 1..=max {
            if let Some(v) = as_array.get(&idx) {
                list.push(v.clone());
            } else {
                list.push(JsonValue::Null);
            }
        }
        Ok(JsonValue::Array(list))
    } else {
        for (k, v) in as_array {
            as_object.insert(k.to_string(), v);
        }
        Ok(JsonValue::Object(as_object))
    }
}

fn json_to_lua(lua: &Lua, value: &JsonValue) -> mlua::Result<Value> {
    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(v) => Ok(Value::Boolean(*v)),
        JsonValue::Number(v) => {
            if let Some(i) = v.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = v.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        JsonValue::String(v) => Ok(Value::String(lua.create_string(v)?)),
        JsonValue::Array(items) => {
            let t = lua.create_table()?;
            for (idx, item) in items.iter().enumerate() {
                t.set((idx + 1) as i64, json_to_lua(lua, item)?)?;
            }
            Ok(Value::Table(t))
        }
        JsonValue::Object(map) => {
            let t = lua.create_table()?;
            for (k, v) in map {
                t.set(k.as_str(), json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(t))
        }
    }
}

