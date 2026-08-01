# Product: Signet

## Problem

Shipping a desktop or mobile app still collides with **signing and install trust**:

- Each OS (and store) has different tools, key formats, and warning UX.
- Paid developer memberships raise authority but are costly for indie and non-profit OSS.
- Self-signing works, but knowledge is fragmented, easy to get wrong, and hostile to agents/scripts.
- Teams use many stacks (Tauri, Electron, Flutter, React Native, Capacitor, native) — the trust problem repeats each time.

Maintainers want a **single, boring workflow** that produces signed artifacts plus an honest trust story users can follow — and a clear ladder when they graduate to official CA / Apple / Azure tooling.

## Thesis

**Signet** is dedicated to **app signing and verification**:

| Job | Meaning |
|-----|---------|
| **Sign** | Identity + host/mobile signing, or helpers for OV / Azure / notarize |
| **Prove** | `TRUST.md`, `SHA256SUMS`, optional minisign/GPG, release attach |
| **Check** | `signet verify` (integrity) + `signet inspect` (host signature presence) |

Self-sign is the default Sign path. Official/paid paths are **facilitated, never faked**. The product win is **repeatability, clarity, and agent-accessible automation** — not fake “verified publisher” status.

## Who it is for

- Independent developers shipping outside (or beside) app stores
- Non-profit / OSS maintainers who prefer self-signing over paid programs where appropriate
- Teams that later buy OV / Azure / Apple membership and still want one CLI ladder
- Humans and coding agents driving releases from an IDE terminal

## Dual path

| Path | Signet role | Docs |
|------|-------------|------|
| Self-signed | Local identity, host sign, Android keystore, iOS IPA package helpers | [identity](identity.md), [signing](signing.md), [android](android.md), [ios](ios.md) |
| Official / paid | Wrap OV thumbprint/PFX, Azure Trusted Signing dlib, Apple `notarytool` | [graduation](graduation.md) |

Play App Signing and App Store Connect remain **external**; Signet documents honesty and does not claim store trust from local keys.

## Surfaces

### CLI (primary)

Binary: `signet`. Scriptable for CI and agents.

| Job | Commands |
|-----|----------|
| Sign | `identity`, `build`, `android`, `ios`, `graduate` |
| Prove | `trust`, `sums-key`, `release` |
| Check | `verify`, `inspect` |
| Project | `init`, `scan`, `doctor`, `self` |
| Hub | `signet` (no args) — TUI + guided setup |

### TUI

Guided flows wrapping the same CLI engines. **Guided setup** runs Sign → Prove → Check with framework pick (Phase 14).

### GUI

Deferred until the public-cut CLI/TUI contract is stable ([roadmap](roadmap.md) Beyond).

## Platforms & frameworks

### Desktop

Windows, macOS, Linux — sign installers/bundles; document SmartScreen / Gatekeeper reality.

### Mobile

Android keystore + APK sign; iOS IPA packaging + free-provisioning honesty. Store programs stay gated.

### Framework adapters

| Framework | Scan | Adapter |
|-----------|------|---------|
| Tauri | Yes | Yes |
| Electron | Yes | Yes |
| Android / iOS | Yes | Yes (helpers) |
| Flutter / RN / Expo / Capacitor | Yes | Yes (`build_command` required; see [frameworks](frameworks.md)) |
| .NET / others | No | Beyond public cut |

## What Signet never does

1. Instruct end users to install publisher certs into **Trusted Root**.
2. Claim self-signing removes SmartScreen or Gatekeeper warnings.
3. Put private keys, PFX passwords, or keystore passwords in `TRUST.md` or git.
4. Mark a framework “supported” in README until build/sign for that path exists (scan-only ≠ supported).

## Trust & check

- Tiers + anti-patterns: **[trust-model.md](trust-model.md)**
- Verify + inspect: **[verify.md](verify.md)**
- Designs: **[specs/backend/](../specs/backend/)**
- Public release program: **[roadmap.md](roadmap.md)** Phases 13–16

## Config & secrets

- `signet.toml` — safe to commit (legacy `selfsign.toml` still loaded)
- `.signet/` — private keys, gitignored (legacy `.selfsign/` still detected)

## Positioning one-liner

**Signet — Sign → Prove → Check for self-signed and official app signing, across desktop and mobile frameworks.**
