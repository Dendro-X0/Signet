# HOOK landing — private draft (Phase 1)

**Status:** Private draft · **not** announced · no public URL yet  
**Host:** private static preview at `docs/launch/preview/` (localhost) · public host TBD after unlock  
**Design:** [specs/launch/hook-landing-design.md](../../specs/launch/hook-landing-design.md)  
**OS pack CTA:** Windows Install Trust Pack  
**As-of:** 2026-09-05  

> Paste into static host after unlock. Until then: copy review + friend co-read only.

---

## Page chrome

| Slot | Copy |
|------|------|
| Brand | **Signet** |
| Nav (minimal) | Demo · GitHub · Pack *(soft)* |
| Title tag | Signet — Prove what buyers can verify |

---

## Viewport 1 — hero (one composition)

**Brand (hero-level):** Signet  

**H1:** Prove what buyers can verify  

**Supporting sentence:** Self-signing proves integrity of your bits and keys. It does not buy SmartScreen silence — and we will not pretend it does.

**CTA group**

| Priority | Label | Target |
|----------|-------|--------|
| Primary | Run the free demo | [docs/demo.md](../demo.md) · `./demo/scripts/happy-path.ps1` |
| Primary alt | Install Signet (CLI) | [README install](../../README.md#install) · GitHub Releases |
| Secondary (soft) | Get the Windows Install Trust Pack | Checkout URL *(TBD — after unlock)* |

**Won’t appear above the fold:** Aff · Orbit/Assess · “remove SmartScreen” · Trusted Root tips · stats strips · multi-product suite.

---

## Scroll — trust table

Integrity (Signet) does not imply reputation (Windows / stores).

| Tier | What it proves | What it does **not** mean |
|------|----------------|---------------------------|
| Checksums only | `SHA256SUMS` matches the file | Host crypto signature |
| Self-signed | Signed with *your* identity | Microsoft / Store trust |
| Community-signed sums | Sums attested (minisign / GPG) | Authenticode pass |
| CA Authenticode | Signature chains to a public CA | Instant SmartScreen silence |

Full model: [docs/trust-model.md](../trust-model.md).

---

## Scroll — free path (GIFT)

```text
Sign → Prove → Check
identity / build → TRUST.md + sums → signet verify + inspect
```

1. Install the free CLI  
2. Run the [demo kit](../demo.md) (fixed fixture — no real app required)  
3. Read honest tiers before you ship  

The CLI stays free. Paying is optional ritual, not a license gate.

---

## Scroll — TRACK sample (≤20% of pack)

**Public sample (when live):** [track-sample.md](./track-sample.md)

Excerpt promise on the page:

> Free sample: a Windows pre-release checklist slice + a demo `TRUST.md` shape — not the full pack, not a timed trial.

Link label: **Preview a free sample** → `track-sample.md` (or hosted mirror after unlock).

---

## Scroll — soft CONVERT

**Headline:** Shipping on Windows and tired of guessing the honesty story?

**Body:** The Indie Install Trust Pack is a one-OS checklist + verify ritual (Windows): what to publish, what to tell users when SmartScreen warns, and what never to advise.  

**Price line (after checkout live):** $19 intro / $29 standard · optional · CLI remains free  

**CTA:** Get the pack *(soft)* · or keep using the free demo  

Soft-no: if they decline → stop. No full-pack trial timer.

---

## Disclosure

Not legal advice. Not affiliated with Microsoft or Apple.  
Self-sign ≠ platform reputation. We never recommend installing publisher certificates into Trusted Root.

---

## Footer

| Link | URL |
|------|-----|
| GitHub | https://github.com/Dendro-X0/Signet |
| Docs (trust) | `docs/trust-model.md` |
| Demo | `docs/demo.md` |
| Pack (checkout) | *TBD after unlock* |

No hard CTAs for Orbit, Assess, or other suite products.

---

## Implementation notes (private)

| Item | Choice |
|------|--------|
| Format | Static preview: `docs/launch/preview/` (localhost) · public host TBD |
| Primary OS message | Windows SmartScreen honesty (pack lock) |
| Checkout | Placeholder until unlock gate 6 |
| L1 copy review | Won’t-list checked in this draft |
| Friend co-read | Unlock gate 3 — use one-liner + won’t-list below |

### Friend co-read card

**One-liner:** Prove what buyers can verify — without lying about SmartScreen.  

**Won’t-list:** No Trusted Root · no “remove SmartScreen” · no guaranteed trust · no Orbit/Assess hard-sell on this page.
