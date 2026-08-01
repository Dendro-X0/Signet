# Design: public release readiness (program)

**Band:** Public release program (Phases 13–16)  
**Status:** ready  
**Depends on:** Phases 6–12 + graduation + inspect + hybrid adapters (engine complete enough)  
**Owners:** `docs/roadmap.md`, child specs below, `docs/handoffs/current-session.md`  
**Plan alignment:** package Signet as an easy signing+verification tool (self-sign **or** facilitate official paths) — not more frameworks first.

## Problem

The signing engine is largely shipped, but public readiness fails the maintainer bar:

- Easy setup / low learning curve (one demo enough)
- Wide ecosystems (already partially met)
- Clear dual job: **self-signing** and **official verification/signing facilitation**
- Demo/GIF/video-ready golden path

Gaps are mostly **product packaging**, not missing adapters.

## Program goal

A tagged **public cut** (suggested **v0.5.0** public preview, then **v1.0.0** when dogfood+polish exit) where:

1. README + docs match reality and teach **Sign → Prove → Check**.
2. A new user completes first signed artifact + verify/inspect in one guided sitting.
3. A recordable demo kit exists in-repo.
4. At least one real-app dogfood note exists.
5. Release tag includes inspect + hybrid adapters; downloader verify story is explicit.

## Non-goals (this program)

- Desktop GUI (post–public-cut Beyond).
- .NET / more adapters before golden path is boring.
- Full Play Console / App Store Connect automation.
- Claiming SmartScreen / Gatekeeper silence.

---

## Phase map

| Phase | Name | Spec | Status gate |
|-------|------|------|-------------|
| **13** | Product narrative & dual-path docs | this file §13 + README/docs edits | docs only |
| **14** | Golden-path onboarding | [golden-path-onboarding-design.md](golden-path-onboarding-design.md) | TUI/CLI UX |
| **15** | Demo kit | [demo-and-dogfood-design.md](demo-and-dogfood-design.md) §Demo | fixtures + scripts |
| **16** | Dogfood + public cut | [demo-and-dogfood-design.md](demo-and-dogfood-design.md) §Cut | process + tag |

**Order is frozen:** 13 → 14 → 15 → 16. Do not implement 14 before 13 narrative is merged (copy drives UX labels).

---

## Mental model (product contract)

Unify user-facing language around three jobs:

| Job | Meaning | Primary commands |
|-----|---------|------------------|
| **Sign** | Create/apply publisher crypto (self-signed **or** OV/Azure/Apple via helpers) | `identity`, `build`, `android`, `ios`, `graduate` |
| **Prove** | Publish integrity artifacts others can check | `trust`, sums / minisign, `release` |
| **Check** | Validate downloads / local artifacts | `verify`, `inspect` |

Self-sign is the default **Sign** path. Official/paid paths are **graduation** — facilitated, never faked.

---

## Phase 13 — Product narrative & dual-path docs

### Goals

1. Rewrite README quick start as a **2-minute story** (install → guided → check), not a capability dump.
2. Align `docs/product.md` command table with shipped surface (`inspect`, `graduate`, `frameworks`, hybrid).
3. Dual-path section: Self-signed vs Official (OV / Azure / notarize / Play honesty).
4. Explicit “what Signet never does” (Root install, fake OS reputation).
5. Point recorders at demo kit (once Phase 15 lands) and `docs/frameworks.md` for width.

### Non-goals

- New CLI commands (unless a one-line alias is required by Phase 14 — defer).
- Changing trust-tier semantics.

### Acceptance (13)

- [ ] README install + quick start ≤ ~40 lines of user-facing steps before deep links.
- [ ] `product.md` lists current commands and Sign/Prove/Check.
- [ ] `trust-model.md` / `graduation.md` / `verify.md` cross-linked from README dual-path.
- [ ] No stale “adapters later / Tauri only” claims where adapters exist.
- [ ] Working rule preserved: no false “supported” for unscanned paths.

### Proof (13)

| Layer | Evidence |
|-------|----------|
| L1 | Maintainer read-through; link check of new README anchors |
| L2 | Optional: `rg` for stale phrases (`adapters next`, `Tauri today` in product surfaces) |

---

## Child specs

| Spec | Phase |
|------|-------|
| [golden-path-onboarding-design.md](golden-path-onboarding-design.md) | 14 |
| [demo-and-dogfood-design.md](demo-and-dogfood-design.md) | 15–16 |

---

## Beyond public cut (not this program)

- Optional desktop GUI
- Homebrew / winget
- Host-sign / notarize the Signet CLI itself (or document checksum-only honesty)
- .NET and further ecosystems
- Update channels beyond GitHub Releases + checksums
- Deeper EAS / Play / ASC automation

---

## Acceptance (program)

- [ ] Phases 13–16 each marked done on roadmap with child specs `implemented`.
- [ ] Public tag cut per Phase 16 checklist.
- [ ] Handoff points Beyond or dogfood follow-ups only.
