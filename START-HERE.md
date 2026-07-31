# Boot

**Read [README.md](README.md) first**, then this file for contributor/agent detail.

**Binary:** `selfsign`  
**Stack:** Rust CLI + TUI (ratatui)  
**Status:** v0.1.0 — see [CHANGELOG.md](CHANGELOG.md)

## What this is

A developer/agent-friendly CLI that makes **self-signed, cross-platform Tauri distribution** repeatable: identity → build/sign → trust docs → release.

Self-signing is **not** a substitute for paid platform developer programs. It is the practical path for independent and non-profit OSS: legitimate local trust, clear install guidance, and fewer one-off signing rituals.

## Quick start

```bash
cargo run -p selfsign -- doctor
cargo run -p selfsign                 # TUI hub + Guided setup
# or scripted:
cargo run -p selfsign -- init --name my-app --path ./my-app
cargo run -p selfsign -- identity create --config ./my-app/selfsign.toml
cargo run -p selfsign -- trust --config ./my-app/selfsign.toml
cargo run -p selfsign -- build --config ./my-app/selfsign.toml
cargo run -p selfsign -- release --tag v0.1.0 --dry-run --config ./my-app/selfsign.toml
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
| Binary name | `selfsign` |
| Primary UX | CLI first; guided TUI for humans |
| Release | GitHub Releases (`gh` or token API) |
| Current phase | v0.1.0 released |

## How to work this repo

- Prefer small, reviewable increments.
- Keep docs synchronized with implemented behavior (not aspirational claims).
- Agents: use subcommands; humans can open the TUI with bare `selfsign`.
