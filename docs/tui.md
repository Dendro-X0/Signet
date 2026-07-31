# TUI

Running `signet` with no subcommand opens the interactive hub (TTY required).

## Layout

1. **Header** — Signet brand
2. **Project status** — config / identity / trust / artifacts + recommended next step
3. **Actions** — guided setup + each CLI surface
4. **Footer** — keybindings

★ marks the recommended action. Number keys `1`–`8` jump to a row.

## Guided flows

Each action leaves the alternate screen, prompts on the normal terminal, then calls the **same** `commands::*` entry points as the CLI (no second behavior path).

| Action | Prompts for |
|--------|-------------|
| Guided setup | Optional scan first, then first-release wizard |
| Scan | Report installers; optional apply + handoff to Guided setup |
| Init | app name, tauri_root, force overwrite |
| Identity | show / create / list (+ CN, org, days) |
| Trust | (none — uses active identity) |
| Build | full / skip-build / no-sign (+ timestamp on Windows) |
| Release | tag, repo, dry-run vs publish, draft/prerelease |

## Console consistency

CLI human output (scan, doctor, guided flows) shares `ui::console`:

- `┌─ Signet · …` banners
- section titles + rule lines
- aligned `key = value` and status columns
- numbered next-steps with `→` reasons
- relative installer paths; max 5 shown per platform
