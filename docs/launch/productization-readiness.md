# Phase 1 productization readiness

**Product:** Signet **Release Trust Desk** (HOOK + CONVERT + GIFT)  
**Goal:** Production-ready micro-entry — live landing, sample Trust card, live checkout, kill window  
**As-of:** 2026-09-05  
**Rule:** No public announce / checkout until [unlock-checklist.md](./unlock-checklist.md) signed  
**CONVERT SoT:** [pack/release-trust-desk-v2.md](./pack/release-trust-desk-v2.md)

## Production = Phase 1 DoD

| DoD item | Status | Evidence / blocker |
|----------|--------|--------------------|
| HOOK + sample Trust card live | **Deploying** | https://dendro-x0.github.io/Signet/ · workflow `deploy-hook.yml` |
| Kit draft complete (EJR / Windows) | **Ready** | `release-trust-desk-v2.md` + export zip |
| Checkout live | **Blocked on Gumroad** | See `pack/export/GUMROAD-LISTING.md` |
| Cafe Kit FAQ (Desk naming) | **Ready** (URLs TBD) | `cafe-kit.md` |
| Kill window dated | **Proposed** | 45 days · paid &lt; 3 → PARK |
| Anti-scam copy review | **Re-pass Desk copy** | Preview + Trust card + Desk draft 2026-09-05 |
| V-bar ≥8/12 | **Pending** maintainer score | |

## Unlock gates → ship

| # | Gate | Status |
|---|------|--------|
| 1–2 | Offer + Windows OS lock | ☑ |
| 3 | Friend co-read | ☑ maintainer self 2026-09-05 (friend deferred) |
| 4 | No wallet ads · no Root advice | ☑ |
| 5 | Kill window | ☑ 45d / &lt;3 |
| 6 | Authorize HOOK + checkout URLs | ☑ HOOK Pages · ☐ Gumroad |
| 7 | Hook design reviewed | ☑ |
| 8 | Pack/Desk design reviewed | ☑ |
| Sign-off | HOOK probe OK · checkout held | ☑ partial |

## Channel (locked recommendation)

```text
HOOK (GitHub Pages) → soft CTA → Gumroad (one SKU: Release Trust Desk)
```

No multi-checkout in Phase 1.

## Maintainer decisions

1. Friend co-read http://127.0.0.1:8765/ + `/trust-card.html`  
2. Confirm kill window · sign unlock  
3. Deploy Pages · create Gumroad SKU · wire CTA  

## After unlock

1. Deploy `docs/launch/preview/` → HOOK URL  
2. Trust card + judgment sample on same host  
3. Export Desk markdown/zip → Gumroad · $29–49  
4. Cafe URLs · kill clock · soft traffic only  
