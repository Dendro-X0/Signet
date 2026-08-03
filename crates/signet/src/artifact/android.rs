//! Android / Gradle adapter — discover APKs and optional Gradle build.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use which::which;

use crate::project::ProjectCtx;

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::{Artifact, ArtifactKind};

const MAX_DEPTH: u32 = 10;

#[derive(Debug, Default, Clone, Copy)]
pub struct AndroidAdapter;

impl AndroidAdapter {
    fn app_root(&self, ctx: &ProjectCtx) -> PathBuf {
        ctx.root.join(&ctx.config.project.app_root)
    }
}

impl FrameworkAdapter for AndroidAdapter {
    fn id(&self) -> &'static str {
        "android"
    }

    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf {
        self.app_root(ctx)
    }

    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()> {
        let app_root = self.app_root(ctx);
        let cmd = ctx.config.project.build_command.trim();
        if !cmd.is_empty() {
            return run_shell_build(&app_root, cmd, &opts.extra_args);
        }
        run_gradlew(&app_root, &opts.extra_args)
    }

    fn discover(&self, ctx: &ProjectCtx, _profile: &str) -> anyhow::Result<Vec<Artifact>> {
        discover_android_artifacts(&self.app_root(ctx))
    }

    fn empty_hint(&self, ctx: &ProjectCtx, _profile: &str) -> String {
        format!(
            "no APKs found under {} (looked for build/outputs/apk, dist, release)\n\
             hint: run Gradle assembleRelease, set build_command, or pass --artifact path.apk",
            self.app_root(ctx).display()
        )
    }
}

fn run_gradlew(app_root: &Path, extra: &[String]) -> anyhow::Result<()> {
    let unix = app_root.join("gradlew");
    let win = app_root.join("gradlew.bat");
    let (program, use_cmd) = if cfg!(windows) && win.is_file() {
        (win, false)
    } else if unix.is_file() {
        (unix, false)
    } else if which("gradle").is_ok() {
        (PathBuf::from("gradle"), false)
    } else {
        anyhow::bail!(
            "no gradlew/gradle found under {} — set [project].build_command or pass --skip-build",
            app_root.display()
        );
    };

    let mut args = vec!["assembleRelease".to_string()];
    args.extend(extra.iter().cloned());
    println!(
        "running android build: {} {} (cwd {})",
        program.display(),
        args.join(" "),
        app_root.display()
    );
    let mut cmd = Command::new(&program);
    cmd.current_dir(app_root).args(&args);
    let _ = use_cmd;
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("android gradle build failed with status {status}");
    }
    Ok(())
}

fn run_shell_build(app_root: &Path, build_command: &str, extra: &[String]) -> anyhow::Result<()> {
    let parts = split_argv(build_command);
    if parts.is_empty() {
        anyhow::bail!("empty build_command");
    }
    let program = &parts[0];
    let mut args: Vec<String> = parts[1..].to_vec();
    args.extend(extra.iter().cloned());
    println!(
        "running android build: {} {} (cwd {})",
        program,
        args.join(" "),
        app_root.display()
    );
    let status = Command::new(program)
        .current_dir(app_root)
        .args(&args)
        .status()?;
    if !status.success() {
        anyhow::bail!("android build failed with status {status}");
    }
    Ok(())
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

pub fn discover_android_artifacts(app_root: &Path) -> anyhow::Result<Vec<Artifact>> {
    let mut out = Vec::new();
    for name in ["dist", "release", "app/build/outputs/apk", "build/outputs/apk"] {
        let dir = app_root.join(name);
        if dir.is_dir() {
            visit_dir(&dir, 0, &mut out)?;
        }
    }
    // Broader walk for **/build/outputs/apk
    visit_for_outputs(app_root, 0, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.dedup_by(|a, b| a.path == b.path);
    Ok(out)
}

fn visit_for_outputs(dir: &Path, depth: u32, out: &mut Vec<Artifact>) -> anyhow::Result<()> {
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
        if matches!(
            name.as_str(),
            "node_modules" | ".git" | ".signet" | ".selfsign" | ".gradle" | "build"
        ) && path.is_dir()
            && name != "build"
        {
            continue;
        }
        if path.is_dir() {
            if name == "apk" && path.to_string_lossy().contains("outputs") {
                visit_dir(&path, 0, out)?;
            } else if name != "node_modules" && name != ".git" {
                // Limit recursion into build trees
                if name == "build" || depth < 4 {
                    visit_for_outputs(&path, depth + 1, out)?;
                }
            }
        }
    }
    Ok(())
}

fn visit_dir(dir: &Path, depth: u32, out: &mut Vec<Artifact>) -> anyhow::Result<()> {
    if depth > MAX_DEPTH {
        return Ok(());
    }
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, depth + 1, out)?;
        } else if let Some(kind) = ArtifactKind::classify_file(&path) {
            if matches!(kind, ArtifactKind::Apk | ArtifactKind::Aab) {
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
    fn discovers_apk_under_outputs() {
        let dir = tempdir().unwrap();
        let apk_dir = dir.path().join("app/build/outputs/apk/release");
        fs::create_dir_all(&apk_dir).unwrap();
        fs::write(apk_dir.join("app-release.apk"), b"APK").unwrap();
        let found = discover_android_artifacts(dir.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, ArtifactKind::Apk);
    }
}
