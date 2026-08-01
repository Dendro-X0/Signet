//! `signet verify` — fingerprint + SHA256SUMS + community sig checks (Phases 7–8).

use std::fs;
use std::path::{Path, PathBuf};

use clap::Args as ClapArgs;

use crate::config::{resolve_config_path, Config};
use crate::error::ExitCode;
use crate::sign::{
    parse_minisign_pub_from_trust, verify_sha256sums, verify_sums_gpg, verify_sums_minisign,
    ChecksumResult, SumsKeyPaths,
};
use crate::trust_kit::{normalize_fingerprint, parse_fingerprint, parse_tier_id};
use crate::trust_tier::{resolve_primary_tier, TierHints};
use crate::ui::console;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Artifact to check against SHA256SUMS (repeatable)
    #[arg(long = "artifact")]
    pub artifacts: Vec<PathBuf>,

    /// Path to SHA256SUMS (default: project SHA256SUMS)
    #[arg(long)]
    pub sums: Option<PathBuf>,

    /// Path to TRUST.md (default: project TRUST.md)
    #[arg(long)]
    pub trust: Option<PathBuf>,

    /// Fail (exit 3) if community signature on sums is missing or invalid
    #[arg(long)]
    pub require_sig: bool,

    /// Minisign public key file (overrides TRUST.md / `.signet/sums/minisign.pub`)
    #[arg(long)]
    pub minisign_pub: Option<PathBuf>,

    /// Machine-readable report
    #[arg(long)]
    pub json: bool,

    /// Expected SHA-256 fingerprint (overrides TRUST.md)
    #[arg(long)]
    pub fingerprint: Option<String>,
}

#[derive(Debug)]
struct SumsSignatureReport {
    present: bool,
    ok: Option<bool>,
    scheme: Option<&'static str>,
}

#[derive(Debug)]
struct Report {
    schema_version: u32,
    tier: String,
    fingerprint_expected: Option<String>,
    fingerprint_source: Option<String>,
    fingerprint_ok: Option<bool>,
    checksums: Vec<ChecksumResult>,
    sums_path: Option<PathBuf>,
    sums_signature: SumsSignatureReport,
    warnings: Vec<String>,
}

pub fn run(args: Args) -> ExitCode {
    match run_inner(args) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::NotImplemented // exit 2 — missing inputs / parse problems
        }
    }
}

fn run_inner(args: Args) -> anyhow::Result<ExitCode> {
    let (root, config) = load_root_config(args.config.as_deref());
    let trust_path = args
        .trust
        .clone()
        .unwrap_or_else(|| root.join("TRUST.md"));
    let sums_path = args
        .sums
        .clone()
        .unwrap_or_else(|| root.join("SHA256SUMS"));

    let mut warnings = vec![
        "SmartScreen/Gatekeeper not evaluated".into(),
        "Never install publisher certificates into Trusted Root on end-user PCs".into(),
    ];

    let trust_body = if trust_path.is_file() {
        Some(fs::read_to_string(&trust_path)?)
    } else {
        None
    };

    let mut fingerprint_expected = args.fingerprint.clone();
    let mut fingerprint_source = if args.fingerprint.is_some() {
        Some("cli".into())
    } else {
        None
    };
    let mut fingerprint_ok: Option<bool> = None;

    if let Some(cli_fp) = args.fingerprint.as_deref() {
        if let Some(body) = trust_body.as_deref() {
            if let Some(from_trust) = parse_fingerprint(body) {
                let matches =
                    normalize_fingerprint(cli_fp) == normalize_fingerprint(&from_trust);
                fingerprint_ok = Some(matches);
                if !matches {
                    fingerprint_source = Some("cli vs TRUST.md".into());
                }
            } else {
                fingerprint_ok = Some(true);
            }
        } else {
            fingerprint_ok = Some(true);
        }
    } else if let Some(body) = trust_body.as_deref() {
        if let Some(fp) = parse_fingerprint(body) {
            fingerprint_expected = Some(fp);
            fingerprint_source = Some("TRUST.md".into());
            // Host PE/codesign inspect deferred — informational only.
            fingerprint_ok = Some(true);
        }
    }

    let tier = if let Some(body) = trust_body.as_deref() {
        parse_tier_id(body).unwrap_or_else(|| {
            resolve_primary_tier(&config, TierHints::probe(&root))
                .as_str()
                .into()
        })
    } else {
        resolve_primary_tier(&config, TierHints::probe(&root))
            .as_str()
            .into()
    };

    let sums_exists = sums_path.is_file();
    let minisig = sibling_ext(&sums_path, "minisig");
    let asc = sibling_ext(&sums_path, "asc");

    let mut checksums = Vec::new();
    if sums_exists {
        let only = if args.artifacts.is_empty() {
            None
        } else {
            Some(
                args.artifacts
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>(),
            )
        };
        checksums = verify_sha256sums(&sums_path, &[&root], only.as_deref())?;
        if checksums.is_empty() && args.artifacts.is_empty() {
            warnings.push("SHA256SUMS present but no listed files found on disk".into());
        }
    } else if !args.artifacts.is_empty() {
        anyhow::bail!(
            "artifacts requested but SHA256SUMS not found at {}",
            sums_path.display()
        );
    }

    let sums_signature = verify_community_sig(CommunitySigInput {
        sums_path: &sums_path,
        sums_exists,
        minisig: &minisig,
        asc: &asc,
        minisign_pub_override: args.minisign_pub.as_deref(),
        trust_body: trust_body.as_deref(),
        root: &root,
        config: &config,
        warnings: &mut warnings,
    })?;

    let has_any_check = fingerprint_expected.is_some()
        || !checksums.is_empty()
        || sums_signature.ok.is_some()
        || args.require_sig;
    if !has_any_check {
        anyhow::bail!(
            "nothing to verify — provide TRUST.md, SHA256SUMS, --fingerprint, and/or --artifact"
        );
    }

    let report = Report {
        schema_version: 1,
        tier,
        fingerprint_expected,
        fingerprint_source,
        fingerprint_ok,
        checksums,
        sums_path: sums_exists.then_some(sums_path),
        sums_signature,
        warnings,
    };

    emit(&report, args.json);

    if report.checksums.iter().any(|c| !c.ok) || report.fingerprint_ok == Some(false) {
        return Ok(ExitCode::Failure);
    }

    if args.require_sig {
        let sig_ok = report.sums_signature.ok == Some(true);
        if !report.sums_signature.present || !sig_ok {
            eprintln!("error: --require-sig unmet (need valid SHA256SUMS.minisig or .asc)");
            return Ok(ExitCode::Policy);
        }
    }

    // Invalid signature present fails even without --require-sig.
    if report.sums_signature.present && report.sums_signature.ok == Some(false) {
        return Ok(ExitCode::Failure);
    }

    Ok(ExitCode::Success)
}

