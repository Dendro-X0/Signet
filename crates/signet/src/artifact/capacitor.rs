//! Capacitor adapter (android/ios wrapper outputs).

use std::path::PathBuf;

use crate::project::ProjectCtx;

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::walk_outputs::{discover_in_dirs, run_required_build_command};
use super::Artifact;

const OUTPUT_DIRS: &[&str] = &[
    "android/app/build/outputs",
    "ios/App/build",
    "ios/build",
    "dist",
    "release",
];

const BUILD_EXAMPLES: &str =
    "`npx cap sync` then Gradle/Xcode pack, or your npm script that produces APK/IPA/desktop installers";

#[derive(Debug, Default, Clone, Copy)]
pub struct CapacitorAdapter;

impl CapacitorAdapter {
    fn app_root(&self, ctx: &ProjectCtx) -> PathBuf {
        ctx.root.join(&ctx.config.project.tauri_root)
    }
}

impl FrameworkAdapter for CapacitorAdapter {
    fn id(&self) -> &'static str {
        "capacitor"
    }

    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf {
        self.app_root(ctx)
    }

    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()> {
        let app_root = self.app_root(ctx);
        run_required_build_command(
            &app_root,
            &ctx.config.project.build_command,
            &opts.extra_args,
            "capacitor",
            BUILD_EXAMPLES,
        )
    }

    fn discover(&self, ctx: &ProjectCtx, _profile: &str) -> anyhow::Result<Vec<Artifact>> {
        discover_in_dirs(&self.app_root(ctx), OUTPUT_DIRS)
    }

    fn empty_hint(&self, ctx: &ProjectCtx, _profile: &str) -> String {
        format!(
            "no Capacitor artifacts under {}/{{android/…,ios/…,dist}}\n\
             hint: set build_command or --skip-build; APK: signet android sign; \
             iOS: signet ios package",
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
    fn discovers_apk() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("android/app/build/outputs/apk/release");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("app-release.apk"), b"APK").unwrap();
        assert_eq!(discover_in_dirs(dir.path(), OUTPUT_DIRS).unwrap().len(), 1);
    }
}
