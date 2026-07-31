use serde::Deserialize;

use super::paths::host_asset_name;

const REPO: &str = "Dendro-X0/Signet";
const UA: &str = concat!("signet/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub struct AssetPick {
    pub tag: String,
    pub version: String,
    pub asset_name: String,
    pub download_url: String,
    pub sums_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

pub fn fetch_latest_release() -> anyhow::Result<AssetPick> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body: GhRelease = ureq::get(&url)
        .set("User-Agent", UA)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| anyhow::anyhow!("GitHub releases API: {e}"))?
        .into_json()
        .map_err(|e| anyhow::anyhow!("parse GitHub release JSON: {e}"))?;

    let asset_name = host_asset_name().to_string();
    if asset_name == "signet-unknown" {
        anyhow::bail!(
            "unsupported host for self-update ({}-{})",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    }

    let asset = body
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "release {} has no asset `{asset_name}` — publish CLI binaries first (see release-cli workflow)",
                body.tag_name
            )
        })?;

    let sums_url = body
        .assets
        .iter()
        .find(|a| a.name == "SHA256SUMS")
        .map(|a| a.browser_download_url.clone());

    let version = body
        .tag_name
        .trim_start_matches('v')
        .to_string();

    Ok(AssetPick {
        tag: body.tag_name,
        version,
        asset_name,
        download_url: asset.browser_download_url.clone(),
        sums_url,
    })
}

pub fn download_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let mut reader = ureq::get(url)
        .set("User-Agent", UA)
        .call()
        .map_err(|e| anyhow::anyhow!("download {url}: {e}"))?
        .into_reader();
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut buf)?;
    Ok(buf)
}

pub fn expected_sha256_from_sums(sums_text: &str, asset_name: &str) -> Option<String> {
    for line in sums_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (hash, name) = line.split_once("  ").or_else(|| line.split_once(" *"))?;
        if name.trim().trim_start_matches('*') == asset_name {
            return Some(hash.trim().to_ascii_lowercase());
        }
    }
    None
}
