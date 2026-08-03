//! Shared depth-capped walk for framework output directories.

use std::fs;
use std::path::Path;

use super::{Artifact, ArtifactKind};

const MAX_DEPTH: u32 = 8;

const SKIP_NAMES: &[&str] = &[
    "node_modules",
    ".git",
    ".signet",
    ".selfsign",
    ".dart_tool",
    "Pods",
    ".gradle",
    "coverage",
];

/// Discover installable artifacts under `app_root / each(dirs)`.
pub fn discover_in_dirs(app_root: &Path, dirs: &[&str]) -> anyhow::Result<Vec<Artifact>> {
    let mut out = Vec::new();
    for name in dirs {
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
        if SKIP_NAMES.iter().any(|s| name == *s) {
            continue;
        }
        if path.is_dir() {
            if path.extension().and_then(|e| e.to_str()) == Some("app")
                || name.ends_with(".app")
            {
                out.push(Artifact::new(path, ArtifactKind::MacApp));
            } else {
                visit_dir(&path, depth + 1, out)?;
            }
        } else if let Some(kind) = ArtifactKind::classify_file(&path) {
            out.push(Artifact::new(path, kind));
        }
    }
    Ok(())
}

/// Split a shell-ish argv string (whitespace + double quotes).
pub fn split_argv(s: &str) -> Vec<String> {
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

/// Run `[project].build_command` in `app_root`. Empty command → error with `examples`.
pub fn run_required_build_command(
    app_root: &Path,
    build_command: &str,
    extra_args: &[String],
    framework_id: &str,
    examples: &str,
) -> anyhow::Result<()> {
    let trimmed = build_command.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "[{framework_id}] [project].build_command is required (no default target guess).\n\
             Examples: {examples}\n\
             Or pass --skip-build to discover/sign existing outputs only."
        );
    }
    let parts = split_argv(trimmed);
    if parts.is_empty() {
        anyhow::bail!("[project].build_command is empty after parse");
    }
    let program = &parts[0];
    let mut args: Vec<String> = parts[1..].to_vec();
    args.extend(extra_args.iter().cloned());

    println!(
        "running {framework_id} build: {} {} (cwd {})",
        program,
        args.join(" "),
        app_root.display()
    );

    let status = spawn_build_command(program, &args, app_root)?;
    if !status.success() {
        anyhow::bail!("{framework_id} build failed with status {status}");
    }
    Ok(())
}

/// Spawn a build program. On Windows, route through `cmd /C` so `pnpm`/`npm` `.cmd` shims work
/// (CreateProcess cannot run the extensionless POSIX npm shims).
pub fn spawn_build_command(
    program: &str,
    args: &[String],
    cwd: &Path,
) -> anyhow::Result<std::process::ExitStatus> {
    use std::process::Command;

    #[cfg(windows)]
    {
        let mut cmd = Command::new("cmd");
        cmd.current_dir(cwd).args(["/D", "/C"]).arg(program);
        for a in args {
            cmd.arg(a);
        }
        Ok(cmd.status()?)
    }

    #[cfg(not(windows))]
    {
        use which::which;
        let resolved = which(program).unwrap_or_else(|_| std::path::PathBuf::from(program));
        Ok(Command::new(resolved).current_dir(cwd).args(args).status()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn discovers_apk_nested() {
        let dir = tempdir().unwrap();
        let apk_dir = dir.path().join("build/app/outputs/flutter-apk");
        fs::create_dir_all(&apk_dir).unwrap();
        fs::write(apk_dir.join("app-release.apk"), b"APK").unwrap();
        let found = discover_in_dirs(dir.path(), &["build/app/outputs"]).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, ArtifactKind::Apk);
    }

    #[test]
    fn split_preserves_quotes() {
        assert_eq!(
            split_argv(r#"flutter build "windows""#),
            vec!["flutter", "build", "windows"]
        );
    }
}
