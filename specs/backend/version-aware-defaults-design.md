# Design: version-aware defaults + detection polish

**Band:** 0.5.x friction  
**Status:** ready  
**Depends on:** scan + guided TUI (Phases 14–15)  
**Owners:** `crates/signet/src/version_detect.rs`, `tui/flows.rs`, `scan/report.rs`, `commands/release.rs`  
**Plan alignment:** Fast Sign→Prove→Check — stop blind `v0.1.0` tags and silent framework mis-labels.

## Problem

Guided release always defaulted `git tag` to `v0.1.0`. Scan next-steps did the same. Projects with a real `Cargo.toml` / `package.json` version felt broken. Separately, a stale `signet.toml` (`framework = "tauri"`) made guided build talk like Tauri on a CLI/Electron tree.

## Goals

1. Detect project version from common manifests (and optional git tag).
2. Default release tags to `v` + that version in TUI and scan suggestions.
3. Warn once when config framework ≠ scan suggestion (no auto-rewrite).

## Non-goals

- Multi-target `[[targets]]`
- Auto-editing `signet.toml` without confirm
- Semver bump / changelog automation

## Version probe order (first hit wins)

1. Root `Cargo.toml`: `[package].version` else `[workspace.package].version`
2. Root `package.json` `"version"`
3. `app.json` / `app.config.json` `"version"` or nested `expo.version` (string)
4. `git describe --tags --abbrev=0` (best-effort)
5. Fallback `"0.1.0"`

`default_release_tag(root)` → ensure leading `v`.

## Framework mismatch

If `signet.toml` loads and scan `preferred` framework differs (case-insensitive, `rn`≡`react-native`), print one note; leave config unchanged.

## Proof

| Layer | Evidence |
|-------|----------|
| L1 | Unit tests for Cargo / package.json / `v`-prefix |
| L1 | `cargo test -p signet` + clippy `-D warnings` |
| L2 | This repo: default tag `v0.5.3` (workspace version) |
