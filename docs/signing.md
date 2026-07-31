# Signing

`signet build` runs `tauri build` (unless `--skip-build`), discovers bundle artifacts, writes `SHA256SUMS`, and signs host-matching outputs with the active identity.

Other frameworks are detected by `scan`; dedicated build adapters are roadmap work.

## Usage

```bash
signet build
signet build --skip-build          # sign existing bundles only
signet build --no-sign             # build/discover + checksums, no crypto sign
signet build --no-timestamp        # Windows: skip Authenticode TSA
signet build --artifact path.exe   # explicit files (skips discovery)
signet build --tauri-arg=--debug   # forwarded to `tauri build` (repeatable)
```

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
