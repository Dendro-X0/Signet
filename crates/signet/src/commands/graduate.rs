//! `signet graduate` — OV / Azure / notarize helpers.

use std::env;
use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand};

use crate::config::Config;
use crate::graduate::{
    azure_sign_files, honesty_notes, notarize, ov_sign_files, staple, AzureSignOptions,
    NotarizeOptions, OvCredential, OvSignOptions,
};
use crate::project::ProjectCtx;
use crate::ship::{
    assess_sign_profile, discover_graduate_files, PlatformSignAction, ShipSignPath,
};
use crate::ui::console;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Print graduation ladder honesty notes
    Notes,
    /// Discover host installers and apply configured graduate backend (`[ship] path = "graduate"`)
    Apply,
    /// Sign with an OV/CA Authenticode cert (thumbprint or PFX) — not Signet self-signed identity
    #[command(name = "ov-sign")]
    OvSign(OvSignArgs),
    /// Sign via Azure Trusted Signing (signtool + dlib + metadata)
    #[command(name = "azure-sign")]
    AzureSign(AzureSignArgs),
    /// Submit to Apple notary service (macOS)
    Notarize(NotarizeArgs),
    /// Staple a notarization ticket (macOS)
    Staple(StapleArgs),
}

#[derive(Debug, ClapArgs)]
pub struct OvSignArgs {
    /// File(s) to sign (.exe / .msi / …)
    #[arg(long = "file", required = true)]
    pub files: Vec<PathBuf>,
    /// Certificate SHA-1 thumbprint (hex)
    #[arg(long)]
    pub thumbprint: Option<String>,
    /// PFX path (alternative to thumbprint)
    #[arg(long)]
    pub pfx: Option<PathBuf>,
    /// PFX password (prefer SIGNET_OV_PFX_PASS)
    #[arg(long)]
    pub pfx_pass: Option<String>,
    /// Skip Authenticode timestamp
    #[arg(long)]
    pub no_timestamp: bool,
}

#[derive(Debug, ClapArgs)]
pub struct AzureSignArgs {
    #[arg(long = "file", required = true)]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, ClapArgs)]
pub struct NotarizeArgs {
    /// Path to .app / .dmg / .pkg / zip to submit
    #[arg(long)]
    pub path: PathBuf,
    /// Keychain profile from `xcrun notarytool store-credentials`
    #[arg(long)]
    pub profile: Option<String>,
    /// Skip stapler after successful submit
    #[arg(long)]
    pub no_staple: bool,
}

#[derive(Debug, ClapArgs)]
pub struct StapleArgs {
    #[arg(long)]
    pub path: PathBuf,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).ok();
    let cfg = ctx.as_ref().map(|c| &c.config);

    match args.action {
        Action::Notes => {
            println!("{}", honesty_notes());
            println!("Full guide: docs/graduation.md");
        }
        Action::Apply => {
            let ctx = ctx.ok_or_else(|| {
                anyhow::anyhow!("signet.toml required for `graduate apply` — run from project root")
            })?;
            run_apply(&ctx)?;
        }
        Action::OvSign(a) => {
            let opts = resolve_ov_opts(cfg, &a)?;
            let signed = ov_sign_files(&a.files, &opts)?;
            for p in signed {
                println!("signed {}", p.display());
            }
        }
        Action::AzureSign(a) => {
            let opts = resolve_azure_opts(cfg)?;
            let signed = azure_sign_files(&a.files, &opts)?;
            for p in signed {
                println!("signed {}", p.display());
            }
        }
        Action::Notarize(a) => {
            let opts = resolve_notarize_opts(cfg, &a)?;
            notarize(&a.path, &opts)?;
            println!("notarized {}", a.path.display());
        }
        Action::Staple(a) => {
            staple(&a.path)?;
            println!("stapled {}", a.path.display());
        }
    }
    Ok(())
}

fn run_apply(ctx: &ProjectCtx) -> anyhow::Result<()> {
    console::banner("graduate · apply");
    let profile = assess_sign_profile(&ctx.config);
    console::note(&profile.summary_line());

    if profile.path == ShipSignPath::SelfSigned {
        console::note(
            "[ship] path = \"self\" — nothing to graduate. Set path = \"graduate\" or use ov-sign / azure-sign / notarize directly.",
        );
        return Ok(());
    }

    let files = discover_graduate_files(&ctx.root, &ctx.config);
    match std::env::consts::OS {
        "windows" => apply_windows(ctx, &profile.windows, &files)?,
        "macos" => apply_macos(ctx, &profile.macos, &files)?,
        "linux" => {
            console::note(
                "Linux graduate path is integrity-first (SHA256SUMS / self). No OV/Azure/notarize apply on this host.",
            );
        }
        other => {
            anyhow::bail!("graduate apply unsupported on OS `{other}`");
        }
    }
    Ok(())
}

