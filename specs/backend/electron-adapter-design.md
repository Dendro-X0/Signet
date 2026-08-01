# Design: Electron adapter

**Phase:** 10  
**Status:** implemented  
**Depends on:** artifact contract (Phase 9); integrity Phases 6–8  
**Owners:** `crates/signet/src/artifact/electron.rs`, `artifact/adapter.rs`, `config.rs`, `scan/report.rs`  
**Plan alignment:** second `FrameworkAdapter` without forking host sign / sums / release.

## Problem

`signet scan` detects Electron, but build/release still require a Tauri tree. Maintainers need discover → host_sign → sums → sums_sign → release for Electron Builder / Forge outputs.

## Goals

1. `framework = "electron"` selects `ElectronAdapter`.
2. Discover installers under common output dirs (`dist/`, `out/`, `release/`).
3. Optional build wrap via configurable command (default `npm run dist`).
4. Reuse Phase 8–9 pipelines unchanged.

## Non-goals

- Injecting `win.certificateFile` / `@electron/osx-sign` into Electron config (Signet signs **post-bundle** artifacts).
- Auto-purchasing CA certificates.
- Replacing Forge makers’ internal signing.

---

## Config

```toml
[project]
name = "my-app"
framework = "electron"
tauri_root = "."           # app root (package.json parent); name kept for compat
build_command = ""         # empty → default `npm run dist`; else space-separated argv
```

| Field | Electron meaning |
|-------|------------------|
| `tauri_root` | Relative app root (legacy field name; path to `package.json` dir) |
| `framework` | `"electron"` |
| `build_command` | Optional override, e.g. `npx electron-builder --publish never` |

`--tauri-arg` on `signet build` still appends to the Electron build argv (shared `BuildOpts.extra_args`).

---

## Discover

Search under `app_root = project_root / tauri_root`:

| Directory | Notes |
|-----------|--------|
| `dist/` | electron-builder default |
| `out/` | Electron Forge default |
| `release/` | common alternate |

Walk depth-capped (max 8). Collect files/dirs classified by `ArtifactKind::classify_*` (exe, msi, dmg, appimage, deb, rpm, zip, `.app`). Skip `node_modules`, `.git`, `.signet`.

`profile` is ignored for path layout (no Cargo profile); kept in the trait for API symmetry.

### Dedup

Same path once; sort by path for stable SHA256SUMS.

---

## Build

When not `--skip-build`:

1. Resolve app root; require `package.json` (warn if missing, still attempt command).
2. Require `npm` or `npx` on PATH (`doctor` optional check when framework=electron).
3. Run `build_command` if set; else `npm run dist`.
4. Append `BuildOpts.extra_args`.
5. Fail with status if non-zero.

**Decision (frozen):** default `npm run dist` (configurable). Prefer post-bundle Signet signing over injecting Electron cert paths.

---

## Scan

Update Electron scan hint: suggest `framework = "electron"` and `signet sums-key` / `signet build --skip-build` when installers already exist — not “Phase later”.

---

## Module layout

| Path | Role |
|------|------|
| `artifact/electron.rs` | `ElectronAdapter` + discover walk |
| `artifact/adapter.rs` | `select_adapter` match `"electron"` |
| `config.rs` | `project.build_command` |

---

## Acceptance

- [x] Design status `ready` then `implemented`.
- [x] `framework = "electron"` discovers fixture under `dist/`.
- [x] Build command configurable; default documented.
- [x] Host sign / sums / release unchanged (call adapter only).
- [x] `cargo test -p signet` + clippy `-D warnings`.

**Status:** implemented (2026-07-31)

## Proof plan

| Layer | Evidence |
|-------|----------|
| L1 | Unit: discover exe under `dist/`; select_adapter electron |
| L2 | `cargo test -p signet` + clippy |
| L3 | Fixture project: `signet build --skip-build --no-sign --config …` |

## Subtraction

- Do not fork `sign_host_artifacts` / `maybe_sign_sums` / GitHub publish.
- Do not parse full `electron-builder.yml` in v1 (directory conventions only).

## Open questions — resolved

| Question | Decision |
|----------|----------|
| Post-bundle vs inject Electron cert config? | **Post-bundle** Signet host sign only |
| Default build command? | **`npm run dist`**, override via `build_command` |
