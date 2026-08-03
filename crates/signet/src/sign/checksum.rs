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
/// Filenames are paths relative to the directory containing `out` when possible
/// (so `signet verify` can find monorepo artifacts next to a root `SHA256SUMS`).
pub fn write_sha256sums(out: &Path, paths: &[PathBuf]) -> anyhow::Result<PathBuf> {
    let sums_dir = out.parent().unwrap_or_else(|| Path::new("."));
    let named: Vec<(String, PathBuf)> = paths
        .iter()
        .filter(|p| p.is_file())
        .map(|path| {
            let name = relative_sums_name(path, sums_dir);
            (name, path.clone())
        })
        .collect();
    write_sha256sums_named(out, &named)
}

/// Path written into SHA256SUMS: relative to `sums_dir` with `/` separators, else basename.
pub fn relative_sums_name(path: &Path, sums_dir: &Path) -> String {
    let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let abs_dir = sums_dir
        .canonicalize()
        .unwrap_or_else(|_| sums_dir.to_path_buf());
    if let Ok(rel) = abs_path.strip_prefix(&abs_dir) {
        return normalize_sums_rel(rel);
    }
    // Best-effort without canonicalize (tests / missing parents).
    if let Ok(rel) = path.strip_prefix(sums_dir) {
        return normalize_sums_rel(rel);
    }
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("artifact")
        .to_string()
}

fn normalize_sums_rel(rel: &Path) -> String {
    let s = rel.to_string_lossy().replace('\\', "/");
    if s.is_empty() {
        "artifact".into()
    } else {
        s
    }
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
    // Basename-only leftover SHA256SUMS (e.g. after release collect): walk for the name.
    if let Some(base) = Path::new(sums_name)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|b| !b.is_empty())
    {
        let mut best: Option<(i32, PathBuf)> = None;
        for root in search_roots {
            if let Some((score, path)) = find_basename_under(root, base) {
                match &best {
                    Some((best_score, _)) if *best_score >= score => {}
                    _ => best = Some((score, path)),
                }
            }
        }
        if let Some((_, path)) = best {
            return path;
        }
    }
    sums_dir.join(sums_name)
}

const BASENAME_WALK_MAX_DEPTH: usize = 14;
const BASENAME_WALK_MAX_VISITS: usize = 50_000;

fn find_basename_under(root: &Path, basename: &str) -> Option<(i32, PathBuf)> {
    let mut best: Option<(i32, PathBuf)> = None;
    let mut visits = 0usize;
    fn walk(
        dir: &Path,
        depth: usize,
        basename: &str,
        visits: &mut usize,
        best: &mut Option<(i32, PathBuf)>,
    ) {
        if depth > BASENAME_WALK_MAX_DEPTH || *visits >= BASENAME_WALK_MAX_VISITS {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            *visits += 1;
            if *visits >= BASENAME_WALK_MAX_VISITS {
                return;
            }
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if path.is_dir() {
                if skip_basename_walk_dir(&name_str) {
                    continue;
                }
                walk(&path, depth + 1, basename, visits, best);
            } else if path.is_file() && name_str == basename {
                let score = basename_path_score(&path);
                match best {
                    Some((best_score, _)) if *best_score >= score => {}
                    _ => *best = Some((score, path)),
                }
            }
        }
    }
    walk(root, 0, basename, &mut visits, &mut best);
    best
}

fn skip_basename_walk_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | ".git"
            | ".signet"
            | ".selfsign"
            | ".next"
            | ".turbo"
            | ".cache"
            | "deps"
            | "incremental"
            | "examples"
            | "fixtures"
            | "testdata"
    ) || name == "debug" // skip target/debug when encountered as a dir name
}

fn basename_path_score(path: &Path) -> i32 {
    let s = path.to_string_lossy().to_ascii_lowercase();
    let mut score = 0i32;
    for (needle, pts) in [
        ("bundle", 40),
        ("nsis", 30),
        ("msi", 25),
        ("release", 20),
        ("dist", 10),
        ("out", 5),
    ] {
        if s.contains(needle) {
            score += pts;
        }
    }
    score
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

/// Freshness of a SHA256SUMS file vs on-disk artifacts and project version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumsFreshness {
    pub listed: usize,
    pub found: usize,
    pub missing: Vec<String>,
    pub sums_versions: Vec<String>,
    pub project_version: Option<String>,
    /// `listed > 0 && found == 0`
    pub empty_disk: bool,
    /// Project version known, sums carry version(s), none match project.
    pub version_mismatch: bool,
}

