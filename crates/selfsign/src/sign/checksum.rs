use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

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
}
