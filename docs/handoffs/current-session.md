# Current session handoff

**Updated:** 2026-08-02  
**Band:** **0.5.x** — v0.5.4 cut (omit-framework resolve + version-aware defaults)

## Next atomic step

Dogfood on ≥1 **real installable app** (Miro Tauri / other) with successful Sign→Prove→Check notes → then v1.0.0 gate per [`specs/backend/v0.5-release-roadmap.md`](../../specs/backend/v0.5-release-roadmap.md).

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none (dogfood needs a real app)

## Canonical owners

| Work | Owner |
|------|--------|
| Omit-framework resolve | `project.rs` (`resolve_framework`), `config.rs`, `artifact/adapter.rs` |
| Version / tag defaults | `version_detect.rs`, `tui/flows.rs`, `commands/release.rs` |
| 0.5.x dogfood | `docs/dogfood/` |
| v1.0.0 | full gate in v0.5-release-roadmap |

## Recently completed

- **v0.5.4:** omit-framework scan resolve; version-aware release tags; `framework = "cli"` in this repo
- v0.5.3: install.ps1 curl retries
- v0.5.2: `~/bin` mirror for Git Bash/Cursor
