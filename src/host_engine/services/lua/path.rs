use std::fmt;
use std::path::{Path, PathBuf};

const MAX_VIRTUAL_PATH_BYTES: usize = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SandboxPathKind {
  File,
  Directory,
  WritableFile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SafeRelativePath {
  relative: PathBuf,
  virtual_path: String,
}

impl SafeRelativePath {
  pub(crate) fn parse(input: &str) -> Result<Self, SandboxPathError> {
    if input.is_empty() {
      return Err(SandboxPathError::Empty);
    }
    if input.len() > MAX_VIRTUAL_PATH_BYTES {
      return Err(SandboxPathError::TooLong);
    }
    if input.contains('\0') {
      return Err(SandboxPathError::ContainsNul);
    }

    let portable = input.replace('\\', "/");
    if portable.starts_with('/') {
      return Err(SandboxPathError::Absolute);
    }

    let mut segments = Vec::new();
    for segment in portable.split('/') {
      match segment {
        "" | "." => continue,
        ".." => return Err(SandboxPathError::ParentTraversal),
        value if !valid_portable_segment(value) => {
          return Err(SandboxPathError::InvalidSegment);
        }
        value => segments.push(value.to_string()),
      }
    }

    let mut relative = PathBuf::new();
    for segment in &segments {
      relative.push(segment);
    }
    let virtual_path = if segments.is_empty() {
      ".".to_string()
    } else {
      segments.join("/")
    };
    Ok(Self {
      relative,
      virtual_path,
    })
  }

  pub(crate) fn virtual_path(&self) -> &str {
    &self.virtual_path
  }

  pub(crate) fn extension(&self) -> Option<&str> {
    self.relative.extension().and_then(|value| value.to_str())
  }

  pub(crate) fn set_extension(&mut self, extension: &str) {
    self.relative.set_extension(extension);
    self.virtual_path = path_to_virtual(&self.relative);
  }

  pub(crate) fn is_normalized(input: &str) -> bool {
    Self::parse(input).is_ok_and(|path| path.virtual_path == input)
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SandboxPathError {
  Empty,
  TooLong,
  ContainsNul,
  Absolute,
  ParentTraversal,
  InvalidSegment,
  RootUnavailable,
  NotFound,
  ParentUnavailable,
  EscapesRoot,
  NotFile,
  NotDirectory,
}

impl fmt::Display for SandboxPathError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(match self {
      Self::Empty => "path cannot be empty",
      Self::TooLong => "path exceeds 8192 bytes",
      Self::ContainsNul => "path contains NUL",
      Self::Absolute => "absolute paths are not allowed",
      Self::ParentTraversal => "parent path '..' is not allowed",
      Self::InvalidSegment => "path contains an invalid segment",
      Self::RootUnavailable => "safe path root is unavailable",
      Self::NotFound => "path was not found",
      Self::ParentUnavailable => "path parent does not exist",
      Self::EscapesRoot => "path escapes its safe root",
      Self::NotFile => "path is not a file",
      Self::NotDirectory => "path is not a directory",
    })
  }
}

pub(crate) fn resolve_sandbox_path(
  root: &Path,
  relative: &SafeRelativePath,
  kind: SandboxPathKind,
) -> Result<PathBuf, SandboxPathError> {
  let canonical_root = root
    .canonicalize()
    .map_err(|_| SandboxPathError::RootUnavailable)?;
  if !canonical_root.is_dir() {
    return Err(SandboxPathError::RootUnavailable);
  }

  let candidate = canonical_root.join(&relative.relative);
  let resolved = if candidate.exists() {
    candidate
      .canonicalize()
      .map_err(|_| SandboxPathError::NotFound)?
  } else if kind == SandboxPathKind::WritableFile {
    let parent = candidate
      .parent()
      .ok_or(SandboxPathError::ParentUnavailable)?
      .canonicalize()
      .map_err(|_| SandboxPathError::ParentUnavailable)?;
    if !parent.starts_with(&canonical_root) {
      return Err(SandboxPathError::EscapesRoot);
    }
    candidate
  } else {
    return Err(SandboxPathError::NotFound);
  };

  if !resolved.starts_with(&canonical_root) {
    return Err(SandboxPathError::EscapesRoot);
  }
  match kind {
    SandboxPathKind::File if !resolved.is_file() => Err(SandboxPathError::NotFile),
    SandboxPathKind::Directory if !resolved.is_dir() => Err(SandboxPathError::NotDirectory),
    SandboxPathKind::WritableFile if resolved.exists() && !resolved.is_file() => {
      Err(SandboxPathError::NotFile)
    }
    _ => Ok(resolved),
  }
}

fn valid_portable_segment(segment: &str) -> bool {
  if segment
    .chars()
    .any(|value| matches!(value, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
    || segment.ends_with([' ', '.'])
  {
    return false;
  }
  let stem = segment
    .split_once('.')
    .map_or(segment, |(stem, _)| stem)
    .to_ascii_uppercase();
  !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
    && !(stem.len() == 4
      && (stem.starts_with("COM") || stem.starts_with("LPT"))
      && stem.as_bytes()[3].is_ascii_digit()
      && stem.as_bytes()[3] != b'0')
}

fn path_to_virtual(path: &Path) -> String {
  let value = path
    .iter()
    .map(|segment| segment.to_string_lossy())
    .collect::<Vec<_>>()
    .join("/");
  if value.is_empty() {
    ".".to_string()
  } else {
    value
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalizes_current_directory_segments_and_rejects_parent_traversal() {
    assert_eq!(SafeRelativePath::parse(".").unwrap().virtual_path(), ".");
    assert_eq!(
      SafeRelativePath::parse("./images/./icon.png")
        .unwrap()
        .virtual_path(),
      "images/icon.png"
    );
    assert_eq!(
      SafeRelativePath::parse("images\\.\\icon.png")
        .unwrap()
        .virtual_path(),
      "images/icon.png"
    );
    assert_eq!(
      SafeRelativePath::parse("images/../secret.txt").unwrap_err(),
      SandboxPathError::ParentTraversal
    );
    assert_eq!(
      SafeRelativePath::parse("../secret.txt").unwrap_err(),
      SandboxPathError::ParentTraversal
    );
  }

  #[test]
  fn rejects_absolute_and_non_portable_paths() {
    for path in [
      "/etc/passwd",
      r"C:\Windows\system.ini",
      r"\\server\share\file",
      "file:stream",
      "CON",
      "file. ",
    ] {
      assert!(SafeRelativePath::parse(path).is_err(), "accepted {path:?}");
    }
  }
}
