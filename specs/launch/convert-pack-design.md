# Design — Indie Install Trust Pack (CONVERT)

**Status:** Design · draft pack file before public checkout  
**SKU:** one · format PDF **or** Markdown zip  
**Price:** $19 intro / $29 standard  
**OS lock:** **Windows** (accepted 2026-09-05; SmartScreen honesty path). macOS deferred.  

## Contents map

| # | Section | Source in repo |
|---|---------|----------------|
| 1 | Integrity vs reputation | [docs/trust-model.md](../../docs/trust-model.md) |
| 2 | Trust tiers | trust-model |
| 3 | Pre-release checklist | [docs/release.md](../../docs/release.md), ship docs |
| 4 | Verify/inspect copy-paste | [docs/verify.md](../../docs/verify.md), [docs/demo.md](../../docs/demo.md) |
| 5 | User warning scripts | New — honest language only |
| 6 | Anti-patterns | trust-model + product won’t |
| 7 | Graduate pointer | [docs/graduation.md](../../docs/graduation.md) |
| 8 | Blank checklist | Template |
| 9 | Disclaimer | Standard |

## Public sample (TRACK)

Publish **one** section (#3 or #5) + demo TRUST excerpt — ≤20% of pack. Not a free full download / trial.

## Delivery

| Step | Artifact |
|------|----------|
| Draft | `docs/launch/pack/install-trust-pack-v0.md` (working draft) |
| Ship | Export PDF or zip · upload to checkout |
| Errata | Changelog note in pack footer (as-of date) |

## Proof plan

| Layer | Check |
|-------|-------|
| L1 | Sections 1–9 present · OS locked in title |
| L2 | Every command runs against current Signet CLI |
| L3 | Anti-scam review (Root / bypass absent) |
| L4 | Checkout delivers file; Cafe FAQ “is CLI free?” = Yes |

## Acceptance

- [x] Working draft in repo (2026-09-05 · Windows · sections 1–9)
- [ ] Checkout SKU live after unlock  
- [ ] Sample page live  
