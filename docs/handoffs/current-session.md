# Current session handoff

**Updated:** 2026-08-01  
**Band:** **0.5.x** toward v1.0 (v0.5.0 preview tagged)

## Next atomic step

Complete real-app dogfood (beyond partial [`docs/dogfood/signet-cli-notes.md`](../dogfood/signet-cli-notes.md)) → friction fixes → then v1.0.0 gate per [`specs/backend/v0.5-release-roadmap.md`](../../specs/backend/v0.5-release-roadmap.md).

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none (spot-check install+verify after `release-cli` for v0.5.0)

## Canonical owners

| Work | Owner |
|------|--------|
| v0.5.0 cut | CHANGELOG, Cargo.toml `0.5.0`, `docs/dogfood/` |
| 0.5.x dogfood | `docs/dogfood/`, demo recording |
| v1.0.0 | full gate in v0.5-release-roadmap |

## Recently completed

- Local CI gate: `cargo test -p signet` + `clippy -D warnings` (clippy fixes in scan)
- v0.5.0 cut docs + partial Signet CLI dogfood notes
- CLI discover Unix CI fix; CLI detection + TUI polish; Phases 13–15
