use serde::{Deserialize, Serialize};

/// 包来源（官方或模组）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageSource {
  Official,
  Mod,
}

/// 包类型（游戏或屏保）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
  Game,
  Screensaver,
}

/// 宿主内部使用的稳定包身份。版本、标题和目录名不参与身份计算。
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct PackageId {
  pub source: PackageSource,
  pub package_type: PackageType,
  pub mod_id: String,
}

impl<'de> Deserialize<'de> for PackageId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct Wire {
      source: PackageSource,
      package_type: PackageType,
      mod_id: String,
    }
    let value = Wire::deserialize(deserializer)?;
    Self::new(value.source, value.package_type, value.mod_id).map_err(serde::de::Error::custom)
  }
}

impl PackageId {
  pub fn new(
    source: PackageSource,
    package_type: PackageType,
    mod_id: impl Into<String>,
  ) -> Result<Self, String> {
    let mod_id = mod_id.into();
    validate_mod_id(&mod_id)?;
    Ok(Self {
      source,
      package_type,
      mod_id,
    })
  }

  pub fn storage_key(&self) -> String {
    format!(
      "{}/{}/{}",
      self.source.as_str(),
      self.package_type.as_str(),
      self.mod_id
    )
  }

  pub fn from_storage_key(value: &str) -> Result<Self, String> {
    let mut parts = value.splitn(3, '/');
    let source = PackageSource::from_str(parts.next().unwrap_or_default())?;
    let package_type = PackageType::from_str(parts.next().unwrap_or_default())?;
    let mod_id = parts.next().ok_or("package id is missing mod_id")?;
    Self::new(source, package_type, mod_id)
  }
}

impl std::fmt::Display for PackageId {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(&self.storage_key())
  }
}

impl PackageSource {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Official => "official",
      Self::Mod => "mod",
    }
  }

  fn from_str(value: &str) -> Result<Self, String> {
    match value {
      "official" => Ok(Self::Official),
      "mod" => Ok(Self::Mod),
      _ => Err(format!("unknown package source '{value}'")),
    }
  }
}

impl PackageType {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Game => "game",
      Self::Screensaver => "screensaver",
    }
  }

  fn from_str(value: &str) -> Result<Self, String> {
    match value {
      "game" => Ok(Self::Game),
      "screensaver" => Ok(Self::Screensaver),
      _ => Err(format!("unknown package type '{value}'")),
    }
  }
}

fn validate_mod_id(mod_id: &str) -> Result<(), String> {
  if mod_id.is_empty() {
    return Err("mod_id is empty".to_string());
  }
  if mod_id.len() > 128 {
    return Err("mod_id exceeds 128 bytes".to_string());
  }
  if !mod_id
    .bytes()
    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
  {
    return Err("mod_id may only contain ASCII letters, digits, '.', '_' and '-'".to_string());
  }
  Ok(())
}
