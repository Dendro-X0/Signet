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
    /// Optional trust-tier declaration and notes for TRUST.md / doctor.
    #[serde(default)]
    pub trust: Trust,
    /// Optional OV / Azure / Apple notarization helper settings (no secrets).
    #[serde(default)]
    pub graduation: Graduation,
    /// Optional monorepo ship targets. Empty → synthesize one from `[project]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<Target>,
}

/// One shippable app/framework under a repo-level `signet.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Target {
    pub id: String,
    pub framework: String,
    #[serde(alias = "tauri_root")]
    pub app_root: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub build_command: String,
}

/// Targets to build: explicit `[[targets]]`, or a synthetic `default` from `[project]`.
pub fn resolve_targets(config: &Config) -> Vec<Target> {
    if !config.targets.is_empty() {
        return config.targets.clone();
    }
    vec![Target {
        id: "default".into(),
        framework: config.project.framework.clone(),
        app_root: config.project.app_root.clone(),
        build_command: config.project.build_command.clone(),
    }]
}

/// Filter by `--target id`. `None` → all targets.
pub fn select_targets<'a>(
    all: &'a [Target],
    filter: Option<&str>,
) -> anyhow::Result<Vec<&'a Target>> {
    match filter.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(all.iter().collect()),
        Some(id) => {
            let found: Vec<_> = all.iter().filter(|t| t.id == id).collect();
            if found.is_empty() {
                let known = all
                    .iter()
                    .map(|t| t.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                anyhow::bail!("unknown target `{id}` — known: {known}");
            }
            Ok(found)
        }
    }
}

/// Reputation graduation helpers — paths and public ids only (see docs/graduation.md).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Graduation {
    /// Windows OV cert SHA-1 thumbprint (hex). Prefer env override for CI.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub ov_thumbprint: String,
    /// Default Authenticode timestamp URL for `graduate ov-sign`.
    #[serde(default = "default_ov_timestamp")]
    pub timestamp_url: String,
    #[serde(default)]
    pub azure: GraduationAzure,
    #[serde(default)]
    pub apple: GraduationApple,
}

impl Default for Graduation {
    fn default() -> Self {
        Self {
            ov_thumbprint: String::new(),
            timestamp_url: default_ov_timestamp(),
            azure: GraduationAzure::default(),
            apple: GraduationApple::default(),
        }
    }
}

fn default_ov_timestamp() -> String {
    "http://timestamp.digicert.com".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraduationAzure {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub dlib: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub metadata: String,
    #[serde(default = "default_azure_timestamp")]
    pub timestamp_url: String,
}

fn default_azure_timestamp() -> String {
    "http://timestamp.acs.microsoft.com".into()
}

impl Default for GraduationAzure {
    fn default() -> Self {
        Self {
            dlib: String::new(),
            metadata: String::new(),
            timestamp_url: default_azure_timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GraduationApple {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub keychain_profile: String,
}

fn default_secrets_dir() -> String {
    SECRETS_DIR_NAME.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    /// Display / release name
    pub name: String,
    /// App / package root relative to config (legacy TOML key: `tauri_root`).
    #[serde(alias = "tauri_root")]
    pub app_root: String,
    /// Framework adapter id (`tauri` / `electron` / `cli` / …).
    /// Empty when omitted from TOML — resolved via scan at use time (see `resolve_framework`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub framework: String,
    /// Optional build argv override (Electron: default `npm run dist` when empty).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub build_command: String,
}

fn default_framework() -> String {
    "tauri".into()
}

/// True when the config file (or in-memory value) named a framework explicitly.
pub fn framework_is_explicit(config: &Config) -> bool {
    !config.project.framework.trim().is_empty()
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

/// Declared trust intent. Does not change host PE/codesign behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Trust {
    /// Optional tier id (see docs/trust-model.md). When set, overrides inference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_tier: Option<String>,
    /// Extra notes appended to the Trust tier section of TRUST.md.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Community checksum signing (minisign / optional GPG).
    #[serde(default)]
    pub checksum_signing: ChecksumSigning,
}

/// Phase 8 — sign `SHA256SUMS` for community verify.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChecksumSigning {
    /// Sign with Signet-managed minisign key under `.signet/sums/` (default true).
    #[serde(default = "default_true")]
    pub minisign: bool,
    /// Opt-in GPG detach-sign → `SHA256SUMS.asc`.
    #[serde(default)]
    pub gpg: bool,
    /// Optional GPG key id / fingerprint; empty uses gpg default key.
    #[serde(default)]
    pub gpg_key_id: String,
}

impl Default for ChecksumSigning {
    fn default() -> Self {
        Self {
            minisign: true,
            gpg: false,
            gpg_key_id: String::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project: Project {
                name: "my-app".into(),
                app_root: ".".into(),
                framework: default_framework(),
                build_command: String::new(),
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
            trust: Trust::default(),
            graduation: Graduation::default(),
            targets: Vec::new(),
        }
    }
}

impl Config {
    pub fn example(name: &str, app_root: &str) -> Self {
        Self {
            project: Project {
                name: name.into(),
                app_root: app_root.into(),
                framework: default_framework(),
                build_command: String::new(),
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

    #[test]
    fn omitted_framework_deserializes_empty() {
        let mut base = Config::example("App", ".");
        base.project.framework.clear();
        let text = toml::to_string_pretty(&base).unwrap();
        assert!(
            !text.contains("framework"),
            "empty framework should be omitted from TOML; got:\n{text}"
        );
        let cfg: Config = toml::from_str(&text).unwrap();
        assert!(cfg.project.framework.is_empty());
        assert!(!framework_is_explicit(&cfg));
    }

    #[test]
    fn legacy_tauri_root_alias_loads_as_app_root() {
        let text = r#"
[project]
name = "App"
tauri_root = "apps/desk"
framework = "tauri"
[platforms]
windows = true
[release]
github = false
"#;
        let cfg: Config = toml::from_str(text).unwrap();
        assert_eq!(cfg.project.app_root, "apps/desk");
        let out = toml::to_string_pretty(&cfg).unwrap();
        assert!(out.contains("app_root"), "serialize prefers app_root:\n{out}");
        assert!(!out.contains("tauri_root"), "should not emit legacy key:\n{out}");
    }

    #[test]
    fn resolve_targets_synthesizes_default() {
        let cfg = Config::example("App", "apps/x");
        let t = resolve_targets(&cfg);
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].id, "default");
        assert_eq!(t[0].app_root, "apps/x");
    }

    #[test]
    fn resolve_targets_uses_explicit_list() {
        let text = r#"
[project]
name = "Mono"
app_root = "."
framework = "tauri"
[platforms]
windows = true
[release]
github = false
[[targets]]
id = "desktop"
framework = "tauri"
app_root = "apps/desk"
build_command = "pnpm desktop:release"
[[targets]]
id = "cli"
framework = "cli"
app_root = "crates/tool"
"#;
        let cfg: Config = toml::from_str(text).unwrap();
        let t = resolve_targets(&cfg);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].id, "desktop");
        assert_eq!(t[1].framework, "cli");
        let one = select_targets(&t, Some("cli")).unwrap();
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].id, "cli");
        assert!(select_targets(&t, Some("missing")).is_err());
    }
}
