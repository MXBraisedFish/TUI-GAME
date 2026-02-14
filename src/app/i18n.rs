use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use anyhow::Result;
use once_cell::sync::Lazy;
use serde_json::Value;

use crate::utils::path_utils;

const REQUIRED_KEYS: [&str; 3] = ["language_name", "language", "confirm_language"];

#[derive(Clone, Debug)]
pub struct LanguagePack {
    pub code: String,
    pub name: String,
    pub dict: HashMap<String, String>,
}

#[derive(Clone, Debug)]
struct I18nState {
    packs: Vec<LanguagePack>,
    fallback: LanguagePack,
    current_code: String,
}

static I18N: Lazy<RwLock<I18nState>> = Lazy::new(|| {
    let fallback = builtin_english_pack();
    RwLock::new(I18nState {
        packs: vec![fallback.clone()],
        fallback: fallback.clone(),
        current_code: fallback.code.clone(),
    })
});

/// Initializes i18n by loading language packs from assets/lang.
pub fn init(default_code: &str) -> Result<()> {
    let mut packs = load_language_packs()?;
    packs.sort_by(|a, b| b.name.cmp(&a.name));

    let fallback = packs
        .iter()
        .find(|pack| pack.code == "us-en")
        .cloned()
        .unwrap_or_else(builtin_english_pack);

    if packs.is_empty() {
        packs.push(fallback.clone());
    }

    let preferred_code = load_persisted_language_code()
        .ok()
        .flatten()
        .unwrap_or_else(|| default_code.to_string());

    let current_code = if packs.iter().any(|pack| pack.code == preferred_code) {
        preferred_code
    } else if packs.iter().any(|pack| pack.code == default_code) {
        default_code.to_string()
    } else if packs.iter().any(|pack| pack.code == "us-en") {
        "us-en".to_string()
    } else {
        fallback.code.clone()
    };

    if let Ok(mut state) = I18N.write() {
        *state = I18nState {
            packs,
            fallback,
            current_code,
        };
    }

    Ok(())
}

/// Returns all valid language packs sorted by Unicode descending of language_name.
pub fn available_languages() -> Vec<LanguagePack> {
    if let Ok(state) = I18N.read() {
        return state.packs.clone();
    }
    vec![builtin_english_pack()]
}

/// Returns current active language code.
pub fn current_language_code() -> String {
    if let Ok(state) = I18N.read() {
        return state.current_code.clone();
    }
    "us-en".to_string()
}

/// Switches active language by code.
pub fn set_language(code: &str) -> bool {
    if let Ok(mut state) = I18N.write() {
        if state.packs.iter().any(|pack| pack.code == code) {
            state.current_code = code.to_string();
            let _ = save_persisted_language_code(code);
            return true;
        }
    }
    false
}

/// Looks up a key in current language with built-in English fallback.
pub fn t(key: &str) -> String {
    if let Ok(state) = I18N.read() {
        if let Some(current_pack) = state
            .packs
            .iter()
            .find(|pack| pack.code == state.current_code)
        {
            if let Some(value) = current_pack.dict.get(key) {
                return value.clone();
            }
        }

        if let Some(value) = state.fallback.dict.get(key) {
            return value.clone();
        }
    }

    format!("[missing-i18n-key:{}]", key)
}

/// Looks up a key in a specific language code with English fallback.
pub fn t_for_code(code: &str, key: &str) -> String {
    if let Ok(state) = I18N.read() {
        if let Some(pack) = state.packs.iter().find(|pack| pack.code == code) {
            if let Some(value) = pack.dict.get(key) {
                return value.clone();
            }
        }

        if let Some(value) = state.fallback.dict.get(key) {
            return value.clone();
        }
    }

    format!("[missing-i18n-key:{}]", key)
}

/// Looks up a key in current language and falls back to provided text when missing.
pub fn t_or(key: &str, fallback: &str) -> String {
    let value = t(key);
    if value.starts_with("[missing-i18n-key:") {
        fallback.to_string()
    } else {
        value
    }
}

fn load_language_packs() -> Result<Vec<LanguagePack>> {
    let mut packs = Vec::new();
    for lang_dir in resolve_lang_dirs() {
        for entry in fs::read_dir(lang_dir)? {
            let path = match entry {
                Ok(e) => e.path(),
                Err(_) => continue,
            };

            if !path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
            {
                continue;
            }

            if let Some(pack) = parse_language_pack(&path)? {
                packs.push(pack);
            }
        }
        if !packs.is_empty() {
            break;
        }
    }

    Ok(packs)
}

