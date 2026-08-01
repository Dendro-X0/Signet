//! Framework adapter boundary.

use std::path::PathBuf;

use crate::config::Config;
use crate::project::ProjectCtx;

use super::tauri::TauriAdapter;
use super::Artifact;

#[derive(Debug, Clone)]
pub struct BuildOpts {
    pub profile: String,
    pub extra_args: Vec<String>,
}

impl Default for BuildOpts {
    fn default() -> Self {
        Self {
            profile: "release".into(),
            extra_args: Vec::new(),
        }
    }
}

pub trait FrameworkAdapter {
    fn id(&self) -> &'static str;

    /// Path shown in CLI (“tauri crate: …”).
    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf;

    /// Run the framework’s production build. Callers skip this when `--skip-build`.
    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()>;

    fn discover(&self, ctx: &ProjectCtx, profile: &str) -> anyhow::Result<Vec<Artifact>>;

    /// Hint when discover returns empty.
    fn empty_hint(&self, ctx: &ProjectCtx, profile: &str) -> String;
}

/// Select adapter from `[project].framework` (default tauri).
pub fn select_adapter(config: &Config) -> anyhow::Result<Box<dyn FrameworkAdapter>> {
    let fw = config.project.framework.trim();
    let fw = if fw.is_empty() { "tauri" } else { fw };
    match fw {
        "tauri" => Ok(Box::new(TauriAdapter)),
        other => anyhow::bail!(
            "framework '{other}' is not supported yet — Signet Phase 9 ships the Tauri adapter; \
             Electron lands in Phase 10 (see docs/roadmap.md)"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn default_selects_tauri() {
        let cfg = Config::example("app", ".");
        let adapter = select_adapter(&cfg).unwrap();
        assert_eq!(adapter.id(), "tauri");
    }

    #[test]
    fn unknown_framework_errors() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "electron".into();
        let err = match select_adapter(&cfg) {
            Ok(_) => panic!("expected error"),
            Err(e) => e.to_string(),
        };
        assert!(err.contains("Phase 10"));
    }
}
