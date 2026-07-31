use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// One verified (or failed) checksum line.
#[derive(Debug, Clone)]
pub struct ChecksumResult {
    pub file: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub ok: bool,
    pub expected: String,
    pub actual: Option<String>,
    pub error: Option<String>,
}

/// Write a SHA256SUMS file (GNU coreutils compatible: `hash  filename`).
pub fn write_sha256sums(out: &Path, paths: &[PathBuf]) -> anyhow::Result<PathBuf> {
    let named: Vec<(String, PathBuf)> = paths
        .iter()
        .filter(|p| p.is_file())
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("artifact")
                .to_string();
            (name, path.clone())
        })
        .collect();
    write_sha256sums_named(out, &named)
}

/// Like [`write_sha256sums`], but uses explicit names (for release asset filenames).
pub fn write_sha256sums_named(
    out: &Path,
    files: &[(String, PathBuf)],
) -> anyhow::Result<PathBuf> {
    let mut body = String::new();
    for (name, path) in files {
        let bytes = fs::read(path)?;
        let digest = Sha256::digest(&bytes);
        let hex = hex::encode(digest);
        body.push_str(&hex);
        body.push_str("  ");
        body.push_str(name);
        body.push('\n');
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(out)?;
    file.write_all(body.as_bytes())?;
    Ok(out.to_path_buf())
}

/// Hash a file as lowercase hex SHA-256.
pub fn sha256_hex_file(path: &Path) -> anyhow::Result<String> {
    let bytes = fs::read(path)?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

/// Parse GNU `SHA256SUMS` body into `(hex, filename)` pairs.
pub fn parse_sha256sums(text: &str) -> anyhow::Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (hash, name) = if let Some((h, rest)) = line.split_once("  ") {
            (h, rest)
        } else if let Some((h, rest)) = line.split_once(" *") {
            (h, rest)
        } else if let Some((h, rest)) = line.split_once('\t') {
            (h, rest.trim())
        } else {
            anyhow::bail!("SHA256SUMS line {}: expected `hash  filename`", lineno + 1);
        };
        let hash = hash.trim().to_ascii_lowercase();
        if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("SHA256SUMS line {}: invalid sha256 hex", lineno + 1);
        }
        let name = name.trim().trim_start_matches('*').to_string();
        if name.is_empty() {
            anyhow::bail!("SHA256SUMS line {}: empty filename", lineno + 1);
        }
        out.push((hash, name));
    }
    Ok(out)
}

/// Verify checksums. If `only_names` is Some, only those basenames (or paths) are checked;
/// otherwise every listed file that exists under `search_roots` is checked.
///
/// Missing files on disk are skipped (not failures) when verifying the full sums file,
/// unless they were explicitly requested via `only_names`.
pub fn verify_sha256sums(
    sums_path: &Path,
    search_roots: &[&Path],
    only_names: Option<&[String]>,
) -> anyhow::Result<Vec<ChecksumResult>> {
    let text = fs::read_to_string(sums_path)?;
    let entries = parse_sha256sums(&text)?;
    let sums_dir = sums_path.parent().unwrap_or_else(|| Path::new("."));

    let mut results = Vec::new();

    if let Some(wanted) = only_names {
        for want in wanted {
            let want_base = Path::new(want)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(want.as_str());
            let entry = entries.iter().find(|(_, name)| {
                name == want || name == want_base || Path::new(name).file_name().and_then(|s| s.to_str()) == Some(want_base)
            });
            match entry {
                None => results.push(ChecksumResult {
                    file: want.clone(),
                    path: PathBuf::from(want),
                    ok: false,
                    expected: String::new(),
                    actual: None,
                    error: Some("not listed in SHA256SUMS".into()),
                }),
                Some((expected, name)) => {
                    let path = resolve_artifact_path(name, want, sums_dir, search_roots);
                    results.push(check_one(name, path, expected));
                }
            }
        }
    } else {
        for (expected, name) in &entries {
            let path = resolve_artifact_path(name, name, sums_dir, search_roots);
            if !path.is_file() {
                continue;
            }
            results.push(check_one(name, path, expected));
        }
    }

    Ok(results)
}

fn resolve_artifact_path(
    sums_name: &str,
    hint: &str,
    sums_dir: &Path,
    search_roots: &[&Path],
) -> PathBuf {
    let hint_path = Path::new(hint);
    if hint_path.is_file() {
        return hint_path.to_path_buf();
    }
    let candidates = [
        sums_dir.join(sums_name),
        PathBuf::from(sums_name),
        PathBuf::from(hint),
    ];
    for c in &candidates {
        if c.is_file() {
            return c.clone();
        }
    }
    for root in search_roots {
        let p = root.join(sums_name);
        if p.is_file() {
            return p;
        }
        if let Some(base) = Path::new(sums_name).file_name() {
            let p = root.join(base);
            if p.is_file() {
                return p;
            }
        }
    }
    sums_dir.join(sums_name)
}

fn check_one(name: &str, path: PathBuf, expected: &str) -> ChecksumResult {
    if !path.is_file() {
        return ChecksumResult {
            file: name.to_string(),
            path,
            ok: false,
            expected: expected.to_string(),
            actual: None,
            error: Some("file not found".into()),
        };
    }
    match sha256_hex_file(&path) {
        Ok(actual) => ChecksumResult {
            file: name.to_string(),
            path,
            ok: actual == expected,
            expected: expected.to_string(),
            actual: Some(actual),
            error: None,
        },
        Err(e) => ChecksumResult {
            file: name.to_string(),
            path,
            ok: false,
            expected: expected.to_string(),
            actual: None,
            error: Some(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn writes_checksum_line() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.bin");
        fs::write(&file, b"hello").unwrap();
        let out = dir.path().join("SHA256SUMS");
        write_sha256sums(&out, &[file]).unwrap();
        let text = fs::read_to_string(out).unwrap();
        assert!(text.contains("  a.bin"));
        assert_eq!(text.split_whitespace().next().unwrap().len(), 64);
    }

    #[test]
    fn verify_sha256sums_ok_and_tamper() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.bin");
        fs::write(&file, b"hello").unwrap();
        let sums = dir.path().join("SHA256SUMS");
        write_sha256sums(&sums, &[file.clone()]).unwrap();

        let ok = verify_sha256sums(&sums, &[dir.path()], None).unwrap();
        assert_eq!(ok.len(), 1);
        assert!(ok[0].ok);

        fs::write(&file, b"tampered").unwrap();
        let bad = verify_sha256sums(&sums, &[dir.path()], None).unwrap();
        assert_eq!(bad.len(), 1);
        assert!(!bad[0].ok);
    }

    #[test]
    fn parse_rejects_bad_hash() {
        assert!(parse_sha256sums("notahash  file.bin").is_err());
    }
}
