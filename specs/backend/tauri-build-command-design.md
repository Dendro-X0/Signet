# Design: Tauri `build_command` override (monorepo)

**Band:** 0.5.x dogfood friction  
**Status:** ready  
**Depends on:** Tauri adapter, Electron `build_command` pattern  
**Owners:** `crates/signet/src/artifact/tauri.rs`  
**Plan alignment:** Miro (and similar) need `pnpm desktop:release` from workspace root — raw `tauri build` from `src-tauri` is insufficient / wrong cwd for monorepos.

## Problem

`TauriAdapter::build` always invokes the Tauri CLI and ignores `[project].build_command`. Dogfood on Miro failed after a partial Next export because Signet cannot run the project’s documented release script; users must leave Signet and rebuild manually.

Separately, Miro’s `frontendDist` was one directory short (`../miro-web/out` → under `miro-desktop/`, not `apps/miro-web/out`).

## Goals

1. If `build_command` is non-empty, run it from **project root** (`signet.toml` dir), same argv split as CLI/Electron adapters; then discover as today.
2. If empty, keep current `tauri build` via `find_tauri_cli` from resolved `src-tauri`.
3. Empty-hint mentions `build_command` and monorepo release scripts.

## Non-goals

- Parsing `tauri.conf.json` / auto-fixing `frontendDist`
- Changing default discover paths

## Windows note

`build_command` is spawned via `cmd /C` on Windows so Node package-manager shims (`pnpm` / `npm`) resolve. Without that, CreateProcess reports `program not found` for extensionless npm PATH entries.

## Proof

| Layer | Evidence |
|-------|----------|
| L1 | Unit: non-empty build_command uses shell path (parse/split test); empty keeps CLI path |
| L1 | `cargo test -p signet` + clippy |
| L2 | Miro: `frontendDist` → `../../miro-web/out`; `signet build` or `pnpm desktop:release` produces bundle |

## Related Miro fix (outside this repo)

`apps/miro-desktop/src-tauri/tauri.conf.json`: `frontendDist` = `../../miro-web/out`.
