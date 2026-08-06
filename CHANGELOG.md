# Changelog

## 0.5.17 — 2026-08-06

### Added

- `signet ship secrets` — assess / dry-run `--push` / `--push --apply` for GitHub Actions secrets from local `.signet/`
- CI readiness on `signet doctor` + `ship --plan` with stable gap IDs (`gap.android.ci_secrets`, …)
- `ship --ci` template: `ship-preflight` job, identity/keystore restore steps, `::error::` pointing at `ship secrets --push --apply`

### Docs

- Spec: `specs/backend/ship-secrets-ci-design.md`; `docs/ship.md` secrets section

## 0.5.16 — 2026-08-04

### Added

- Confirmed browser-open step for GitHub auth setup (TTY only): doctor, `release --dry-run`, live preflight, and guided release
- Opens `https://cli.github.com/` when `gh` is missing, or the classic PAT create page (`repo` scope) when `gh` is installed but not logged in

### Docs

- Spec: `specs/backend/release-auth-browser-open-design.md`; `docs/release.md` Auth

## 0.5.15 — 2026-08-03

### Added

- Shared GitHub auth assessor: `gh auth status` or `GH_TOKEN`/`GITHUB_TOKEN` (installed `gh` alone is not enough)
- `signet doctor` prints a numbered setup guide when `github-auth` is not ready
- `signet release --dry-run` reports `auth:`; live release preflights with the same guide
- Guided Release / setup blocks publish until auth is ready

### Docs

- Spec: `specs/backend/release-auth-guided-design.md`; expanded `docs/release.md` Auth

## 0.5.14 — 2026-08-03

### Added

- `[ship] path = "self" | "graduate"` — dual Sign profile on the same multi-OS plan
- `signet ship --plan` shows per-OS Sign path (self / azure / ov / notarize / missing)
- `signet graduate apply` — discover host installers and run configured graduate backend
- `signet ship --ci` emits `graduate apply` on Windows/macOS when path=graduate

### Docs

- Spec: `specs/backend/ship-graduate-profile-design.md` (multi-platform ship slice F)

## 0.5.13 — 2026-08-03

### Added

- `[platforms].android` / `[platforms].ios` (default false); mobile `[[targets]]` (expo/RN/flutter/capacitor/android/ios) also declare commitment
- Ship coverage gap includes android/ios; `ship --ci` emits `ship-android` / `ship-ios` jobs when declared
- Release `classify_kind`: `.apk`/`.aab` → `android`, `.ipa` → `ios`; collect accepts `.aab`

### Docs

- Spec: `specs/backend/mobile-ship-loop-design.md` (multi-platform ship slice G)
- `docs/ship.md` / config-schema mobile platforms

## 0.5.12 — 2026-08-03

### Added

- `signet ship --ci` — emit `.github/workflows/signet-ship.yml` matrix from declared `[platforms]`
- `signet ship --collect DIR` — merge multi-host installers into `dist/signet-ship/` and rewrite `SHA256SUMS`
- `signet release` coverage gate: fails on declared-platform gap unless `--allow-partial` (dry-run warns only)
- Release collect attaches `dist/signet-ship/` assets

### Docs

- `docs/ship.md`; spec `specs/backend/ship-ci-collect-design.md` (slices D–E)

## 0.5.11 — 2026-08-03

### Fixed

- `signet release --dry-run` is read-only: no longer rewrites or creates `SHA256SUMS` / signatures on disk

### Docs

- Spec: `specs/backend/release-dry-run-readonly-design.md` (multi-platform ship slice C)

## 0.5.10 — 2026-08-03

### Fixed

- Multi-target `signet build` soft-skips unpaid/failed siblings (e.g. Expo without `build_command`) and still signs discovered desktop artifacts
- Clear target **debt** report; `--strict-targets` fails after signing successful siblings; `--target id` remains hard-fail

### Docs

- Spec: `specs/backend/soft-fail-targets-design.md` (multi-platform ship slice B)

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
