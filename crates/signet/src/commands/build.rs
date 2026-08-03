use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::android::{keystore_paths, sign_apks};
use crate::artifact::{
    requires_explicit_build_command, select_adapter, Artifact, ArtifactKind, BuildOpts,
};
use crate::config::{select_targets, Target};
use crate::identity::load_active;
use crate::project::ProjectCtx;
use crate::sign::{
    host_signable, maybe_sign_sums, sign_host_artifacts, tool_hint_for_host, write_sha256sums,
    SignOptions,
};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Build only this `[[targets]].id` (default: all targets)
    #[arg(long)]
    pub target: Option<String>,

    /// Fail the run if any target was skipped or errored (still signs successful siblings first)
    #[arg(long)]
    pub strict_targets: bool,

    /// Skip framework build; only discover + sign existing artifacts
    #[arg(long)]
    pub skip_build: bool,

    /// Build without signing (still writes SHA256SUMS when artifacts exist)
    #[arg(long)]
    pub no_sign: bool,

    /// Skip minisign/GPG signing of SHA256SUMS
    #[arg(long)]
    pub no_sums_sign: bool,

    /// Fail if minisign cannot sign SHA256SUMS
    #[arg(long)]
    pub require_sums_sign: bool,

    /// Fail if GPG checksum signing was requested but did not succeed
    #[arg(long)]
    pub require_gpg: bool,

    /// Cargo/Tauri profile (default: release)
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Do not request an Authenticode timestamp (Windows)
    #[arg(long)]
    pub no_timestamp: bool,

    /// Extra args forwarded to the framework build (Tauri: `tauri build`)
    #[arg(long = "tauri-arg")]
    pub tauri_args: Vec<String>,

    /// Explicit artifact paths to sign (skips discovery when set)
    #[arg(long = "artifact")]
    pub artifacts: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct TargetDebt {
    id: String,
    framework: String,
    reason: String,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your app directory first")
    })?;

    let coverage = crate::ship::assess_coverage(&ctx.root, &ctx.config);
    println!("{}", coverage.summary_line());
    if coverage.has_gap() {
        println!(
            "note: ship gap = {} — this host signs {} only; see `signet ship --plan`",
            coverage.gap.join(", "),
            coverage.host_can_sign
        );
    }

    let all = ctx.targets();
    let selected = select_targets(&all, args.target.as_deref())?;
    let soft = args.target.is_none();

    let mut discovered: Vec<Artifact> = Vec::new();
    let mut saw_ios = false;
    let mut debt: Vec<TargetDebt> = Vec::new();

    if args.artifacts.is_empty() {
        for target in &selected {
            match process_target(&ctx, target, &args, soft, &mut saw_ios, &mut discovered) {
                Ok(()) => {}
                Err(e) if soft => {
                    let reason = format!("{e:#}");
                    println!(
                        "warning: target {} ({}) skipped — {reason}",
                        target.id, target.framework
                    );
                    debt.push(TargetDebt {
                        id: target.id.clone(),
                        framework: target.framework.clone(),
                        reason,
                    });
                }
                Err(e) => return Err(e),
            }
        }
    } else {
        println!("using {} explicit --artifact path(s)", args.artifacts.len());
        discovered = args.artifacts.iter().map(Artifact::from_path).collect();
    }

    if discovered.is_empty() {
        let mut parts: Vec<String> = debt
            .iter()
            .map(|d| format!("target {} ({}): {}", d.id, d.framework, d.reason))
            .collect();
        for t in &selected {
            let tctx = ctx.with_target(t);
            let hint = select_adapter(&tctx.root, &tctx.config)
                .map(|a| a.empty_hint(&tctx, &args.profile))
                .unwrap_or_else(|e| e.to_string());
            parts.push(hint);
        }
        anyhow::bail!("{}", parts.join("\n"));
    }

    println!("discovered {} artifact(s):", discovered.len());
    for a in &discovered {
        println!("  [{}] {}", a.kind.as_str(), a.path.display());
    }
    if !debt.is_empty() {
        println!("note: {} target(s) unpaid/failed — continuing with discovered artifacts", debt.len());
        for d in &debt {
            println!("  debt: {} ({}) — {}", d.id, d.framework, d.reason);
        }
        println!("hint: set [[targets]].build_command, pass --target <id>, or --skip-build");
    }

    let checksum_paths: Vec<PathBuf> = discovered
        .iter()
        .filter(|a| a.path.is_file())
        .map(|a| a.path.clone())
        .collect();
    let sums_path = ctx.root.join("SHA256SUMS");
    if !checksum_paths.is_empty() {
        write_sha256sums(&sums_path, &checksum_paths)?;
        println!("wrote {}", sums_path.display());
        emit_sums_sign(&ctx, &sums_path, &args)?;
    }

    if args.no_sign {
        println!("signing skipped (--no-sign)");
        println!(
            "note: OS warnings (SmartScreen / Gatekeeper) are unchanged by checksums alone"
        );
        return finish_with_debt(&debt, args.strict_targets);
    }

    let apks: Vec<Artifact> = discovered
        .iter()
        .filter(|a| a.kind == ArtifactKind::Apk)
        .cloned()
        .collect();
    let host = host_signable(&discovered);

    if saw_ios || discovered.iter().any(|a| a.kind == ArtifactKind::Ipa) {
        println!("note: {}", crate::ios::honesty_notes());
        println!(
            "note: IPA/device signing stays in Xcode — `signet ios package --app …` for IPA layout"
        );
    }

    if !apks.is_empty() {
        sign_android_flow(&ctx, &apks, &checksum_paths, &sums_path, &args)?;
    }

    if host.is_empty() && apks.is_empty() {
        if saw_ios || discovered.iter().any(|a| a.kind == ArtifactKind::Ipa) {
            println!("done — see docs/ios.md");
            return finish_with_debt(&debt, args.strict_targets);
        }
        println!("nothing to host-sign (no host-matching or APK artifacts)");
        return finish_with_debt(&debt, args.strict_targets);
    }

    if host.is_empty() {
        return finish_with_debt(&debt, args.strict_targets);
    }

    let identity = load_active(&ctx.identity_root()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet identity create` first (or pass --no-sign)")
    })?;
    println!(
        "signing with identity '{}' ({})",
        identity.meta.name, identity.meta.fingerprint_sha256
    );
    println!("host tools: {}", tool_hint_for_host());

    let opts = SignOptions {
        timestamp: !args.no_timestamp,
        ..SignOptions::default()
    };
    let report = sign_host_artifacts(&identity, &host, &opts)?;

    for w in &report.warnings {
        println!("warning: {w}");
    }
    for s in &report.signed {
        println!("signed: {} ({})", s.path.display(), s.method);
        if let Some(note) = &s.note {
            println!("  note: {note}");
        }
    }
    for (path, reason) in &report.skipped {
        println!("skipped: {} — {reason}", path.display());
    }

    if report.signed.is_empty() && report.skipped.is_empty() {
        println!("nothing signed (no host-matching artifacts)");
    } else if report.signed.is_empty() {
        anyhow::bail!("signing produced no successful artifacts");
    }

    let mut refresh = checksum_paths;
    for s in &report.signed {
        if s.path.is_file() && !refresh.iter().any(|p| p == &s.path) {
            refresh.push(s.path.clone());
        }
    }
    if !refresh.is_empty() {
        write_sha256sums(&sums_path, &refresh)?;
        println!("wrote {} (post-sign)", sums_path.display());
        emit_sums_sign(&ctx, &sums_path, &args)?;
    }

    println!("done — compare fingerprints via `signet identity show` / TRUST.md");
    finish_with_debt(&debt, args.strict_targets)
}

fn finish_with_debt(debt: &[TargetDebt], strict: bool) -> anyhow::Result<()> {
    if strict && !debt.is_empty() {
        let summary = debt
            .iter()
            .map(|d| format!("{} ({})", d.id, d.framework))
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!(
            "--strict-targets: unpaid/failed targets remain: {summary}"
        );
    }
    Ok(())
}

fn process_target(
    ctx: &ProjectCtx,
    target: &Target,
    args: &Args,
    soft: bool,
    saw_ios: &mut bool,
    discovered: &mut Vec<Artifact>,
) -> anyhow::Result<()> {
    println!("── target {} ({}) ──", target.id, target.framework);
    let tctx = ctx.with_target(target);
    let adapter = select_adapter(&tctx.root, &tctx.config)?;
    let label = adapter.label_root(&tctx);
    println!("{} root: {}", adapter.id(), label.display());

    let unpaid = !args.skip_build
        && requires_explicit_build_command(&target.framework)
        && target.build_command.trim().is_empty()
        && tctx.config.project.build_command.trim().is_empty();

    if unpaid {
        let reason = format!(
            "unpaid recipe: set build_command for target `{}` (framework {}), or --skip-build / --target",
            target.id, target.framework
        );
        if soft {
            // Still try discover in case artifacts already exist.
            match adapter.discover(&tctx, &args.profile) {
                Ok(arts) => {
                    for art in arts {
                        if !discovered.iter().any(|a| a.path == art.path) {
                            discovered.push(art);
                        }
                    }
                }
                Err(e) => println!("note: discover after unpaid skip failed: {e}"),
            }
            return Err(anyhow::anyhow!("{reason}"));
        }
        anyhow::bail!("{reason}");
    }

    if !args.skip_build {
        adapter.build(
            &tctx,
            &BuildOpts {
                profile: args.profile.clone(),
                extra_args: args.tauri_args.clone(),
            },
        )?;
    } else {
        println!("skipping {} build (--skip-build)", adapter.id());
    }

    if adapter.id() == "ios" {
        *saw_ios = true;
    }

    for art in adapter.discover(&tctx, &args.profile)? {
        if !discovered.iter().any(|a| a.path == art.path) {
            discovered.push(art);
        }
    }
    Ok(())
}

fn sign_android_flow(
    ctx: &ProjectCtx,
    apk_arts: &[Artifact],
    checksum_paths: &[PathBuf],
    sums_path: &std::path::Path,
    args: &Args,
) -> anyhow::Result<()> {
    let paths = keystore_paths(&ctx.secrets_dir());
    let apks: Vec<PathBuf> = apk_arts.iter().map(|a| a.path.clone()).collect();
    if apks.is_empty() {
        return Ok(());
    }
    println!(
        "signing {} APK(s) with android keystore ({})",
        apks.len(),
        paths.keystore.display()
    );
    println!("note: local keystore ≠ Play App Signing key — see docs/android.md");

    let report = sign_apks(&paths, &apks)?;
    for w in &report.warnings {
        println!("warning: {w}");
    }
    for (p, method) in &report.signed {
        println!("signed: {} ({method})", p.display());
    }
    for (p, reason) in &report.skipped {
        println!("skipped: {} — {reason}", p.display());
    }
    if report.signed.is_empty() {
        anyhow::bail!("android signing produced no successful APKs");
    }

    let mut refresh = checksum_paths.to_vec();
    for (p, _) in &report.signed {
        if p.is_file() && !refresh.iter().any(|x| x == p) {
            refresh.push(p.clone());
        }
    }
    if !refresh.is_empty() {
        write_sha256sums(sums_path, &refresh)?;
        println!("wrote {} (post-sign)", sums_path.display());
        emit_sums_sign(ctx, sums_path, args)?;
    }
    println!("done — compare Android cert digest via `signet android keystore show` / TRUST.md");
    Ok(())
}

fn emit_sums_sign(
    ctx: &ProjectCtx,
    sums_path: &std::path::Path,
    args: &Args,
) -> anyhow::Result<()> {
    let report = maybe_sign_sums(
        sums_path,
        &ctx.secrets_dir(),
        &ctx.config.trust.checksum_signing,
        args.no_sums_sign,
        args.require_sums_sign,
        args.require_gpg,
    )?;
    for w in &report.warnings {
        println!("warning: {w}");
    }
    if let Some(p) = &report.minisig {
        println!("wrote {}", p.display());
    }
    if let Some(p) = &report.asc {
        println!("wrote {}", p.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpaid_expo_is_soft_debt_reason() {
        assert!(requires_explicit_build_command("expo"));
        assert!(requires_explicit_build_command("flutter"));
        assert!(!requires_explicit_build_command("tauri"));
        assert!(!requires_explicit_build_command("electron"));
    }

    #[test]
    fn strict_finish_fails_on_debt() {
        let debt = vec![TargetDebt {
            id: "mobile".into(),
            framework: "expo".into(),
            reason: "unpaid".into(),
        }];
        assert!(finish_with_debt(&debt, true).is_err());
        assert!(finish_with_debt(&debt, false).is_ok());
        assert!(finish_with_debt(&[], true).is_ok());
    }
}
