# Design: Multi-platform ship (desktop + mobile, dual path)

**Band:** post-0.5.x / pre-v1.0 product gap (or 0.6.0 if maintainer prefers)  
**Status:** draft — product north star from Miro dogfood (2026-08-03)  
**Depends on:** artifact contract, `[[targets]]`, platforms intent, graduation helpers, release attach  
**Owners:** `commands/build.rs`, `commands/release.rs`, new ship/CI surface, `docs/product.md`  
**Plan alignment:** Product thesis already claims Win/macOS/Linux + mobile + dual path. Miro dogfood proved local `signet build` only completes **one host OS**. That is a **product failure relative to the thesis**, not an acceptable “works as designed” end state.

---

## Problem (user / business)

Indie and OSS maintainers do not buy a signing tool to run `signtool` once on Windows.

They buy (or adopt) a tool so that **shipping the same app across the platforms they declared** is:

1. **Rapid** — one mental model, few commands, CI copy-paste without inventing scripts.
2. **Complete** — Windows + macOS + Linux desktop when declared; Android / iOS when declared.
3. **Dual-path** — self-signed by default; assisted OV / Azure / Apple / store honesty when they graduate.
4. **Honest** — never fake SmartScreen / Gatekeeper / Play / App Store trust.

If Signet only reliably signs the OS you are sitting on, it competes with a 20-line script and loses. The dedicated product must own **cross-platform facilitation**: orchestration, trust kit continuity, and official-path assists — not merely host crypto wrappers.

---

## Non-goals (still true)

- Cross-compiling Tauri/Electron GUI installers on the wrong OS (Apple/Microsoft/toolchain reality).
- Storing Apple/Google/Azure secrets in git or `TRUST.md`.
- Claiming store or OS reputation from self-sign.

Cross-compile limits are **implementation constraints**. They must not define the **user-visible product boundary**. The product boundary is: *I declared platforms X; Signet gets me to signed+proven release assets for X with minimal friction.*

---

## Jobs to be done

| Actor | Job | Success looks like |
|-------|-----|--------------------|
| Solo maintainer (Windows laptop) | Ship Miro desktop to Win + Mac + Linux | Runs one guided/CI path; mac/linux jobs run elsewhere; one GitHub Release with all installers + one trust kit |
| Same maintainer | Add Android APK later | Same identity/trust story; `[[targets]]` mobile; CI row or local android host |
| Graduating team | Switch Windows to Azure Trusted Signing; macOS to notarize | Same ship matrix; path flag / `[graduation]` profile; no rewrite of discover/sums/release |
| Agent / CI | Fail closed | Missing declared platform = failed ship, not silent Windows-only success |

---

## Product principle

**`[platforms]` and `[[targets]]` are commitments, not documentation.**

Today they are mostly intent + TRUST copy. Required shift:

| Today | Required |
|-------|----------|
| Host signs whatever is on disk | Ship **plan** derived from platforms × targets × path (self \| graduate) |
| Other OS “need a matching host” (footnote) | Signet **generates and drives** that matching work (CI workflow + collect + release) |
| Expo without `build_command` aborts after desktop build | Soft-fail / skip with plan debt; never wipe a successful sibling |
| `release --dry-run` mutates sums | Dry-run is read-only |
| Doctor green on Windows with macos=true | Doctor/ship report **coverage gap** until mac assets exist or CI is wired |

---

## Proposed surface: `signet ship` (name flexible)

One user-facing verb above `build` / `release`:

```text
signet ship                 # local host slice + print gap / next CI
signet ship --ci            # emit or refresh GitHub Actions (or documented generic CI)
signet ship --collect DIR   # merge remote artifacts into release set + rewrite sums
signet ship --release       # publish after coverage gate (or --allow-partial)
```

### Semantics

1. **Plan** — from `signet.toml`: platforms × targets × path (`self` \| `graduate`).
2. **Execute local slice** — build+sign what this host can (desktop host sign, android if tools present).
3. **Report gap** — explicit checklist of missing OS/target artifacts vs plan.
4. **CI contract** — checked-in workflow (or `signet ship --ci` output) runs the same plan on `windows-latest` / `macos-latest` / `ubuntu-latest` (+ mobile jobs when configured).
5. **Collect** — download/matrix artifacts into one tree; single `SHA256SUMS` + minisig; one `TRUST.md`.
6. **Release** — coverage gate: refuse tag publish if declared platform missing unless `--allow-partial`.

