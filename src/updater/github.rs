use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

use crate::utils::path_utils;

const GITHUB_API_LATEST: &str = "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";
const FALLBACK_RELEASE_URL: &str = "https://github.com/MXBraisedFish/TUI-GAME/releases/latest";
pub const GITHUB_TOKEN: &str = "";
pub const CURRENT_VERSION_TAG: &str = "v0.1.5";

#[derive(Clone, Debug)]
pub struct UpdateNotification {
    pub latest_version: String,
    pub release_url: String,
}

#[derive(Clone, Debug)]
pub enum UpdaterEvent {
    NewVersion(UpdateNotification),
    NoUpdate,
}

#[derive(Debug)]
pub struct Updater {
    receiver: Receiver<UpdaterEvent>,
}

#[derive(Clone, Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    html_url: Option<String>,
}

impl Updater {
    /// Starts a background updater check thread.
    pub fn spawn(current_version: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let current = normalize_tag(current_version);

        thread::spawn(move || {
            if let Ok(result) = check_for_update(&current) {
                match result {
                    Some(notification) => {
                        let _ = tx.send(UpdaterEvent::NewVersion(notification));
                    }
                    None => {
                        let _ = tx.send(UpdaterEvent::NoUpdate);
                    }
                }
            }
        });

        Self { receiver: rx }
    }

    /// Non-blocking poll for updater events.
    pub fn try_recv(&self) -> Option<UpdaterEvent> {
        self.receiver.try_recv().ok()
    }
}

fn check_for_update(current_version: &str) -> Result<Option<UpdateNotification>> {
    ensure_cache_initialized()?;

    let client = Client::builder().timeout(Duration::from_secs(8)).build()?;
    let mut req = client
        .get(GITHUB_API_LATEST)
        .header(USER_AGENT, "tui-game-updater")
        .header(ACCEPT, "application/vnd.github+json");

    if !GITHUB_TOKEN.is_empty() {
        req = req.header(AUTHORIZATION, format!("Bearer {}", GITHUB_TOKEN));
    }

    let response = match req.send() {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    if !response.status().is_success() {
        return Ok(None);
    }

    let payload: ReleaseResponse = match response.json() {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

    let latest_tag = normalize_tag(&payload.tag_name);
    let release_url = payload
        .html_url
        .unwrap_or_else(|| FALLBACK_RELEASE_URL.to_string());

    if latest_tag != normalize_tag(current_version) {
        Ok(Some(UpdateNotification {
            latest_version: latest_tag,
            release_url,
        }))
    } else {
        Ok(None)
    }
}

fn ensure_cache_initialized() -> Result<()> {
    let path = path_utils::updater_cache_file()?;
    if path.exists() {
        return Ok(());
    }
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, format!("\"{}\"\n", CURRENT_VERSION_TAG))?;
    Ok(())
}

fn normalize_tag(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return CURRENT_VERSION_TAG.to_string();
    }
    if trimmed.starts_with('v') || trimmed.starts_with('V') {
        format!("v{}", trimmed[1..].trim())
    } else {
        format!("v{}", trimmed)
    }
}

/// Runs external updater script (version.bat/version.sh) and returns whether it was started.
pub fn run_external_update_script(notification: &UpdateNotification) -> Result<bool> {
    let runtime = path_utils::runtime_dir()?;
    let bat = runtime.join("version.bat");
    let sh = runtime.join("version.sh");

    let Some(script) = select_version_script(&bat, &sh) else {
        return Ok(false);
    };

    let ext = script
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if ext == "bat" {
        let _child = Command::new("cmd")
            .arg("/C")
            .arg(script.as_os_str())
            .arg(notification.latest_version.as_str())
            .arg(notification.release_url.as_str())
            .spawn()?;
        return Ok(true);
    }

    let _child = Command::new("sh")
        .arg(script.as_os_str())
        .arg(notification.latest_version.as_str())
        .arg(notification.release_url.as_str())
        .spawn()?;
    Ok(true)
}

fn select_version_script(bat: &Path, sh: &Path) -> Option<PathBuf> {
    if bat.exists() {
        return Some(bat.to_path_buf());
    }
    if sh.exists() {
        return Some(sh.to_path_buf());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{normalize_tag, select_version_script};

    #[test]
    fn normalize_tag_adds_prefix() {
        assert_eq!(normalize_tag("0.1.4"), "v0.1.4");
        assert_eq!(normalize_tag("v0.1.4"), "v0.1.4");
    }

    #[test]
    fn script_selection_prefers_bat_then_sh() {
        let base = std::env::temp_dir().join("tui_game_updater_script_select");
        let _ = std::fs::create_dir_all(&base);
        let bat = base.join("version.bat");
        let sh = base.join("version.sh");

        let _ = std::fs::remove_file(&bat);
        let _ = std::fs::remove_file(&sh);
        assert!(select_version_script(&bat, &sh).is_none());

        let _ = std::fs::write(&sh, "echo sh");
        assert_eq!(select_version_script(&bat, &sh), Some(sh.clone()));

        let _ = std::fs::write(&bat, "echo bat");
        assert_eq!(select_version_script(&bat, &sh), Some(bat.clone()));

        let _ = std::fs::remove_file(&bat);
        let _ = std::fs::remove_file(&sh);
        let _ = std::fs::remove_dir_all(&base);
    }
}