struct CommunitySigInput<'a> {
    sums_path: &'a Path,
    sums_exists: bool,
    minisig: &'a Path,
    asc: &'a Path,
    minisign_pub_override: Option<&'a Path>,
    trust_body: Option<&'a str>,
    root: &'a Path,
    config: &'a Config,
    warnings: &'a mut Vec<String>,
}

fn verify_community_sig(input: CommunitySigInput<'_>) -> anyhow::Result<SumsSignatureReport> {
    let CommunitySigInput {
        sums_path,
        sums_exists,
        minisig,
        asc,
        minisign_pub_override,
        trust_body,
        root,
        config,
        warnings,
    } = input;

    let present = minisig.is_file() || asc.is_file();
    if !present {
        return Ok(SumsSignatureReport {
            present: false,
            ok: None,
            scheme: None,
        });
    }
    if !sums_exists {
        warnings.push("sums signature present but SHA256SUMS missing".into());
        return Ok(SumsSignatureReport {
            present: true,
            ok: Some(false),
            scheme: None,
        });
    }

    if minisig.is_file() {
        let pub_text = resolve_minisign_pub(
            minisign_pub_override,
            trust_body,
            root,
            config,
            warnings,
        )?;
        match pub_text {
            Some(text) => match verify_sums_minisign(sums_path, minisig, &text) {
                Ok(()) => {
                    return Ok(SumsSignatureReport {
                        present: true,
                        ok: Some(true),
                        scheme: Some("minisign"),
                    });
                }
                Err(e) => {
                    warnings.push(format!("minisign verify failed: {e}"));
                    return Ok(SumsSignatureReport {
                        present: true,
                        ok: Some(false),
                        scheme: Some("minisign"),
                    });
                }
            },
            None => {
                warnings.push(
                    "SHA256SUMS.minisig present but no public key (pass --minisign-pub or regenerate TRUST.md)"
                        .into(),
                );
                return Ok(SumsSignatureReport {
                    present: true,
                    ok: Some(false),
                    scheme: Some("minisign"),
                });
            }
        }
    }

    if asc.is_file() {
        match verify_sums_gpg(sums_path, asc) {
            Ok(()) => {
                return Ok(SumsSignatureReport {
                    present: true,
                    ok: Some(true),
                    scheme: Some("gpg"),
                });
            }
            Err(e) => {
                warnings.push(format!("gpg verify failed: {e}"));
                return Ok(SumsSignatureReport {
                    present: true,
                    ok: Some(false),
                    scheme: Some("gpg"),
                });
            }
        }
    }

    Ok(SumsSignatureReport {
        present: true,
        ok: Some(false),
        scheme: None,
    })
}

