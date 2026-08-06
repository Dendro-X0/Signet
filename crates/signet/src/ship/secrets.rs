//! Push or print GitHub Actions secrets from local `.signet/` (ship slice S1).

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::android::keystore_paths;
use crate::project::ProjectCtx;
use crate::release::assess_github_auth;
use crate::ship::ci_readiness::{assess_ci_readiness, required_ci_secrets};
use crate::ui::console;

#[derive(Debug, Clone)]
pub struct SecretsArgs {
    /// Prepare GitHub Actions secret recipe (dry-run unless --apply)
    pub push: bool,
    /// Actually run `gh secret set` (requires --push + gh auth)
    pub apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretLine {
    pub name: &'static str,
    pub local_ok: bool,
    pub detail: String,
}

pub fn run_secrets(ctx: &ProjectCtx, args: SecretsArgs) -> anyhow::Result<()> {
    if args.apply && !args.push {
        anyhow::bail!("--apply requires --push");
    }

    console::banner("ship · secrets");
    let needed = required_ci_secrets(&ctx.config);
    let lines = assess_local_secrets(ctx, &needed);

    console::section("required CI secrets");
    for line in &lines {
        let status = if line.local_ok { "local-ok" } else { "MISSING" };
        console::kv(28, line.name, &format!("{status} — {}", line.detail));
    }

    let readiness = assess_ci_readiness(&ctx.root, &ctx.config);
    console::blank();
    console::note(&readiness.summary_line());
    for g in &readiness.gaps {
        console::note(&format!("{}: {} — {}", g.id, g.detail, g.next));
    }

    if !args.push {
        console::blank();
        console::note("Dry-run assess only. Next: `signet ship secrets --push` (recipe) or `--push --apply`.");
        return Ok(());
    }

    let auth = assess_github_auth();
    if !auth.ready() {
        console::blank();
        for line in auth.setup_guide().lines() {
            console::note(line);
        }
        let _ = crate::release::offer_open_auth_setup(&auth);
        anyhow::bail!(
            "cannot push secrets: {}\nhint: `gh auth login` or set GH_TOKEN, then retry",
            auth.summary_line()
        );
    }

    if args.apply {
        apply_secrets(ctx, &lines)?;
        console::ok_line("pushed secrets via gh secret set");
    } else {
        print_push_recipe(ctx, &lines)?;
        console::blank();
        console::note("Dry-run push recipe only. Pass --apply to run `gh secret set`.");
    }

    let after = assess_ci_readiness(&ctx.root, &ctx.config);
    console::blank();
    console::note(&after.summary_line());
    Ok(())
}

fn assess_local_secrets(ctx: &ProjectCtx, needed: &[&'static str]) -> Vec<SecretLine> {
    let secrets = ctx.secrets_dir();
    let mut out = Vec::new();
    for &name in needed {
        out.push(match name {
            "SIGNET_ANDROID_KEYSTORE_BASE64" => {
                let p = keystore_paths(&secrets);
                SecretLine {
                    name,
                    local_ok: p.keystore.is_file(),
                    detail: if p.keystore.is_file() {
                        format!("{}", p.keystore.display())
                    } else {
                        "run `signet android keystore create`".into()
                    },
                }
            }
            "SIGNET_ANDROID_META_BASE64" => {
                let p = keystore_paths(&secrets);
                SecretLine {
                    name,
                    local_ok: p.meta.is_file(),
                    detail: if p.meta.is_file() {
                        format!("{}", p.meta.display())
                    } else {
                        "run `signet android keystore create` (writes meta.toml)".into()
                    },
                }
            }
            "SIGNET_ANDROID_STORE_PASS" => {
                let ok = std::env::var("SIGNET_ANDROID_STORE_PASS")
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
                SecretLine {
                    name,
                    local_ok: ok,
                    detail: if ok {
                        "set in this shell (value not printed)".into()
                    } else {
                        "export SIGNET_ANDROID_STORE_PASS=… then --push --apply".into()
                    },
                }
            }
            "SIGNET_IDENTITY_BUNDLE_BASE64" => {
                let id = secrets.join("identity");
                let active = id.join("active").is_file();
                SecretLine {
                    name,
                    local_ok: active,
                    detail: if active {
                        format!("{}", id.display())
                    } else {
                        "run `signet identity create`".into()
                    },
                }
            }
            "SIGNET_SUMS_KEY_BASE64" => {
                let key = secrets.join("sums/minisign.key");
                let want = ctx.config.trust.checksum_signing.minisign;
                SecretLine {
                    name,
                    local_ok: !want || key.is_file(),
                    detail: if !want {
                        "minisign disabled in config — optional".into()
                    } else if key.is_file() {
                        format!("{}", key.display())
                    } else {
                        "run `signet sums-key create`".into()
                    },
                }
            }
            other => SecretLine {
                name: other,
                local_ok: false,
                detail: "unknown secret".into(),
            },
        });
    }
    out
}

fn print_push_recipe(ctx: &ProjectCtx, lines: &[SecretLine]) -> anyhow::Result<()> {
    console::section("gh secret set recipe (dry-run)");
    for line in lines {
        if !line.local_ok && line.name != "SIGNET_ANDROID_STORE_PASS" {
            console::note(&format!("skip {}: {}", line.name, line.detail));
            continue;
        }
        match line.name {
            "SIGNET_ANDROID_STORE_PASS" => {
                println!(
                    "# printf %s \"$SIGNET_ANDROID_STORE_PASS\" | gh secret set SIGNET_ANDROID_STORE_PASS"
                );
            }
            "SIGNET_ANDROID_KEYSTORE_BASE64" => {
                let p = keystore_paths(&ctx.secrets_dir()).keystore;
                println!(
                    "# base64 -w0 '{}' | gh secret set SIGNET_ANDROID_KEYSTORE_BASE64",
                    p.display()
                );
            }
            "SIGNET_ANDROID_META_BASE64" => {
                let p = keystore_paths(&ctx.secrets_dir()).meta;
                println!(
                    "# base64 -w0 '{}' | gh secret set SIGNET_ANDROID_META_BASE64",
                    p.display()
                );
            }
            "SIGNET_IDENTITY_BUNDLE_BASE64" => {
                println!(
                    "# tar -C '{}' -czf - identity | base64 -w0 | gh secret set SIGNET_IDENTITY_BUNDLE_BASE64",
                    ctx.secrets_dir().display()
                );
            }
            "SIGNET_SUMS_KEY_BASE64" => {
                let p = ctx.secrets_dir().join("sums/minisign.key");
                println!(
                    "# base64 -w0 '{}' | gh secret set SIGNET_SUMS_KEY_BASE64",
                    p.display()
                );
            }
            _ => {}
        }
    }
    Ok(())
}

fn apply_secrets(ctx: &ProjectCtx, lines: &[SecretLine]) -> anyhow::Result<()> {
    for line in lines {
        match line.name {
            "SIGNET_ANDROID_STORE_PASS" => {
                let pass = std::env::var("SIGNET_ANDROID_STORE_PASS").map_err(|_| {
                    anyhow::anyhow!("SIGNET_ANDROID_STORE_PASS must be set in this shell for --apply")
                })?;
                if pass.trim().is_empty() {
                    anyhow::bail!("SIGNET_ANDROID_STORE_PASS is empty");
                }
                gh_secret_set(line.name, pass.as_bytes())?;
                console::ok_line(&format!("set {}", line.name));
            }
            "SIGNET_ANDROID_KEYSTORE_BASE64" => {
                let p = keystore_paths(&ctx.secrets_dir()).keystore;
                if !p.is_file() {
                    anyhow::bail!("missing keystore at {}", p.display());
                }
                let b64 = base64_encode(&fs::read(&p)?);
                gh_secret_set(line.name, b64.as_bytes())?;
                console::ok_line(&format!("set {}", line.name));
            }
            "SIGNET_ANDROID_META_BASE64" => {
                let p = keystore_paths(&ctx.secrets_dir()).meta;
                if !p.is_file() {
                    anyhow::bail!("missing android meta at {}", p.display());
                }
                let b64 = base64_encode(&fs::read(&p)?);
                gh_secret_set(line.name, b64.as_bytes())?;
                console::ok_line(&format!("set {}", line.name));
            }
            "SIGNET_IDENTITY_BUNDLE_BASE64" => {
                let tar = tar_identity_bundle(&ctx.secrets_dir())?;
                let b64 = base64_encode(&tar);
                gh_secret_set(line.name, b64.as_bytes())?;
                console::ok_line(&format!("set {}", line.name));
            }
            "SIGNET_SUMS_KEY_BASE64" => {
                if !ctx.config.trust.checksum_signing.minisign {
                    continue;
                }
                let p = ctx.secrets_dir().join("sums/minisign.key");
                if !p.is_file() {
                    anyhow::bail!("missing sums key at {}", p.display());
                }
                let b64 = base64_encode(&fs::read(&p)?);
                gh_secret_set(line.name, b64.as_bytes())?;
                console::ok_line(&format!("set {}", line.name));
            }
            _ => {}
        }
    }
    Ok(())
}

fn gh_secret_set(name: &str, value: &[u8]) -> anyhow::Result<()> {
    let mut child = Command::new("gh")
        .args(["secret", "set", name])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn gh: {e}"))?;
    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("gh stdin unavailable"))?;
        stdin.write_all(value)?;
    }
    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("gh secret set {name} failed with {status}");
    }
    Ok(())
}

