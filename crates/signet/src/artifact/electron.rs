//! Electron Builder / Forge adapter (Phase 10).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use which::which;

use crate::project::ProjectCtx;

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::{Artifact, ArtifactKind};

const OUTPUT_DIRS: &[&str] = &["dist", "out", "release"];
const MAX_DEPTH: u32 = 8;

#[derive(Debug, Default, Clone, Copy)]
pub struct ElectronAdapter;

impl ElectronAdapter {
    pub fn app_root(&self, ctx: &ProjectCtx) -> PathBuf {
        ctx.root.join(&ctx.config.project.tauri_root)
    }
}

impl FrameworkAdapter for ElectronAdapter {
    fn id(&self) -> &'static str {
        "electron"
    }

    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf {
        self.app_root(ctx)
    }

    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()> {
        let app_root = self.app_root(ctx);
        if !app_root.join("package.json").is_file() {
            eprintln!(
                "warning: no package.json under {} — continuing with build_command anyway",
                app_root.display()
            );
        }

        let (program, mut args) = resolve_build_argv(&ctx.config.project.build_command)?;
        args.extend(opts.extra_args.iter().cloned());

        println!(
            "running electron build: {} {} (cwd {})",
            program,
            args.join(" "),
            app_root.display()
        );

        let status = Command::new(&program)
            .current_dir(&app_root)
            .args(&args)
            .status()?;
        if !status.success() {
            anyhow::bail!("electron build failed with status {status}");
        }
        Ok(())
    }

    fn discover(&self, ctx: &ProjectCtx, _profile: &str) -> anyhow::Result<Vec<Artifact>> {
        let app_root = self.app_root(ctx);
        discover_electron_artifacts(&app_root)
    }

    fn empty_hint(&self, ctx: &ProjectCtx, _profile: &str) -> String {
        let root = self.app_root(ctx);
        format!(
            "no Electron installers found under {}/{{dist,out,release}}\n\
             hint: run your packager (`npm run dist` / electron-builder / Forge), \
             or pass --artifact <path>, or --skip-build after a local build",
            root.display()
        )
    }
}

/// Parse `build_command` or default to `npm run dist`.
fn resolve_build_argv(build_command: &str) -> anyhow::Result<(String, Vec<String>)> {
    let trimmed = build_command.trim();
    if trimmed.is_empty() {
        ensure_npm()?;
        return Ok(("npm".into(), vec!["run".into(), "dist".into()]));
    }
    let parts: Vec<String> = split_argv(trimmed);
    if parts.is_empty() {
        anyhow::bail!("[project].build_command is empty after parse");
    }
    let program = parts[0].clone();
    if which(&program).is_err() && program != "npm" && program != "npx" {
        // Still try — PATH may differ at runtime; warn only for missing npm/npx defaults.
    }
    if program == "npm" || program == "npx" {
        ensure_npm_or_npx(&program)?;
    }
    Ok((program, parts[1..].to_vec()))
}

fn ensure_npm() -> anyhow::Result<()> {
    if which("npm").is_ok() {
        return Ok(());
    }
    anyhow::bail!(
        "npm not found on PATH — install Node.js/npm, set [project].build_command, \
         or pass --skip-build to sign existing artifacts only"
    );
}

fn ensure_npm_or_npx(program: &str) -> anyhow::Result<()> {
    if which(program).is_ok() {
        return Ok(());
    }
    anyhow::bail!(
        "{program} not found on PATH — install Node.js, or pass --skip-build"
    );
}

fn split_argv(s: &str) -> Vec<String> {
    // Minimal split: whitespace; quoted segments supported for common cases.
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

pub fn discover_electron_artifacts(app_root: &Path) -> anyhow::Result<Vec<Artifact>> {
    let mut out = Vec::new();
    for name in OUTPUT_DIRS {
        let dir = app_root.join(name);
        if dir.is_dir() {
            visit_dir(&dir, 0, &mut out)?;
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
        if name == "node_modules" || name == ".git" || name == ".signet" || name == ".selfsign" {
            continue;
        }
        if path.is_dir() {
            if path.extension().and_then(|e| e.to_str()) == Some("app") {
                out.push(Artifact::new(path, ArtifactKind::MacApp));
            } else {
                visit_dir(&path, depth + 1, out)?;
            }
        } else if let Some(kind) = ArtifactKind::classify_file(&path) {
            // Prefer installers over loose zip of unpacked dirs when both exist — keep all classified.
            out.push(Artifact::new(path, kind));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use which::which;

    #[test]
    fn discovers_exe_under_dist() {
        let dir = tempdir().unwrap();
        let dist = dir.path().join("dist");
        fs::create_dir_all(&dist).unwrap();
        fs::write(dist.join("App Setup 1.0.0.exe"), b"MZ").unwrap();
        let found = discover_electron_artifacts(dir.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, ArtifactKind::WindowsExe);
    }

    #[test]
    fn discovers_appimage_under_out() {
        let dir = tempdir().unwrap();
        let out_dir = dir.path().join("out/make");
        fs::create_dir_all(&out_dir).unwrap();
        fs::write(out_dir.join("App-1.0.0.AppImage"), b"AI").unwrap();
        let found = discover_electron_artifacts(dir.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, ArtifactKind::LinuxAppImage);
    }

    #[test]
    fn skips_node_modules() {
        let dir = tempdir().unwrap();
        let nm = dir.path().join("dist/node_modules/pkg");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("tool.exe"), b"MZ").unwrap();
        fs::create_dir_all(dir.path().join("dist")).unwrap();
        fs::write(dir.path().join("dist/real.exe"), b"MZ").unwrap();
        let found = discover_electron_artifacts(dir.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].path.ends_with("real.exe"));
    }

    #[test]
    fn default_build_argv_when_npm_present() {
        if which("npm").is_err() {
            return;
        }
        let (prog, args) = resolve_build_argv("").unwrap();
        assert_eq!(prog, "npm");
        assert_eq!(args, vec!["run", "dist"]);
    }

    #[test]
    fn parses_custom_build_command() {
        let parts = split_argv(r#"npx electron-builder --publish never"#);
        assert_eq!(parts[0], "npx");
        assert_eq!(parts[1], "electron-builder");
        assert!(parts.iter().any(|p| p == "--publish"));
    }

    #[test]
    fn electron_adapter_via_config() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("dist")).unwrap();
        fs::write(root.join("dist/setup.exe"), b"MZ").unwrap();
        fs::write(root.join("package.json"), r#"{"name":"demo"}"#).unwrap();
        fs::write(
            root.join("signet.toml"),
            r#"
[project]
name = "demo"
tauri_root = "."
framework = "electron"

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
        let arts = ElectronAdapter.discover(&ctx, "release").unwrap();
        assert_eq!(arts.len(), 1);
    }

    #[test]
    fn split_preserves_quotes() {
        let parts = split_argv(r#"foo "bar baz" qux"#);
        assert_eq!(parts, vec!["foo", "bar baz", "qux"]);
    }
}
