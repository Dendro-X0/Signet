# Design: OV / Azure / notarize graduation helpers

**Band:** Later (roadmap — graduation ladder)  
**Status:** implemented  
**Depends on:** Phases 6–9 (trust tiers, verify, artifact contract); host sign already exists for self-signed  
**Owners:** `crates/signet/src/graduate/`, `commands/graduate.rs`, `docs/graduation.md`, `trust_kit.rs`, `commands/doctor.rs`  
**Plan alignment:** wrap paid/CA/Apple tooling honestly; never claim SmartScreen silence or Gatekeeper pass from helpers alone.

## Problem

Self-signed Signet builds stay at integrity tiers (`self_signed_host`, checksums). Maintainers who buy an OV/EV Authenticode cert, use Azure Trusted Signing (Artifact Signing), or Apple Developer ID + notarization need a **repeatable CLI ladder** without Signet pretending those programs are free or automatic.

## Goals

1. Public doc (`docs/graduation.md`) describing the ladder and what Signet will / will not do.
2. `signet graduate notes` — short honesty summary.
3. Windows helpers:
   - `ov-sign` — Authenticode via existing OV/CA cert (thumbprint or PFX), **not** the Signet self-signed identity.
   - `azure-sign` — wrap `signtool` + Azure Code Signing dlib + metadata JSON (Trusted Signing).
4. macOS helpers:
   - `notarize` — `xcrun notarytool submit --wait` with a Keychain profile.
   - `staple` — `xcrun stapler staple`.
5. Doctor optional checks for `notarytool` / `stapler` (macOS) and graduation tooling hints (Windows).
6. TRUST.md pointer when `declared_tier` is `ca_authenticode` or `apple_notarized`.

## Non-goals

- Purchasing certificates, Apple membership, or Azure subscriptions.
- Silencing SmartScreen / Gatekeeper / storing Root trust instructions.
- Replacing `signet build` self-sign path (graduation is explicit opt-in commands).
- Perfect detection of CA chain or notarization ticket status on every OS (declare tiers; best-effort doctor only).
- Linux CA store / distro packaging.

---

## Trust honesty (required)

| Path | Signet role |
|------|-------------|
| Self-signed (default) | Existing identity + host sign |
| OV / EV Authenticode | Sign with **maintainer-supplied** cert; declare `ca_authenticode` when true |
| Azure Trusted Signing | Wrap Microsoft dlib + metadata; Azure auth stays outside Signet |
| Apple notarization | Submit + staple via Apple tools; declare `apple_notarized` when true |

**Never** instruct end users to install publisher certs into Trusted Root.

Helpers print: graduation improves *reputation eligibility*; it does not guarantee OS silence.

---

## Config (optional)

```toml
[graduation]
# Windows OV — certificate SHA-1 thumbprint (hex, no spaces). Empty = require CLI flag / env.
ov_thumbprint = ""
# Default Authenticode timestamp URL for OV / Azure helpers.
timestamp_url = "http://timestamp.digicert.com"

[graduation.azure]
# Paths relative to project root (or absolute). Secrets stay in Azure / Key Vault — not here.
dlib = ""           # Azure.CodeSigning.Dlib.dll
metadata = ""       # Trusted Signing metadata JSON
timestamp_url = "http://timestamp.acs.microsoft.com"

[graduation.apple]
# Keychain profile name created via `xcrun notarytool store-credentials`
keychain_profile = ""
```

Env overrides (never write secrets into `signet.toml`):

| Env | Role |
|-----|------|
| `SIGNET_OV_THUMBPRINT` | OV cert thumbprint |
| `SIGNET_OV_PFX` | Path to PFX (alternative to thumbprint) |
| `SIGNET_OV_PFX_PASS` | PFX password (required with PFX) |
| `SIGNET_AZURE_DLIB` | Override dlib path |
| `SIGNET_AZURE_METADATA` | Override metadata JSON path |
| `SIGNET_NOTARY_PROFILE` | Override Apple Keychain profile |

---

## CLI

```text
signet graduate notes
signet graduate ov-sign --file PATH [--file PATH ...] [--thumbprint HEX] [--pfx PATH]
signet graduate azure-sign --file PATH [--file PATH ...]
signet graduate notarize --path PATH [--profile NAME] [--no-staple]
signet graduate staple --path PATH
```

### `ov-sign`

1. Resolve `signtool`.
2. Resolve credential: `--thumbprint` / env / config, **or** `--pfx` / `SIGNET_OV_PFX` (+ pass).
3. Reject if neither present — do **not** fall back to Signet self-signed identity.
4. `signtool sign /fd SHA256 /td SHA256 /tr <tsa> (/sha1 …|/f … /p …) <file>`
5. Print honesty note + suggest setting `trust.declared_tier = "ca_authenticode"` when the cert truly chains to a public CA.

### `azure-sign`

1. Resolve `signtool`, dlib, metadata (config/env).
2. Fail clearly if dlib or metadata missing (point at `docs/graduation.md`).
3. `signtool sign /fd SHA256 /td SHA256 /tr <acs tsa> /dlib <dlib> /dmdf <metadata> <file>`
4. Azure login/identity is **out of band** (Azure CLI / env / dlib config). Signet does not store Azure secrets.

### `notarize` / `staple`

1. Require macOS host; resolve `xcrun`.
2. `xcrun notarytool submit <path> --keychain-profile <profile> --wait`
3. Unless `--no-staple`: `xcrun stapler staple <path>`
4. Suggest `trust.declared_tier = "apple_notarized"` only after successful notarization of a **Developer ID** signed build (document that ad-hoc/self-issued Signet identity is insufficient).

---

## Doctor / scan / TRUST

- Doctor (macOS): optional `notarytool`, `stapler`.
- Doctor (Windows): optional note that OV/Azure need certs/dlib beyond self-sign OpenSSL/PFX.
- Scan: one line pointing at `docs/graduation.md` / `signet graduate notes` when desktop installers found.
- TRUST: when declared tier is `ca_authenticode` or `apple_notarized`, add a short **Graduation** blurb linking to `docs/graduation.md` (no secrets).

---

## Acceptance

- [x] Design `ready` → implement → `implemented`.
- [x] `docs/graduation.md` with ladder honesty.
- [x] `signet graduate notes` prints honesty without network.
- [x] `ov-sign` / `azure-sign` unit-testable argument builders (no live Azure/Apple).
- [x] `notarize` / `staple` command builders + clear errors off-macOS.
- [x] Doctor checks added; TRUST graduation blurb for declared tiers.
- [x] `cargo test -p signet` and `cargo clippy -p signet -- -D warnings` green.

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet` |
| L2 | `cargo clippy -p signet -- -D warnings` |
| L3 | Manual on maintainer machine with real OV/Azure/Apple credentials (optional; not CI) |

## Open questions (frozen defaults)

1. **Separate from `signet build`?** Yes — graduation is explicit; build stays self-sign by default.
2. **Import OV into `.signet/identity`?** No — OV/Azure credentials stay external; thumbprint/PFX/dlib only.
3. **Auto-set `declared_tier`?** No — maintainer declares; helpers only suggest.
