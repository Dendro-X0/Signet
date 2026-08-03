# Current session handoff

**Updated:** 2026-08-03  
**Band:** **0.5.x → multi-platform ship** — slices A–G done

## Next atomic step

Fix order **#7** from [`docs/dogfood/signet-shortcomings.md`](../dogfood/signet-shortcomings.md): **Release auth guided path** (item 10) — or dogfood Miro with graduate/`--ci` / `--collect`.

Optional: **tag v0.5.14** + push when ready (includes 0.5.13 mobile + 0.5.14 graduate if not tagged yet).

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none

## Canonical owners

| Work | Owner |
|------|--------|
| Ship CI / collect / coverage / mobile / graduate profile | `ship/*`, `commands/ship.rs`, `commands/graduate.rs`, `commands/release.rs` |
| Release auth guided (next) | `commands/release.rs`, `commands/doctor.rs`, guided/TUI |
| Multi-platform ship (north star) | `specs/backend/multi-platform-ship-design.md` |

## Recently completed

- **v0.5.14 / F:** `[ship].path` self\|graduate; plan + CI + `graduate apply`
- **v0.5.13 / G:** mobile commitment + coverage gap + CI android/ios jobs + release classify
- **v0.5.12 / D–E:** `ship --ci`, `ship --collect`, release coverage gate + staging attach
- **v0.5.11 / C:** dry-run read-only
- **v0.5.10 / B:** soft-fail targets
- **v0.5.9 / A:** coverage plan
