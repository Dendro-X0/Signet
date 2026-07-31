# Roadmap

Status reflects intent. Only mark a phase done when the repo contains matching implementation **and** the phase’s design spec (if any) is marked implemented.

**Spec gate:** Phases 6+ require a design under [`specs/backend/`](../specs/backend/) before code. Stubs mean **do not implement**.

## Done — through v0.2.0

- [x] Product definition, Signet rebrand, multi-framework thesis
- [x] CLI skeleton: `init`, `identity`, `trust`, `build`, `release`, `doctor`, `scan`, TUI
- [x] Identity + TRUST.md
- [x] Windows / macOS / Linux host signing; Tauri `build`
- [x] GitHub Releases + checksums
- [x] TUI guided flows

**Verify (baseline):**

```bash
cargo test -p signet
cargo run -p signet -- doctor
```

## Phase 6 — Trust clarity

**Spec:** [trust-tiers-and-verify-design.md](../specs/backend/trust-tiers-and-verify-design.md) (Phase 6 section)  
**Public doc:** [trust-model.md](trust-model.md)

- [x] Tier ids in `TRUST.md` + doctor
- [x] Root-install anti-pattern in generated trust copy
- [x] Align docs with integrity vs reputation layers

**Exit:** A reader of `TRUST.md` can name the tier and know self-sign ≠ OS reputation.

## Phase 7 — Verify

**Spec:** [trust-tiers-and-verify-design.md](../specs/backend/trust-tiers-and-verify-design.md)

- [x] `signet verify` (fingerprint + SHA256SUMS)
- [x] Exit codes 0/1/2; JSON report
- [x] Unit + fixture proof (L1–L3 in spec)

**Exit:** Agents can fail a CI job on checksum/fingerprint mismatch without custom scripts.

## Distribution — CLI self-install / update / uninstall

**Spec:** [self-update-design.md](../specs/backend/self-update-design.md) · **User doc:** [install.md](install.md)

- [x] One-command installers (`install.sh` / `install.ps1`)
- [x] `signet self status|update|uninstall`
- [x] TUI Update / Uninstall Signet
- [x] Tag workflow `release-cli.yml` for binaries + SHA256SUMS

**Exit:** Users can install with one command and manage the CLI from the hub without cargo.

## Phase 8 — Checksum signing


**Spec:** [checksum-signing-design.md](../specs/backend/checksum-signing-design.md)

- [ ] `signet sums-key` + minisign on `SHA256SUMS`
- [ ] Optional GPG `.asc`
- [ ] Wire build/release/verify/`--require-sig`

**Exit:** Release assets include verifiable signed checksums; tier can report `community_signed_sums`.

## Phase 9 — Artifact contract

**Spec:** [artifact-contract-design.md](../specs/backend/artifact-contract-design.md) (stub → ready before code)

- [ ] Framework-agnostic discover → sign → sums → release types
- [ ] Tauri path refactored behind the contract

**Exit:** A second adapter can plug in without forking release/trust.

## Phase 10 — Electron adapter

**Spec:** [electron-adapter-design.md](../specs/backend/electron-adapter-design.md) — **blocked on Phase 9**

- [ ] Discover Electron Builder/Forge outputs
- [ ] Optional build command wrap
- [ ] Reuse host sign + sums + release

## Phase 11 — Android helpers

**Spec:** [android-signing-design.md](../specs/backend/android-signing-design.md) — **blocked on Phases 6–9**

- [ ] Keystore create/import under `.signet/`
- [ ] APK sign via platform tools
- [ ] Honest Play App Signing documentation (external)

## Phase 12 — iOS packaging notes / IPA helpers

- [ ] Design stub + honest free-provisioning (7-day) docs
- [ ] IPA packaging helpers where feasible without claiming App Store trust

## Later

- OV / Azure Artifact Signing / notarization *helpers* (graduation ladder)
- Flutter / React Native / Capacitor adapters
- Optional desktop GUI
- Update channels beyond checksums + GitHub Releases

## Working rules

1. No README “supported” claim until build/sign for that path exists in-repo. Scan-only = awareness.
2. Never instruct end users to install certificates into Trusted Root.
3. One open implementation band at a time unless the handoff says otherwise — see [`docs/handoffs/current-session.md`](handoffs/current-session.md).
