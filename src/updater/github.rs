use std::cmp::Ordering;
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

// github的release的地址API
const GITHUB_API_LATEST: &str =
    "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";
// 备用URL
const FALLBACK_RELEASE_URL: &str =
    "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest";
// 这里是开发者测试防止限制API
pub const GITHUB_TOKEN: &str = "";
// 硬编码版本,避免文件被篡改导致错误(记的更新啊!)
pub const CURRENT_VERSION_TAG: &str = "0.10.2";

// 派生宏,实话说我没搞明白,但AI告诉我这么写合适就这么写了

// 更新通知
#[derive(Clone, Debug)]
pub struct UpdateNotification {
    pub latest_version: String, // 最新版本号
    pub release_url: String, // 发布页面URL
}

// 更新器主体
#[derive(Clone, Debug)]
pub enum UpdaterEvent {
    LatestVersion(UpdateNotification), // 当前已是最新版本
    NewVersion(UpdateNotification), // 发现新版本
    NoUpdate, // 没有更新
}

// 接收更新事件
#[derive(Debug)]
pub struct Updater {
    receiver: Receiver<UpdaterEvent>, // 接收更新事件的通道
}

// GitHub API 响应结构
#[derive(Clone, Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String, // 发布的标签名
    html_url: Option<String>, // 发布页面的 URL
}

// 版本更新主体
impl Updater {
    /// Starts a background updater check thread.
    pub fn spawn(current_version: &str) -> Self {
        // 创建通道
        let (tx, rx) = mpsc::channel();

        // 规范化当前版本
        let current = normalize_tag(current_version);

        // 写入缓存文件
        let _ = write_current_version_cache(&current);

        // 启动后台线程检查更新
        thread::spawn(move || {
            if let Ok(result) = fetch_latest_release() {
                if let Some(latest) = result {
                    // 发送LatestVersion事件
                    let _ = tx.send(UpdaterEvent::LatestVersion(latest.clone()));

                    // 判断是否有新版本
                    if is_version_newer(&latest.latest_version, &current) {
                        let _ = tx.send(UpdaterEvent::NewVersion(latest));
                    } else {
                        let _ = tx.send(UpdaterEvent::NoUpdate);
                    }
                }
            }
        });

        Self { receiver: rx }
    }

    // 非阻塞接收事
    pub fn try_recv(&self) -> Option<UpdaterEvent> {
        self.receiver.try_recv().ok()
    }
}

// 获取最后一个release
fn fetch_latest_release() -> Result<Option<UpdateNotification>> {
    // 创建HTTP客户端(8秒后超时)
    let client = Client::builder().timeout(Duration::from_secs(8)).build()?;

    // 构建请求
    let mut req = client
        .get(GITHUB_API_LATEST)
        .header(USER_AGENT, "tui-game-updater")
        .header(ACCEPT, "application/vnd.github+json");

        // 如果有token,添加到请求头(开发者提高API限流)
    if !GITHUB_TOKEN.is_empty() {
        req = req.header(AUTHORIZATION, format!("Bearer {}", GITHUB_TOKEN));
    }

    // 发送请求
    let response = match req.send() {
        Ok(r) => r,
        Err(_) => return Ok(None), // 网络错误返回None
    };

    // 检查HTTP状态码
    if !response.status().is_success() {
        return Ok(None); // API错误返回None
    }

    let payload: ReleaseResponse = match response.json() {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

    // 构造UpdateNotification
    let latest_tag = normalize_tag(&payload.tag_name);
    let release_url = payload
        .html_url
        .unwrap_or_else(|| FALLBACK_RELEASE_URL.to_string());

    Ok(Some(UpdateNotification {
        latest_version: latest_tag,
        release_url,
    }))
}

// 写入当前版本缓存
fn write_current_version_cache(current_version: &str) -> Result<()> {
    let path = path_utils::updater_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, format!("\"{}\"\n", normalize_tag(current_version)))?;
    Ok(())
}

// 版本格式化
fn normalize_tag(raw: &str) -> String {
    let trimmed = raw.trim();

    // 空字符串处理
    if trimmed.is_empty() {
        return format!("v{}", CURRENT_VERSION_TAG.trim_start_matches(['v', 'V']));
    }

    // 是否处理添加v符号
    if trimmed.starts_with('v') || trimmed.starts_with('V') {
        format!("v{}", trimmed[1..].trim())
    } else {
        format!("v{}", trimmed)
    }
}

// 拆解版本号用于逐级检查
fn parse_version_segments(version: &str) -> Option<Vec<u64>> {
    let clean = version.trim().trim_start_matches(['v', 'V']);
    if clean.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    for part in clean.split('.') {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let Ok(num) = part.parse::<u64>() else {
            return None;
        };
        out.push(num);
    }

    if out.is_empty() { None } else { Some(out) }
}

// 远程版本和当前版本的逐级比较
fn compare_versions(remote: &str, current: &str) -> Option<Ordering> {
    let a = parse_version_segments(remote)?;
    let b = parse_version_segments(current)?;
    let max_len = a.len().max(b.len());

    for i in 0..max_len {
        let av = *a.get(i).unwrap_or(&0);
        let bv = *b.get(i).unwrap_or(&0);
        match av.cmp(&bv) {
            Ordering::Equal => {}
            non_eq => return Some(non_eq),
        }
    }

    Some(Ordering::Equal)
}

// 远程是否更新了
fn is_version_newer(remote: &str, current: &str) -> bool {
    matches!(compare_versions(remote, current), Some(Ordering::Greater))
}

// 运行更新脚本
// windows版本
#[cfg(target_os = "windows")]
pub fn run_external_update_script(notification: &UpdateNotification) -> Result<bool> {
    let runtime = path_utils::runtime_dir()?;
    let bat = runtime.join("version.bat");
    let sh = runtime.join("version.sh");

    let Some(script) = select_version_script(&bat, &sh) else {
        return Ok(false);
    };

    let _child = Command::new("cmd")
        .arg("/C")
        .arg(script.as_os_str())
        .arg(notification.latest_version.as_str())
        .arg(notification.release_url.as_str())
        .spawn()?;
    Ok(true)
}

// unix版本
#[cfg(not(target_os = "windows"))]
pub fn run_external_update_script(notification: &UpdateNotification) -> Result<bool> {
    let runtime = path_utils::runtime_dir()?;
    let bat = runtime.join("version.bat");
    let sh = runtime.join("version.sh");

    let Some(script) = select_version_script(&bat, &sh) else {
        return Ok(false);
    };

    let _child = Command::new("sh")
        .arg(script.as_os_str())
        .arg(notification.latest_version.as_str())
        .arg(notification.release_url.as_str())
        .spawn()?;
    Ok(true)
}

// 选择运行的脚本
fn select_version_script(bat: &Path, sh: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if bat.exists() {
            return Some(bat.to_path_buf());
        }
        if sh.exists() {
            return Some(sh.to_path_buf());
        }
        return None;
    }
    #[cfg(not(target_os = "windows"))]
    {
        if sh.exists() {
            return Some(sh.to_path_buf());
        }
        if bat.exists() {
            return Some(bat.to_path_buf());
        }
        None
    }
}
