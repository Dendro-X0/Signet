//! `signet android` — keystore + APK signing helpers.

use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand};

use crate::android::{
    create_keystore, import_keystore, keystore_paths, load_meta, read_cert_sha256, sign_apks,
    store_pass,
};
use crate::project::ProjectCtx;

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
    /// Manage the Android release keystore under `.signet/android/`
    Keystore(KeystoreArgs),
    /// Sign one or more APKs with the project keystore
    Sign(SignArgs),
}

#[derive(Debug, ClapArgs)]
pub struct KeystoreArgs {
    #[command(subcommand)]
    pub action: KeystoreAction,
}

#[derive(Debug, Subcommand)]
pub enum KeystoreAction {
    /// Create a new JKS keystore (requires SIGNET_ANDROID_STORE_PASS)
    Create(CreateArgs),
    /// Import an existing keystore file
    Import(ImportArgs),
    /// Show alias and certificate digest
    Show,
}

#[derive(Debug, ClapArgs)]
pub struct CreateArgs {
    #[arg(long, default_value = "signet")]
    pub alias: String,
    /// Certificate DN for keytool
    #[arg(long, default_value = "CN=Signet Android,O=Signet,C=US")]
    pub dname: String,
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, ClapArgs)]
pub struct ImportArgs {
    #[arg(long)]
    pub keystore: PathBuf,
    #[arg(long, default_value = "signet")]
    pub alias: String,
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, ClapArgs)]
pub struct SignArgs {
    /// APK path (repeatable)
    #[arg(long = "apk", required = true)]
    pub apks: Vec<PathBuf>,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your project directory first")
    })?;

    match args.action {
        Action::Keystore(k) => match k.action {
            KeystoreAction::Create(c) => {
                let paths = create_keystore(&ctx.secrets_dir(), &c.alias, &c.dname, c.force)?;
                println!("created android keystore under {}", paths.dir.display());
                println!("  keystore: {}", paths.keystore.display());
                if let Ok(meta) = load_meta(&paths) {
                    if let Some(fp) = meta.cert_sha256 {
                        println!("  cert SHA-256: {fp}");
                    }
                }
                println!("note: this is a local/sideload (or Play *upload*) key — not the Play App Signing key.");
                println!("see docs/android.md");
            }
            KeystoreAction::Import(i) => {
                let paths =
                    import_keystore(&ctx.secrets_dir(), &i.keystore, &i.alias, i.force)?;
                println!("imported keystore → {}", paths.keystore.display());
                println!("alias: {}", i.alias);
                println!("see docs/android.md for Play App Signing honesty");
            }
            KeystoreAction::Show => {
                let paths = keystore_paths(&ctx.secrets_dir());
                if !paths.exists() {
                    anyhow::bail!(
                        "no android keystore at {} — run `signet android keystore create`",
                        paths.dir.display()
                    );
                }
                let meta = load_meta(&paths)?;
                println!("keystore: {}", paths.keystore.display());
                println!("alias: {}", meta.alias);
                println!("store_type: {}", meta.store_type);
                println!("created_at: {}", meta.created_at);
                let fp = meta.cert_sha256.clone().or_else(|| {
                    let pass = store_pass().ok()?;
                    read_cert_sha256(&paths.keystore, &meta.alias, &pass).ok()
                });
                match fp {
                    Some(fp) => println!("cert SHA-256: {fp}"),
                    None => println!(
                        "cert SHA-256: (set SIGNET_ANDROID_STORE_PASS to refresh from keytool)"
                    ),
                }
                println!("note: {}", meta.note);
            }
        },
        Action::Sign(s) => {
            let paths = keystore_paths(&ctx.secrets_dir());
            let report = sign_apks(&paths, &s.apks)?;
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
                anyhow::bail!("no APKs signed");
            }
        }
    }
    Ok(())
}
