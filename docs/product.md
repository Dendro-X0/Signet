# Product: Signet

## Problem

Shipping a desktop or mobile app still collides with **signing and install trust**:

- Each OS (and store) has different tools, key formats, and warning UX.
- Paid developer memberships raise authority but are costly for indie and non-profit OSS.
- Self-signing works, but knowledge is fragmented, easy to get wrong, and hostile to agents/scripts.
- Teams use many stacks (Tauri, Electron, Flutter, React Native, Capacitor, native) — the trust problem repeats each time.

Maintainers want a **single, boring workflow** that produces signed artifacts plus an honest trust story users can follow.

## Thesis

**Signet** owns the **certificate/identity lifecycle, platform signing hooks, trust documentation, and release packaging** for self-signed apps — starting with Tauri desktop, expanding to other desktop and mobile frameworks.

Authority stays honest: self-signed builds may still show OS warnings; store notarization / Play App Signing remain gated. The product win is **repeatability, clarity, and agent-accessible automation**—not fake “verified publisher” status.

## Who it is for

- Independent developers shipping outside (or beside) app stores
- Non-profit / OSS maintainers who prefer self-signing over paid programs where appropriate
- Humans and coding agents driving releases from an IDE terminal

## Surfaces

### CLI (primary)

Binary: `signet`. Scriptable for CI and agents.

| Command | Purpose |
|---------|---------|
| `signet init` | Project config + local signing layout |
| `signet identity` | Create, import, list, show fingerprint |
| `signet build` | Build + sign (Tauri today; adapters next) |
| `signet trust` | Emit trust/install docs |
| `signet release` | Checksums + publish (e.g. GitHub Releases) |
| `signet doctor` | Host tooling / prereqs |
| `signet scan` | Detect frameworks + installers; suggest config |
| `signet verify` | Verify fingerprints + SHA256SUMS (Phase 7) |
| `signet` (no args) | TUI hub |

### TUI

Guided flows wrapping the same CLI engines.

### GUI

Deferred until the CLI contract is stable.

## Platform goals

### Desktop (v1 shipping)

Windows, macOS, Linux — sign installers / bundles; document SmartScreen / Gatekeeper reality.

### Mobile (detect now, deepen next)

Android and iOS projects and artifacts are discovered by `scan`. Store signing helpers stay honest about Apple / Google gates and land as explicit later work.

### Frameworks

| Framework | Scan | Build+sign adapter |
|-----------|------|--------------------|
| Tauri | Yes | Yes (current) |
| Electron | Yes | Planned |
| Flutter / RN / Capacitor / native | Partial / planned | Planned |

## Trust model

Signet must never imply store-equivalent trust from self-signing alone. It should always produce fingerprints, checksums, honest install docs, and clear exit codes.

Integrity vs reputation, trust tiers, and anti-patterns: **[`docs/trust-model.md`](trust-model.md)**.  
Verify downloads: **[`docs/verify.md`](verify.md)**.  
Phase designs: **[`specs/backend/`](../specs/backend/)**.

## Config & secrets

- `signet.toml` — safe to commit (legacy `selfsign.toml` still loaded)
- `.signet/` — private keys, gitignored (legacy `.selfsign/` still detected)

## Positioning one-liner

**Signet — CLI + TUI to identity, sign, explain, and release self-signed apps across desktop and mobile frameworks.**
