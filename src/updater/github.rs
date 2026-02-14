use std::fs;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use semver::Version;
use serde::Deserialize;

use crate::utils::path_utils;

const GITHUB_API_LATEST: &str = "https://api.github.com/repos/your-username/tui-game/releases/latest";
const FALLBACK_RELEASE_URL: &str = "https://github.com/your-username/tui-game/releases/latest";
const CHECK_COOLDOWN_SECONDS: u64 = 24 * 60 * 60;
pub const GITHUB_TOKEN: &str = "";

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

#[derive(Clone, Debug)]
struct CacheState {
    last_check_unix: u64,
    latest_version: Option<String>,
    release_url: Option<String>,
}

impl Updater {
    /// Starts a background updater check thread.
    pub fn spawn(current_version: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let current = current_version.to_string();

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
    let current = parse_version(current_version)?;
    if let Some(cache) = load_cache()? {
        let now = now_unix();
        if now.saturating_sub(cache.last_check_unix) < CHECK_COOLDOWN_SECONDS {
            if let Some(version) = cache.latest_version {
                let latest = parse_version(&version)?;
                if latest > current {
                    return Ok(Some(UpdateNotification {
                        latest_version: format!("v{}", latest),
                        release_url: cache
                            .release_url
                            .unwrap_or_else(|| FALLBACK_RELEASE_URL.to_string()),
                    }));
                }
            }
            return Ok(None);
        }
    }

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

    let latest = parse_version(&payload.tag_name)?;
    let release_url = payload
        .html_url
        .unwrap_or_else(|| FALLBACK_RELEASE_URL.to_string());

    let cache = CacheState {
        last_check_unix: now_unix(),
        latest_version: Some(format!("v{}", latest)),
        release_url: Some(release_url.clone()),
    };
    let _ = save_cache(&cache);

    if latest > current {
        Ok(Some(UpdateNotification {
            latest_version: format!("v{}", latest),
            release_url,
        }))
    } else {
        Ok(None)
    }
}

/// Runs external updater script (version.bat/version.sh) and returns whether it was started.
pub fn run_external_update_script(notification: &UpdateNotification) -> Result<bool> {
    let script = path_utils::version_script_file()?;
    if !script.exists() {
        return Ok(false);
    }

    #[cfg(target_os = "windows")]
    {
        let _child = Command::new("cmd")
            .arg("/C")
            .arg(script.as_os_str())
            .arg(notification.latest_version.as_str())
            .arg(notification.release_url.as_str())
            .spawn()?;
        return Ok(true);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _child = Command::new("sh")
            .arg(script.as_os_str())
            .arg(notification.latest_version.as_str())
            .arg(notification.release_url.as_str())
            .spawn()?;
        Ok(true)
    }
}
fn parse_version(input: &str) -> Result<Version> {
    let normalized = input.trim().trim_start_matches('v');
    Ok(Version::parse(normalized)?)
}

fn load_cache() -> Result<Option<CacheState>> {
    let path = path_utils::updater_cache_file()?;
    if !path.exists() {
        path_utils::ensure_parent_dir(&path)?;
        fs::write(&path, "0\n\n\n")?;
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let mut lines = content.lines();
    let last_check_unix = lines
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let latest_version = lines
        .next()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let release_url = lines
        .next()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let cache = CacheState {
        last_check_unix,
        latest_version,
        release_url,
    };
    Ok(Some(cache))
}

fn save_cache(state: &CacheState) -> Result<()> {
    let path = path_utils::updater_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    let latest = state.latest_version.clone().unwrap_or_default();
    let url = state.release_url.clone().unwrap_or_default();
    let content = format!("{}\n{}\n{}\n", state.last_check_unix, latest, url);
    fs::write(path, content)?;
    Ok(())
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::parse_version;

    #[test]
    fn parse_version_accepts_prefixed_tag() {
        let parsed = parse_version("v1.2.3").expect("version should parse");
        assert_eq!(parsed, Version::new(1, 2, 3));
    }
}



