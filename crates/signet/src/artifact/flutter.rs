//! Flutter desktop / mobile output adapter.

use std::path::PathBuf;

use crate::project::ProjectCtx;

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::walk_outputs::{discover_in_dirs, run_required_build_command};
use super::Artifact;

const OUTPUT_DIRS: &[&str] = &[
    "build/windows",
    "build/macos",
    "build/linux",
    "build/app/outputs",
    "build/ios",
    "build/ios/iphoneos",
    "dist",
    "release",
];

const BUILD_EXAMPLES: &str =
    "`flutter build apk`, `flutter build windows`, `flutter build macos`, `flutter build linux`";

#[derive(Debug, Default, Clone, Copy)]
pub struct FlutterAdapter;

impl FlutterAdapter {
    fn app_root(&self, ctx: &ProjectCtx) -> PathBuf {
        ctx.root.join(&ctx.config.project.app_root)
    }
}

impl FrameworkAdapter for FlutterAdapter {
    fn id(&self) -> &'static str {
        "flutter"
    }

    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf {
        self.app_root(ctx)
    }

    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()> {
        let app_root = self.app_root(ctx);
        if !app_root.join("pubspec.yaml").is_file() {
            eprintln!(
                "warning: no pubspec.yaml under {} — continuing anyway",
                app_root.display()
            );
        }
        run_required_build_command(
            &app_root,
            &ctx.config.project.build_command,
            &opts.extra_args,
            "flutter",
            BUILD_EXAMPLES,
        )
    }

    fn discover(&self, ctx: &ProjectCtx, _profile: &str) -> anyhow::Result<Vec<Artifact>> {
        discover_in_dirs(&self.app_root(ctx), OUTPUT_DIRS)
    }

    fn empty_hint(&self, ctx: &ProjectCtx, _profile: &str) -> String {
        format!(
            "no Flutter artifacts under {}/{{build/…,dist,release}}\n\
             hint: set build_command (e.g. flutter build apk) or --skip-build after a local build; \
             APK signing: signet android sign",
            self.app_root(ctx).display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn discovers_windows_exe() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("build/windows/x64/runner/Release");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("app.exe"), b"MZ").unwrap();
        let found = discover_in_dirs(dir.path(), OUTPUT_DIRS).unwrap();
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn flutter_adapter_via_config() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let out = root.join("build/app/outputs/flutter-apk");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("app-release.apk"), b"APK").unwrap();
        fs::write(root.join("pubspec.yaml"), "name: demo\nflutter:\n  assets: []\n").unwrap();
        write_signet_toml(root, "flutter");
        let ctx = ProjectCtx::load(Some(&root.join("signet.toml"))).unwrap();
        let arts = FlutterAdapter.discover(&ctx, "release").unwrap();
        assert_eq!(arts.len(), 1);
    }

    fn write_signet_toml(root: &std::path::Path, framework: &str) {
        fs::write(
            root.join("signet.toml"),
            format!(
                r#"
[project]
name = "demo"
app_root = "."
framework = "{framework}"
build_command = "flutter build apk"

[platforms]
windows = true
macos = true
linux = true

[release]
github = true
repo = ""
attach_trust = true

secrets_dir = ".signet"
"#
            ),
        )
        .unwrap();
    }
}
