# Current session handoff

**Updated:** 2026-08-03  
**Band:** **0.5.x** — v0.5.6 config simplification + `[[targets]]`

## Next atomic step

v1.0.0 gate: narrative stable / drop preview framing; optional demo recording; spot-check install+verify on release assets — per [`specs/backend/v0.5-release-roadmap.md`](../../specs/backend/v0.5-release-roadmap.md).

Optional: cut **tag v0.5.6** + `release-cli` when ready to publish.

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none

## Canonical owners

| Work | Owner |
|------|--------|
| Inspect honesty / sums paths | `inspect/probe.rs`, `sign/checksum.rs` |
| `app_root` + guided/scan apply | `config.rs`, `tui/flows.rs`, `commands/scan.rs` |
| Multi-target | `config.rs` (`Target`), `project.rs`, `commands/build.rs`, `release/collect.rs` |
| Dogfood | `docs/dogfood/miro-notes.md` |
| v1.0.0 | full gate in v0.5-release-roadmap |

## Recently completed

- **v0.5.6:** inspect self-sign honesty; relative SHA256SUMS; `app_root`; `[[targets]]` + `--target`
- **v0.5.5:** Tauri `build_command` + Windows pnpm `cmd /C`; Miro Sign→Prove→Check
- Miro: `frontendDist` → `../../miro-web/out`
