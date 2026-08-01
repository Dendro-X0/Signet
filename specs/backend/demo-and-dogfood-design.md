# Design: demo kit + dogfood + public cut (Phases 15–16)

**Phases:** 15 (demo kit), 16 (dogfood + public cut)  
**Status:** ready  
**Depends on:** Phase 13 narrative, Phase 14 golden path (for “one demo” claim)  
**Owners:** `examples/` (or `demo/`), scripts, `CHANGELOG`, release tag, optional `docs/demo.md`  
**Plan alignment:** recordable GIF/video path; real-app proof before calling the version “official.”

---

## Phase 15 — Demo kit

### Problem

Maintainers plan to record GIFs/videos once the official version is complete. Without a fixed kit, demos drift and viewers cannot replay.

### Goals

1. In-repo **minimal demo app** (prefer Electron or plain folder of fake installers + Tauri-optional — **decision:** ship a **fixture tree** that does not require full GUI app runtime for CI, plus a **scripted happy path** against that fixture).
2. Scripts:
   - `demo/record-happy-path.sh` (+ `.ps1`) calling: doctor → init/config → identity → trust → build `--skip-build` → verify → inspect.
3. `docs/demo.md`: exact window size hints, commands, expected on-screen phrases for recording.
4. Sample downloadable story: “verify this release” using Signet’s own `SHA256SUMS` from GitHub Releases.

### Non-goals

- Hosting video files in git.
- Automating GIF capture in CI (document manual recording).

### Fixture layout (frozen)

```text
demo/
  README.md                 # points to docs/demo.md
  fixture/
    signet.toml             # framework = electron (or generic), skip heavy build
    dist/HelloSignet.exe    # tiny/non-functional bytes OK for inspect/sums demos on Windows
    # optional: .AppImage / .apk placeholders for classify/inspect unknown-tool paths
  scripts/
    happy-path.sh
    happy-path.ps1
```

CI: do **not** require signing the fixture PE in PR CI; unit tests already cover inspect/sums. Script is for humans/recorders.

### Acceptance (15)

- [ ] `demo/` + `docs/demo.md` exist.
- [ ] Happy-path script runs on a clean machine with Signet installed (document prereqs).
- [ ] README links “2-minute demo” → `docs/demo.md`.

### Proof (15)

| Layer | Evidence |
|-------|----------|
| L1 | Script dry-run locally |
| L2 | Links from README resolve |

---

## Phase 16 — Dogfood + public cut

### Problem

Public “official” without dogfood hides friction. Tag must include features already on `main` (inspect + hybrid) that post-date v0.4.0.

### Goals

1. **Dogfood gate:** run Signet on ≥1 real project (maintainer’s Miro / Deco / KnotTrace / other) and file a short `docs/dogfood/<app>-notes.md` (commands used, framework, blockers, time-to-first-sign).
2. **Public cut checklist** before tag:

| # | Check |
|---|--------|
| 1 | Phases 13–15 done (or 15 waived only if demo script uses CLI-only and 14 TUI deferred — **not allowed:** cut without 13) |
| 2 | Dogfood notes merged |
| 3 | CHANGELOG Unreleased → version section |
| 4 | Workspace version bump |
| 5 | START-HERE / README status version |
| 6 | `cargo test -p signet` + clippy `-D warnings` |
| 7 | Tag `vX.Y.Z` → `release-cli` green |
| 8 | Spot-check install + `signet verify` against new release SHA256SUMS |
| 9 | Announce dual-path + demo link in release body (edit if needed) |

3. Version policy (frozen):

| Tag | When |
|-----|------|
| **v0.5.0** | Public preview: 13–15 + engine on main; dogfood notes may be “partial” |
| **v1.0.0** | 13–16 complete; dogfood notes show successful Sign→Prove→Check on a real app; narrative stable |

### Non-goals

- Marketing site.
- Guaranteeing brew/winget on first public cut (Beyond).

### Acceptance (16)

- [ ] Dogfood notes path exists and is linked from handoff or `docs/demo.md`.
- [ ] Tagged release with green `release-cli`.
- [ ] Roadmap marks Phases 13–16 done; Beyond listed separately.
- [ ] Handoff next step = Beyond item or maintenance.

### Proof (16)

| Layer | Evidence |
|-------|----------|
| L2 | CI release-cli success URL |
| L3 | Installer + verify against release assets |
| L4 | Dogfood notes with commands + outcome |

---

## Recording guidance (for maintainer)

1. Use Phase 15 script once silently, then re-run with TUI Guided for the video.
2. Show honesty line once (self-sign ≠ SmartScreen silence).
3. Optional second short clip: `signet graduate notes` only (no secrets on screen).
