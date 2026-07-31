# Repo scan

`signet scan` walks the current repository (or `--path`) for installable apps and existing installers, then suggests a desktop signing configuration and concrete next commands.

## Usage

```bash
signet scan
signet scan --path ./my-app
signet scan --json
signet scan --apply              # write/update signet.toml platforms
signet scan --apply --force      # also replace name / tauri_root
```

## What it finds

- Framework markers: Tauri, Electron, Android, iOS, …
- Existing installers / bundles on disk
- Whether `signet.toml` / legacy `selfsign.toml` and an active identity already exist

Skips `.git`, `node_modules`, `.signet`, `.selfsign`, etc. Depth-capped for speed.

## Honesty

- **Desktop** (Windows / macOS / Linux): suggestions feed `signet init` / `build` / `release`.
- **Android / iOS**: listed for awareness. Store signing still uses Play App Signing / Apple certificates — Signet does **not** claim to replace those.
- **Non-Tauri desktop**: detected now; dedicated build adapters are roadmap.

## TUI

With no `signet.toml` (or legacy config), the hub recommends Scan first.
