# Ship CI + collect + release coverage gate (slices D–E)

## Plan alignment

- **Handoff:** shortcomings fix order #4 (items 1, 9)
- **Parent:** `specs/backend/multi-platform-ship-design.md` slices D–E
- **Band:** 0.5.12
- **In scope:** `signet ship --ci` workflow emit; `signet ship --collect DIR`; release fail-closed on coverage gap unless `--allow-partial`; release discover `dist/signet-ship/`
- **Out of scope:** graduate profile in CI (slice F); mobile matrix (G); Signet SaaS

## Contracts

### `signet ship --ci [--force]`

Write `.github/workflows/signet-ship.yml` from declared `[platforms]`:

- Matrix rows only for `windows` / `macos` / `linux` that are `true`
- Each job: checkout → install Signet (GitHub release binary hint) → `signet build --require-sums-sign` (with comments for identity secrets / framework setup)
- Upload OS artifact named `signet-<os>`
- Header comment: collect locally with `signet ship --collect` after downloading artifacts

Refuse overwrite without `--force`.

### `signet ship --collect DIR`

1. Recursively find installers under `DIR` (exe/msi/dmg/appimage/deb/rpm + sibling .sig)
2. Copy into `{root}/dist/signet-ship/` (unique basenames)
3. Also include any files already in `dist/signet-ship/`
4. Rewrite project `SHA256SUMS` for staging files (relative paths); optional minisign via config
5. Print coverage after

### `signet release` coverage gate

After load config, `assess_coverage`:

- Live publish: **bail** if `has_gap()` unless `--allow-partial`
- `--dry-run`: **warn** only (still read-only)

### Release collect

When `dist/signet-ship/` exists, include its files in release asset set (in addition to adapter discover).

## Acceptance

- [ ] AC1 — `--ci` writes workflow; matrix matches declared platforms
- [ ] AC2 — `--collect` copies foreign installers; sums updated; coverage improves
- [ ] AC3 — `release` fails on gap without `--allow-partial`
- [ ] AC4 — dry-run warns on gap, does not fail solely for gap
- [ ] AC5 — unit tests for template matrix + collect copy

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet ship::` |
| L1 | `cargo clippy -p signet -- -D warnings` |
