# Current session handoff

**Updated:** 2026-07-31  
**Band:** Public release program — **Phase 13** next (narrative / dual-path docs)

## Next atomic step

Implement **Phase 13** from [`specs/backend/public-release-readiness-design.md`](../../specs/backend/public-release-readiness-design.md) §13: rewrite README + `docs/product.md` for Sign → Prove → Check and dual-path (self-sign vs official); remove stale Tauri-only claims. **No Phase 14 code until 13 is done.**

**PAUSED / CANCELLED:** none  
**Blocked for coding:** Phase 14+ until 13 acceptance checked

## Canonical owners

| Work | Owner |
|------|--------|
| Public release program | `docs/roadmap.md`, `specs/backend/public-release-*.md` |
| Phase 13 narrative | `README.md`, `docs/product.md`, trust/graduation/verify cross-links |
| Phase 14 golden path | `tui/`, guided flows (spec ready) |
| Phase 15–16 demo/cut | `demo/`, `docs/demo.md`, `docs/dogfood/` (spec ready) |

## Specs (ready)

- [public-release-readiness-design.md](../../specs/backend/public-release-readiness-design.md)
- [golden-path-onboarding-design.md](../../specs/backend/golden-path-onboarding-design.md)
- [demo-and-dogfood-design.md](../../specs/backend/demo-and-dogfood-design.md)

## Recently completed

- Inspect + Flutter/RN/Expo/Capacitor on `main` (`3d10a8f`)
- v0.4.0 release-cli green (engine tag; public cut will be 0.5/1.0 after Phases 13–16)