fn tar_identity_bundle(secrets_dir: &Path) -> anyhow::Result<Vec<u8>> {
    let identity = secrets_dir.join("identity");
    if !identity.join("active").is_file() {
        anyhow::bail!("no active identity under {}", identity.display());
    }
    // Portable: zip the identity directory (no external tar required on Windows).
    let cursor = std::io::Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    add_dir_to_zip(&mut zip, &identity, Path::new("identity"), opts)?;
    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    dir: &Path,
    prefix: &Path,
    opts: SimpleFileOptions,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = prefix.join(entry.file_name());
        let name_str = name
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("non-utf8 path in identity bundle"))?
            .replace('\\', "/");
        if path.is_dir() {
            zip.add_directory(format!("{name_str}/"), opts)?;
            add_dir_to_zip(zip, &path, &name, opts)?;
        } else if path.is_file() {
            zip.start_file(name_str, opts)?;
            let mut f = fs::File::open(&path)?;
            std::io::copy(&mut f, zip)?;
        }
    }
    Ok(())
}

fn base64_encode(bytes: &[u8]) -> String {
    // Standard base64 without padding issues for gh secret set stdin.
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(T[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(T[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Target};
    use crate::ship::ci_readiness::required_ci_secrets;

    #[test]
    fn android_target_requires_keystore_secrets() {
        let mut cfg = Config::example("app", ".");
        cfg.platforms.windows = false;
        cfg.platforms.macos = false;
        cfg.platforms.linux = false;
        cfg.targets.push(Target {
            id: "mobile".into(),
            framework: "expo".into(),
            app_root: "apps/m".into(),
            build_command: String::new(),
        });
        let names = required_ci_secrets(&cfg);
        assert!(names.contains(&"SIGNET_ANDROID_KEYSTORE_BASE64"));
        assert!(names.contains(&"SIGNET_ANDROID_STORE_PASS"));
    }

    #[test]
    fn base64_roundtrip_len() {
        let s = base64_encode(b"hello");
        assert_eq!(s, "aGVsbG8=");
    }
}
