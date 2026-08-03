# Current session handoff

**Updated:** 2026-08-03  
**Band:** **0.5.x → multi-platform ship** — slices A–C done

## Next atomic step

Fix order **#4** from [`docs/dogfood/signet-shortcomings.md`](../dogfood/signet-shortcomings.md): **CI template + collect + release coverage gate** (items 1, 9) — design → implement → dogfood. Parent: [`specs/backend/multi-platform-ship-design.md`](../../specs/backend/multi-platform-ship-design.md) slices D–E.

Optional: **tag v0.5.11** (+ any untagged 0.5.10 soft-fail commit) + push when ready.

**PAUSED / CANCELLED:** none  
**Blocked for coding:** none

## Canonical owners

| Work | Owner |
|------|--------|
| Dry-run read-only | `release/collect.rs`, `commands/release.rs` |
| Soft-fail targets | `commands/build.rs` |
| Coverage / ship plan | `ship/coverage.rs`, `commands/ship.rs` |
| CI + collect (next) | ship + workflows |
| Multi-platform ship (north star) | `specs/backend/multi-platform-ship-design.md` |

## Recently completed

- **v0.5.11 / slice C:** `release --dry-run` read-only (no sums rewrite)
- **v0.5.10 / slice B:** soft-fail unpaid `[[targets]]`; debt; `--strict-targets`
- **v0.5.9 / slice A:** platform coverage; `ship --plan`
