# Design — Release Trust Desk kit (CONVERT v2)

**Status:** Design · replaces thin Install Trust Pack outline  
**Upstream:** `functional-scope-redesign-sao-v2.md`  
**SKU:** Release Trust Desk (Windows) · PDF and/or Markdown zip + templates  
**Price:** $29–49 standard  

## Deliverables

| ID | Artifact | Layer |
|----|----------|-------|
| D1 | Integrity≠reputation + trust tiers (short) | J |
| D2 | OS-warn decision trees (SmartScreen paths) | J |
| D3 | User-message bank (honest only) | J/R |
| D4 | Trust card template (buyer-facing) | E |
| D5 | Verify / inspect transcript template | E |
| D6 | Worked Windows release fixture | E/R |
| D7 | Pre-tag → post-release ritual checklist | R |
| D8 | Anti-patterns + cost of mistake | J |
| D9 | Graduate pointer + as-of sheet | J |
| D10 | Blank checklist (app/version) | R |
| D11 | Disclaimer | — |

## Soft tool (prefer GIFT)

- `signet trust report` or script that emits Trust card skeleton from config  
- Honesty-constrained message helper — **must refuse** Root/bypass tips  

## Public sample (TRACK)

Trust card example **or** one OS-warn tree branch — ≤20% of kit.

## Proof

| Layer | Check |
|-------|-------|
| L1 | D1–D11 present · Windows locked in title |
| L2 | Commands match current CLI · fixture verifies |
| L3 | Anti-scam: no Root/bypass |
| L4 | V-bar provisional score recorded in launch notes |

## Draft path

`docs/launch/pack/release-trust-desk-v2.md` (new) · retire install-trust-pack-v0 as legacy stub with pointer
