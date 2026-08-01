# Design: Android signing helpers

**Phase:** 11  
**Status:** implemented  
**Depends on:** Phases 6–9 (trust, verify, checksum signing, artifact contract)  
**Owners:** `crates/signet/src/android/`, `commands/android.rs`, `artifact/android.rs`, `trust_kit.rs`, `commands/doctor.rs`  
**Plan alignment:** sideload/F-Droid-style self-sign; honest Play App Signing docs — never conflate upload key with Play app signing key.

## Problem

Every installable APK/AAB must be signed. Indie/OSS self-signing uses a **local keystore**. Play distribution uses **Play App Signing** (upload key ≠ distribution signing key). Signet helps the first path and documents the second.

## Goals

1. Create/import release keystore under `.signet/android/` (gitignored).
2. Sign APKs via `apksigner` (preferred) with `jarsigner` fallback.
3. `signet android` CLI + optional `framework = "android"` adapter for discover/build/sign.
4. Emit Android cert digest into `TRUST.md`; doctor checks tooling.
5. Public doc: Play upload key vs app signing key.

## Non-goals

- Replacing Play App Signing or storing Google Play Console credentials.
- Claiming Play Store install trust from a local keystore.
- iOS (Phase 12).
- Signing AABs as “Play-ready” (document upload-key flow only; APK is the primary Signet path).

---

## Trust honesty (required)

| Channel | Signet role |
|---------|-------------|
| Sideload / F-Droid-style | Manage developer keystore; checksums + cert SHA-256 digest |
| Google Play | Document upload key vs app signing key; local keystore is **not** the Play distribution key |

Tier: local APK sign → integrity (`self_signed_host` / checksum tiers as appropriate). Use declared `play_managed` only when the maintainer ships via Play App Signing.

**Never** tell users to install the app cert into a system trust store.

---

## Keystore layout (gitignored)

```text
.signet/android/
  release.jks          # or .keystore
  meta.toml            # alias, store_type, created_at, cert_sha256
```

Passwords **only** via env (never `signet.toml`):

| Env | Role |
|-----|------|
| `SIGNET_ANDROID_STORE_PASS` | Keystore password (required for create/sign) |
| `SIGNET_ANDROID_KEY_PASS` | Key password (default: same as store pass) |

**Decision (frozen):** one release keystore per app project under `.signet/android/` (not mapped from desktop X.509 identity).

### Commands

```text
signet android keystore create [--alias signet] [--force]
signet android keystore import --keystore PATH --alias ALIAS [--force]
signet android keystore show
signet android sign --apk PATH [--apk PATH ...]
```

Create uses `keytool -genkeypair` (RSA 2048, long validity). Import copies the file and records alias + digests via `keytool -list -v`.

---

## APK signing

**Decision (frozen):** prefer `apksigner` (build-tools); fall back to `jarsigner` if apksigner missing.

Discovery of tools:

1. `which apksigner` / `which jarsigner` / `which keytool`
2. `$ANDROID_HOME` / `$ANDROID_SDK_ROOT` → `build-tools/<latest>/apksigner(.bat)`

`signet android sign` and `signet build` (when `framework = "android"` or discovered APKs with keystore present) sign in place (or write `-signed.apk` if input is read-only — prefer in-place when writable).

AAB: do not auto-sign as Play distribution. Print note pointing at `docs/android.md`.

---

## `framework = "android"` adapter

| Stage | Behavior |
|-------|----------|
| `label_root` | `tauri_root` as app/Gradle root |
| `build` | `build_command` if set; else `gradlew assembleRelease` / `gradlew.bat` |
| `discover` | Walk `**/build/outputs/apk/**`, `dist/`, `release/` for `.apk` (depth-capped); skip `node_modules` |

Host PE/codesign path is skipped for Android adapter; use android keystore signing instead.

---

## TRUST.md / doctor / scan

- TRUST: **Android** subsection with cert SHA-256 digest (when keystore meta exists) + Play honesty blurb.
- Doctor: `keytool`, `apksigner` (optional), `android-keystore` present check.
- Scan: when Android markers found, suggest `signet android keystore create` and `framework = "android"` for APK-only apps.

---

## Acceptance

- [x] Design ready → implemented.
- [x] Keystore create/import/show under `.signet/android/`.
- [x] APK sign via apksigner or jarsigner when tools + env pass present.
- [x] `docs/android.md` Play honesty.
- [x] Doctor + TRUST updates.
- [x] `cargo test -p signet` + clippy `-D warnings`.

**Status:** implemented (2026-07-31)

## Proof plan

| Layer | Evidence |
|-------|----------|
| L1 | Unit: paths/meta round-trip; classify apk; adapter discover fixture |
| L2 | `cargo test -p signet` + clippy |
| L3 | Manual/optional: keytool available → create keystore (env pass) |

## Open questions — resolved

| Question | Decision |
|----------|----------|
| apksigner vs jarsigner? | **apksigner first**, jarsigner fallback |
| Shared identity vs per-app keystore? | **Per-app** `.signet/android/` |
