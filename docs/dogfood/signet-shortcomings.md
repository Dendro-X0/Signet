# Signet — current shortcomings

**Source:** Miro dogfood (Signet 0.5.8, Windows host, 2026-08-03) + product bar (multi-platform desktop/mobile, self-sign + assisted official).  
**Use:** Address one item at a time.  
**Related:** [`specs/backend/multi-platform-ship-design.md`](../../specs/backend/multi-platform-ship-design.md)

---

## Product / coverage

1. **No multi-platform ship orchestration** — Declaring `windows/macos/linux = true` does not drive a plan, CI matrix, artifact collect, or coverage gate. Other OSes are a footnote (“need a matching host”), not a Signet-owned path. Without this, the tool does not earn “all platforms” positioning.

2. **`[platforms]` is documentation-ish, not a commitment** — Doctor/build/release can succeed Windows-only while macOS/Linux remain declared and empty. No fail-closed “missing declared platform” check on release.

3. **Official (graduate) path is not on the same ship plan** — OV / Azure / notarize helpers exist, but there is no unified “self vs graduate” profile that runs across the same multi-OS release flow.

4. **Mobile is second-class in the end-to-end loop** — Expo/RN/Flutter adapters need `build_command`; scan drafts targets, but ship/release does not carry mobile through the same Prove → Check → Release story as desktop.

---

## Build / targets

5. **Sibling target failure aborts the whole `signet build`** — After a successful desktop Tauri build, a `[[targets]]` entry without `build_command` (e.g. Expo) errors out and never reaches signing for the artifacts already produced.

6. **No soft-skip / debt reporting for unpaid targets** — Missing recipes should warn and continue (or require `--target`), not kill the pipeline mid-run.

7. **No upfront coverage warning** — Building with `macos/linux=true` on Windows does not clearly state at start: “this host will only produce/sign Windows; ship gap = N platforms.”

---

## Release / trust kit

8. **`signet release --dry-run` mutates `SHA256SUMS`** — Rewrites paths to basenames (and related side effects) even in dry-run; dry-run should be read-only.

9. **No collect/merge step for multi-host artifacts** — Cannot ingest Win + Mac + Linux outputs into one sums + minisig + TRUST set as a first-class command.

10. **Live `signet release` still needs GitHub auth** — Doctor reports missing `gh` / `GH_TOKEN`; fine as a prerequisite, but the publish path is incomplete without clear guided setup for that gate.

---

## Honesty / UX

11. **Narrative oversells relative to local capability** — Product/README promise cross-desktop + mobile + dual path; a Windows maintainer’s one-command experience is still Windows desktop self-sign. Gap should be explicit in doctor/ship until orchestration exists.

12. **Guided/TUI path does not surface the multi-OS gap** — Guided setup can feel “done” after host Sign → Prove → Check without listing undeclared-vs-missing platforms.

---

## Out of scope as “bugs” (constraints to design around, not ignore)

- Cannot cross-compile/sign macOS/Linux GUI installers on Windows (toolchain/OS reality).
- Cannot fake SmartScreen / Gatekeeper / Play / App Store trust.
- Store upload (Play / App Store Connect / full EAS) remains external.

These do **not** excuse missing orchestration, collect, coverage gates, or dual-path ship UX.

---

## Suggested fix order (optional)

| # | Item | Status |
|---|------|--------|
| 1 | Coverage report / treat platforms as commitment (shortcomings 1–2, 7, 11–12) | **Done in 0.5.9** (`signet ship --plan`, doctor/build/guided) — release fail-closed still slice E |
| 2 | Soft-fail unpaid `[[targets]]` (5–6) | **Done in 0.5.10** — debt skip + `--strict-targets` |
| 3 | `release --dry-run` read-only (8) | **Done in 0.5.11** |
| 4 | CI template + collect + release coverage gate (1, 9) | **Done in 0.5.12** |
| 5 | Graduate on same ship plan (3) | next |
| 6 | Mobile through full loop (4) | |
| 7 | Release auth guided path (10) | |
