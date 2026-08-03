use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use which::which;

use super::collect::ReleaseFile;

#[derive(Debug, Clone)]
pub struct GitHubPublishOpts {
    pub repo: String, // owner/name
    pub tag: String,
    pub name: String,
    pub notes: String,
    pub draft: bool,
    pub prerelease: bool,
    /// Replace assets on an existing release with the same tag
    pub clobber: bool,
}

#[derive(Debug, Clone)]
pub struct PublishResult {
    pub url: String,
    pub method: String,
}

pub fn detect_github_repo(explicit: Option<&str>, config_repo: &str, cwd: &Path) -> anyhow::Result<String> {
    if let Some(repo) = explicit {
        let repo = repo.trim();
        if !repo.is_empty() {
            return normalize_repo(repo);
        }
    }
    if !config_repo.trim().is_empty() {
        return normalize_repo(config_repo);
    }
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(cwd)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
            parse_github_remote(&url)
        }
        _ => anyhow::bail!(
            "cannot detect GitHub repo — set `[release] repo = \"owner/name\"` in signet.toml, \
             pass --repo owner/name, or run inside a git checkout with origin pointing at GitHub"
        ),
    }
}

fn normalize_repo(repo: &str) -> anyhow::Result<String> {
    if let Ok(parsed) = parse_github_remote(repo) {
        return Ok(parsed);
    }
    let parts: Vec<_> = repo.split('/').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        return Ok(format!("{}/{}", parts[0], parts[1].trim_end_matches(".git")));
    }
    anyhow::bail!("invalid GitHub repo '{repo}' (expected owner/name)");
}

pub fn parse_github_remote(url: &str) -> anyhow::Result<String> {
    let url = url.trim();
    // git@github.com:owner/repo.git
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let rest = rest.trim_end_matches(".git");
        return Ok(rest.to_string());
    }
    // https://github.com/owner/repo.git
    for prefix in [
        "https://github.com/",
        "http://github.com/",
        "ssh://git@github.com/",
        "git://github.com/",
    ] {
        if let Some(rest) = url.strip_prefix(prefix) {
            let rest = rest.trim_end_matches('/').trim_end_matches(".git");
            let parts: Vec<_> = rest.split('/').collect();
            if parts.len() >= 2 {
                return Ok(format!("{}/{}", parts[0], parts[1]));
            }
        }
    }
    // already owner/repo
    let parts: Vec<_> = url.split('/').collect();
    if parts.len() == 2 && !url.contains(':') {
        return Ok(format!("{}/{}", parts[0], parts[1].trim_end_matches(".git")));
    }
    anyhow::bail!("not a GitHub remote URL: {url}");
}

pub fn publish_github_release(
    opts: &GitHubPublishOpts,
    files: &[ReleaseFile],
) -> anyhow::Result<PublishResult> {
    if which("gh").is_ok() {
        match publish_with_gh(opts, files) {
            Ok(r) => return Ok(r),
            Err(err) => {
                eprintln!("warning: gh release failed ({err}); trying GitHub HTTP API");
            }
        }
    }
    publish_with_api(opts, files)
}

fn publish_with_gh(opts: &GitHubPublishOpts, files: &[ReleaseFile]) -> anyhow::Result<PublishResult> {
    use std::fs;
    use tempfile::tempdir;

    let mut create = Command::new("gh");
    create
        .args(["release", "create", &opts.tag])
        .args(["--repo", &opts.repo])
        .args(["--title", &opts.name])
        .args(["--notes", &opts.notes]);
    if opts.draft {
        create.arg("--draft");
    }
    if opts.prerelease {
        create.arg("--prerelease");
    }
    let create_status = create.status()?;
    if !create_status.success() {
        eprintln!(
            "note: `gh release create` exited {create_status} (release may already exist); uploading assets"
        );
    }

    let staging = tempdir()?;
    for file in files {
        let upload_path = if file.path.file_name().and_then(|s| s.to_str()) == Some(file.asset_name.as_str())
        {
            file.path.clone()
        } else {
            let staged = staging.path().join(&file.asset_name);
            fs::copy(&file.path, &staged)?;
            staged
        };

        let mut upload = Command::new("gh");
        upload
            .args(["release", "upload", &opts.tag])
            .args(["--repo", &opts.repo])
            .arg(&upload_path);
        if opts.clobber {
            upload.arg("--clobber");
        }
        let status = upload.status()?;
        if !status.success() {
            anyhow::bail!(
                "gh release upload failed for {} with status {status}",
                file.asset_name
            );
        }
        println!("uploaded {}", file.asset_name);
    }

    let url = format!("https://github.com/{}/releases/tag/{}", opts.repo, opts.tag);
    Ok(PublishResult {
        url,
        method: "gh".into(),
    })
}

fn github_token() -> anyhow::Result<String> {
    for key in ["GH_TOKEN", "GITHUB_TOKEN"] {
        if let Ok(v) = std::env::var(key) {
            if !v.trim().is_empty() {
                return Ok(v);
            }
        }
    }
    anyhow::bail!(
        "{}",
        crate::release::assess_github_auth().preflight_error()
    )
}

