# Current session handoff

**Updated:** 2026-08-03  
**Band:** **0.5.x** — v0.5.8 scan-apply hardening + basename resolve

## Next atomic step

Optional: **tag v0.5.8** + push + `release-cli` when ready to publish.

Then v1.0.0 gate: narrative stable / drop preview framing; optional demo recording; spot-check install+verify on release assets — per [`specs/backend/v0.5-release-roadmap.md`](../../specs/backend/v0.5-release-roadmap.md).

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none

## Canonical owners

| Work | Owner |
|------|--------|
| scan --apply platforms / targets | `commands/scan.rs`, `scan/report.rs` |
| Basename resolve / stale | `sign/checksum.rs` |
| Dogfood | `docs/dogfood/miro-notes.md` |
| v1.0.0 | full gate in v0.5-release-roadmap |

## Recently completed

- **v0.5.8:** apply never shrinks platforms without `--force`; draft `[[targets]]` on existing toml; basename walk for sums; skip identity hint when present
- **v0.5.7:** verify stale warnings + `--fail-stale`; post-sign sums log; scan `[[targets]]`/platforms notes; `identity status`
- **v0.5.6:** inspect self-sign honesty; relative SHA256SUMS; `app_root`; `[[targets]]` + `--target`
