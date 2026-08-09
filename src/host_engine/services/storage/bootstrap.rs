use std::fs;
use std::io::ErrorKind;

use super::layout;
use super::service::StorageService;
use crate::host_engine::services::{LogService, LogSource};

/// 确保存储目录和默认文件存在，缺失时自动创建。
pub fn ensure_storage_layout(storage: &StorageService, log: &mut LogService) {
  ensure_required_directories(storage, log);
  migrate_legacy_log(storage, log);
  ensure_default_files(storage, log);
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
