use std::path::Path;

use crate::sign::{discover_artifacts, resolve_src_tauri};
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct ProjectStatus {
    pub has_config: bool,
    pub has_identity: bool,
    pub has_trust: bool,
    pub has_artifacts: bool,
    pub app_name: Option<String>,
}

impl ProjectStatus {
    pub fn probe(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref();
        let config_path = root.join("selfsign.toml");
        let has_config = config_path.is_file();
        let mut app_name = None;
        let mut has_artifacts = false;

        if has_config {
            if let Ok(cfg) = Config::load(&config_path) {
                app_name = Some(cfg.project.name.clone());
                let src = resolve_src_tauri(root, &cfg.project.tauri_root);
                if let Ok(arts) = discover_artifacts(&src, "release") {
                    has_artifacts = arts.iter().any(|a| a.path.is_file());
                }
            }
        }

        let has_identity = root.join(".selfsign/identity/active").is_file();
        let has_trust = root.join("TRUST.md").is_file();

        Self {
            has_config,
            has_identity,
            has_trust,
            has_artifacts,
            app_name,
        }
    }

    pub fn recommended_action(&self) -> &'static str {
        if !self.has_config {
            "scan"
        } else if !self.has_identity {
            "identity"
        } else if !self.has_trust {
            "trust"
        } else if !self.has_artifacts {
            "build"
        } else {
            "release"
        }
    }

    pub fn next_hint(&self) -> String {
        match self.recommended_action() {
            "scan" => "run Scan to find installers & suggest config".into(),
            "identity" => "create a signing identity".into(),
            "trust" => "emit TRUST.md".into(),
            "build" => "build & sign artifacts".into(),
            "release" => {
                let name = self.app_name.as_deref().unwrap_or("app");
                format!("publish {name} (try dry-run first)")
            }
            _ => "choose an action".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn recommends_guided_without_config() {
        let dir = tempdir().unwrap();
        let status = ProjectStatus::probe(dir.path());
        assert!(!status.has_config);
        assert_eq!(status.recommended_action(), "scan");
    }

    #[test]
    fn recommends_identity_after_init() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("selfsign.toml"),
            r#"
[project]
name = "x"
tauri_root = "."
[platforms]
windows = true
macos = true
linux = true
[release]
github = true
"#,
        )
        .unwrap();
        let status = ProjectStatus::probe(dir.path());
        assert!(status.has_config);
        assert_eq!(status.recommended_action(), "identity");
    }
}
