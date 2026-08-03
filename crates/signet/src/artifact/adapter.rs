//! Framework adapter boundary.

use std::path::PathBuf;

use crate::config::Config;
use crate::project::ProjectCtx;

use super::android::AndroidAdapter;
use super::capacitor::CapacitorAdapter;
use super::cli::CliAdapter;
use super::electron::ElectronAdapter;
use super::expo::ExpoAdapter;
use super::flutter::FlutterAdapter;
use super::ios::IosAdapter;
use super::react_native::ReactNativeAdapter;
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

/// Frameworks that refuse to invent a default build (empty `build_command` is unpaid debt).
pub fn requires_explicit_build_command(framework: &str) -> bool {
    matches!(
        framework.trim().to_ascii_lowercase().as_str(),
        "flutter" | "react-native" | "rn" | "expo" | "capacitor" | "ios"
    )
}

/// Select adapter from effective framework (explicit config, else scan, else tauri).
pub fn select_adapter(
    root: &std::path::Path,
    config: &Config,
) -> anyhow::Result<Box<dyn FrameworkAdapter>> {
    let resolved = crate::project::resolve_framework(root, config);
    let fw = resolved.as_str();
    match fw {
        "tauri" => Ok(Box::new(TauriAdapter)),
        "electron" => Ok(Box::new(ElectronAdapter)),
        "android" => Ok(Box::new(AndroidAdapter)),
        "ios" => Ok(Box::new(IosAdapter)),
        "flutter" => Ok(Box::new(FlutterAdapter)),
        "react-native" | "rn" => Ok(Box::new(ReactNativeAdapter)),
        "expo" => Ok(Box::new(ExpoAdapter)),
        "capacitor" => Ok(Box::new(CapacitorAdapter)),
        "cli" | "rust" | "rust-cli" => Ok(Box::new(CliAdapter)),
        other => anyhow::bail!(
            "framework '{other}' is not supported yet — supported: tauri, electron, android, ios, \
             flutter, react-native (rn), expo, capacitor, cli (see docs/frameworks.md)"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::Path;

    fn root() -> &'static Path {
        Path::new(".")
    }

    #[test]
    fn default_selects_tauri() {
        let cfg = Config::example("app", ".");
        let adapter = select_adapter(root(), &cfg).unwrap();
        assert_eq!(adapter.id(), "tauri");
    }

    #[test]
    fn selects_electron() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "electron".into();
        let adapter = select_adapter(root(), &cfg).unwrap();
        assert_eq!(adapter.id(), "electron");
    }

    #[test]
    fn selects_android() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "android".into();
        let adapter = select_adapter(root(), &cfg).unwrap();
        assert_eq!(adapter.id(), "android");
    }

    #[test]
    fn selects_ios() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "ios".into();
        let adapter = select_adapter(root(), &cfg).unwrap();
        assert_eq!(adapter.id(), "ios");
    }

    #[test]
    fn selects_flutter() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "flutter".into();
        assert_eq!(select_adapter(root(), &cfg).unwrap().id(), "flutter");
    }

    #[test]
    fn selects_rn_alias() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "rn".into();
        assert_eq!(select_adapter(root(), &cfg).unwrap().id(), "react-native");
    }

    #[test]
    fn selects_expo_and_capacitor() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "expo".into();
        assert_eq!(select_adapter(root(), &cfg).unwrap().id(), "expo");
        cfg.project.framework = "capacitor".into();
        assert_eq!(select_adapter(root(), &cfg).unwrap().id(), "capacitor");
    }

    #[test]
    fn selects_cli() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "cli".into();
        assert_eq!(select_adapter(root(), &cfg).unwrap().id(), "cli");
        cfg.project.framework = "rust-cli".into();
        assert_eq!(select_adapter(root(), &cfg).unwrap().id(), "cli");
    }

    #[test]
    fn unknown_framework_errors() {
        let mut cfg = Config::example("app", ".");
        cfg.project.framework = "dotnet".into();
        let err = match select_adapter(root(), &cfg) {
            Ok(_) => panic!("expected error"),
            Err(e) => e.to_string(),
        };
        assert!(err.contains("not supported"));
    }
}
