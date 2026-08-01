# Dogfood notes — Signet (this repo)

**Status:** partial (v0.5.0 preview)  
**App:** Signet CLI itself (`Dendro-X0/Signet`)  
**Framework:** `cli` (Rust workspace — not an installable UI app)  
**Date:** 2026-08-01  
**Host:** Windows (maintainer machine); CI also covers ubuntu/macos unit tests

## Goal

Exercise Sign → Prove → Check on the Signet workspace before the public preview tag. Full third-party app dogfood remains a **0.5.x** item.

## Commands used

```bash
cargo run -p signet -- scan
# suggested: framework=cli, root=.

cargo run -p signet -- doctor
cargo test -p signet
cargo clippy -p signet -- -D warnings

# Demo fixture path (Electron fake installers):
./demo/scripts/happy-path.ps1   # or happy-path.sh
```

Guided Init on this repo now auto-suggests `framework=cli` instead of defaulting to Tauri.

## Outcome

| Step | Result |
|------|--------|
| Scan / detect | Correctly prefers Rust workspace over nested `demo/fixture` Electron |
| Doctor | Host tooling OK on Windows |
| Demo happy-path | Completes Sign→Prove→Check on fixture (`--no-sign` for fake PE) |
| Self-sign Signet.exe | Not required for this partial note; release assets use GitHub `release-cli` |

## Blockers / friction

- Nested demo Electron markers previously skewed init toward Electron — fixed by path preference + `cli` adapter.
- Unix CI required executable-bit handling for CLI discover tests — fixed before tag.
- No separate consumer GUI app dogfood yet (preview allows partial notes).

## Time-to-first-useful check

~ minutes for `scan` + demo happy-path once Signet is on `PATH` / `cargo run`.

## Follow-up (0.5.x)

- [ ] Dogfood on ≥1 real installable app (Tauri/Electron/…) with successful host sign + verify
- [ ] Link recording / GIF from `docs/demo.md`
