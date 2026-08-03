//! Expo adapter (local / EAS-exported outputs).

use std::path::PathBuf;

use crate::project::ProjectCtx;

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::walk_outputs::{discover_in_dirs, run_required_build_command};
use super::Artifact;

const OUTPUT_DIRS: &[&str] = &[
    "dist",
    "android/app/build/outputs",
    "ios/build",
    "release",
];

const BUILD_EXAMPLES: &str =
    "`npx eas-cli build --local` or platform pack after `npx expo export`; prefer --skip-build for CI artifacts";

#[derive(Debug, Default, Clone, Copy)]
pub struct ExpoAdapter;

impl ExpoAdapter {
    fn app_root(&self, ctx: &ProjectCtx) -> PathBuf {
        ctx.root.join(&ctx.config.project.app_root)
    }
}

impl FrameworkAdapter for ExpoAdapter {
    fn id(&self) -> &'static str {
        "expo"
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
            "expo",
            BUILD_EXAMPLES,
        )
    }

    fn discover(&self, ctx: &ProjectCtx, _profile: &str) -> anyhow::Result<Vec<Artifact>> {
        discover_in_dirs(&self.app_root(ctx), OUTPUT_DIRS)
    }

    fn empty_hint(&self, ctx: &ProjectCtx, _profile: &str) -> String {
        format!(
            "no Expo artifacts under {}/{{dist,android/…,ios/build}}\n\
             hint: EAS cloud builds are external — download APK/IPA then --skip-build / \
             signet android sign / signet ios package",
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
    fn discovers_ipa_under_dist() {
        let dir = tempdir().unwrap();
        let dist = dir.path().join("dist");
        fs::create_dir_all(&dist).unwrap();
        fs::write(dist.join("app.ipa"), b"IPA").unwrap();
        let found = discover_in_dirs(dir.path(), OUTPUT_DIRS).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, super::super::ArtifactKind::Ipa);
    }
}
