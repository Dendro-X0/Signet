# Changelog

## 0.2.0 — 2026-07-31

Rebrand to **Signet**. Product scope expands beyond Tauri-only: identity, signing, trust, and release for self-signed **desktop and mobile** apps across frameworks (Tauri deepest today; Electron / mobile adapters next).

### Changed

- Binary and crate: `signet` (was `selfsign`)
- Config / secrets: `signet.toml` + `.signet/` (legacy `selfsign.toml` / `.selfsign/` still detected)
- Docs and TUI copy reflect multi-framework thesis
- Repository: [github.com/Dendro-X0/Signet](https://github.com/Dendro-X0/Signet)

### Notes

- `signet build` remains Tauri-first; other frameworks are scan/roadmap until adapters land.

## 0.1.0 — 2026-07-31

First public release as `selfsign`.

### Added

- CLI: `init`, `identity`, `trust`, `build`, `release`, `doctor`, `scan`
- Interactive TUI hub with guided setup and shared console formatting
- Local ECDSA code-signing identity under `.selfsign/` + `TRUST.md` generation
- Host signing: Windows (`signtool`), macOS (`codesign`), Linux (OpenSSL detached + `SHA256SUMS`)
- GitHub Releases publish via `gh` or `GH_TOKEN` / `GITHUB_TOKEN`
- Repo scan for Tauri / Electron / Android / iOS markers and existing installers

### Notes

- Self-signed builds may still trigger SmartScreen / Gatekeeper warnings.
- Mobile store signing is detected by scan but not performed by the tool.
