# Signing

`signet build` selects a [`FrameworkAdapter`](../specs/backend/artifact-contract-design.md) (`project.framework`: `tauri`, `electron`, `android`, or `ios`), runs the framework build (unless `--skip-build`), discovers artifacts, writes `SHA256SUMS`, and signs where applicable.

**Tauri:** default build is `tauri build` from the resolved `src-tauri`. For monorepos, set `build_command` (e.g. `pnpm desktop:release`) — Signet runs it from the directory containing `signet.toml`. App root field is `app_root` (legacy alias: `tauri_root`).

**Multi-target (monorepo):** optional `[[targets]]` entries (`id`, `framework`, `app_root`, `build_command`). `signet build` runs all; `signet build --target <id>` runs one. Shared identity / `SHA256SUMS` / `TRUST.md`.

**Electron:** set `framework = "electron"`, optional `build_command` (default `npm run dist`). Discover looks under `dist/`, `out/`, and `release/`. Signet signs **post-bundle** installers (does not inject `win.certificateFile`).

**Android:** set `framework = "android"` or use `signet android keystore|sign`. See [android.md](android.md) — local keystore ≠ Play App Signing key.

**iOS:** set `framework = "ios"` or use `signet ios package`. See [ios.md](ios.md) — free provisioning ~7 days; packaging ≠ App Store trust. Build requires explicit `build_command` (no scheme guessing).

**Hybrid (Flutter / RN / Expo / Capacitor):** set `framework` + required `build_command`. See [frameworks.md](frameworks.md).

**Rust CLI:** set `framework = "cli"` (auto-detected for Cargo workspaces / binary crates). Default build is `cargo build --release`; Signet signs the host binary under `target/<profile>/`.

**Release tags:** Guided release and `signet release` (when `--tag` is omitted) default to a tag from the project version — `Cargo.toml` / `package.json` / Expo `app.json` / latest git tag — e.g. workspace `0.5.4` → `v0.5.4`. See [`specs/backend/version-aware-defaults-design.md`](../specs/backend/version-aware-defaults-design.md).

**Graduation (OV / Azure / notarize):** explicit helpers — not part of default `signet build`. See [graduation.md](graduation.md) and `signet graduate notes`.

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
