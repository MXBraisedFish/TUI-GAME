use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use mlua::{Lua, Table};

use crate::utils::path_utils;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub script_path: PathBuf,
}

/// Scans scripts directory for Lua game files and reads GAME_META when available.
pub fn scan_scripts() -> Result<Vec<GameMeta>> {
    let scripts_dir = path_utils::scripts_dir()?;
    scan_scripts_in(&scripts_dir)
}

fn scan_scripts_in(dir: &Path) -> Result<Vec<GameMeta>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("lua"))
                .unwrap_or(false)
        })
        .collect();

    entries.sort();

    let mut games = Vec::new();
    for path in entries {
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut name = id.replace('_', " ");
        let mut description = "No description available.".to_string();

        if let Ok(content) = fs::read_to_string(&path) {
            let content = content.trim_start_matches('\u{feff}');
            let lua = Lua::new();
            if lua.load(content).exec().is_ok() {
                let globals = lua.globals();
                if let Ok(meta) = globals.get::<Table>("GAME_META") {
                    if let Ok(v) = meta.get::<String>("name") {
                        if !v.trim().is_empty() {
                            name = v;
                        }
                    }
                    if let Ok(v) = meta.get::<String>("description") {
                        if !v.trim().is_empty() {
                            description = v;
                        }
                    }
                }
            }
        }

        games.push(GameMeta {
            id,
            name,
            description,
            script_path: path,
        });
    }

    Ok(games)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::scan_scripts_in;

    #[test]
    fn scan_scripts_finds_lua_files_only() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_dir = std::env::temp_dir().join(format!("tui_game_scan_test_{}", unique));
        let result = (|| {
            fs::create_dir_all(&temp_dir)?;
            fs::write(
                temp_dir.join("alpha.lua"),
                "GAME_META = { name = 'Alpha', description = 'A test game' }",
            )?;
            fs::write(temp_dir.join("beta.txt"), "ignore")?;
            fs::write(temp_dir.join("gamma.LUA"), "print('b')")?;

            let games = scan_scripts_in(&temp_dir)?;
            Ok::<usize, anyhow::Error>(games.len())
        })();

        let _ = fs::remove_dir_all(&temp_dir);
        let len = result.expect("scan_scripts_in should succeed");
        assert_eq!(len, 2);
    }
}
