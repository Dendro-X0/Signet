# Current session handoff

**Updated:** 2026-09-05  
**Band:** **Launch Phase 1** — Install Trust HOOK + CONVERT (commercial micro-entry)

## Next atomic step

1. Friend co-read unlock gate 3 using private preview: `http://127.0.0.1:8765/` (`docs/launch/preview/`).  
2. Finish unlock gates 4–8 on [unlock-checklist.md](../launch/unlock-checklist.md) (kill window, authorize URLs).  
3. Public HOOK / checkout only after unlock signed.

**PAUSED / CANCELLED:** none for launch band  
**Blocked for coding:** public checkout / announce until unlock signed  

## Parallel (optional engineering)

**v0.5.17** release is live: https://github.com/Dendro-X0/Signet/releases/tag/v0.5.17  
Clippy fix on `main` for `needless_match` in `ship/coverage.rs` (CI run 33964921459). Miro/Clavis dogfood remains optional and must not displace Phase 1 DoD.

## Canonical owners

| Work | Owner |
|------|--------|
| Launch Phase 1 | `docs/launch/*`, `specs/launch/*` |
| Pack draft | `docs/launch/pack/install-trust-pack-v0.md` |
| HOOK draft | `docs/launch/hook-landing-draft.md` |
| HOOK static preview | `docs/launch/preview/` (localhost) |
| TRACK sample | `docs/launch/track-sample.md` · `preview/track-sample.html` |
| CLI / verify / demo | existing `docs/demo.md`, `docs/trust-model.md`, crates |
| Cafe L0 | `docs/launch/cafe-kit.md` |

## Recently completed

- **2026-09-05:** Private static HOOK preview served at `127.0.0.1:8765` (`docs/launch/preview/`)
- **2026-09-05:** Private HOOK draft + TRACK sample (`hook-landing-draft.md`, `track-sample.md`)
- **2026-09-05:** Expanded Windows pack draft (sections 1–9) — `docs/launch/pack/install-trust-pack-v0.md`
- **2026-09-05:** Phase 1 step 1 — accepted `phase1.md` + pack outline; **OS lock = Windows** (unlock gates 1–2)
- **2026-09-04:** Launch docs + specs seeded from strategy-research-lab (S-M2 / H4)
- **v0.5.17 / S1–S3:** `ship secrets`, CI readiness (prior band)
