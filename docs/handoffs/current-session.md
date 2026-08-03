# Current session handoff

**Updated:** 2026-08-03  
**Band:** **0.5.x** — Miro dogfood green; v0.5.5 cut (Tauri build_command + Windows pnpm)

## Next atomic step

v1.0.0 gate: narrative stable / drop preview framing; optional demo recording; spot-check install+verify on release assets — per [`specs/backend/v0.5-release-roadmap.md`](../../specs/backend/v0.5-release-roadmap.md).

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none

## Canonical owners

| Work | Owner |
|------|--------|
| Tauri monorepo build | `artifact/tauri.rs`, `walk_outputs.rs` (`spawn_build_command`) |
| Miro dogfood | `docs/dogfood/miro-notes.md` (+ Miro `frontendDist` fix outside this repo) |
| v1.0.0 | full gate in v0.5-release-roadmap |

## Recently completed

- **v0.5.5:** Tauri `build_command` from project root; Windows `cmd /C` for pnpm/npm; Miro Sign→Prove→Check dogfood
- Miro: `frontendDist` → `../../miro-web/out`
- **v0.5.4:** omit-framework scan resolve; version-aware release tags
