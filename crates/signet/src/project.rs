//! Resolve project root, config, and secrets paths from `signet.toml`.

use std::path::{Path, PathBuf};

use crate::config::{resolve_config_path, resolve_targets, Config, ConfigError, Target};
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

    /// View of this project with `[project]` fields overridden by a `[[targets]]` entry.
    pub fn with_target(&self, target: &Target) -> Self {
        let mut config = self.config.clone();
        config.project.framework = target.framework.clone();
        config.project.app_root = target.app_root.clone();
        config.project.build_command = target.build_command.clone();
        // Single-target view — adapters read `[project]` only.
        config.targets.clear();
        Self {
            config_path: self.config_path.clone(),
            root: self.root.clone(),
            config,
        }
    }

    pub fn targets(&self) -> Vec<Target> {
        resolve_targets(&self.config)
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

    #[test]
    fn with_target_overrides_project_fields() {
        let mut cfg = Config::example("Mono", ".");
        cfg.project.framework = "tauri".into();
        let target = crate::config::Target {
            id: "desk".into(),
            framework: "electron".into(),
            app_root: "apps/e".into(),
            build_command: "npm run dist".into(),
        };
        let ctx = ProjectCtx {
            config_path: PathBuf::from("signet.toml"),
            root: PathBuf::from("."),
            config: cfg,
        };
        let view = ctx.with_target(&target);
        assert_eq!(view.config.project.framework, "electron");
        assert_eq!(view.config.project.app_root, "apps/e");
        assert!(view.config.targets.is_empty());
    }
}
