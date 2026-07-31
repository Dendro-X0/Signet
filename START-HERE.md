# Boot

**Read [README.md](README.md) first**, then this file for contributor/agent detail.

**Product:** Signet  
**Binary:** `signet`  
**Stack:** Rust CLI + TUI (ratatui)  
**Status:** v0.2.0 — see [CHANGELOG.md](CHANGELOG.md)  
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
2. [`docs/tui.md`](docs/tui.md) — hub & guided flows
3. [`docs/scan.md`](docs/scan.md) — repo installer self-check
4. [`docs/identity.md`](docs/identity.md)
5. [`docs/signing.md`](docs/signing.md)
6. [`docs/release.md`](docs/release.md)
7. [`docs/roadmap.md`](docs/roadmap.md)

## Decisions locked

| Decision | Choice |
|----------|--------|
| Language | Rust |
| Product / binary | Signet / `signet` |
| Primary UX | CLI first; guided TUI for humans |
| Scope | Desktop + mobile; multi-framework (Tauri first) |
| Release | GitHub Releases (`gh` or token API) |
| Current phase | v0.2.0 rebrand + multi-framework thesis |

## How to work this repo

- Prefer small, reviewable increments.
- Keep docs synchronized with implemented behavior (not aspirational claims).
- Agents: use subcommands; humans can open the TUI with bare `signet`.
