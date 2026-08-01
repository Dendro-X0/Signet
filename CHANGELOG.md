# Changelog

## Unreleased

### Added

- Graduation helpers: `signet graduate notes|ov-sign|azure-sign|notarize|staple`, `docs/graduation.md`, `[graduation]` config
- Phase 12 iOS helpers: `signet ios package|notes`, `framework = "ios"`, `docs/ios.md` (7-day free provisioning honesty)
- Phase 11 Android helpers: `signet android keystore|sign`, `framework = "android"`, `docs/android.md`
- Phase 10 Electron adapter: `framework = "electron"`, discover `dist`/`out`/`release`, optional `build_command`
- Phase 9 artifact contract: `artifact/` module, `FrameworkAdapter` + `TauriAdapter`, `project.framework`
- Phase 8 checksum signing: `signet sums-key create|show`, minisign on `SHA256SUMS` → `.minisig`, optional GPG `.asc`
- `signet build` / `release` flags: `--no-sums-sign`, `--require-sums-sign`, `--require-gpg`
- `signet verify --require-sig` hard-fails (exit 3); `--minisign-pub` for distributed verify
- Config: `[trust.checksum_signing]`; doctor checks `sums-minisign-key` and optional `gpg`

## 0.3.1 — 2026-07-31

### Fixed

- `release-cli` builds Intel macOS via cross-compile on `macos-latest` (avoids stuck `macos-13` runners)

## 0.3.0 — 2026-07-31

CLI distribution and integrity verify path.

### Added

- `signet verify` — fingerprint + SHA256SUMS checks (exit codes for CI)
- Trust tiers in `TRUST.md` + doctor `trust-tier` (`[trust]` config)
- One-command installers (`install.sh` / `install.ps1`)
- `signet self status|update|uninstall` + TUI Update / Uninstall Signet
- GitHub Actions: `ci.yml` (test) and `release-cli.yml` (multi-platform binaries on tag)

### Notes

- Installer-managed installs live under `~/.signet-cli/` or `%LOCALAPPDATA%\Signet\`
- Project `.signet/` dirs are never removed by CLI uninstall

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