impl SumsFreshness {
    pub fn is_stale(&self) -> bool {
        self.empty_disk || self.version_mismatch
    }

    /// Human warnings suitable for `signet verify` notes.
    pub fn warnings(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.empty_disk {
            out.push(format!(
                "stale SHA256SUMS: listed {} file(s) but none found on disk — rebuild (`signet build`) or prune SHA256SUMS",
                self.listed
            ));
        } else if self.found < self.listed && self.listed > 0 {
            out.push(format!(
                "SHA256SUMS: {}/{} listed file(s) found on disk ({} missing) — rebuild or prune stale entries",
                self.found,
                self.listed,
                self.listed - self.found
            ));
        }
        if self.version_mismatch {
            let proj = self.project_version.as_deref().unwrap_or("?");
            let sums = if self.sums_versions.is_empty() {
                "(none)".into()
            } else {
                self.sums_versions.join(", ")
            };
            out.push(format!(
                "stale SHA256SUMS: artifact version(s) [{sums}] ≠ project version {proj} — rebuild or prune sums"
            ));
        }
        out
    }
}

/// Assess whether SHA256SUMS looks stale relative to disk and optional project version.
pub fn assess_sums_freshness(
    sums_path: &Path,
    search_roots: &[&Path],
    project_version: Option<&str>,
) -> anyhow::Result<SumsFreshness> {
    let text = fs::read_to_string(sums_path)?;
    let entries = parse_sha256sums(&text)?;
    let sums_dir = sums_path.parent().unwrap_or_else(|| Path::new("."));

    let mut missing = Vec::new();
    let mut found = 0usize;
    let mut version_set = std::collections::BTreeSet::new();

    for (_, name) in &entries {
        let path = resolve_artifact_path(name, name, sums_dir, search_roots);
        if path.is_file() {
            found += 1;
        } else {
            missing.push(name.clone());
        }
        let base = Path::new(name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(name.as_str());
        for v in extract_semver_tokens(base) {
            version_set.insert(v);
        }
    }

    let listed = entries.len();
    let sums_versions: Vec<String> = version_set.into_iter().collect();
    let project_norm = project_version.map(normalize_version);
    let version_mismatch = match (&project_norm, sums_versions.is_empty()) {
        (Some(proj), false) => !sums_versions.iter().any(|v| normalize_version(v) == *proj),
        _ => false,
    };

    Ok(SumsFreshness {
        listed,
        found,
        missing,
        sums_versions,
        project_version: project_norm,
        empty_disk: listed > 0 && found == 0,
        version_mismatch,
    })
}

fn normalize_version(v: &str) -> String {
    v.trim()
        .trim_start_matches(['v', 'V'])
        .trim()
        .to_string()
}

/// Extract `X.Y.Z` (optional pre-release suffix ignored) tokens from a filename.
pub fn extract_semver_tokens(name: &str) -> Vec<String> {
    let bytes = name.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            let mut dots = 0u8;
            let mut j = i;
            while j < bytes.len() {
                let b = bytes[j];
                if b.is_ascii_digit() {
                    j += 1;
                } else if b == b'.' && dots < 2 && j + 1 < bytes.len() && bytes[j + 1].is_ascii_digit()
                {
                    dots += 1;
                    j += 1;
                } else {
                    break;
                }
            }
            if dots == 2 {
                let token = &name[start..j];
                // Avoid matching lone IP-ish or year.month.day when surrounded oddly —
                // require token boundaries that look like version separators.
                let before_ok = start == 0
                    || matches!(
                        bytes[start - 1],
                        b'_' | b'-' | b'/' | b' ' | b'(' | b'[' | b'v' | b'V'
                    );
                let after_ok = j >= bytes.len()
                    || matches!(
                        bytes[j],
                        b'_' | b'-' | b'/' | b' ' | b')' | b']' | b'.'
                    );
                if before_ok && after_ok {
                    out.push(token.to_string());
                }
                i = j;
                continue;
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    out
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
    fn writes_relative_nested_path() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("apps/desk/bundle");
        fs::create_dir_all(&nested).unwrap();
        let file = nested.join("setup.exe");
        fs::write(&file, b"mz").unwrap();
        let out = dir.path().join("SHA256SUMS");
        write_sha256sums(&out, &[file]).unwrap();
        let text = fs::read_to_string(&out).unwrap();
        assert!(
            text.contains("apps/desk/bundle/setup.exe") || text.contains("apps\\desk\\bundle\\setup.exe"),
            "expected relative path in sums, got:\n{text}"
        );
        let ok = verify_sha256sums(&out, &[dir.path()], None).unwrap();
        assert_eq!(ok.len(), 1);
        assert!(ok[0].ok, "{:?}", ok[0]);
    }

    #[test]
    fn verify_finds_relative_entry_without_search_root_hit() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("out");
        fs::create_dir_all(&nested).unwrap();
        let file = nested.join("app.exe");
        fs::write(&file, b"data").unwrap();
        let sums = dir.path().join("SHA256SUMS");
        write_sha256sums(&sums, &[file]).unwrap();
        // Empty search roots — must resolve via sums_dir + relative name.
        let ok = verify_sha256sums(&sums, &[], None).unwrap();
        assert_eq!(ok.len(), 1);
        assert!(ok[0].ok);
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

    #[test]
    fn extract_semver_from_installer_names() {
        assert_eq!(
            extract_semver_tokens("Miro Desktop_0.2.0_x64-setup.exe"),
            vec!["0.2.0".to_string()]
        );
        assert_eq!(
            extract_semver_tokens("app-1.0.0.dmg"),
            vec!["1.0.0".to_string()]
        );
        assert!(extract_semver_tokens("plain-setup.exe").is_empty());
    }

    #[test]
    fn freshness_empty_disk_and_version_mismatch() {
        let dir = tempdir().unwrap();
        let sums = dir.path().join("SHA256SUMS");
        fs::write(
            &sums,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  Miro_0.2.0_setup.exe\n",
        )
        .unwrap();
        let f = assess_sums_freshness(&sums, &[dir.path()], Some("0.3.0")).unwrap();
        assert!(f.empty_disk);
        assert!(f.version_mismatch);
        assert!(f.is_stale());
        let warns = f.warnings();
        assert!(warns.iter().any(|w| w.contains("none found on disk")));
        assert!(warns.iter().any(|w| w.contains("0.2.0")));
    }

    #[test]
    fn freshness_ok_when_file_and_version_match() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("App_1.0.0.exe");
        fs::write(&file, b"mz").unwrap();
        let sums = dir.path().join("SHA256SUMS");
        write_sha256sums(&sums, &[file]).unwrap();
        let f = assess_sums_freshness(&sums, &[dir.path()], Some("v1.0.0")).unwrap();
        assert!(!f.empty_disk);
        assert!(!f.version_mismatch);
        assert!(!f.is_stale());
        assert_eq!(f.found, 1);
    }

    #[test]
    fn resolves_basename_under_nested_bundle() {
        let dir = tempdir().unwrap();
        let nested = dir
            .path()
            .join("apps/desk/src-tauri/target/release/bundle/nsis");
        fs::create_dir_all(&nested).unwrap();
        let file = nested.join("Miro Desktop_0.3.0_x64-setup.exe");
        fs::write(&file, b"mz-setup").unwrap();
        let sums = dir.path().join("SHA256SUMS");
        // Basename-only line (release-style leftover).
        let digest = sha256_hex_file(&file).unwrap();
        fs::write(
            &sums,
            format!("{digest}  Miro Desktop_0.3.0_x64-setup.exe\n"),
        )
        .unwrap();

        let ok = verify_sha256sums(&sums, &[dir.path()], None).unwrap();
        assert_eq!(ok.len(), 1, "{ok:?}");
        assert!(ok[0].ok, "{:?}", ok[0]);

        let fresh = assess_sums_freshness(&sums, &[dir.path()], Some("0.3.0")).unwrap();
        assert!(!fresh.empty_disk);
        assert_eq!(fresh.found, 1);
        assert!(!fresh.is_stale());
    }
}
