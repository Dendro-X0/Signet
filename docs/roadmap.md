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

- [x] `signet sums-key` + minisign on `SHA256SUMS`
- [x] Optional GPG `.asc`
- [x] Wire build/release/verify/`--require-sig`

**Exit:** Release assets include verifiable signed checksums; tier can report `community_signed_sums`.

## Phase 9 — Artifact contract

**Spec:** [artifact-contract-design.md](../specs/backend/artifact-contract-design.md)

- [x] Framework-agnostic discover → sign → sums → release types
- [x] Tauri path refactored behind the contract

**Exit:** A second adapter can plug in without forking release/trust.

## Phase 10 — Electron adapter

**Spec:** [electron-adapter-design.md](../specs/backend/electron-adapter-design.md)

- [x] Discover Electron Builder/Forge outputs
- [x] Optional build command wrap
- [x] Reuse host sign + sums + release

## Phase 11 — Android helpers

**Spec:** [android-signing-design.md](../specs/backend/android-signing-design.md) · **User doc:** [android.md](android.md)

- [x] Keystore create/import under `.signet/`
- [x] APK sign via platform tools
- [x] Honest Play App Signing documentation (external)

## Phase 12 — iOS packaging notes / IPA helpers

**Spec:** [ios-signing-design.md](../specs/backend/ios-signing-design.md) · **User doc:** [ios.md](ios.md)

- [x] Design + honest free-provisioning (7-day) docs
- [x] IPA packaging helpers without claiming App Store trust

## Graduation helpers — OV / Azure / notarize

**Spec:** [graduation-helpers-design.md](../specs/backend/graduation-helpers-design.md) · **User doc:** [graduation.md](graduation.md)

- [x] Design + honesty docs (ladder; no SmartScreen/Gatekeeper guarantees)
- [x] `signet graduate notes|ov-sign|azure-sign|notarize|staple`
- [x] Optional `[graduation]` config; doctor / TRUST / scan hooks

## Later (engine — done)

- [x] Host signature inspect (`signet inspect`) — signed/unsigned/adhoc/unknown per artifact platform
- [x] Flutter / React Native / Expo / Capacitor adapters (`docs/frameworks.md`)

## v0.5.0 — Public preview cut

**Roadmap spec:** [v0.5-release-roadmap.md](../specs/backend/v0.5-release-roadmap.md)  
**Program:** [public-release-readiness-design.md](../specs/backend/public-release-readiness-design.md)  
**Cut detail:** [demo-and-dogfood-design.md](../specs/backend/demo-and-dogfood-design.md) §Cut

Goal: easy setup, dual path (self-sign **or** official facilitation), low learning curve (one demo), signing+verification-first packaging — **before** more frameworks/GUI.

```text
v0.4.0 → (main since 0.4.0) → Phase 16 cut → v0.5.0 preview → 0.5.x → v1.0.0
```

### Already on main (credit toward 0.5.0)

Public-release Phases **13 → 14 → 15** are done (order was frozen; only Phase 16 remains).

#### Phase 13 — Product narrative & dual-path docs

**Spec:** [public-release-readiness-design.md](../specs/backend/public-release-readiness-design.md) §13

- [x] README 2-minute story; Sign → Prove → Check
- [x] Align `product.md` with shipped commands (inspect, graduate, hybrid)
- [x] Dual-path: self-signed vs OV/Azure/notarize/Play honesty
- [x] Remove stale “Tauri-only / adapters later” claims where false

#### Phase 14 — Golden-path onboarding

**Spec:** [golden-path-onboarding-design.md](../specs/backend/golden-path-onboarding-design.md)

- [x] TUI Guided setup uses Sign / Prove / Check
- [x] Framework pick from scan + `build_command` when required
- [x] Guided ends with verify/inspect Check
- [x] Hub exposes Verify / Inspect (+ graduate hint)

#### Phase 15 — Demo kit

**Spec:** [demo-and-dogfood-design.md](../specs/backend/demo-and-dogfood-design.md) §Demo

- [x] `demo/` fixture + happy-path scripts
- [x] `docs/demo.md` recording guide
- [x] README links the demo

#### Engine & UX since v0.4.0

- [x] Host signature inspect (`signet inspect`) — also listed under Later (engine)
- [x] Flutter / React Native / Expo / Capacitor adapters (`docs/frameworks.md`)
- [x] CLI / Rust workspace detection (`framework = "cli"`); prefer root over nested `demo/` / fixtures ([project-kind-detection-design.md](../specs/backend/project-kind-detection-design.md))
- [x] Guided init auto-suggests detected framework
- [x] TUI hub + console: shared cyan panels and TTY-aware color hierarchy

### Phase 16 — Cut to v0.5.0

**Specs:** [v0.5-release-roadmap.md](../specs/backend/v0.5-release-roadmap.md) · [demo-and-dogfood-design.md](../specs/backend/demo-and-dogfood-design.md) §Cut

- [ ] Real-app dogfood notes (`docs/dogfood/<app>-notes.md`) — **partial OK** for preview
- [ ] CHANGELOG Unreleased → `## 0.5.0` (include CLI detect + TUI polish)
- [ ] Workspace version bump `0.4.0` → `0.5.0`; START-HERE / README status
- [ ] `cargo test -p signet` + clippy `-D warnings`
- [ ] Tag **v0.5.0** → `release-cli` green
- [ ] Spot-check install + `signet verify` against release `SHA256SUMS`
- [ ] Release body: dual-path + demo link

**Exit:** Public preview tag matches the “official enough to demo” bar.  
**Blocked for coding:** maintainer dogfood notes (process + tag).

### After tag — 0.5.x → v1.0.0

| Band | Focus | Exit |
|------|--------|------|
| **0.5.x** | Complete dogfood; friction fixes from notes; publish demo recording; unmanaged-cargo install honesty if needed | Successful Sign→Prove→Check on ≥1 real app documented |
| **v1.0.0** | Phase 16 acceptance fully checked; narrative stable; no “preview” framing | Tag per full gate in cut specs |

## Beyond v1.0

Deferred — do not pull into the 0.5.0 cut:

- Optional desktop GUI
- Homebrew / winget
- Host-sign / notarize Signet CLI itself (or document checksum-only honesty)
- .NET / other desktop ecosystems
- Update channels beyond checksums + GitHub Releases
- Deeper EAS / Play / App Store Connect automation

## Working rules

1. No README “supported” claim until build/sign for that path exists in-repo. Scan-only = awareness.
2. Never instruct end users to install certificates into Trusted Root.
3. One open implementation band at a time unless the handoff says otherwise — see [`docs/handoffs/current-session.md`](handoffs/current-session.md).
4. Public-release program order is frozen: **13 → 14 → 15 → 16** (13–15 done; **16** is the open cut band).
5. Version policy: **v0.5.0** = preview (partial dogfood OK); **v1.0.0** = full dogfood + stable narrative — see [v0.5-release-roadmap.md](../specs/backend/v0.5-release-roadmap.md).
