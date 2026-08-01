# Current session handoff

**Updated:** 2026-08-01  
**Band:** **0.5.x** — version-aware defaults + detection polish

## Next atomic step

Complete real-app dogfood (beyond partial [`docs/dogfood/signet-cli-notes.md`](../dogfood/signet-cli-notes.md)) → then v1.0.0 gate per [`specs/backend/v0.5-release-roadmap.md`](../../specs/backend/v0.5-release-roadmap.md).

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none

## Canonical owners

| Work | Owner |
|------|--------|
| Version / tag defaults | `version_detect.rs`, `tui/flows.rs`, `commands/release.rs` |
| 0.5.x dogfood | `docs/dogfood/` |
| v1.0.0 | full gate in v0.5-release-roadmap |

## Recently completed

- Version-aware release tag defaults + framework mismatch note (scan vs signet.toml)
- v0.5.3: install.ps1 curl retries
- v0.5.2: `~/bin` mirror for Git Bash/Cursor
