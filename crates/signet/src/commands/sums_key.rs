//! `signet sums-key` — manage minisign keys for SHA256SUMS attestation.

use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand};

use crate::project::ProjectCtx;
use crate::sign::{create_sums_key, read_public_key_text, SumsKeyPaths};

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
    /// Create a minisign keypair under `.signet/sums/`
    Create(CreateArgs),
    /// Print the public key and paths
    Show,
}

#[derive(Debug, ClapArgs)]
pub struct CreateArgs {
    /// Overwrite an existing sums key
    #[arg(long)]
    pub force: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your project directory first")
    })?;

    match args.action {
        Action::Create(c) => {
            let paths = create_sums_key(&ctx.secrets_dir(), c.force)?;
            println!("created minisign keypair under {}", paths.dir.display());
            println!("  public: {}", paths.public.display());
            println!("  secret: {} (gitignored — never commit)", paths.secret.display());
            if std::env::var("SIGNET_MINISIGN_PASSWORD")
                .map(|p| !p.is_empty())
                .unwrap_or(false)
            {
                println!("  secret key encrypted with SIGNET_MINISIGN_PASSWORD");
            } else {
                println!(
                    "  secret key unencrypted (optional: set SIGNET_MINISIGN_PASSWORD before create)"
                );
            }
            println!("next: `signet build` / `signet release` will write SHA256SUMS.minisig");
            println!("      regenerate TRUST.md with `signet trust` to publish the public key");
        }
        Action::Show => {
            let paths = SumsKeyPaths::from_secrets_dir(&ctx.secrets_dir());
            if !paths.exists() {
                anyhow::bail!(
                    "no sums key at {} — run `signet sums-key create`",
                    paths.dir.display()
                );
            }
            let pub_text = read_public_key_text(&paths.public)?;
            println!("public key ({})", paths.public.display());
            println!("{pub_text}");
            if !pub_text.ends_with('\n') {
                println!();
            }
            println!("secret: {}", paths.secret.display());
        }
    }
    Ok(())
}
