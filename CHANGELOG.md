# Changelog

## 0.1.0 — 2026-07-31

First public release of `selfsign`.

### Added

- CLI: `init`, `identity`, `trust`, `build`, `release`, `doctor`, `scan`
- Interactive TUI hub with guided setup and shared console formatting
- Local ECDSA code-signing identity under `.selfsign/` + `TRUST.md` generation
- Host signing: Windows (`signtool`), macOS (`codesign`), Linux (OpenSSL detached + `SHA256SUMS`)
- GitHub Releases publish via `gh` or `GH_TOKEN` / `GITHUB_TOKEN`
- Repo scan for Tauri / Electron / Android / iOS markers and existing installers

### Notes

- Self-signed builds may still trigger SmartScreen / Gatekeeper warnings.
- Mobile store signing is detected by scan but not performed by selfsign.
