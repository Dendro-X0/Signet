# `release --dry-run` read-only (ship slice C)

## Plan alignment

- **Handoff:** shortcomings fix order #3 (item 8)
- **Parent:** `specs/backend/multi-platform-ship-design.md` slice C
- **Band:** 0.5.11
- **In scope:** dry-run must not rewrite `SHA256SUMS` / `.minisig` / `.asc`
- **Out of scope:** collect merge from multi-host; coverage fail-closed on live release

## Behavior

| Mode | Collect |
|------|---------|
| `--dry-run` | Discover assets; **do not** write/sign sums; attach existing `SHA256SUMS*` + `TRUST.md` if present |
| live release | Current behavior: rewrite sums to flat basenames + optional minisign |

Dry-run notes:
- Print `dry-run is read-only — will not rewrite SHA256SUMS (live release flattens to asset basenames)`.
- If no `SHA256SUMS` on disk: still list installers; warn that live release would create/rewrite sums; skip hard `verify_checksums_cover` fail when sums absent (or soft-warn).
- If sums present: `verify_checksums_cover` as today (may warn/fail if basenames missing from relative-path sums — print hint that live release rewrites).

## Ownership

| Piece | Module |
|-------|--------|
| `CollectOpts.read_only` | `release/collect.rs` |
| Pass `read_only: dry_run` | `commands/release.rs` |

## Acceptance

- [ ] AC1 — dry-run leaves existing `SHA256SUMS` bytes unchanged
- [ ] AC2 — dry-run does not create `SHA256SUMS` when absent
- [ ] AC3 — live collect still writes basename sums (existing test)
- [ ] AC4 — dry-run prints read-only note

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet release::` |
| L1 | `cargo clippy -p signet -- -D warnings` |
