//! React Native adapter (local android/ios outputs).

use std::path::PathBuf;

use crate::project::ProjectCtx;

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::walk_outputs::{discover_in_dirs, run_required_build_command};
use super::Artifact;

const OUTPUT_DIRS: &[&str] = &[
    "android/app/build/outputs",
    "ios/build",
    "dist",
    "release",
];

const BUILD_EXAMPLES: &str =
    "`npx react-native run-android --mode=release` (then locate APK), or your CI pack script";

#[derive(Debug, Default, Clone, Copy)]
pub struct ReactNativeAdapter;

impl ReactNativeAdapter {
    fn app_root(&self, ctx: &ProjectCtx) -> PathBuf {
        ctx.root.join(&ctx.config.project.tauri_root)
    }
}

impl FrameworkAdapter for ReactNativeAdapter {
    fn id(&self) -> &'static str {
        "react-native"
    }

    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf {
        self.app_root(ctx)
    }

    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()> {
        let app_root = self.app_root(ctx);
        if !app_root.join("package.json").is_file() {
            eprintln!(
                "warning: no package.json under {} — continuing anyway",
                app_root.display()
            );
        }
        run_required_build_command(
            &app_root,
            &ctx.config.project.build_command,
            &opts.extra_args,
            "react-native",
            BUILD_EXAMPLES,
        )
    }

    fn discover(&self, ctx: &ProjectCtx, _profile: &str) -> anyhow::Result<Vec<Artifact>> {
        discover_in_dirs(&self.app_root(ctx), OUTPUT_DIRS)
    }

    fn empty_hint(&self, ctx: &ProjectCtx, _profile: &str) -> String {
        format!(
            "no React Native artifacts under {}/{{android/app/build/outputs,ios/build,dist}}\n\
             hint: set build_command or --skip-build; APK: signet android sign; \
             iOS: signet ios package after Xcode archive",
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
        let found = discover_in_dirs(dir.path(), OUTPUT_DIRS).unwrap();
        assert_eq!(found.len(), 1);
    }
}
