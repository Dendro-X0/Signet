//! Tauri framework adapter — wraps existing discover + tauri CLI build.

use std::path::PathBuf;

use crate::project::ProjectCtx;
use crate::sign::{discover_artifacts, find_tauri_cli, resolve_src_tauri};

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::walk_outputs::run_required_build_command;
use super::Artifact;

#[derive(Debug, Default, Clone, Copy)]
pub struct TauriAdapter;

impl FrameworkAdapter for TauriAdapter {
    fn id(&self) -> &'static str {
        "tauri"
    }

    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf {
        resolve_src_tauri(&ctx.root, &ctx.config.project.app_root)
    }

    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()> {
        let cmd = ctx.config.project.build_command.trim();
        if !cmd.is_empty() {
            // Monorepo release scripts (e.g. `pnpm desktop:release`) run from signet.toml root.
            return run_required_build_command(
                &ctx.root,
                cmd,
                &opts.extra_args,
                "tauri",
                "pnpm desktop:release | npm run tauri build",
            );
        }

        let src_tauri = self.label_root(ctx);
        let Some(cli) = find_tauri_cli() else {
            anyhow::bail!(
                "tauri CLI not found — install with: cargo install tauri-cli --version \"^2\"\n\
                 or set [project].build_command (e.g. pnpm desktop:release), \
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
        let has_cmd = !ctx.config.project.build_command.trim().is_empty();
        if has_cmd {
            format!(
                "no artifacts found under {}/target/{profile}/bundle\n\
                 hint: ensure build_command succeeded (check frontendDist / web `out/`), \
                 or pass --artifact <path>",
                src.display()
            )
        } else {
            format!(
                "no artifacts found under {}/target/{profile}/bundle\n\
                 hint: run a successful `tauri build` (or set build_command for monorepos), \
                 or pass --artifact <path>",
                src.display()
            )
        }
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
app_root = "."
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

    #[test]
    fn empty_hint_mentions_build_command_when_set() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::write(
            root.join("signet.toml"),
            r#"
[project]
name = "demo"
app_root = "."
framework = "tauri"
build_command = "pnpm desktop:release"

[platforms]
windows = true
macos = false
linux = false

[release]
github = false
repo = ""
attach_trust = false

secrets_dir = ".signet"
"#,
        )
        .unwrap();
        let ctx = ProjectCtx::load(Some(&root.join("signet.toml"))).unwrap();
        let hint = TauriAdapter.empty_hint(&ctx, "release");
        assert!(hint.contains("build_command"), "{hint}");
    }
}
