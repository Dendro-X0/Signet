# Design: iOS packaging notes / IPA helpers

**Phase:** 12  
**Status:** implemented  
**Depends on:** Phases 6–9 (trust honesty, verify, artifact contract); Phase 11 pattern for mobile docs  
**Owners:** `crates/signet/src/ios/`, `commands/ios.rs`, `artifact/ios.rs`, `docs/ios.md`, `trust_kit.rs`, `commands/doctor.rs`  
**Plan alignment:** help local/dev IPA packaging; never claim App Store, TestFlight, or notarization trust.

## Problem

iOS distribution is gated by Apple’s programs. Free Apple ID **development** provisioning typically expires in **~7 days**. Paid Developer Program enables Ad Hoc / App Store / TestFlight. Signet must document this honestly and offer only helpers that do not pretend otherwise.

## Goals

1. Public doc (`docs/ios.md`) covering free provisioning (7-day), development vs distribution, App Store out of scope.
2. `signet ios package` — build a `.ipa` from an existing `.app` (`Payload/` zip layout).
3. Optional `framework = "ios"` adapter: discover `.app` / `.ipa` under common build output dirs; optional `build_command` (e.g. `xcodebuild`).
4. Doctor hints for `codesign` / `xcodebuild` when framework is ios (macOS host).
5. TRUST.md iOS honesty subsection (no private keys; no “install this cert into trust store”).

## Non-goals

- App Store Connect upload, notarization, or paid certificate purchase.
- Claiming free provisioning lasts longer than Apple allows.
- Replacing Xcode’s automatic signing UI for complex multi-target apps.
- Running full iOS simulator/device install orchestration (document only).

---

## Trust honesty (required)

| Channel | Signet role |
|---------|-------------|
| Free Apple ID / development | Document ~7-day provisioning; package IPA from already-signed `.app` when present |
| Ad Hoc (paid) | Document only — maintainer uses Apple tooling |
| App Store / TestFlight | **Out of scope** — never claim Signet enables store trust |

Never instruct end users to install developer certificates into system trust stores.

Tier: use `apple_notarized` / store tiers only when **declared** by maintainer for real notarized/store builds — not inferred from a local `.ipa`.

---

## IPA packaging

An IPA is a zip archive:

```text
Payload/
  AppName.app/
```

`signet ios package --app path/to/App.app [--out path/to/App.ipa]`:

1. Validate `.app` is a directory (bundle).
2. Stage `Payload/<name>.app` in a temp dir (copy or hardlink tree).
3. Zip to `.ipa` (store paths with forward slashes; no extra top-level junk).
4. Print honesty note: packaging ≠ App Store distribution; free provisioning may expire in ~7 days.

Cross-platform: zip packaging works wherever the `.app` tree is available (typically produced on macOS).

---

## `framework = "ios"` adapter

| Stage | Behavior |
|-------|----------|
| `label_root` | `tauri_root` as Xcode/Tauri iOS project root |
| `build` | `build_command` if set; else try `xcodebuild -scheme …` only when clearly configured — **v1: require `build_command` or `--skip-build`** (avoid guessing schemes) |
| `discover` | Walk common outputs for `.app` / `.ipa` (depth-capped): `build/`, `DerivedData`-style relative dirs under app root, `dist/`, `release/`, Tauri `src-tauri/gen/apple` / gen ios paths |

Signing: reuse existing macOS host `codesign` path when desktop `.app` is host-signable; for iOS `.app`, do **not** invent a second identity store in Phase 12 — package only, document that signing is Xcode/Apple’s job.

---

## CLI

```text
signet ios package --app App.app [--out App.ipa]
signet ios notes          # print short honesty summary (or point at docs/ios.md)
```

Optional later (not required for exit): `signet ios verify-app --app` wrapping `codesign -dv`.

---

## Doctor / scan / TRUST

- Doctor (when `framework = "ios"` or always optional): `codesign`, `xcodebuild` presence.
- Scan: update iOS note to point at `docs/ios.md` + `signet ios package`.
- TRUST: short **iOS** subsection with free-provisioning / App Store honesty when platforms.ios or ios framework.

---

## Acceptance

- [x] Design ready → implemented.
- [x] `docs/ios.md` with 7-day free provisioning honesty.
- [x] `signet ios package` creates valid IPA zip layout from fixture `.app`.
- [x] `framework = "ios"` discover works on fixture.
- [x] Doctor + scan + TRUST updates.
- [x] `cargo test -p signet` + clippy `-D warnings`.

**Status:** implemented (2026-07-31)

## Proof plan

| Layer | Evidence |
|-------|----------|
| L1 | Unit: package temp `.app` → `.ipa` contains `Payload/…` |
| L2 | `cargo test -p signet` + clippy |
| L3 | Optional on macOS: package a real `.app` if present |

## Open questions — resolved

| Question | Decision |
|----------|----------|
| Auto `xcodebuild` without scheme? | **No** — require `build_command` or `--skip-build` |
| Sign iOS app with Signet desktop identity? | **No** in v1 — package + docs only |
