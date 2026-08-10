use std::fs;
use std::io::ErrorKind;

use super::layout;
use super::service::StorageService;
use crate::host_engine::services::{LogService, LogSource};

/// 确保存储目录和默认文件存在，缺失时自动创建。
pub fn ensure_storage_layout(storage: &StorageService, log: &mut LogService) {
  ensure_required_directories(storage, log);
  migrate_legacy_log(storage, log);
  migrate_legacy_package_logs(storage, log);
  ensure_default_files(storage, log);
}

fn migrate_legacy_package_logs(storage: &StorageService, log: &mut LogService) {
  for source in ["official", "mod"] {
    for kind in ["game", "screensaver"] {
      let legacy = storage.path(&format!("data/log/package/{source}/{kind}"));
      if !legacy.is_dir() {
        continue;
      }
      let target = storage.path(&format!("data/log/{kind}"));
      let Ok(entries) = fs::read_dir(&legacy) else {
        log.warn(
          LogSource::Storage,
          format!(
            "Failed to read legacy package log directory {}",
            legacy.display()
          ),
        );
        continue;
      };
      for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("log") {
          continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
          continue;
        };
        let destination = target.join(format!("{source}_{name}"));
        let result = if destination.exists() {
          fs::read(&path).and_then(|bytes| {
            use std::io::Write;
            let mut file = fs::OpenOptions::new().append(true).open(&destination)?;
            if !bytes.is_empty() {
              file.write_all(&bytes)?;
              file.flush()?;
            }
            fs::remove_file(&path)
          })
        } else {
          fs::rename(&path, &destination)
        };
        if let Err(error) = result {
          log.warn(
            LogSource::Storage,
            format!(
              "Failed to migrate legacy package log {}: {error}",
              path.display()
            ),
          );
        }
      }
      let _ = fs::remove_dir(&legacy);
    }
  }
  let package_root = storage.path("data/log/package");
  for source in ["official", "mod"] {
    let _ = fs::remove_dir(package_root.join(source));
  }
  let _ = fs::remove_dir(package_root);
}

fn migrate_legacy_log(storage: &StorageService, log: &mut LogService) {
  let legacy = storage.path(layout::LEGACY_TUI_LOG_FILE);
  let current = storage.path(layout::TUI_LOG_FILE);
  if !legacy.is_file() {
    return;
  }
  let result = if current.exists() {
    fs::read(&legacy).and_then(|bytes| {
      use std::io::Write;
      let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&current)?;
      if !bytes.is_empty() {
        if current.metadata().is_ok_and(|metadata| metadata.len() > 0) {
          file.write_all(b"\n")?;
        }
        file.write_all(&bytes)?;
        file.flush()?;
      }
      fs::remove_file(&legacy)
    })
  } else {
    fs::rename(&legacy, &current)
  };
  match result {
    Ok(()) => log.info(LogSource::Storage, "Migrated tui_log.txt to tui_log.log"),
    Err(error) => log.warn(
      LogSource::Storage,
      format!("Failed to migrate legacy log file: {error}"),
    ),
  }
}

fn ensure_required_directories(storage: &StorageService, log: &mut LogService) {
  for relative_dir in layout::REQUIRED_DIRECTORIES {
    let path = storage.path(relative_dir);
    if let Err(error) = fs::create_dir_all(&path) {
      log.error(
        LogSource::Storage,
        format!("Failed to create directory {}: {}", path.display(), error),
      );
      log.fatal(
        LogSource::Boot,
        format!(
          "Critical directory creation failed: {}: {err}",
          path.display(),
          err = error
        ),
      );
    }
  }
}

fn ensure_default_files(storage: &StorageService, log: &mut LogService) {
  for (relative_file, default_content) in layout::DEFAULT_FILES {
    let path = storage.path(relative_file);
    match fs::metadata(&path) {
      Ok(metadata) => {
        if metadata.is_file() && metadata.len() > 0 {
          continue;
        }
      }
      Err(error) => {
        if error.kind() != ErrorKind::NotFound {
          log.warn(
            LogSource::Storage,
            format!("Cannot access file {}: {}", path.display(), error),
          );
        }
      }
    }
    if let Some(parent) = path.parent() {
      if let Err(error) = fs::create_dir_all(parent) {
        log.error(
          LogSource::Storage,
          format!(
            "Failed to create parent directory {}: {}",
            parent.display(),
            error
          ),
        );

        continue;
      }
    }
    if let Err(error) = fs::write(&path, default_content) {
      log.error(
        LogSource::Storage,
        format!(
          "Failed to create default file {}: {}",
          path.display(),
          error
        ),
      );
    }
  }
}
