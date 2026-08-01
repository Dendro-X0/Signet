//! iOS adapter — discover .app / .ipa; build only via explicit build_command.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::project::ProjectCtx;

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::{Artifact, ArtifactKind};

const MAX_DEPTH: u32 = 8;

#[derive(Debug, Default, Clone, Copy)]
pub struct IosAdapter;

impl IosAdapter {
    fn app_root(&self, ctx: &ProjectCtx) -> PathBuf {
        ctx.root.join(&ctx.config.project.tauri_root)
    }
}

impl FrameworkAdapter for IosAdapter {
    fn id(&self) -> &'static str {
        "ios"
    }

    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf {
        self.app_root(ctx)
    }

    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()> {
        let cmd = ctx.config.project.build_command.trim();
        if cmd.is_empty() {
            anyhow::bail!(
                "iOS build requires [project].build_command (e.g. xcodebuild …) or pass --skip-build\n\
                 Signet will not guess an Xcode scheme. See docs/ios.md"
            );
        }
        let app_root = self.app_root(ctx);
        let parts = split_argv(cmd);
        if parts.is_empty() {
            anyhow::bail!("empty build_command");
        }
        let program = &parts[0];
        let mut args: Vec<String> = parts[1..].to_vec();
        args.extend(opts.extra_args.iter().cloned());
        println!(
            "running ios build: {} {} (cwd {})",
            program,
            args.join(" "),
            app_root.display()
        );
        let status = Command::new(program)
            .current_dir(&app_root)
            .args(&args)
            .status()?;
        if !status.success() {
            anyhow::bail!("ios build failed with status {status}");
        }
        Ok(())
    }

    fn discover(&self, ctx: &ProjectCtx, _profile: &str) -> anyhow::Result<Vec<Artifact>> {
        discover_ios_artifacts(&self.app_root(ctx))
    }

    fn empty_hint(&self, ctx: &ProjectCtx, _profile: &str) -> String {
        format!(
            "no .app / .ipa found under {}\n\
             hint: build with Xcode / set build_command, use `signet ios package --app …`, \
             or pass --artifact",
            self.app_root(ctx).display()
        )
    }
}

fn split_argv(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for ch in s.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn discover_ios_artifacts(app_root: &Path) -> anyhow::Result<Vec<Artifact>> {
    let mut out = Vec::new();
    for name in [
        "build",
        "dist",
        "release",
        "src-tauri/gen/apple",
        "src-tauri/gen/ios",
        "gen/apple",
        "gen/ios",
    ] {
        let dir = app_root.join(name);
        if dir.is_dir() {
            visit_dir(&dir, 0, &mut out)?;
        }
    }
    // Also scan app root shallow for loose .ipa
    if app_root.is_dir() {
        for entry in fs::read_dir(app_root)?.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ArtifactKind::Ipa) = ArtifactKind::classify_file(&path) {
                    out.push(Artifact::new(path, ArtifactKind::Ipa));
                }
            } else if path.extension().and_then(|e| e.to_str()) == Some("app") {
                out.push(Artifact::new(path, ArtifactKind::MacApp));
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.dedup_by(|a, b| a.path == b.path);
    Ok(out)
}

fn visit_dir(dir: &Path, depth: u32, out: &mut Vec<Artifact>) -> anyhow::Result<()> {
    if depth > MAX_DEPTH {
        return Ok(());
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if matches!(name.as_str(), "node_modules" | ".git" | ".signet" | "pods") {
            continue;
        }
        if path.is_dir() {
            if path.extension().and_then(|e| e.to_str()) == Some("app") {
                // iOS/macOS app bundle — classify as MacApp kind (bundle); IPA packaging is separate.
                out.push(Artifact::new(path, ArtifactKind::MacApp));
            } else {
                visit_dir(&path, depth + 1, out)?;
            }
        } else if let Some(kind) = ArtifactKind::classify_file(&path) {
            if kind == ArtifactKind::Ipa {
                out.push(Artifact::new(path, kind));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn discovers_ipa_and_app() {
        let dir = tempdir().unwrap();
        let build = dir.path().join("build/Release-iphoneos");
        fs::create_dir_all(build.join("Demo.app")).unwrap();
        fs::create_dir_all(dir.path().join("dist")).unwrap();
        fs::write(dir.path().join("dist/Demo.ipa"), b"PK").unwrap();
        let found = discover_ios_artifacts(dir.path()).unwrap();
        assert!(found.iter().any(|a| a.kind == ArtifactKind::MacApp));
        assert!(found.iter().any(|a| a.kind == ArtifactKind::Ipa));
    }
}
