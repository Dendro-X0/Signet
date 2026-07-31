# Signing (Phase 3)

`selfsign build` runs `tauri build` (unless `--skip-build`), discovers bundle artifacts, writes `SHA256SUMS`, and signs host-matching outputs with the active identity.

## Command

```bash
selfsign build
selfsign build --skip-build          # sign existing bundles only
selfsign build --no-sign             # build/discover + checksums, no crypto sign
selfsign build --no-timestamp        # Windows: skip Authenticode TSA
selfsign build --artifact path.exe   # explicit files (skips discovery)
selfsign build --tauri-arg=--debug   # forwarded to `tauri build` (repeatable)
```

## Host behavior

| Host | Method | Tools |
|------|--------|-------|
| Windows | Authenticode via `signtool` after OpenSSL PEM→PFX export | Windows SDK SignTool, OpenSSL |
| macOS | `codesign` with temp keychain import; ad-hoc fallback (`-`) | codesign, security, OpenSSL |
| Linux | Detached `openssl dgst -sha256` signatures (`.sig`) + checksums | OpenSSL |

Checksums (`SHA256SUMS` in the project root) are always written for discovered files.

## Honesty

- **Windows:** SmartScreen may still warn for self-signed / low-reputation certificates.
- **macOS:** Gatekeeper may block; **notarization is not performed** (Apple Developer account).
- **Linux:** Detached signatures + checksums; distro package trust policies still apply.

## Artifact discovery

Looks under `{src-tauri}/target/{profile}/bundle/**` and the profile output dir for:

- Windows: `.exe`, `.msi` / `.msix`
- macOS: `.app`, `.dmg`
- Linux: `.AppImage`, `.deb`, `.rpm`

`src-tauri` is resolved from `project.tauri_root` (child `src-tauri/` preferred).

## Prerequisites

```bash
selfsign doctor
```

Windows specifically needs SignTool (SDK) and OpenSSL on PATH for PFX export from the Phase 2 PEM identity.
