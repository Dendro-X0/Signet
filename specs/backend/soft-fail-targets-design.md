# Soft-fail unpaid `[[targets]]` (ship slice B)

## Plan alignment

- **Handoff:** shortcomings fix order #2 (items 5–6)
- **Parent:** `specs/backend/multi-platform-ship-design.md` slice B
- **Band:** 0.5.10
- **In scope:** multi-target `signet build` continues after unpaid / failed sibling; debt report; `--strict-targets`
- **Out of scope:** auto recipes for Expo; CI matrix; coverage gate

## Behavior

### Default (all targets, no `--target`)

For each `[[targets]]` entry:

1. If framework requires explicit `build_command`, command empty, and not `--skip-build` → **debt skip** (do not call build; still attempt discover in case artifacts exist).
2. Else run build + discover; on **error** → record debt, continue to next target (do not `?`-abort the loop).
3. After loop: if any artifacts discovered → checksum/sign as today; print debt summary.
4. If **no** artifacts and any debt → fail with combined debt + empty hints.
5. If **no** artifacts and no debt → existing empty hints bail.

### `--target id`

Hard-fail that target (user asked for one surface). No soft-skip.

### `--strict-targets`

Any debt (skip or error) → non-zero exit **after** signing successful siblings (so desktop still gets signed, but CI can fail closed).

### Debt message shape

```text
warning: target mobile (expo) skipped — unpaid recipe: set [[targets]].build_command or pass --target desktop / --skip-build
…
note: target debt: mobile — fix recipe or exclude from default build
```

## Ownership

| Piece | Module |
|-------|--------|
| Loop soft-fail + debt | `commands/build.rs` |
| “Needs build_command” | reuse policy from `tui/framework_pick::requires_build_command` via small `artifact` helper to avoid TUI dep from build… **or** call `framework_pick` from build (already used elsewhere?). Prefer `artifact::requires_explicit_build_command` duplicated match for layering. |

## Acceptance

- [ ] AC1 — tauri success + expo empty build_command → desktop artifacts signed; expo in debt; exit 0
- [ ] AC2 — `--target mobile` with empty cmd → hard fail
- [ ] AC3 — `--strict-targets` with debt → exit error after signing siblings
- [ ] AC4 — unit/integration-style test on build debt helper or fixture loop

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet` |
| L1 | `cargo clippy -p signet -- -D warnings` |