fn parse_language_pack(path: &Path) -> Result<Option<LanguagePack>> {
    let code = match path.file_stem().and_then(|s| s.to_str()) {
        Some(stem) => stem.to_ascii_lowercase(),
        None => return Ok(None),
    };

    let content = fs::read_to_string(path)?;
    let content = content.trim_start_matches('\u{feff}');
    let value: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    let object = match value.as_object() {
        Some(map) => map,
        None => return Ok(None),
    };

    if !REQUIRED_KEYS
        .iter()
        .all(|key| object.get(*key).and_then(Value::as_str).is_some())
    {
        return Ok(None);
    }

    let mut dict = HashMap::new();
    for (key, value) in object {
        if let Some(text) = value.as_str() {
            dict.insert(key.to_string(), text.to_string());
        }
    }

    let name = match dict.get("language_name") {
        Some(v) => v.clone(),
        None => return Ok(None),
    };

    Ok(Some(LanguagePack { code, name, dict }))
}

fn resolve_lang_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let candidate = ancestor.join("assets").join("lang");
            if candidate.exists() {
                dirs.push(candidate);
            }
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            for ancestor in parent.ancestors() {
                let candidate = ancestor.join("assets").join("lang");
                if candidate.exists() && !dirs.iter().any(|d| d == &candidate) {
                    dirs.push(candidate);
                }
            }
        }
    }

    dirs
}

fn load_persisted_language_code() -> Result<Option<String>> {
    let path = path_utils::language_pref_file()?;
    if !path.exists() {
        return Ok(None);
    }

    let code = fs::read_to_string(path)?;
    let normalized = code.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        Ok(None)
    } else {
        Ok(Some(normalized))
    }
}

fn save_persisted_language_code(code: &str) -> Result<()> {
    let path = path_utils::language_pref_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, format!("{}\n", code.trim().to_ascii_lowercase()))?;
    Ok(())
}

fn builtin_english_pack() -> LanguagePack {
    let mut dict = HashMap::new();
    dict.insert("language_name".to_string(), "English".to_string());
    dict.insert("language".to_string(), "Language".to_string());
    dict.insert(
        "confirm_language".to_string(),
        "[Enter] Confirm language. [ESC] or [Q] Return to main menu".to_string(),
    );
    dict.insert("menu.play".to_string(), "Play Game".to_string());
    dict.insert("menu.continue".to_string(), "Continue".to_string());
    dict.insert("menu.settings".to_string(), "Settings".to_string());
    dict.insert("menu.about".to_string(), "About".to_string());
    dict.insert("menu.quit".to_string(), "Quit".to_string());
    dict.insert(
        "settings.hub.language".to_string(),
        "Language".to_string(),
    );
    dict.insert(
        "settings.hub.uninstall".to_string(),
        "Uninstall TUI GAME".to_string(),
    );
    dict.insert(
        "settings.hub.back_hint".to_string(),
        "[ESC]/[Q] Back to main menu".to_string(),
    );
    dict.insert(
        "placeholder.settings".to_string(),
        "Settings page is under construction. Please check back later.".to_string(),
    );
    dict.insert(
        "placeholder.about".to_string(),
        "TUI GAME\nVersion: 0.1.0\nAuthor: 123\nGitHub: https://github.com/your-username/tui-game".to_string(),
    );
    dict.insert(
        "placeholder.continue".to_string(),
        "Continue feature is not implemented yet.".to_string(),
    );
    dict.insert("updater.new_version".to_string(), "New version available".to_string());
    dict.insert("updater.press_u".to_string(), "Press U to open release page".to_string());
    dict.insert("updater.no_update".to_string(), "You are up to date".to_string());
    dict.insert("warning.size_title".to_string(), "Terminal Too Small".to_string());
    dict.insert("warning.required".to_string(), "Required size".to_string());
    dict.insert("warning.current".to_string(), "Current size".to_string());
    dict.insert(
        "warning.enlarge_hint".to_string(),
        "Please enlarge terminal window to continue.".to_string(),
    );
    dict.insert(
        "common.back_hint".to_string(),
        "Press ESC or Q to return to main menu".to_string(),
    );
    dict.insert(
        "confirm.new_game_overwrite".to_string(),
        "There is a save from {game}. Starting a new game will overwrite it. Continue?"
            .to_string(),
    );
    dict.insert(
        "confirm.new_game_yes".to_string(),
        "[Y] Start New Game".to_string(),
    );
    dict.insert("confirm.new_game_no".to_string(), "[N] Cancel".to_string());
    dict.insert("games.empty".to_string(), "No Lua games found in scripts/".to_string());
    dict.insert(
        "games.run_pending".to_string(),
        "Press Enter to run selected game (runtime framework pending)".to_string(),
    );

    LanguagePack {
        code: "us-en".to_string(),
        name: "English".to_string(),
        dict,
    }
}
