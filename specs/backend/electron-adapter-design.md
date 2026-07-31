# Design stub: Electron adapter

**Phase:** 10  
**Status:** stub — **blocked on Phase 9**  
**Depends on:** artifact contract (Phase 9); integrity Phases 6–8 preferred  

## Problem

`signet scan` already detects Electron markers. Maintainers still need discover → sign → sums → release without a Tauri tree.

## In scope (when implemented)

- Discover Electron Builder / Forge outputs (`dist/`, `out/`, common NSIS/DMG/AppImage names).
- Optional wrap of `electron-builder` / npm scripts via config (`[project] framework = "electron"`, build command).
- Reuse host sign + checksum signing + release unchanged.

## Out of scope

- Replacing `@electron/osx-sign` / Windows sign config inside Electron Forge (Signet signs final artifacts or documents handoff).
- Auto-purchasing CA certificates.

## Scan hooks (already / extend)

- Detect `package.json` + `electron` dependency / `electron-builder` config.
- Suggest `signet.toml` platforms from existing installers.

## Do not implement until

- [`artifact-contract-design.md`](artifact-contract-design.md) is elevated from stub to ready and implemented.
- Maintainer opens Phase 10 in [`docs/handoffs/current-session.md`](../../docs/handoffs/current-session.md).

## Open questions

- Prefer signing only post-bundle artifacts vs injecting Electron `win.certificateFile`?
- Default build command: `npm run dist` vs configurable argv list?
