# Current session handoff

**Updated:** 2026-08-03  
**Band:** **0.5.x → multi-platform ship** — slices A–G + release auth done

## Next atomic step

Shortcomings fix-order queue is complete through **#7**. Prefer **dogfood Miro** with `ship --ci` / `--collect` / graduate, or pick the next product item from [`docs/dogfood/signet-shortcomings.md`](../dogfood/signet-shortcomings.md) (remaining narrative/UX items 11–12 if still open).

Optional: **tag v0.5.15** + push when ready.

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none

## Canonical owners

| Work | Owner |
|------|--------|
| Ship / graduate / mobile | `ship/*`, `commands/ship.rs`, `commands/graduate.rs` |
| Release auth | `release/auth.rs`, `commands/release.rs`, `commands/doctor.rs` |
| Multi-platform ship (north star) | `specs/backend/multi-platform-ship-design.md` |

## Recently completed

- **v0.5.15 / #7:** GitHub auth guided path (doctor / dry-run / preflight / guided)
- **v0.5.14 / F:** `[ship].path` self\|graduate; plan + CI + `graduate apply`
- **v0.5.13 / G:** mobile commitment + coverage gap + CI android/ios jobs + release classify
- **v0.5.12 / D–E:** `ship --ci`, `ship --collect`, release coverage gate + staging attach
