//! Apple notarization + staple helpers (`notarytool` / `stapler`).

use std::path::Path;
use std::process::Command;

use super::honesty_notes;

#[derive(Debug, Clone)]
pub struct NotarizeOptions {
    pub keychain_profile: String,
    pub staple: bool,
}

impl Default for NotarizeOptions {
    fn default() -> Self {
        Self {
            keychain_profile: String::new(),
            staple: true,
        }
    }
}

pub fn build_notarize_argv(opts: &NotarizeOptions, path: &Path) -> anyhow::Result<Vec<String>> {
    if opts.keychain_profile.trim().is_empty() {
        anyhow::bail!(
            "Apple Keychain profile required — pass --profile, SIGNET_NOTARY_PROFILE, or \
             [graduation.apple].keychain_profile (create with: xcrun notarytool store-credentials)"
        );
    }
    Ok(vec![
        "notarytool".into(),
        "submit".into(),
        path.display().to_string(),
        "--keychain-profile".into(),
        opts.keychain_profile.clone(),
        "--wait".into(),
    ])
}

pub fn build_staple_argv(path: &Path) -> Vec<String> {
    vec![
        "stapler".into(),
        "staple".into(),
        path.display().to_string(),
    ]
}

fn require_macos() -> anyhow::Result<()> {
    if cfg!(target_os = "macos") {
        Ok(())
    } else {
        anyhow::bail!("Apple notarize/staple helpers require macOS (xcrun notarytool / stapler)")
    }
}

pub fn notarize(path: &Path, opts: &NotarizeOptions) -> anyhow::Result<()> {
    require_macos()?;
    if !path.exists() {
        anyhow::bail!("path not found: {}", path.display());
    }
    let args = build_notarize_argv(opts, path)?;
    let status = Command::new("xcrun").args(&args).status()?;
    if !status.success() {
        anyhow::bail!(
            "notarytool submit failed ({status}). Use a Developer ID–signed build — Signet \
             self-signed / ad-hoc identity is not sufficient for notarization. See docs/graduation.md"
        );
    }
    if opts.staple {
        staple(path)?;
    }
    eprintln!("note: {}", honesty_notes());
    eprintln!(
        "hint: after a successful notarization of a Developer ID build, set \
         trust.declared_tier = \"apple_notarized\" in signet.toml"
    );
    Ok(())
}

pub fn staple(path: &Path) -> anyhow::Result<()> {
    require_macos()?;
    if !path.exists() {
        anyhow::bail!("path not found: {}", path.display());
    }
    let args = build_staple_argv(path);
    let status = Command::new("xcrun").args(&args).status()?;
    if !status.success() {
        anyhow::bail!("stapler staple failed for {}: {status}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notarize_argv() {
        let opts = NotarizeOptions {
            keychain_profile: "SignetNotary".into(),
            staple: true,
        };
        let args = build_notarize_argv(&opts, Path::new("App.app")).unwrap();
        assert_eq!(args[0], "notarytool");
        assert!(args.contains(&"--keychain-profile".into()));
        assert!(args.contains(&"SignetNotary".into()));
        assert!(args.contains(&"--wait".into()));
    }

    #[test]
    fn notarize_rejects_empty_profile() {
        let err = build_notarize_argv(&NotarizeOptions::default(), Path::new("App.app")).unwrap_err();
        assert!(err.to_string().contains("profile"));
    }

    #[test]
    fn staple_argv() {
        let args = build_staple_argv(Path::new("App.dmg"));
        assert_eq!(args, vec!["stapler", "staple", "App.dmg"]);
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn notarize_errors_off_macos() {
        let err = notarize(
            Path::new("."),
            &NotarizeOptions {
                keychain_profile: "x".into(),
                staple: false,
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("macOS"));
    }
}
