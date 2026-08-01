//! Tauri framework adapter — wraps existing discover + tauri CLI build.

use std::path::PathBuf;

use crate::project::ProjectCtx;
use crate::sign::{discover_artifacts, find_tauri_cli, resolve_src_tauri};

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::Artifact;

#[derive(Debug, Default, Clone, Copy)]
pub struct TauriAdapter;

impl FrameworkAdapter for TauriAdapter {
    fn id(&self) -> &'static str {
        "tauri"
    }

    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf {
        resolve_src_tauri(&ctx.root, &ctx.config.project.tauri_root)
    }

    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()> {
        let src_tauri = self.label_root(ctx);
        let Some(cli) = find_tauri_cli() else {
            anyhow::bail!(
                "tauri CLI not found — install with: cargo install tauri-cli --version \"^2\"\n\
                 or pass --skip-build to sign existing artifacts only"
            );
        };
        println!("running tauri build ({})…", opts.profile);
        cli.run_build(&src_tauri, &opts.extra_args)?;
        Ok(())
    }

    fn discover(&self, ctx: &ProjectCtx, profile: &str) -> anyhow::Result<Vec<Artifact>> {
        let src_tauri = self.label_root(ctx);
        discover_artifacts(&src_tauri, profile)
    }

    fn empty_hint(&self, ctx: &ProjectCtx, profile: &str) -> String {
        let src = self.label_root(ctx);
        format!(
            "no artifacts found under {}/target/{profile}/bundle\n\
             hint: run a successful `tauri build` first, or pass --artifact <path>",
            src.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn tauri_adapter_discovers_fixture() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let bundle = root.join("src-tauri/target/release/bundle/nsis");
        fs::create_dir_all(&bundle).unwrap();
        fs::write(bundle.join("App_0.1.0_x64-setup.exe"), b"MZ").unwrap();
        fs::write(
            root.join("signet.toml"),
            r#"
[project]
name = "demo"
tauri_root = "."
framework = "tauri"

[platforms]
windows = true
macos = true
linux = true

[release]
github = true
repo = ""
attach_trust = true

secrets_dir = ".signet"
"#,
        )
        .unwrap();

        let ctx = ProjectCtx::load(Some(&root.join("signet.toml"))).unwrap();
        let arts = TauriAdapter.discover(&ctx, "release").unwrap();
        assert_eq!(arts.len(), 1);
        assert_eq!(arts[0].kind.as_str(), "windows-exe");
        assert_eq!(arts[0].name_for_sums, "App_0.1.0_x64-setup.exe");
    }
}