Self vs graduate is a **profile on the same plan**, not a separate product.

---

## Dual path (same plan)

| Path | Per-platform Sign action |
|------|---------------------------|
| **Self** | Windows Authenticode (Signet identity); macOS `codesign` adhoc/identity; Linux detached/openssl + sums; Android keystore; iOS package honesty |
| **Graduate** | Windows OV / Azure Trusted Signing; macOS Developer ID + `notarytool` + staple; Linux still integrity-first; Play/App Store remain external with docs |

`signet graduate` stays the helper; `ship` chooses which Sign backend per platform from `[graduation]` + env.

---

## Miro acceptance (dogfood gate)

Miro already declares `windows/macos/linux = true` and `miro-desktop` + `miro-mobile`.

**Accept when a Windows-primary maintainer can:**

1. Keep that `signet.toml` unchanged in intent.
2. Run `signet ship --ci` (or merge a Signet-maintained workflow template).
3. After matrix green + `signet ship --collect`, have Win + Mac + Linux desktop installers under one sums file with one fingerprint story.
4. Optionally run Windows Azure graduate on the Windows job without forking the trust kit.
5. Mobile: either a documented target recipe **or** explicit `platforms`/target debt that does not abort desktop ship.

Until then, “multi-platform supported” in marketing is oversell relative to Miro’s business need.

---

## Implementation slices (suggested order)

| Slice | Deliverable | Proof |
|-------|-------------|-------|
| **A** | Coverage report: `signet doctor` / `ship --plan` lists declared vs present artifacts; warn when macos/linux true on Windows with zero foreign assets | Miro: plan shows 2/3 desktop missing |
| **B** | Soft-fail targets missing `build_command`; `--target` default guidance; don’t abort sibling success | `signet build` signs desktop even if Expo unpaid |
| **C** | `release --dry-run` read-only | sums unchanged |
| **D** | CI workflow template + `signet ship --ci` | Matrix jobs call `signet build --require-sums-sign` per OS |
| **E** | `signet ship --collect` + coverage gate on `release` | One release with 3 OS installers from Miro matrix |
| **F** | Graduate profile in ship plan | Windows job uses `graduate azure-sign` when configured |
| **G** | Mobile rows (Android first; iOS macOS-hosted) | Optional until desktop matrix proven |

---

## Alternatives considered

| Option | Why reject as sole answer |
|--------|---------------------------|
| “Document that users must run Signet on 3 machines manually” | True today; fails “rapid” and “dedicated tool” bar |
| Cross-compile everything from Windows | Not realistic for Tauri/macOS signing/notarization |
| SaaS signing service | Out of scope for local-first OSS Signet; CI orchestration is enough |
| Windows-only MVP forever | Maintainer intent: scrap-worthy; contradicts `product.md` |

---

## Risks

- CI secret distribution for graduate path (Azure/Apple) — document patterns; never commit secrets.
- Partial releases during migration — `--allow-partial` explicit only.
- Template drift across frameworks — keep ship plan framework-agnostic; adapters only fill build/discover.

---

## Proof layers (when implementing)

| Layer | Check |
|-------|-------|
| L1 | Unit: plan coverage from toml + fake artifact set |
| L2 | Fixture: collect merges 3 OS fake installers → one sums |
| L3 | Miro dogfood: matrix (or simulated collect) → verify on Windows host |
| L4 | Optional: live `signet release` with multi-OS assets |

---

## Doc updates when shipping

- `docs/product.md` — state orchestration as core, not footnote
- `docs/signing.md` — ship / CI / collect
- `docs/roadmap.md` — Phase or 0.6 band
- `docs/dogfood/miro-notes.md` — re-run acceptance

---

## Decision needed from maintainer

1. Band name: finish inside **0.5.x** friction vs open **0.6.0 multi-platform ship**?
2. First slice A–C only (honesty + don’t abort) before CI template, or A+D together?
3. Is GitHub Actions the first-class CI, with generic “any CI” docs second?