fn apply_windows(
    ctx: &ProjectCtx,
    action: &PlatformSignAction,
    files: &[PathBuf],
) -> anyhow::Result<()> {
    match action {
        PlatformSignAction::Azure => {
            if files.is_empty() {
                anyhow::bail!(
                    "no Windows installers found to azure-sign — build first or place artifacts under dist/signet-ship/"
                );
            }
            let opts = resolve_azure_opts(Some(&ctx.config))?;
            let signed = azure_sign_files(files, &opts)?;
            for p in signed {
                console::ok_line(&format!("azure-signed {}", p.display()));
            }
        }
        PlatformSignAction::Ov => {
            if files.is_empty() {
                anyhow::bail!(
                    "no Windows installers found to ov-sign — build first or place artifacts under dist/signet-ship/"
                );
            }
            let opts = resolve_ov_opts_for_apply(&ctx.config)?;
            let signed = ov_sign_files(files, &opts)?;
            for p in signed {
                console::ok_line(&format!("ov-signed {}", p.display()));
            }
        }
        PlatformSignAction::GraduateMissing(msg) => {
            anyhow::bail!("Windows graduate not configured: {msg}");
        }
        other => {
            anyhow::bail!("unexpected Windows graduate action: {}", other.label());
        }
    }
    Ok(())
}

fn apply_macos(
    ctx: &ProjectCtx,
    action: &PlatformSignAction,
    files: &[PathBuf],
) -> anyhow::Result<()> {
    match action {
        PlatformSignAction::Notarize => {
            if files.is_empty() {
                anyhow::bail!(
                    "no macOS .app/.dmg found to notarize — build first or place artifacts under dist/signet-ship/"
                );
            }
            let opts = resolve_notarize_opts(
                Some(&ctx.config),
                &NotarizeArgs {
                    path: files[0].clone(),
                    profile: None,
                    no_staple: false,
                },
            )?;
            for path in files {
                notarize(path, &opts)?;
                console::ok_line(&format!("notarized {}", path.display()));
            }
        }
        PlatformSignAction::GraduateMissing(msg) => {
            anyhow::bail!("macOS graduate not configured: {msg}");
        }
        other => {
            anyhow::bail!("unexpected macOS graduate action: {}", other.label());
        }
    }
    Ok(())
}

fn resolve_ov_opts_for_apply(cfg: &Config) -> anyhow::Result<OvSignOptions> {
    resolve_ov_opts(
        Some(cfg),
        &OvSignArgs {
            files: Vec::new(),
            thumbprint: None,
            pfx: None,
            pfx_pass: None,
            no_timestamp: false,
        },
    )
}

fn resolve_ov_opts(cfg: Option<&Config>, a: &OvSignArgs) -> anyhow::Result<OvSignOptions> {
    let g = cfg.map(|c| &c.graduation);
    let timestamp_url = g
        .map(|g| g.timestamp_url.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://timestamp.digicert.com".into());

    let pfx = a
        .pfx
        .clone()
        .or_else(|| env::var_os("SIGNET_OV_PFX").map(PathBuf::from));
    let credential = if let Some(pfx) = pfx {
        let password = a
            .pfx_pass
            .clone()
            .or_else(|| env::var("SIGNET_OV_PFX_PASS").ok())
            .unwrap_or_default();
        OvCredential::Pfx { path: pfx, password }
    } else {
        let tp = a
            .thumbprint
            .clone()
            .or_else(|| env::var("SIGNET_OV_THUMBPRINT").ok())
            .or_else(|| {
                g.and_then(|g| {
                    if g.ov_thumbprint.is_empty() {
                        None
                    } else {
                        Some(g.ov_thumbprint.clone())
                    }
                })
            })
            .unwrap_or_default();
        OvCredential::Thumbprint(tp)
    };

    Ok(OvSignOptions {
        credential,
        timestamp_url,
        timestamp: !a.no_timestamp,
    })
}

fn resolve_azure_opts(cfg: Option<&Config>) -> anyhow::Result<AzureSignOptions> {
    let g = cfg.map(|c| &c.graduation);
    let azure = g.map(|g| &g.azure);
    let dlib = env::var_os("SIGNET_AZURE_DLIB")
        .map(PathBuf::from)
        .or_else(|| {
            azure.and_then(|a| {
                if a.dlib.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&a.dlib))
                }
            })
        })
        .unwrap_or_default();
    let metadata = env::var_os("SIGNET_AZURE_METADATA")
        .map(PathBuf::from)
        .or_else(|| {
            azure.and_then(|a| {
                if a.metadata.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&a.metadata))
                }
            })
        })
        .unwrap_or_default();
    let timestamp_url = azure
        .map(|a| a.timestamp_url.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://timestamp.acs.microsoft.com".into());
    Ok(AzureSignOptions {
        dlib,
        metadata,
        timestamp_url,
    })
}

fn resolve_notarize_opts(cfg: Option<&Config>, a: &NotarizeArgs) -> anyhow::Result<NotarizeOptions> {
    let profile = a
        .profile
        .clone()
        .or_else(|| env::var("SIGNET_NOTARY_PROFILE").ok())
        .or_else(|| {
            cfg.and_then(|c| {
                let p = &c.graduation.apple.keychain_profile;
                if p.is_empty() {
                    None
                } else {
                    Some(p.clone())
                }
            })
        })
        .unwrap_or_default();
    Ok(NotarizeOptions {
        keychain_profile: profile,
        staple: !a.no_staple,
    })
}
