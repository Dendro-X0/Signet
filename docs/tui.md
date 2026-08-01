# TUI

Running `signet` with no subcommand opens the interactive hub (TTY required).

## Layout

1. **Header** — Signet · Sign → Prove → Check
2. **Project status** — config / identity / trust / artifacts + recommended next step
3. **Actions** — guided setup + each CLI surface
4. **Footer** — keybindings

★ marks the recommended action. Number keys `1`–`9` jump to early rows; use arrows for the rest.

## Guided setup (Phase 14)

**Guided setup** walks **Sign → Prove → Check** in one sitting:

1. Doctor (optional)
2. Scan → suggest framework
3. Init with framework pick (+ `build_command` when required)
4. Sign · identity
5. Prove · TRUST.md
6. Sign · build / skip-build
7. Prove · release dry-run (optional)
8. Check · verify + inspect
9. Hint: `signet graduate notes` for OV / Azure / notarize

Each action leaves the alternate screen, prompts on the normal terminal, then calls the **same** `commands::*` entry points as the CLI (no second behavior path).

| Action | Prompts for |
|--------|-------------|
| Guided setup | Doctor, scan, framework, identity, trust, build, check |
| Scan | Report installers; optional apply + handoff to Guided setup |
| Init | app name, app root, framework, build_command |
| Identity | show / create / list (+ CN, org, days) |
| Trust | (none — uses active identity) |
| Build | full / skip-build / no-sign (+ timestamp on Windows) |
| Verify | runs `signet verify` |
| Inspect | discovered artifacts or path prompt |
| Graduate notes | honesty text for official Sign path |
| Release | tag, repo, dry-run vs publish, draft/prerelease |

## Console consistency

CLI human output (scan, doctor, guided flows) shares `ui::console`:

- `┌─ Signet · …` banners
- section titles + rule lines
- aligned `key = value` and status columns
- numbered next-steps with `→` reasons
- relative installer paths; max 5 shown per platform
