//! Non-secret project configuration (`signet.toml`).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CONFIG_FILE_NAME: &str = "signet.toml";
pub const SECRETS_DIR_NAME: &str = ".signet";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config not found at {0}")]
    NotFound(PathBuf),
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid config TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

/// Checked-in project config. Never store private keys here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub project: Project,
    pub platforms: Platforms,
    pub release: Release,
    /// Relative path to the secrets directory (gitignored). Default: `.signet`
    #[serde(default = "default_secrets_dir")]
    pub secrets_dir: String,
}

fn default_secrets_dir() -> String {
    SECRETS_DIR_NAME.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    /// Display / release name
    pub name: String,
    /// Path to the Tauri app root (directory containing `src-tauri`), relative to config
    pub tauri_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Platforms {
    #[serde(default = "default_true")]
    pub windows: bool,
    #[serde(default = "default_true")]
    pub macos: bool,
    #[serde(default = "default_true")]
    pub linux: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Release {
    /// Publish to GitHub Releases when `signet release` is used
    #[serde(default = "default_true")]
    pub github: bool,
    /// Optional `owner/repo`. If empty, detected from `git remote get-url origin`.
    #[serde(default)]
    pub repo: String,
    /// Attach TRUST.md to the GitHub Release when present
    #[serde(default = "default_true")]
    pub attach_trust: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project: Project {
                name: "my-app".into(),
                tauri_root: ".".into(),
            },
            platforms: Platforms {
                windows: true,
                macos: true,
                linux: true,
            },
            release: Release {
                github: true,
                repo: String::new(),
                attach_trust: true,
            },
            secrets_dir: default_secrets_dir(),
        }
    }
}

impl Config {
    pub fn example(name: &str, tauri_root: &str) -> Self {
        Self {
            project: Project {
                name: name.into(),
                tauri_root: tauri_root.into(),
            },
            ..Self::default()
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(ConfigError::NotFound(path.to_path_buf()));
        }
        let text = fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        let header = "# signet project config (safe to commit)\n\
                      # Private keys live under secrets_dir — see docs/secrets-layout.md\n\n";
        fs::write(path, format!("{header}{text}"))?;
        Ok(())
    }

    pub fn secrets_path(&self, project_root: impl AsRef<Path>) -> PathBuf {
        project_root.as_ref().join(&self.secrets_dir)
    }
}

/// Resolve config path: explicit override, else `signet.toml`, else legacy `selfsign.toml`.
pub fn resolve_config_path(explicit: Option<&Path>) -> PathBuf {
    if let Some(path) = explicit {
        return path.to_path_buf();
    }
    let modern = PathBuf::from(CONFIG_FILE_NAME);
    if modern.exists() {
        return modern;
    }
    let legacy = PathBuf::from("selfsign.toml");
    if legacy.exists() {
        return legacy;
    }
    modern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_default_toml() {
        let cfg = Config::default();
        let text = toml::to_string_pretty(&cfg).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(cfg, parsed);
    }
}
