# Product: selfsign

## Problem

Tauri already delivers one codebase and relatively small installers across OSes. The recurring tax for indie and OSS maintainers is **signing and install trust**:

- Each OS has different tools, key formats, and warning UX (SmartScreen, Gatekeeper, package trust).
- Paid developer memberships raise authority but are costly or undesirable for non-profit OSS.
- Self-signing works, but the knowledge is fragmented, easy to get wrong, and hostile to agents/scripts.

Maintainers want a **single, boring workflow** that produces signed artifacts plus an honest trust story users can follow.

## Thesis

`selfsign` owns the **certificate/identity lifecycle, platform signing hooks around Tauri builds, trust documentation, and release packaging** for self-signed apps.

Authority stays honest: self-signed builds may still show OS warnings. The product win is **repeatability, clarity, and agent-accessible automation**—not fake “verified publisher” status.

## Who it is for

- Independent developers shipping Tauri apps outside app stores
- Non-profit / OSS maintainers who prefer self-signing over paid programs
- Humans and coding agents driving releases from an IDE terminal

## Surfaces

### CLI (primary)

Scriptable, non-interactive when flags and config are present. Suitable for CI and agents.

Planned command shape (names may refine; intent is stable):

| Command | Purpose |
|---------|---------|
| `selfsign init` | Project config + local signing layout |
| `selfsign identity` | Create, import, list, show fingerprint |
| `selfsign build` | Invoke Tauri build and sign platform artifacts |
| `selfsign trust` | Emit trust/install docs (fingerprint, verify steps, why warnings appear) |
| `selfsign release` | Checksums + publish artifacts (e.g. GitHub Releases) |
| `selfsign doctor` | Check tooling/prereqs per host OS |
| `selfsign` (no args) | Enter TUI hub |

### TUI (interactive layer)

Clean, attractive terminal UI for guided flows when the user is present:

- First-run init and identity creation
- Platform-aware build/sign checklist
- Trust kit preview
- Release confirmation

TUI wraps the same operations as the CLI; it must not invent a second behavior path.

### GUI

Out of scope until the CLI contract is stable and useful.

## Platform goals (v1)

All three targets are **in scope** for a real v1—not stubs forever. Depth will still land in phases (see roadmap), but each OS must reach a usable signed-artifact path.

### Windows

- Create or import a code-signing certificate suitable for self-signed EXEs/MSIs (and related Tauri bundles).
- Sign build outputs with the project identity.
- Document SmartScreen reality: warnings are expected without reputation / paid certs; provide fingerprint and verify steps.

### macOS

- Support local/ad-hoc or self-managed signing suitable for distribution outside the App Store where tooling allows.
- Be explicit about Gatekeeper and the notarization cliff (Apple-gated; not claimed as solved by self-sign alone).
- Document what users must allow / verify for OSS installs.

### Linux

- Sign or attach integrity metadata appropriate to chosen bundle formats (e.g. AppImage, deb, or rpm—aligned with what Tauri emits for the project).
- Prefer formats and metadata that make updates and checksum verification straightforward.
- Usually fewer scary OS dialogs; still ship a clear trust kit.

## Trust model (product honesty)

`selfsign` must never imply:

- SmartScreen / Gatekeeper silence equivalent to paid Apple/Microsoft programs
- App Store or Play Store distribution
- Notarization as a free/self-signed default on macOS

It **should** always produce:

- Stable identity fingerprints
- Checksums for release artifacts
- Human-readable install/trust instructions (`TRUST.md` or equivalent)
- Agent-readable structured status (exit codes + concise stderr/stdout)

## Configuration & secrets

- Project-level config checked into git (non-secret): app id, platforms, release targets, public fingerprint references.
- Key material **never** committed; local encrypted or OS keychain/store as implementation allows.
- `doctor` and clear errors when secrets or platform tools are missing.

## Success criteria (product)

A maintainer (or agent) can, on a supported host:

1. `selfsign init` once for a Tauri app
2. Create or import an identity
3. `selfsign build` and get signed artifacts for the host’s target OS (and documented cross-build limits)
4. `selfsign trust` and get install guidance that matches reality
5. `selfsign release` and publish artifacts + checksums without hand-rolled scripts

Cross-compilation and remote builders may be limited early; product docs must state host→target matrix honestly as implementation lands.

## Non-goals (near term)

- Full desktop GUI
- Purchasing or automating paid Apple Developer / Microsoft Authenticode EV enrollment
- Guaranteeing warning-free installs on Windows/macOS
- Replacing Tauri itself (wrap and extend; do not fork the app framework)
- Arbitrary Electron/non-Tauri binary signing as a v1 promise (may come later)

## Positioning one-liner

**selfsign — CLI + TUI to identity, sign, explain, and release self-signed Tauri apps across Windows, macOS, and Linux.**
