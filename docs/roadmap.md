# Roadmap

Status reflects intent. Only mark a phase done when the repo contains matching implementation.

## Phase 0 — Product definition

- [x] Lock Rust, binary name `selfsign`, CLI + TUI, three-platform scope
- [x] Write `START-HERE.md`, `docs/product.md`, this roadmap
- [ ] Maintainer review / tweak of command names and trust wording

## Phase 1 — CLI skeleton

- [x] Cargo workspace / binary `selfsign`
- [x] `clap` subcommands + config schema + TUI hub + doctor

## Phase 2 — Identity + trust kit

- [x] Create / import / show identity; fingerprint; `trust` → TRUST.md

## Phase 3 — Sign per platform

- [x] Windows / macOS / Linux sign backends; `selfsign build`; doctor tooling checks

## Phase 4 — Release

- [x] Artifact collection + GitHub Releases (`gh` or token API) + TRUST attach

## Phase 5 — TUI polish (current)

- [x] Status-aware hub (config / identity / trust / artifacts)
- [x] Guided setup wizard + guided init / identity / build / release
- [x] Same `commands::*` engines as CLI
- [x] Low-clutter cyan-accent terminal UI; non-TTY hint for agents

**Exit:** New users can complete a first release path without memorizing flags (via Guided setup).

**Verify:**

```bash
cargo test -p selfsign
cargo run -p selfsign -- doctor
# interactive:
cargo run -p selfsign
```

## Later (explicitly after v1)

- Paid-cert / notarization *helpers* (still honest about cost and gates)
- Non-Tauri binary support
- Optional desktop GUI
- Update channels / repo metadata beyond checksums + Releases

## Working rule

Do not claim a platform is “supported” in README until Phase 3 exit criteria for that OS are met in-repo.
