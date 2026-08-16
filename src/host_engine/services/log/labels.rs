use std::collections::HashMap;

use crate::host_engine::services::i18n::I18nService;

use super::{LogLevel, LogPhase, LogSource};

#[derive(Clone, Debug)]
pub struct LogLabels {
  values: HashMap<&'static str, String>,
}

impl LogLabels {
  pub fn new() -> Self {
    let mut labels = Self {
      values: HashMap::new(),
    };
    labels.insert_defaults();
    labels
  }

  pub fn refresh_from_i18n(&mut self, i18n: &I18nService) {
    self.insert_defaults();
    let missing_template = i18n.get_runtime_text("language_warning", "language_warning.missing");
    for key in log_label_keys() {
      let value = i18n.get_runtime_text("log", key);
      let missing = missing_template.replace("{value:missing_key}", key);
      if value != missing {
        self.values.insert(key, value);
      }
    }
  }

  pub fn phase(&self, phase: LogPhase) -> &str {
    self.label(phase.key(), phase.default_label())
  }

  pub fn source(&self, source: LogSource) -> &str {
    self.label(source.key(), source.default_label())
  }

  pub fn level(&self, level: LogLevel) -> &str {
    self.label(level.key(), level.default_label())
  }

  fn label(&self, key: &'static str, fallback: &'static str) -> &str {
    self.values.get(key).map(String::as_str).unwrap_or(fallback)
  }

  fn insert_defaults(&mut self) {
    self.values.insert(LogPhase::Boot.key(), "Boot".to_string());
    self
      .values
      .insert(LogPhase::Runtime.key(), "Runtime".to_string());
    self
      .values
      .insert(LogPhase::Shutdown.key(), "Shutdown".to_string());
    self
      .values
      .insert(LogPhase::Crash.key(), "Crash".to_string());

    for source in [
      LogSource::Engine,
      LogSource::Boot,
      LogSource::Runtime,
      LogSource::Shutdown,
      LogSource::Terminal,
      LogSource::Render,
      LogSource::Input,
      LogSource::Storage,
      LogSource::Audio,
      LogSource::Pack,
      LogSource::Lua,
      LogSource::Game,
      LogSource::Screensaver,
      LogSource::Overlay,
      LogSource::Ui,
      LogSource::Crash,
      LogSource::I18n,
    ] {
      self
        .values
        .insert(source.key(), source.default_label().to_string());
    }

    for level in [
      LogLevel::Trace,
      LogLevel::Debug,
      LogLevel::Info,
      LogLevel::Warn,
      LogLevel::Error,
      LogLevel::Fatal,
    ] {
      self
        .values
        .insert(level.key(), level.default_label().to_string());
    }
  }
}

impl Default for LogLabels {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::*;

  #[test]
  fn refresh_resets_missing_labels_to_defaults() {
    let mut labels = LogLabels::new();
    let mut translated = I18nService::new();
    translated.insert_runtime_namespace(
      "log",
      HashMap::from([
        ("log.service.lua".to_string(), "脚本".to_string()),
        ("log.service.game".to_string(), "游戏".to_string()),
        ("log.service.screensaver".to_string(), "屏保".to_string()),
        ("log.level.warn".to_string(), "警告".to_string()),
      ]),
    );
    labels.refresh_from_i18n(&translated);
    assert_eq!(labels.source(LogSource::Lua), "脚本");
    assert_eq!(labels.source(LogSource::Game), "游戏");
    assert_eq!(labels.source(LogSource::Screensaver), "屏保");
    assert_eq!(labels.level(LogLevel::Warn), "警告");

    labels.refresh_from_i18n(&I18nService::new());
    assert_eq!(labels.source(LogSource::Lua), "Lua");
  }
}

pub fn log_label_keys() -> &'static [&'static str] {
  &[
    "log.phase.boot",
    "log.phase.runtime",
    "log.phase.shutdown",
    "log.phase.crash",
    "log.service.engine",
    "log.service.boot",
    "log.service.runtime",
    "log.service.shutdown",
    "log.service.terminal",
    "log.service.render",
    "log.service.input",
    "log.service.storage",
    "log.service.audio",
    "log.service.package",
    "log.service.lua",
    "log.service.game",
    "log.service.screensaver",
    "log.service.overlay",
    "log.service.ui",
    "log.service.crash",
    "log.service.i18n",
    "log.level.trace",
    "log.level.debug",
    "log.level.info",
    "log.level.warn",
    "log.level.error",
    "log.level.fatal",
  ]
}
