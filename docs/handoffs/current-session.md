# Current session handoff

**Updated:** 2026-08-03  
**Band:** **0.5.x → multi-platform ship** — slices A–E done

## Next atomic step

Fix order **#5** from [`docs/dogfood/signet-shortcomings.md`](../dogfood/signet-shortcomings.md): **Graduate on same ship plan** (item 3) — or dogfood Miro with `--ci` / `--collect` first. Parent: [`specs/backend/multi-platform-ship-design.md`](../../specs/backend/multi-platform-ship-design.md) slice F.

Optional: **tag v0.5.12** + push when ready.

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none

## Canonical owners

| Work | Owner |
|------|--------|
| Ship CI / collect / coverage gate | `ship/ci_template.rs`, `ship/collect_dir.rs`, `commands/ship.rs`, `commands/release.rs` |
| Graduate on ship plan (next) | `commands/ship.rs`, `graduate/*` |
| Multi-platform ship (north star) | `specs/backend/multi-platform-ship-design.md` |

## Recently completed

- **v0.5.12 / D–E:** `ship --ci`, `ship --collect`, release coverage gate + staging attach
- **v0.5.11 / C:** dry-run read-only
- **v0.5.10 / B:** soft-fail targets
- **v0.5.9 / A:** coverage plan
