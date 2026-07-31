//! `signet verify` — fingerprint + SHA256SUMS checks (Phase 7).

use std::fs;
use std::path::{Path, PathBuf};

use clap::Args as ClapArgs;

use crate::config::{resolve_config_path, Config};
use crate::error::ExitCode;
use crate::sign::{verify_sha256sums, ChecksumResult};
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

    /// Prefer failing without community sums signature (Phase 8 — soft-warn in 7.0)
    #[arg(long)]
    pub require_sig: bool,

    /// Machine-readable report
    #[arg(long)]
    pub json: bool,

    /// Expected SHA-256 fingerprint (overrides TRUST.md)
    #[arg(long)]
    pub fingerprint: Option<String>,
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
    sums_signature_present: bool,
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
            // Host PE/codesign inspect deferred to Phase 7.1 — informational only.
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
    let sums_signature_present = sibling_sig_exists(&sums_path);

    if args.require_sig {
        warnings.push(
            "--require-sig: community checksum signing lands in Phase 8; soft warning only".into(),
        );
        if !sums_signature_present {
            warnings.push("no SHA256SUMS.minisig / .asc found beside sums".into());
        }
    }

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

    let has_any_check = fingerprint_expected.is_some() || !checksums.is_empty();
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
        sums_signature_present,
        warnings,
    };

    emit(&report, args.json);

    if report.checksums.iter().any(|c| !c.ok) || report.fingerprint_ok == Some(false) {
        return Ok(ExitCode::Failure);
    }
    Ok(ExitCode::Success)
}

fn sibling_sig_exists(sums_path: &Path) -> bool {
    let parent = sums_path.parent().unwrap_or_else(|| Path::new("."));
    let base = sums_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("SHA256SUMS");
    parent.join(format!("{base}.minisig")).is_file()
        || parent.join(format!("{base}.asc")).is_file()
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
    console::kv(
        14,
        "sums-sig",
        if report.sums_signature_present {
            "present (Phase 8 verify not wired)"
        } else {
            "absent"
        },
    );
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
    println!(
        "{{\n  \"schema_version\": {},\n  \"tier\": \"{}\",\n  \"fingerprint_expected\": {fp},\n  \"fingerprint_source\": {fps},\n  \"fingerprint_ok\": {fp_ok},\n  \"checksums\": [{checksums}],\n  \"sums_signature\": {{\"present\": {}, \"ok\": null, \"scheme\": null}},\n  \"warnings\": [{warnings}]\n}}",
        report.schema_version,
        json_escape(&report.tier),
        report.sums_signature_present,
    );
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
