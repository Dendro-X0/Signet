# Boot

**Read [README.md](README.md) first**, then this file for contributor/agent detail.

**Product:** Signet  
**Binary:** `signet`  
**Stack:** Rust CLI + TUI (ratatui)  
**Status:** v0.3.1 — see [CHANGELOG.md](CHANGELOG.md)  
**Repo:** [github.com/Dendro-X0/Signet](https://github.com/Dendro-X0/Signet)

## What this is

A developer/agent-friendly CLI for **identity → sign → trust docs → release** of self-signed apps across desktop and mobile frameworks. Tauri is the deepest path today; `scan` already sees Electron / Android / iOS, and more adapters follow.

Self-signing is **not** a substitute for paid platform developer programs. It is the practical path for independent and non-profit OSS: legitimate local trust, clear install guidance, and fewer one-off signing rituals.

## Quick start

```bash
cargo run -p signet -- doctor
cargo run -p signet                 # TUI hub + Guided setup
# or scripted:
cargo run -p signet -- init --name my-app --path ./my-app
cargo run -p signet -- identity create --config ./my-app/signet.toml
cargo run -p signet -- trust --config ./my-app/signet.toml
cargo run -p signet -- build --config ./my-app/signet.toml
cargo run -p signet -- release --tag v0.2.0 --dry-run --config ./my-app/signet.toml
```

## Read next

1. [`docs/product.md`](docs/product.md)
2. [`docs/trust-model.md`](docs/trust-model.md) — integrity vs reputation
3. [`docs/roadmap.md`](docs/roadmap.md) — phases + spec gate
4. [`specs/backend/README.md`](specs/backend/README.md) — design contracts (**read before coding Phases 6+**)
5. [`docs/handoffs/current-session.md`](docs/handoffs/current-session.md) — next atomic step
6. [`docs/tui.md`](docs/tui.md) — hub & guided flows
7. [`docs/scan.md`](docs/scan.md) — repo installer self-check
8. [`docs/identity.md`](docs/identity.md) / [`docs/signing.md`](docs/signing.md) / [`docs/release.md`](docs/release.md)

## Decisions locked

| Decision | Choice |
|----------|--------|
| Language | Rust |
| Product / binary | Signet / `signet` |
| Primary UX | CLI first; guided TUI for humans |
| Scope | Desktop + mobile; multi-framework (Tauri first) |
| Release | GitHub Releases (`gh` or token API) |
| Near-term | Integrity first (trust tiers → verify → checksum signing) before adapters |
| Specs | Hybrid: `docs/` public + `specs/backend/` contracts |
| Current phase | Dist CLI self-update done; next = Phase 8 checksum signing |

## How to work this repo

- Prefer small, reviewable increments.
- Keep docs synchronized with implemented behavior (not aspirational claims).
- **Phases 6+:** design status must be `ready` (not `stub`) before code; follow the handoff atomic step.
- Agents: use subcommands; humans can open the TUI with bare `signet`.
