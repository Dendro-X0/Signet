# Design: host signature inspect (signed / unsigned / platform)

**Band:** Later (integrity UX — complements verify)  
**Status:** implemented  
**Depends on:** Phase 7 verify, Phase 9 artifact kinds, Android/iOS tooling discovery  
**Owners:** `crates/signet/src/inspect/`, `commands/inspect.rs`, `docs/verify.md` (cross-link)  
**Plan alignment:** answer “is this artifact signed?” and “for which platform?” without claiming OS reputation (SmartScreen / Gatekeeper).

## Problem

`signet scan` classifies installers by extension and prints a **signing hint**, not live status.  
`signet verify` checks fingerprint + `SHA256SUMS` (+ community sigs) and **defers** host PE/codesign inspect.  
Maintainers and agents need a best-effort report: signed / unsigned / unknown per file, with target platform.

## Goals

1. `signet inspect` CLI: one or more `--file` paths (required in v1; optional discover later).
2. Classify each file’s **target platform** via `ArtifactKind` (Windows / macOS / Linux / Android / iOS / unknown).
3. Probe **host signature presence** with platform tools when available:
   - Windows PE/MSI: `signtool verify /pa`
   - macOS `.app` / `.dmg`: `codesign --verify` (+ note ad-hoc when detectable)
   - Android APK: `apksigner verify` (reuse Android tool discovery)
   - Linux: sibling OpenSSL `.sig` presence (detached integrity, not Authenticode)
   - IPA: macOS only — best-effort `codesign` on nested `.app` after zip list **or** mark `unknown` with honesty if too heavy; **v1: unknown + hint to package/codesign tooling** unless path is already `.app`
4. Human table + `--json` for agents.
5. Never equate “signed” with SmartScreen silence, Gatekeeper pass, Play, or notarization.

## Non-goals

- Perfect CA-chain or notarization ticket validation (graduation / Apple tooling).
- Mutating files or re-signing (`build` / `graduate` / `android sign` own that).
- Cross-OS PE inspection without tools (report `unknown` + reason).
- Flutter/RN/.NET adapters (separate Later band).

---

## Status model

| `status` | Meaning |
|----------|---------|
| `signed` | Tool reported a cryptographic signature present |
| `unsigned` | Tool ran and reported no signature (or Linux: no sibling `.sig`) |
| `adhoc` | macOS codesign present but ad-hoc (`Signature=adhoc` / identity `-`) |
| `unknown` | Wrong host, missing tool, unsupported format, or inconclusive |
| `error` | Tool failed unexpectedly (still exit 0 for inspect unless `--strict`) |

Fields per file:

```text
path, kind, platform, status, method, detail
```

`platform` = intended ship platform from kind (not the machine running inspect).

### Exit codes

| Code | When |
|------|------|
| 0 | Inspect completed (including unsigned / unknown) |
| 1 | `--strict` and any `unsigned` or `error` |
| 2 | Usage / I/O (missing file) |

---

## CLI

```text
signet inspect --file PATH [--file PATH ...] [--json] [--strict]
```

Optional later (not required for exit): discover from `SHA256SUMS` or `signet scan` roots.

---

## Probe details (frozen)

### Windows (`windows-exe` / `windows-msi`)

- Require `signtool` (same discovery as sign).
- `signtool verify /pa <file>` → exit 0 ⇒ `signed`, else `unsigned` (if tool present).
- Off Windows without signtool ⇒ `unknown`.

### macOS (`macos-app` / `macos-dmg`)

- Require `codesign`.
- `codesign --verify --verbose=2 <path>` success ⇒ at least signed; parse `codesign -dv` stderr for `Signature=adhoc` ⇒ `adhoc`, else `signed`.
- Off macOS ⇒ `unknown`.

### Android (`android-apk`)

- Prefer `apksigner verify <apk>`; success ⇒ `signed`.
- Missing tool ⇒ `unknown` (point at `signet android` / SDK).

### Linux (`linux-appimage` / `deb` / `rpm`)

- If `<file>.sig` exists ⇒ `signed` with method `openssl-detached-sibling`.
- Else ⇒ `unsigned` (checksum-only possible via verify; inspect does not hash).

### iOS (`ios-ipa`)

- v1: `unknown`, detail: packaging ≠ codesign inspect; use macOS + Developer tools for `.app`.

---

## Acceptance

- [x] Design ready → implemented.
- [x] `signet inspect --file` reports platform + status for fixture paths (unit-tested parsers / status mapping).
- [x] Windows integration optional when signtool present (unsigned unsigned PE or signed tiny exe from existing integration test pattern).
- [x] Docs: short section in `docs/verify.md` + roadmap Later check.
- [x] `cargo test -p signet` + `cargo clippy -p signet -- -D warnings` green.

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet` |
| L2 | `cargo clippy -p signet -- -D warnings` |
| L3 | Manual `signet inspect --file <signed.exe>` on maintainer Windows (optional) |
