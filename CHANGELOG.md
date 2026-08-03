# Changelog

## 0.5.9 — 2026-08-03

### Added

- `signet ship --plan` — declared `[platforms]` vs on-disk artifacts coverage report
- Build prints upfront ship-coverage / host-only capability line
- Doctor `ship-coverage` optional check (warns on gap)
- Guided finish surfaces multi-OS gap when declared platforms are missing

### Docs

- Spec: `specs/backend/ship-coverage-design.md` (multi-platform ship slice A)

## 0.5.8 — 2026-08-03

### Fixed

- `scan --apply` no longer shrinks `[platforms]` without `--force` (preserves shipping intent)
- `scan --apply` drafts `[[targets]]` on existing configs when empty and ≥2 installable apps (excludes nested rust_cli)
- Basename-only `SHA256SUMS` entries resolve via bounded tree walk (verify + stale assess match)
- `scan --apply` skips “identity create” when an identity already exists

### Docs

- Spec: `specs/backend/scan-apply-hardening-design.md`

## 0.5.7 — 2026-08-03

### Added

- `signet verify` stale-sums detection: warn when listed files are missing or filename versions ≠ project version; `--fail-stale` for hard fail
- Scan notes/next-steps for multi-installable apps (`[[targets]]` / `scan --apply`); platforms intent vs host capability one-liner
- `signet identity status` alias for `identity show`

### Fixed

- Always log `wrote …SHA256SUMS (post-sign)` after Authenticode/APK resign rewrite

### Docs

- Spec: `specs/backend/fault-tolerance-design.md`
- Clarify build relative paths vs release flat basenames in `docs/signing.md` / `docs/verify.md`

## 0.5.6 — 2026-08-03

### Fixed

- Windows `inspect`: self-signed Authenticode reports **signed** (presence vs `/pa` trust), not false `unsigned`
- `SHA256SUMS` uses paths relative to the sums file so `signet verify` finds monorepo artifacts

### Added

- Config field `app_root` (legacy alias `tauri_root`); guided init can accept scan suggestion in one confirm
- `signet scan --apply` fills omitted `framework` / `app_root`; multi-project scans draft `[[targets]]`
- Monorepo `[[targets]]` + `signet build --target <id>` (shared identity / sums / TRUST.md)

### Docs

- Specs: `config-simplification-design.md`, `multi-target-design.md`
- Updated Miro dogfood notes for Check honesty

## 0.5.5 — 2026-08-03

### Fixed

- Windows: run `build_command` via `cmd /C` so `pnpm` / `npm` shims resolve (fixes `program not found` on monorepo Tauri builds)

### Changed

- Tauri adapter: non-empty `[project].build_command` runs from the project root (monorepo scripts like `pnpm desktop:release`); empty still uses `tauri build`

### Docs

- Miro dogfood notes: `docs/dogfood/miro-notes.md` (Sign→Prove→Check on real Tauri app)

## 0.5.4 — 2026-08-02

### Fixed

- Omitted `[project].framework` no longer defaults to Tauri — resolved via scan (same preference as Init)
- This repo’s `signet.toml` sets `framework = "cli"` explicitly

### Added

- Version-aware release tag defaults from Cargo / package.json / Expo / git

## 0.5.3 — 2026-08-01

### Fixed

- Windows `install.ps1`: prefer `curl.exe` with retries (avoids flaky `Invoke-WebRequest` TLS EOF); document curl fallback in `docs/install.md`

## 0.5.2 — 2026-08-01

### Fixed

- Windows installer mirrors into `%USERPROFILE%\bin` so Git Bash / Cursor can run `signet` without restarting the IDE
- Broadcast `WM_SETTINGCHANGE` after PATH updates; write install receipt without UTF-8 BOM
- Treat the home shim as installer-managed for `self status|update|uninstall`

## 0.5.1 — 2026-08-01

### Fixed

- Installer PATH shadow: prepend managed `bin` on Windows; warn when `~/.cargo/bin/signet` still wins
- `signet self status` reports cargo-vs-installer shadow when a receipt exists
- Docs: `docs/install.md` cargo vs installer troubleshooting

## 0.5.0 — 2026-08-01

Public **preview** cut: Sign → Prove → Check packaging, CLI project detection, and TUI polish.

### Added

- Phase 15 demo kit: `demo/fixture` + `demo/scripts/happy-path.{sh,ps1}` + `docs/demo.md`
- Phase 14 golden-path TUI: Sign → Prove → Check guided setup, framework/`build_command` pick, hub Verify / Inspect / Graduate notes
- Phase 13 public narrative: Sign → Prove → Check, dual-path (self-sign vs official), README/product rewrite
- Public release program + **v0.5.0 roadmap**: `specs/backend/v0.5-release-roadmap.md`, Phases 13–16
- Hybrid adapters: `framework = flutter|react-native|expo|capacitor` (+ `rn` alias), `docs/frameworks.md`, scan detection
- `signet inspect` — best-effort signed/unsigned/adhoc/unknown report per artifact platform (`docs/verify.md`)
- CLI / Rust workspace detection (`framework = "cli"`); prefer root over nested `demo/` / fixtures; guided init auto-suggest
- Partial dogfood notes: `docs/dogfood/signet-cli-notes.md`

### Changed

- TUI hub: shared cyan panel frames; Sign / Prove / Check accent in hints; green/dim status marks
- Console / prompts: TTY-aware color hierarchy (`NO_COLOR` / `SIGNET_FORCE_COLOR`)

### Fixed

- CLI binary discovery tests on Unix (host `+x` bins + cross-compiled `.exe` without `+x`)

## 0.4.0 — 2026-07-31

Framework adapters, checksum signing, and reputation graduation helpers.

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
