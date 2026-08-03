//! Rust CLI / binary adapter — `cargo build` + discover host binaries under `target/<profile>/`.

use std::fs;
use std::path::{Path, PathBuf};

use which::which;

use crate::project::ProjectCtx;

use super::adapter::{BuildOpts, FrameworkAdapter};
use super::{Artifact, ArtifactKind};

#[derive(Debug, Default, Clone, Copy)]
pub struct CliAdapter;

impl CliAdapter {
    fn app_root(&self, ctx: &ProjectCtx) -> PathBuf {
        ctx.root.join(&ctx.config.project.tauri_root)
    }
}

impl FrameworkAdapter for CliAdapter {
    fn id(&self) -> &'static str {
        "cli"
    }

    fn label_root(&self, ctx: &ProjectCtx) -> PathBuf {
        self.app_root(ctx)
    }

    fn build(&self, ctx: &ProjectCtx, opts: &BuildOpts) -> anyhow::Result<()> {
        let app_root = self.app_root(ctx);
        let (program, mut args) = resolve_build_argv(&ctx.config.project.build_command, &opts.profile)?;
        args.extend(opts.extra_args.iter().cloned());

        println!(
            "running cli build: {} {} (cwd {})",
            program,
            args.join(" "),
            app_root.display()
        );

        let status = super::walk_outputs::spawn_build_command(&program, &args, &app_root)?;
        if !status.success() {
            anyhow::bail!("cli build failed with status {status}");
        }
        Ok(())
    }

    fn discover(&self, ctx: &ProjectCtx, profile: &str) -> anyhow::Result<Vec<Artifact>> {
        let app_root = self.app_root(ctx);
        discover_cli_binaries(&app_root, profile)
    }

    fn empty_hint(&self, ctx: &ProjectCtx, profile: &str) -> String {
        let root = self.app_root(ctx);
        format!(
            "no host binaries found under {}/target/{profile}\n\
             hint: run `cargo build --{profile}` (or set build_command), then retry; \
             or pass --artifact <path>",
            root.display()
        )
    }
}

fn resolve_build_argv(build_command: &str, profile: &str) -> anyhow::Result<(String, Vec<String>)> {
    let trimmed = build_command.trim();
    if trimmed.is_empty() {
        ensure_cargo()?;
        let mut args = vec!["build".into()];
        if profile != "debug" && profile != "dev" {
            if profile == "release" {
                args.push("--release".into());
            } else {
                args.push("--profile".into());
                args.push(profile.to_string());
            }
        }
        return Ok(("cargo".into(), args));
    }
    let parts: Vec<String> = trimmed.split_whitespace().map(|s| s.to_string()).collect();
    if parts.is_empty() {
        anyhow::bail!("[project].build_command is empty after parse");
    }
    Ok((parts[0].clone(), parts[1..].to_vec()))
}

fn ensure_cargo() -> anyhow::Result<()> {
    if which("cargo").is_err() {
        anyhow::bail!("cargo not found on PATH — install Rust from https://rustup.rs/");
    }
    Ok(())
}

fn discover_cli_binaries(app_root: &Path, profile: &str) -> anyhow::Result<Vec<Artifact>> {
    let mut out = Vec::new();
    // Prefer local target/, then walk up for Cargo workspace `target/`.
    let mut dir = app_root.to_path_buf();
    for _ in 0..6 {
        let candidate = dir.join("target").join(profile);
        if candidate.is_dir() {
            collect_bins_in_dir(&candidate, &mut out)?;
            if !out.is_empty() {
                return Ok(out);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    Ok(out)
}

fn collect_bins_in_dir(dir: &Path, out: &mut Vec<Artifact>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".pdb")
            || lower.ends_with(".d")
            || lower.ends_with(".rlib")
            || lower.ends_with(".rmeta")
            || lower.ends_with(".dll")
            || lower.ends_with(".so")
            || lower.ends_with(".dylib")
            || lower.ends_with(".lib")
            || lower.ends_with(".exp")
            || lower.starts_with('.')
        {
            continue;
        }
        // Cargo stamps / dotted junk — allow PE (.exe) only among extensions.
        if name.contains('.') && !lower.ends_with(".exe") {
            continue;
        }
        #[cfg(windows)]
        {
            if !lower.ends_with(".exe") {
                continue;
            }
        }
        #[cfg(not(windows))]
        {
            // Host Cargo bins are extensionless + executable. `.exe` may be a
            // cross-compiled PE sitting in target/ — include without +x.
            if !lower.ends_with(".exe") {
                use std::os::unix::fs::PermissionsExt;
                let meta = entry.metadata()?;
                if meta.permissions().mode() & 0o111 == 0 {
                    continue;
                }
            }
        }

        let kind = ArtifactKind::classify_explicit(&path);
        out.push(Artifact {
            path: path.clone(),
            kind,
            name_for_sums: name.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn discovers_release_exe() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let bin_dir = root.join("target/release");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("mytool.pdb"), b"x").unwrap();
        fs::create_dir_all(root.join("target/release/deps")).unwrap();
        fs::write(root.join("target/release/deps/noise"), b"x").unwrap();

        #[cfg(windows)]
        {
            fs::write(bin_dir.join("mytool.exe"), b"MZ").unwrap();
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let bin = bin_dir.join("mytool");
            fs::write(&bin, b"#!/bin/sh\n").unwrap();
            let mut perms = fs::metadata(&bin).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&bin, perms).unwrap();
        }

        let arts = discover_cli_binaries(root, "release").unwrap();
        assert_eq!(arts.len(), 1, "expected one host binary, got {arts:?}");
        #[cfg(windows)]
        assert!(arts[0].path.ends_with("mytool.exe"));
        #[cfg(not(windows))]
        assert!(arts[0].path.ends_with("mytool"));
    }

    #[test]
    #[cfg(not(windows))]
    fn discovers_cross_compiled_exe_on_unix() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let bin_dir = root.join("target/release");
        fs::create_dir_all(&bin_dir).unwrap();
        // No +x — still discoverable as PE for checksum / optional signing tooling.
        fs::write(bin_dir.join("mytool.exe"), b"MZ").unwrap();
        let arts = discover_cli_binaries(root, "release").unwrap();
        assert_eq!(arts.len(), 1);
        assert!(arts[0].path.ends_with("mytool.exe"));
    }
}
