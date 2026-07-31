use std::fs;
use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::identity::load_active;
use crate::project::ProjectCtx;
use crate::trust_kit::render_trust_md;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Output path for TRUST.md (default: <project>/TRUST.md)
    #[arg(long)]
    pub out: Option<PathBuf>,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your Tauri app directory first")
    })?;
    let identity = load_active(&ctx.identity_root()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet identity create` first")
    })?;

    let out = args
        .out
        .unwrap_or_else(|| ctx.root.join("TRUST.md"));

    let md = render_trust_md(&ctx.config, &identity);
    if md.contains("BEGIN PRIVATE KEY") || md.contains(&identity.key_pem) {
        anyhow::bail!("internal error: trust kit attempted to embed private key material");
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, md)?;
    println!("wrote {}", out.display());
    println!("fingerprint: {}", identity.meta.fingerprint_sha256);
    println!("safe to commit TRUST.md — never commit .signet/");
    Ok(())
}
