//! Build a `.ipa` (zip) from an existing `.app` bundle.

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Debug)]
pub struct PackageResult {
    pub ipa_path: PathBuf,
    pub app_name: String,
}

/// Create `Payload/<App>.app/…` inside a zip written as `.ipa`.
pub fn package_ipa(app_path: &Path, out_ipa: &Path) -> anyhow::Result<PackageResult> {
    if !app_path.is_dir() {
        anyhow::bail!(
            "expected an .app bundle directory, got: {}",
            app_path.display()
        );
    }
    let app_name = app_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid .app name"))?
        .to_string();
    if !app_name.to_ascii_lowercase().ends_with(".app") {
        anyhow::bail!("bundle name should end with .app (got {app_name})");
    }

    if let Some(parent) = out_ipa.parent() {
        fs::create_dir_all(parent)?;
    }
    if out_ipa.exists() {
        fs::remove_file(out_ipa)?;
    }

    let file = File::create(out_ipa)?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Ensure Payload/ directory entry exists for picky installers.
    zip.add_directory("Payload/", opts)?;
    zip.add_directory(format!("Payload/{app_name}/"), opts)?;

    add_dir_to_zip(&mut zip, app_path, &format!("Payload/{app_name}"), opts)?;
    zip.finish()?;

    Ok(PackageResult {
        ipa_path: out_ipa.to_path_buf(),
        app_name,
    })
}

fn add_dir_to_zip(
    zip: &mut ZipWriter<File>,
    dir: &Path,
    zip_prefix: &str,
    opts: SimpleFileOptions,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let zip_path = format!("{zip_prefix}/{name}");
        if path.is_dir() {
            zip.add_directory(format!("{zip_path}/"), opts)?;
            add_dir_to_zip(zip, &path, &zip_path, opts)?;
        } else if path.is_file() {
            zip.start_file(&zip_path, opts)?;
            let mut f = File::open(&path)?;
            io::copy(&mut f, zip)?;
        }
    }
    Ok(())
}

/// Default output path beside the `.app`.
pub fn default_ipa_path(app_path: &Path) -> PathBuf {
    let stem = app_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("App");
    app_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}.ipa"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;
    use zip::ZipArchive;

    #[test]
    fn packages_app_into_ipa_payload_layout() {
        let dir = tempdir().unwrap();
        let app = dir.path().join("Demo.app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("Info.plist"), b"<plist/>").unwrap();
        fs::write(app.join("Demo"), b"\0").unwrap(); // stub binary

        let ipa = dir.path().join("Demo.ipa");
        let result = package_ipa(&app, &ipa).unwrap();
        assert_eq!(result.app_name, "Demo.app");
        assert!(ipa.is_file());

        let f = File::open(&ipa).unwrap();
        let mut archive = ZipArchive::new(f).unwrap();
        let names: Vec<_> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(names.iter().any(|n| n.starts_with("Payload/Demo.app/")));
        assert!(names.iter().any(|n| n.ends_with("Info.plist")));

        let mut plist = archive
            .by_name("Payload/Demo.app/Info.plist")
            .unwrap();
        let mut buf = String::new();
        plist.read_to_string(&mut buf).unwrap();
        assert!(buf.contains("plist"));
    }
}
