use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use crate::utils::path_utils;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct GameStats {
    pub high_score: u32,
    pub max_duration_sec: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LightsOutBest {
    pub max_size: usize,
    pub min_steps: u64,
    pub min_time_sec: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryFlipBest {
    pub difficulty: usize,
    pub min_steps: u64,
    pub min_time_sec: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MinesweeperBest {
    pub d1_min_time_sec: Option<u64>,
    pub d2_min_time_sec: Option<u64>,
    pub d3_min_time_sec: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MazeEscapeBest {
    pub max_area: usize,
    pub max_cols: usize,
    pub max_rows: usize,
    pub max_mode: usize,
    pub min_time_sec: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SolitaireBest {
    pub freecell_min_time_sec: Option<u64>,
    pub klondike_min_time_sec: Option<u64>,
    pub spider_min_time_sec: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SudokuBest {
    pub difficulty: usize,
    pub min_time_sec: u64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct StatsFile {
    #[serde(default)]
    games: HashMap<String, GameStats>,
}

/// Loads per-game stats from local config file.
pub fn load_stats() -> HashMap<String, GameStats> {
    match load_stats_inner() {
        Ok(map) => map,
        Err(_) => HashMap::new(),
    }
}

/// Updates per-game stats using max(high_score) and max(max_duration_sec).
pub fn update_game_stats(game_id: &str, score: u32, duration_sec: u64) -> Result<()> {
    let path = stats_file_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, "{\n  \"games\": {}\n}\n")?;
    }

    let content = fs::read_to_string(&path)?;
    let mut parsed: StatsFile = serde_json::from_str(&content).unwrap_or_default();
    let entry = parsed.games.entry(game_id.to_string()).or_default();
    entry.high_score = entry.high_score.max(score);
    entry.max_duration_sec = entry.max_duration_sec.max(duration_sec);

    let payload = serde_json::to_string_pretty(&parsed)?;
    fs::write(path, payload)?;
    Ok(())
}

fn load_stats_inner() -> Result<HashMap<String, GameStats>> {
    let path = stats_file_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, "{\n  \"games\": {}\n}\n")?;
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path)?;
    let parsed: StatsFile = serde_json::from_str(&content).unwrap_or_default();
    Ok(parsed.games)
}

/// Formats duration seconds into HH:MM:SS.
pub fn format_duration(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

/// Loads lights_out best record from shared Lua save file.
pub fn load_lights_out_best() -> Option<LightsOutBest> {
    let path = lua_saves_file_path();
    if !path.exists() {
        return None;
    }

    let raw = fs::read_to_string(path).ok()?;
    let root = serde_json::from_str::<JsonValue>(&raw).ok()?;
    let object = root.as_object()?;
    let best = object.get("lights_out_best")?.as_object()?;

    let max_size = best.get("max_size")?.as_u64()? as usize;
    let min_steps = best.get("min_steps")?.as_u64()?;
    let min_time_sec = best.get("min_time_sec")?.as_u64()?;

    Some(LightsOutBest {
        max_size,
        min_steps,
        min_time_sec,
    })
}

/// Loads memory_flip best record from shared Lua save file.
pub fn load_memory_flip_best() -> Option<MemoryFlipBest> {
    let path = lua_saves_file_path();
    if !path.exists() {
        return None;
    }

    let raw = fs::read_to_string(path).ok()?;
    let root = serde_json::from_str::<JsonValue>(&raw).ok()?;
    let object = root.as_object()?;
    let best = object.get("memory_flip_best")?.as_object()?;

    let difficulty = best.get("difficulty")?.as_u64()? as usize;
    let min_steps = best.get("min_steps")?.as_u64()?;
    let min_time_sec = best.get("min_time_sec")?.as_u64()?;

    Some(MemoryFlipBest {
        difficulty,
        min_steps,
        min_time_sec,
    })
}

/// Loads minesweeper best record (official difficulties) from shared Lua save file.
pub fn load_minesweeper_best() -> Option<MinesweeperBest> {
    let path = lua_saves_file_path();
    if !path.exists() {
        return None;
    }

    let raw = fs::read_to_string(path).ok()?;
    let root = serde_json::from_str::<JsonValue>(&raw).ok()?;
    let object = root.as_object()?;
    let best = object.get("minesweeper_best")?.as_object()?;

    Some(MinesweeperBest {
        d1_min_time_sec: best.get("1").and_then(JsonValue::as_u64),
        d2_min_time_sec: best.get("2").and_then(JsonValue::as_u64),
        d3_min_time_sec: best.get("3").and_then(JsonValue::as_u64),
    })
}

/// Loads maze_escape best record from shared Lua save file.
pub fn load_maze_escape_best() -> Option<MazeEscapeBest> {
    let path = lua_saves_file_path();
    if !path.exists() {
        return None;
    }

    let raw = fs::read_to_string(path).ok()?;
    let root = serde_json::from_str::<JsonValue>(&raw).ok()?;
    let object = root.as_object()?;
    let best = object.get("maze_escape_best")?.as_object()?;

    let max_area = best.get("max_area").and_then(JsonValue::as_u64)? as usize;
    let max_cols = best.get("max_cols").and_then(JsonValue::as_u64).unwrap_or(0) as usize;
    let max_rows = best.get("max_rows").and_then(JsonValue::as_u64).unwrap_or(0) as usize;
    let max_mode = best.get("max_mode").and_then(JsonValue::as_u64)? as usize;
    let min_time_sec = best.get("min_time_sec").and_then(JsonValue::as_u64);

    Some(MazeEscapeBest {
        max_area,
        max_cols,
        max_rows,
        max_mode,
        min_time_sec,
    })
}

/// Loads solitaire best records (FreeCell/Klondike/Spider) from shared Lua save file.
pub fn load_solitaire_best() -> Option<SolitaireBest> {
    let path = lua_saves_file_path();
    if !path.exists() {
        return None;
    }

    let raw = fs::read_to_string(path).ok()?;
    let root = serde_json::from_str::<JsonValue>(&raw).ok()?;
    let object = root.as_object()?;

    // New format: solitaire_best_v2
    if let Some(best) = object.get("solitaire_best_v2").and_then(JsonValue::as_object) {
        let freecell = best.get("freecell").and_then(JsonValue::as_u64);
        let klondike = best.get("klondike").and_then(JsonValue::as_u64);
        let spider1 = best.get("spider1").and_then(JsonValue::as_u64);
        let spider2 = best.get("spider2").and_then(JsonValue::as_u64);
        let spider3 = best.get("spider3").and_then(JsonValue::as_u64);
        let spider = [spider1, spider2, spider3]
            .into_iter()
            .flatten()
            .filter(|v| *v > 0)
            .min();

        return Some(SolitaireBest {
            freecell_min_time_sec: freecell.filter(|v| *v > 0),
            klondike_min_time_sec: klondike.filter(|v| *v > 0),
            spider_min_time_sec: spider,
        });
    }

    // Legacy format fallback: solitaire_best
    if let Some(best) = object.get("solitaire_best").and_then(JsonValue::as_object) {
        let freecell = best
            .get("freecell")
            .and_then(JsonValue::as_u64)
            .or_else(|| best.get("foundation").and_then(JsonValue::as_u64));
        let klondike = best
            .get("klondike")
            .and_then(JsonValue::as_u64)
            .or_else(|| best.get("tableau").and_then(JsonValue::as_u64));
        let spider = best.get("spider").and_then(JsonValue::as_u64);

        return Some(SolitaireBest {
            freecell_min_time_sec: freecell.filter(|v| *v > 0),
            klondike_min_time_sec: klondike.filter(|v| *v > 0),
            spider_min_time_sec: spider.filter(|v| *v > 0),
        });
    }

    None
}

/// Loads sudoku best record from shared Lua save file.
pub fn load_sudoku_best() -> Option<SudokuBest> {
    let path = lua_saves_file_path();
    if !path.exists() {
        return None;
    }

    let raw = fs::read_to_string(path).ok()?;
    let root = serde_json::from_str::<JsonValue>(&raw).ok()?;
    let object = root.as_object()?;
    let best = object.get("sudoku_best")?.as_object()?;

    let difficulty = best
        .get("difficulty")
        .and_then(JsonValue::as_u64)
        .or_else(|| best.get("d").and_then(JsonValue::as_u64))? as usize;
    let min_time_sec = best
        .get("min_time_sec")
        .and_then(JsonValue::as_u64)
        .or_else(|| best.get("t").and_then(JsonValue::as_u64))?;

    if !(1..=5).contains(&difficulty) || min_time_sec == 0 {
        return None;
    }

    Some(SudokuBest {
        difficulty,
        min_time_sec,
    })
}

/// Loads 24-points best time from shared Lua save file.
pub fn load_twenty_four_best_time() -> Option<u64> {
    let path = lua_saves_file_path();
    if !path.exists() {
        return None;
    }

    let raw = fs::read_to_string(path).ok()?;
    let root = serde_json::from_str::<JsonValue>(&raw).ok()?;
    let object = root.as_object()?;
    let best = object.get("twenty_four_best_time")?;

    if let Some(sec) = best.as_u64() {
        return (sec > 0).then_some(sec);
    }

    if let Some(best_obj) = best.as_object() {
        let sec = best_obj
            .get("time_sec")
            .and_then(JsonValue::as_u64)
            .or_else(|| best_obj.get("best_time_sec").and_then(JsonValue::as_u64))?;
        return (sec > 0).then_some(sec);
    }

    None
}

fn stats_file_path() -> PathBuf {
    match path_utils::stats_file() {
        Ok(path) => path,
        Err(_) => PathBuf::from("stats.json"),
    }
}

fn lua_saves_file_path() -> PathBuf {
    match path_utils::lua_saves_file() {
        Ok(path) => path,
        Err(_) => PathBuf::from("lua_saves.json"),
    }
}

#[cfg(test)]
mod tests {
    use super::format_duration;

    #[test]
    fn format_duration_works() {
        assert_eq!(format_duration(1250), "00:20:50");
        assert_eq!(format_duration(3661), "01:01:01");
    }
}
