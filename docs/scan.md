# Scan (repo self-check)

`selfsign scan` walks the current repository (or `--path`) for installable apps and existing installers, then suggests a desktop signing configuration and concrete next commands.

## Command

```bash
selfsign scan
selfsign scan --path ./my-app
selfsign scan --json
selfsign scan --apply              # write/update selfsign.toml platforms
selfsign scan --apply --force      # also replace name / tauri_root
```

## What it detects

| Kind | Markers |
|------|---------|
| Tauri | `src-tauri/tauri.conf.json`, optional `gen/android` / `gen/apple` |
| Electron | `package.json` with electron / electron-builder |
| Android | Gradle + `AndroidManifest.xml`, `.apk` / `.aab` |
| iOS | `.xcodeproj` / `.xcworkspace`, `.ipa` |
| Installers | `.exe` `.msi` `.dmg` `.pkg` `.AppImage` `.deb` `.rpm` `.app` |

Skips `.git`, `node_modules`, `.selfsign`, etc. Depth-capped for speed.

## Honesty

- **Desktop** (Windows / macOS / Linux): suggestions feed `selfsign init` / `build` / `release`.
- **Android / iOS**: listed for awareness. Store signing still uses Play App Signing / Apple certificates — selfsign does **not** claim to replace those.

## TUI

Hub → **Scan** runs the report, optionally `--apply`, then can continue into **Guided setup**.
With no `selfsign.toml`, the hub recommends Scan first.
