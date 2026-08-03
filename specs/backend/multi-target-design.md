# Design: multi-target `[[targets]]` (Phase B)

**Band:** 0.5.x → v0.5.7  
**Status:** ready  
**Depends on:** Phase A (`app_root`, honest Check)  
**Owners:** `config.rs`, `project.rs`, `commands/build.rs`, `release/collect.rs`, `tui/flows.rs`  
**Plan alignment:** Repo-level signing — one `signet.toml` for monorepo apps.

## Problem

Monorepos (Miro) need one Signet root but multiple shippable apps. Today a single `[project].framework` + `app_root` forces one adapter per config file.

## Model

```toml
[project]
name = "Miro"

[[targets]]
id = "desktop"
framework = "tauri"
app_root = "apps/miro-desktop"
build_command = "pnpm desktop:release"
```

- **No `[[targets]]`:** synthesize one target from `[project]` (`id = "default"`).
- **With `[[targets]]`:** each entry has `id`, `framework`, `app_root`, optional `build_command`.
- Shared: `.signet/identity`, one `TRUST.md`, one `SHA256SUMS` (relative paths).
- CLI: `signet build` all targets; `signet build --target desktop` one.

## Resolution

```text
targets = config.targets non-empty
  ? config.targets
  : [Target { id: "default", framework, app_root, build_command from project }]
```

Build loops: for each selected target, temporarily apply target fields onto a `ProjectCtx` view (or clone Config with project overridden), run adapter build/discover, merge artifacts.

## Scan --apply

When multiple projects detected, `--apply` may write `[[targets]]` draft (non-destructive merge when file exists).

## Non-goals

- Parallel builds; dependency graph between targets
- Per-target identity keys
- Auto frontendDist fixes

## Proof

| Layer | Evidence |
|-------|----------|
| L1 | Parse TOML with `[[targets]]`; empty → synthetic default |
| L1 | `cargo test -p signet` + clippy |
| L2 | Fixture or Miro: `--target` filters; all-targets discover unions artifacts |
