# Scan apply hardening + stale basename resolve (0.5.8)

## Plan alignment

- **Handoff:** Maintainer continue-improve after Miro 0.5.7 dogfood (`docs/dogfood/miro-notes.md` remaining friction).
- **Band:** 0.5.8
- **PAUSED/CANCELLED:** none
- **In scope:** `scan --apply` platforms non-shrink; draft `[[targets]]` on existing configs; skip identity hint when present; deepen basename resolve for verify/stale.
- **Out of scope:** phase timings; live `gh` auth; Expo default build loop.

## Contracts

### `scan --apply` (existing `signet.toml`, no `--force`)

| Field | Behavior |
|-------|----------|
| `[platforms].*` | **Never shrink** (`true`→`false`). May expand `false`→`true` when scan suggests true. |
| `framework` / `app_root` | Fill only when empty (unchanged). |
| `[[targets]]` | If empty and ≥2 **installable** apps → draft targets. Never overwrite non-empty without `--force`. |

### `scan --apply --force`

- May set platforms to suggested values (can shrink).
- May replace name / app_root / framework.
- May replace `[[targets]]` with a fresh draft when ≥2 installable apps (or clear if <2).

### New config (no file yet)

- Draft `[[targets]]` from **installable** projects only (exclude nested `rust_cli`).
- Platforms = suggested (first write).

### Post-apply next step

- Print `signet identity create` **only** when `!report.has_identity`.

### Basename resolve (shared by verify + stale assess)

When shallow resolve fails and the sums entry is a basename (or basename of a relative path), walk each search root for a file with that exact name:

- Skip: `node_modules`, `.git`, `.signet`, `.selfsign`, `target/debug`, `target/deps`, `.next`, `dist/cache` (and similar noise).
- Cap: depth ≤ 14, max files visited ≤ 50_000 (bail early on first match preferred under `bundle` / `release` / `nsis` / `msi` when scoring).

Same `resolve_artifact_path` used by `verify_sha256sums` and `assess_sums_freshness`.

## Acceptance

- [ ] AC1 — apply without force does not flip `macos/linux` true→false
- [ ] AC2 — apply on Miro-shaped tree with empty targets drafts tauri+expo `[[targets]]`
- [ ] AC3 — apply does not suggest identity create when identity exists
- [ ] AC4 — stale/verify finds basename-only sums entry under nested `bundle/nsis/`
- [ ] AC5 — unit tests for merge_platforms + basename walk + draft targets filter

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet` |
| L1 | `cargo clippy -p signet -- -D warnings` |
