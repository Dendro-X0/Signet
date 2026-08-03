# Design: omitted `framework` resolves via scan

**Band:** 0.5.x friction  
**Status:** ready  
**Depends on:** project-kind detection + version-aware mismatch note  
**Owners:** `config.rs`, `project.rs`, `artifact/adapter.rs`  
**Plan alignment:** Dogfood/friction — stop silent Tauri on CLI (and other) trees when `signet.toml` never set `framework`.

## Problem

`[project].framework` defaults to `"tauri"` when the key is missing. Legacy configs (this repo included) often only set `name` + `tauri_root`, so guided build / discover behave as Tauri despite scan suggesting `cli`. Mismatch note warns but does not fix the wrong adapter.

## Goals

1. Missing or blank `framework` in TOML → resolve via the same scan preference as Init.
2. Explicit `framework = "…"` always wins (mismatch note still applies).
3. In-memory defaults / `init` continue to write an explicit framework (usually `tauri` or user choice).

## Non-goals

- Auto-rewrite of `signet.toml` without Init / user edit
- Changing explicit wrong values
- Multi-target `[[targets]]`

## Resolution

```text
trim(config.project.framework)
  non-empty → use as-is
  empty     → scan_repository(root) → preferred framework id
              else fallback "tauri"
```

Serde: `#[serde(default)]` on `framework` → empty string when key omitted (no longer default `"tauri"` on deserialize). `Config::default` / `example` / `init` still set a concrete string so round-trips stay explicit.

`select_adapter` takes `root` + `config` and uses `resolve_framework` before matching.

## Proof

| Layer | Evidence |
|-------|----------|
| L1 | Unit: TOML without `framework` on a CLI fixture → `cli` |
| L1 | Unit: explicit `framework = "tauri"` unchanged |
| L1 | `cargo test -p signet` + clippy `-D warnings` |
| L2 | This repo `signet.toml` sets `framework = "cli"` (explicit dogfood honesty) |
