use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

/// Returns current project root.
pub fn project_root() -> Result<PathBuf> {
    Ok(std::env::current_dir()?)
}

/// Returns runtime directory (where executable resides).
pub fn runtime_dir() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    project_root()
}

/// Returns app data directory near runtime executable.
pub fn app_data_dir() -> Result<PathBuf> {
    let dir = runtime_dir()?.join("tui-game-data");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns scripts directory path.
pub fn scripts_dir() -> Result<PathBuf> {
    let runtime_scripts = runtime_dir()?.join("scripts");
    if runtime_scripts.exists() {
        return Ok(runtime_scripts);
    }
    Ok(project_root()?.join("scripts"))
}

/// Returns updater cache file path in app data directory.
pub fn updater_cache_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("updater_cache.json"))
}

/// Returns language preference file path in app data directory.
pub fn language_pref_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("language_pref.txt"))
}

/// Returns Lua shared save file path in app data directory.
pub fn lua_saves_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("lua_saves.json"))
}

/// Returns game stats file path in app data directory.
pub fn stats_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("stats.json"))
}

/// Returns external version updater script path near runtime executable.
pub fn version_script_file() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        return Ok(runtime_dir()?.join("version.bat"));
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(runtime_dir()?.join("version.sh"))
    }
}

/// Ensures parent directory exists for a file path.
pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
