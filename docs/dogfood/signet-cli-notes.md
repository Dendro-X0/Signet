# Dogfood notes — Signet (this repo)

**Status:** partial (0.5.x) — CLI path exercised; installable GUI app still open  
**App:** Signet CLI itself (`Dendro-X0/Signet`)  
**Framework:** `cli` (Rust workspace — not an installable UI app)  
**Date:** 2026-08-01  
**Host:** Windows (maintainer machine); CI also covers ubuntu/macos unit tests

## Goal

Exercise Sign → Prove → Check on the Signet workspace. Full third-party **installable app** dogfood remains the v1.0 gate item.

## Commands used

```bash
cargo run -p signet -- scan
# suggested: framework=cli, root=., next tag from workspace version

cargo run -p signet -- doctor
cargo test -p signet
cargo clippy -p signet -- -D warnings

# Discover + checksums against release binary (after cargo build --release):
cargo run -p signet -- build --skip-build --no-sign

# Demo fixture path (Electron fake installers):
./demo/scripts/happy-path.ps1   # or happy-path.sh
```

Guided Init auto-suggests `framework=cli`. Committed `signet.toml` sets `framework = "cli"`. If `framework` is omitted, build/doctor resolve via scan (no silent Tauri).

## Outcome

| Step | Result |
|------|--------|
| Scan / detect | Prefers Rust workspace over nested `demo/fixture` Electron |
| Config | Explicit `framework = "cli"`; omitted key would also resolve to `cli` |
| Doctor | Host tooling OK on Windows |
| Demo happy-path | Completes Sign→Prove→Check on fixture (`--no-sign` for fake PE) |
| Self-sign Signet.exe | Optional locally; release assets use GitHub `release-cli` + checksums |

## Blockers / friction (addressed)

- Nested demo Electron skewed init → path preference + `cli` adapter.
- Unix CI executable-bit for CLI discover → fixed.
- Omitted `framework` → silent Tauri → now scan-resolve + explicit `cli` in this repo.
- Blind `v0.1.0` release tags → version-aware defaults.
- Windows PATH / Cursor / cargo shadow → 0.5.1–0.5.3 installers.

## Still open for v1.0

- [ ] Dogfood on ≥1 real installable app (Tauri/Electron/…) with successful host sign + verify
- [ ] Link recording / GIF from `docs/demo.md`
- [ ] Spot-check install + `signet verify` against latest release `SHA256SUMS`

## Time-to-first-useful check

~ minutes for `scan` + demo happy-path once Signet is on `PATH` / `cargo run`.
