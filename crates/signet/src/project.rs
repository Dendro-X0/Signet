//! Resolve project root, config, and secrets paths from `signet.toml`.

use std::path::{Path, PathBuf};

use crate::config::{resolve_config_path, Config, ConfigError};
use crate::scan::{framework_id_for_kind, preferred_project, scan_repository};

#[derive(Debug, Clone)]
pub struct ProjectCtx {
    #[allow(dead_code)] // useful for diagnostics / future relative resolution
    pub config_path: PathBuf,
    /// Directory containing `signet.toml`
    pub root: PathBuf,
    pub config: Config,
}

/// Effective `[project].framework`: explicit config wins; omitted/blank → scan preference.
pub fn resolve_framework(root: &Path, config: &Config) -> String {
    let fw = config.project.framework.trim();
    if !fw.is_empty() {
        return fw.to_string();
    }
    match scan_repository(root) {
        Ok(report) => preferred_project(&report.root, &report.projects)
            .map(|p| framework_id_for_kind(p.kind).to_string())
            .unwrap_or_else(|| "tauri".into()),
        Err(_) => "tauri".into(),
    }
}

impl ProjectCtx {
    pub fn load(explicit_config: Option<&Path>) -> Result<Self, ConfigError> {
        let config_path = resolve_config_path(explicit_config);
        let config = Config::load(&config_path)?;
        let root = config_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        Ok(Self {
            config_path,
            root,
            config,
        })
    }

    pub fn framework(&self) -> String {
        resolve_framework(&self.root, &self.config)
    }

    pub fn secrets_dir(&self) -> PathBuf {
        self.config.secrets_path(&self.root)
    }

    pub fn identity_root(&self) -> PathBuf {
        self.secrets_dir().join("identity")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn explicit_framework_wins() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "electron".into();
        assert_eq!(resolve_framework(Path::new("."), &cfg), "electron");
    }

    #[test]
    fn omitted_framework_scans_this_repo_as_cli() {
        let mut cfg = Config::example("Signet", ".");
        cfg.project.framework.clear();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root");
        assert_eq!(resolve_framework(root, &cfg), "cli");
    }
}
