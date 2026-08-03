use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand};

use crate::identity::{
    create_identity, import_identity, list_identities, load_active, load_named, read_active,
    set_active, CreateOptions, ImportOptions,
};
use crate::project::ProjectCtx;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub action: Option<Action>,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Create a new local signing identity (ECDSA P-256 self-signed cert)
    Create(CreateArgs),
    /// Import an existing PEM certificate + private key
    Import(ImportArgs),
    /// List identities for this project
    List,
    /// Show fingerprint for the active (or named) identity
    #[command(visible_alias = "status")]
    Show(ShowArgs),
    /// Mark an identity as active
    Use(UseArgs),
}

#[derive(Debug, ClapArgs)]
pub struct CreateArgs {
    /// Identity directory name under .signet/identity/
    #[arg(long, default_value = "default")]
    pub name: String,

    /// Certificate common name (CN)
    #[arg(long)]
    pub cn: Option<String>,

    /// Organization (O)
    #[arg(long, default_value = "")]
    pub org: String,

    /// Validity period in days
    #[arg(long, default_value_t = 825)]
    pub days: u32,

    /// Overwrite an existing identity with the same name
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, ClapArgs)]
pub struct ImportArgs {
    /// Identity directory name under .signet/identity/
    #[arg(long, default_value = "imported")]
    pub name: String,

    /// Path to PEM certificate
    #[arg(long)]
    pub cert: PathBuf,

    /// Path to PEM private key
    #[arg(long)]
    pub key: PathBuf,

    /// Overwrite an existing identity with the same name
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, ClapArgs)]
pub struct ShowArgs {
    /// Show a named identity instead of the active one
    #[arg(long)]
    pub name: Option<String>,
}

#[derive(Debug, ClapArgs)]
pub struct UseArgs {
    /// Identity name to activate
    pub name: String,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your Tauri app directory first")
    })?;
    let identity_root = ctx.identity_root();

    match args.action.unwrap_or(Action::Show(ShowArgs { name: None })) {
        Action::Create(a) => {
            let cn = a
                .cn
                .unwrap_or_else(|| format!("{} Code Signing", ctx.config.project.name));
            let rec = create_identity(
                &identity_root,
                &CreateOptions {
                    name: a.name,
                    common_name: cn,
                    organization: a.org,
                    days: a.days,
                    force: a.force,
                },
            )?;
            println!("created identity '{}'", rec.meta.name);
            println!("  path:        {}", rec.dir.display());
            println!("  CN:          {}", rec.meta.common_name);
            println!("  fingerprint: {}", rec.meta.fingerprint_sha256);
            println!("  valid until: {}", rec.meta.not_after);
            println!("next: signet trust");
            Ok(())
        }
        Action::Import(a) => {
            let rec = import_identity(
                &identity_root,
                &ImportOptions {
                    name: a.name,
                    cert_path: a.cert,
                    key_path: a.key,
                    force: a.force,
                },
            )?;
            println!("imported identity '{}'", rec.meta.name);
            println!("  fingerprint: {}", rec.meta.fingerprint_sha256);
            Ok(())
        }
        Action::List => {
            let active = read_active(&identity_root).ok().map(|a| a.name);
            let list = list_identities(&identity_root)?;
            if list.is_empty() {
                println!("no identities — run `signet identity create`");
                return Ok(());
            }
            for meta in list {
                let mark = if active.as_deref() == Some(meta.name.as_str()) {
                    "*"
                } else {
                    " "
                };
                println!("{mark} {:<16} {}", meta.name, meta.fingerprint_sha256);
            }
            Ok(())
        }
        Action::Show(a) => {
            let rec = match a.name {
                Some(name) => load_named(&identity_root, &name)?,
                None => load_active(&identity_root)?,
            };
            show_record(&rec)
        }
        Action::Use(a) => {
            set_active(&identity_root, &a.name)?;
            println!("active identity set to '{}'", a.name);
            Ok(())
        }
    }
}

fn show_record(rec: &crate::identity::IdentityRecord) -> anyhow::Result<()> {
    println!("name:         {}", rec.meta.name);
    println!("CN:           {}", rec.meta.common_name);
    if !rec.meta.organization.is_empty() {
        println!("org:          {}", rec.meta.organization);
    }
    println!("fingerprint:  {}", rec.meta.fingerprint_sha256);
    println!("not_before:   {}", rec.meta.not_before);
    println!("not_after:    {}", rec.meta.not_after);
    println!("algorithm:    {}", rec.meta.key_algorithm);
    println!("cert:         {}", rec.dir.join("cert.pem").display());
    println!(
        "key:          {} (private — do not commit)",
        rec.dir.join("key.pem").display()
    );
    Ok(())
}