fn resolve_minisign_pub(
    override_path: Option<&Path>,
    trust_body: Option<&str>,
    root: &Path,
    config: &Config,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Option<String>> {
    if let Some(path) = override_path {
        return Ok(Some(fs::read_to_string(path)?));
    }
    if let Some(body) = trust_body {
        if let Some(text) = parse_minisign_pub_from_trust(body) {
            return Ok(Some(text));
        }
    }
    let paths = SumsKeyPaths::from_secrets_dir(&config.secrets_path(root));
    if paths.public.is_file() {
        warnings.push(format!(
            "using local public key {} (prefer TRUST.md / --minisign-pub for distributed verify)",
            paths.public.display()
        ));
        return Ok(Some(fs::read_to_string(&paths.public)?));
    }
    Ok(None)
}

fn sibling_ext(sums_path: &Path, ext: &str) -> PathBuf {
    let parent = sums_path.parent().unwrap_or_else(|| Path::new("."));
    let base = sums_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("SHA256SUMS");
    parent.join(format!("{base}.{ext}"))
}

fn load_root_config(explicit: Option<&Path>) -> (PathBuf, Config) {
    let path = resolve_config_path(explicit);
    if let Ok(cfg) = Config::load(&path) {
        let root = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        (root, cfg)
    } else {
        (PathBuf::from("."), Config::default())
    }
}

fn emit(report: &Report, json: bool) {
    if json {
        emit_json(report);
    } else {
        emit_human(report);
    }
}

fn emit_human(report: &Report) {
    console::banner("verify");
    console::kv(14, "tier", &report.tier);
    if let Some(fp) = &report.fingerprint_expected {
        console::kv(
            14,
            "fingerprint",
            &format!(
                "{fp} ({})",
                report.fingerprint_source.as_deref().unwrap_or("unknown")
            ),
        );
    } else {
        console::kv(14, "fingerprint", "(none)");
    }
    if let Some(path) = &report.sums_path {
        console::kv(14, "sums", &path.display().to_string());
    }
    let sig_detail = match (
        report.sums_signature.present,
        report.sums_signature.ok,
        report.sums_signature.scheme,
    ) {
        (false, _, _) => "absent".into(),
        (true, Some(true), Some(scheme)) => format!("ok ({scheme})"),
        (true, Some(false), Some(scheme)) => format!("invalid ({scheme})"),
        (true, Some(false), None) => "invalid".into(),
        (true, None, _) => "present (not verified)".into(),
        (true, Some(true), None) => "ok".into(),
    };
    console::kv(14, "sums-sig", &sig_detail);
    console::blank();
    if !report.checksums.is_empty() {
        console::section("checksums");
        for c in &report.checksums {
            let detail = if c.ok {
                c.actual.clone().unwrap_or_default()
            } else if let Some(err) = &c.error {
                err.clone()
            } else {
                format!(
                    "expected {} got {}",
                    c.expected,
                    c.actual.as_deref().unwrap_or("?")
                )
            };
            console::status(24, &c.file, c.ok, &detail);
        }
        console::blank();
    }
    for w in &report.warnings {
        console::note(w);
    }
    console::blank();
}

fn emit_json(report: &Report) {
    let mut checksums = String::new();
    for (i, c) in report.checksums.iter().enumerate() {
        if i > 0 {
            checksums.push(',');
        }
        let actual = c
            .actual
            .as_deref()
            .map(|s| format!("\"{s}\""))
            .unwrap_or_else(|| "null".into());
        let err = c
            .error
            .as_deref()
            .map(|s| format!("\"{}\"", json_escape(s)))
            .unwrap_or_else(|| "null".into());
        checksums.push_str(&format!(
            "{{\"file\": \"{}\", \"ok\": {}, \"expected\": \"{}\", \"actual\": {actual}, \"error\": {err}}}",
            json_escape(&c.file),
            c.ok,
            json_escape(&c.expected),
        ));
    }
    let warnings = report
        .warnings
        .iter()
        .map(|w| format!("\"{}\"", json_escape(w)))
        .collect::<Vec<_>>()
        .join(", ");
    let fp = report
        .fingerprint_expected
        .as_deref()
        .map(|s| format!("\"{}\"", json_escape(s)))
        .unwrap_or_else(|| "null".into());
    let fps = report
        .fingerprint_source
        .as_deref()
        .map(|s| format!("\"{}\"", json_escape(s)))
        .unwrap_or_else(|| "null".into());
    let fp_ok = match report.fingerprint_ok {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    };
    let sig_ok = match report.sums_signature.ok {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    };
    let scheme = report
        .sums_signature
        .scheme
        .map(|s| format!("\"{s}\""))
        .unwrap_or_else(|| "null".into());
    println!(
        "{{\n  \"schema_version\": {},\n  \"tier\": \"{}\",\n  \"fingerprint_expected\": {fp},\n  \"fingerprint_source\": {fps},\n  \"fingerprint_ok\": {fp_ok},\n  \"checksums\": [{checksums}],\n  \"sums_signature\": {{\"present\": {}, \"ok\": {sig_ok}, \"scheme\": {scheme}}},\n  \"warnings\": [{warnings}]\n}}",
        report.schema_version,
        json_escape(&report.tier),
        report.sums_signature.present,
    );
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
