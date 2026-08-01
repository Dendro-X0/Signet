# Signing

`signet build` selects a [`FrameworkAdapter`](../specs/backend/artifact-contract-design.md) (`project.framework`: `tauri`, `electron`, or `android`), runs the framework build (unless `--skip-build`), discovers artifacts, writes `SHA256SUMS`, and signs outputs.

**Electron:** set `framework = "electron"`, optional `build_command` (default `npm run dist`). Discover looks under `dist/`, `out/`, and `release/`. Signet signs **post-bundle** installers (does not inject `win.certificateFile`).

**Android:** set `framework = "android"` or use `signet android keystore|sign`. See [android.md](android.md) — local keystore ≠ Play App Signing key.

iOS helpers are roadmap Phase 12.

## Usage

```bash
signet sums-key create             # once per project — `.signet/sums/` minisign key
signet build
signet build --skip-build          # sign existing bundles only
signet build --no-sign             # build/discover + checksums, no host crypto sign
signet build --no-sums-sign        # skip minisign/GPG on SHA256SUMS
signet build --require-sums-sign   # fail if minisign cannot sign checksums
signet build --no-timestamp        # Windows: skip Authenticode TSA
signet build --artifact path.exe   # explicit files (skips discovery)
signet build --tauri-arg=--debug   # forwarded to `tauri build` (repeatable)
```

With a sums key present, build/release also write `SHA256SUMS.minisig` (and optional `SHA256SUMS.asc` when `[trust.checksum_signing].gpg = true`).
## Host backends

| OS | Tooling |
|----|---------|
| Windows | `signtool` (+ OpenSSL to build PFX from PEM) |
| macOS | `codesign` |
| Linux | OpenSSL detached `.sig` + `SHA256SUMS` |

## Doctor

```bash
signet doctor
```

Checks host signing tools and GitHub auth without printing secrets.