#[derive(Debug, Deserialize)]
struct ApiRelease {
    id: u64,
    html_url: String,
    #[serde(default)]
    #[allow(dead_code)]
    upload_url: String,
}

fn publish_with_api(opts: &GitHubPublishOpts, files: &[ReleaseFile]) -> anyhow::Result<PublishResult> {
    let token = github_token()?;
    let agent = ureq::AgentBuilder::new()
        .user_agent("signet-cli")
        .build();

    let release = match get_release_by_tag(&agent, &token, &opts.repo, &opts.tag) {
        Ok(existing) => {
            if opts.clobber {
                // delete existing assets with matching names before upload
                delete_conflicting_assets(&agent, &token, &opts.repo, existing.id, files)?;
            }
            // update notes/title lightly
            existing
        }
        Err(_) => create_release(&agent, &token, opts)?,
    };

    for file in files {
        upload_asset(&agent, &token, &opts.repo, release.id, file)?;
    }

    Ok(PublishResult {
        url: release.html_url,
        method: "api".into(),
    })
}

fn create_release(
    agent: &ureq::Agent,
    token: &str,
    opts: &GitHubPublishOpts,
) -> anyhow::Result<ApiRelease> {
    let url = format!("https://api.github.com/repos/{}/releases", opts.repo);
    let body = ureq::json!({
        "tag_name": opts.tag,
        "name": opts.name,
        "body": opts.notes,
        "draft": opts.draft,
        "prerelease": opts.prerelease,
    });
    let resp = agent
        .post(&url)
        .set("Accept", "application/vnd.github+json")
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-GitHub-Api-Version", "2022-11-28")
        .send_json(body);

    match resp {
        Ok(r) => Ok(r.into_json()?),
        Err(ureq::Error::Status(code, r)) => {
            let text = read_body(r);
            anyhow::bail!("GitHub create release failed ({code}): {text}");
        }
        Err(e) => Err(e.into()),
    }
}

fn get_release_by_tag(
    agent: &ureq::Agent,
    token: &str,
    repo: &str,
    tag: &str,
) -> anyhow::Result<ApiRelease> {
    let url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
    let resp = agent
        .get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call();
    match resp {
        Ok(r) => Ok(r.into_json()?),
        Err(ureq::Error::Status(code, r)) => {
            let text = read_body(r);
            anyhow::bail!("get release by tag failed ({code}): {text}");
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug, Deserialize)]
struct ApiAsset {
    id: u64,
    name: String,
}

fn delete_conflicting_assets(
    agent: &ureq::Agent,
    token: &str,
    repo: &str,
    release_id: u64,
    files: &[ReleaseFile],
) -> anyhow::Result<()> {
    let url = format!("https://api.github.com/repos/{repo}/releases/{release_id}/assets");
    let resp = agent
        .get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call();
    let assets: Vec<ApiAsset> = match resp {
        Ok(r) => r.into_json()?,
        Err(ureq::Error::Status(_, r)) => {
            let _ = read_body(r);
            return Ok(());
        }
        Err(_) => return Ok(()),
    };
    let wanted: Vec<&str> = files.iter().map(|f| f.asset_name.as_str()).collect();
    for asset in assets {
        if wanted.contains(&asset.name.as_str()) {
            let del = format!("https://api.github.com/repos/{repo}/releases/assets/{}", asset.id);
            let _ = agent
                .delete(&del)
                .set("Accept", "application/vnd.github+json")
                .set("Authorization", &format!("Bearer {token}"))
                .set("X-GitHub-Api-Version", "2022-11-28")
                .call();
        }
    }
    Ok(())
}

fn upload_asset(
    agent: &ureq::Agent,
    token: &str,
    repo: &str,
    release_id: u64,
    file: &ReleaseFile,
) -> anyhow::Result<()> {
    let bytes = fs::read(&file.path)?;
    let url = format!(
        "https://uploads.github.com/repos/{repo}/releases/{release_id}/assets?name={}",
        urlencoding_rough(&file.asset_name)
    );
    let resp = agent
        .post(&url)
        .set("Accept", "application/vnd.github+json")
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/octet-stream")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .send_bytes(&bytes);

    match resp {
        Ok(_) => {
            println!("uploaded {}", file.asset_name);
            Ok(())
        }
        Err(ureq::Error::Status(code, r)) => {
            let text = read_body(r);
            anyhow::bail!("upload {} failed ({code}): {text}", file.asset_name);
        }
        Err(e) => Err(e.into()),
    }
}

fn urlencoding_rough(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn read_body(resp: ureq::Response) -> String {
    let mut s = String::new();
    let _ = resp.into_reader().read_to_string(&mut s);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ssh_and_https_remotes() {
        assert_eq!(
            parse_github_remote("git@github.com:acme/app.git").unwrap(),
            "acme/app"
        );
        assert_eq!(
            parse_github_remote("https://github.com/acme/app.git").unwrap(),
            "acme/app"
        );
        assert_eq!(
            parse_github_remote("https://github.com/acme/app").unwrap(),
            "acme/app"
        );
    }
}
