# Roadmap

Status reflects intent. Only mark a phase done when the repo contains matching implementation.

## Phase 0 — Product definition

- [x] Lock Rust, CLI + TUI, three-platform desktop scope
- [x] Rebrand to **Signet**; multi-framework desktop + mobile thesis
- [x] Write `START-HERE.md`, `docs/product.md`, this roadmap
- [ ] Maintainer review / tweak of command names and trust wording

## Phase 1 — CLI skeleton

- [x] Cargo workspace / binary `signet`
- [x] `clap` subcommands + config schema + TUI hub + doctor

## Phase 2 — Identity + trust kit

- [x] Create / import / show identity; fingerprint; `trust` → TRUST.md

## Phase 3 — Sign per platform (desktop)

- [x] Windows / macOS / Linux sign backends; `signet build` (Tauri); doctor tooling checks

## Phase 4 — Release

- [x] Artifact collection + GitHub Releases (`gh` or token API) + TRUST attach

## Phase 5 — TUI polish

- [x] Status-aware hub (config / identity / trust / artifacts)
- [x] Guided setup wizard + guided init / identity / build / release
- [x] Same `commands::*` engines as CLI
- [x] Low-clutter cyan-accent terminal UI; non-TTY hint for agents

**Exit:** New users can complete a first release path without memorizing flags (via Guided setup).

**Verify:**

```bash
cargo test -p signet
cargo run -p signet -- doctor
# interactive:
cargo run -p signet
```

## Phase 6 — Framework adapters (next)

- [ ] Electron build + sign path
- [ ] Flutter / React Native / Capacitor desktop or side-car flows where applicable
- [ ] Deeper Android / iOS helpers (still honest about Play / Apple gates)
- [ ] Shared “artifact contract” so release/trust stay framework-agnostic

## Later

- Paid-cert / notarization *helpers* (still honest about cost and gates)
- Optional desktop GUI
- Update channels / repo metadata beyond checksums + Releases

## Working rule

Do not claim a framework or platform is “supported” in README until build/sign for that path exists in-repo. Scan-only detection is documented as awareness, not support.
