# Design: config simplification + Check honesty (Phase A)

**Band:** 0.5.x → v0.5.6  
**Status:** ready  
**Depends on:** Miro dogfood; inspect + checksum modules  
**Owners:** `inspect/probe.rs`, `sign/checksum.rs`, `config.rs`, `tui/flows.rs`  
**Plan alignment:** Repo-level signing UX plan — Phase A before `[[targets]]`.

## Problems

1. `inspect` uses `signtool verify /pa` only → self-signed artifacts report **unsigned** after a successful Sign.
2. `SHA256SUMS` stores basenames → `verify` skips all lines when files live under `apps/.../bundle/`.
3. Config field `tauri_root` confuses non-Tauri projects; guided flow asks too many questions when scan already knows the answer.

## Goals

1. Inspect: presence of Authenticode ≠ OS trust; self-signed → status `signed` with honest detail.
2. Sums: write paths relative to the `SHA256SUMS` directory; verify resolves those paths (basename fallback kept).
3. Rename surface to `app_root` (serde alias `tauri_root`); guided/scan `--apply` fill minimal config with defaults.

## Non-goals

- Multi-target `[[targets]]` (Phase B)
- Changing SmartScreen / `/pa` trust
- Auto-editing third-party `tauri.conf.json`

## Windows inspect

1. `signtool verify /v <path>` — parse output for signature presence (`No signature found` → unsigned).
2. If present: status `signed`; run `/pa` only to enrich detail (trusted chain vs self-signed / untrusted root).
3. Never map “signature present but `/pa` failed” to `unsigned`.

## SHA256SUMS

`write_sha256sums`: for each path, prefer `strip_prefix(sums_dir)` with `/` separators; else basename.  
`verify_sha256sums`: resolve relative to `sums_dir` first (already partially done); ensure nested relative names work.

## Config

```toml
[project]
name = "…"
app_root = "apps/miro-desktop"   # alias: tauri_root
framework = "tauri"
build_command = "…"              # optional
```

Serialize as `app_root`. Accept either key on load.

## Proof

| Layer | Evidence |
|-------|----------|
| L1 | Unit: relative sums write/verify; parse_signtool_has_signature |
| L1 | Windows integration: signed PE inspects as signed |
| L1 | `cargo test -p signet` + clippy `-D warnings` |
| L2 | Miro: verify finds files; inspect not false-unsigned |
