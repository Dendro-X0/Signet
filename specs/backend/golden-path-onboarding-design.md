# Design: golden-path onboarding (Phase 14)

**Phase:** 14  
**Status:** ready  
**Depends on:** Phase 13 narrative (Sign / Prove / Check labels)  
**Owners:** `crates/signet/src/tui/`, `commands/init.rs` / `scan`, optional thin CLI glue, `docs/install.md`  
**Plan alignment:** one sitting from install → signed artifact → check; demo/GIF friendly.

## Problem

Guided setup is still Tauri-shaped. Hybrid users must know `framework` + `build_command`. `verify` and `inspect` are separate and easy to miss. Public users need a **single obvious path**.

## Goals

1. TUI **Guided setup** teaches Sign → Prove → Check with explicit step titles.
2. After scan (or manual pick), guided flow sets `project.framework` from detected kinds (Tauri / Electron / Flutter / RN / Expo / Capacitor / Android / iOS) with confirmation.
3. Prompt for `build_command` when the selected adapter requires it (hybrid / iOS); offer `--skip-build` path when artifacts already exist.
4. Guided path ends with **Check**: run `verify` (if sums/TRUST present) and `inspect` on discovered primary artifact(s) when tools allow.
5. Hub menu exposes Check actions (Verify / Inspect) and points Official signing at `graduate notes` (not buried).
6. Optional CLI: `signet doctor --fix` or guided already covers — **prefer TUI**; add `signet quickstart` only if TUI cannot be non-interactive for demos (decision below).

## Non-goals

- Replacing framework build systems.
- Auto-purchasing OV / Apple / Azure credentials.
- GUI (Beyond).
- Merging `verify`+`inspect` into one binary command in v1 of this phase (orchestration in guided is enough; optional later `signet check`).

---

## Decisions (frozen)

1. **Primary surface for golden path = TUI Guided setup** (matches “watch a demo”).
2. **No new `signet quickstart` in Phase 14** unless demo kit (Phase 15) cannot drive TUI; prefer a non-interactive **script** in the demo kit that calls CLI verbs in order.
3. Framework selection: scan suggestion → confirm list; default first high-confidence project kind; always allow override.
4. Official path in guided: after successful self-sign Check, print short “Need OV / notarize / Azure? → `signet graduate notes`” — do not run graduate in the default wizard.

---

## Guided flow (target)

```text
1. Doctor (optional skip if recently ok)
2. Scan → pick app root + framework
3. Init (if needed) writing framework (+ build_command prompt)
4. Sign: identity create/show
5. Prove: trust generate
6. Sign: build (or skip-build) + honesty notes
7. Prove: remind sums / release dry-run optional
8. Check: verify + inspect sample artifact
9. Next: release tag OR graduate notes
```

Step titles in UI must use Sign / Prove / Check words from Phase 13.

---

## Init / config touchpoints

When guided writes `signet.toml`:

- Set `project.framework` from choice.
- Set `project.tauri_root` to chosen app root (legacy field).
- Set `project.build_command` when required; leave empty only for adapters with defaults (Electron `npm run dist`, Tauri internal).

---

## Acceptance

- [ ] Guided setup shows Sign/Prove/Check sectioning.
- [ ] Framework pick works for at least: tauri, electron, flutter (fixture or mock).
- [ ] Required `build_command` prompted for flutter/rn/expo/capacitor/ios.
- [ ] Check step invokes verify and/or inspect without requiring the user to know subcommands.
- [ ] Hub lists Verify / Inspect (and Graduate notes entry or help text).
- [ ] `cargo test -p signet` + clippy `-D warnings`.
- [ ] Phase 13 docs mention the guided path.

## Proof

| Layer | Command / evidence |
|-------|---------------------|
| L1 | Unit tests for framework→config mapping helpers |
| L2 | `cargo test -p signet` + clippy |
| L3 | Manual TUI walk on Windows or macOS (maintainer) |

## Open → closed

| Question | Decision |
|----------|----------|
| Merge verify+inspect CLI? | Not in 14; guided orchestrates |
| `signet check` alias? | Defer to Beyond unless demo script needs it |
