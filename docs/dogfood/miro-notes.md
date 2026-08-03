# Dogfood notes — Miro Desktop

**Status:** Sign → Prove → Check green on Windows (self-signed; SmartScreen still warns)  
**App:** Miro (`E:\Web Projects\miro-workspace\miro`) — Tauri v2 desktop  
**Framework:** `tauri` · `tauri_root = apps/miro-desktop` · `build_command = pnpm desktop:release`  
**Date:** 2026-08-03  
**Host:** Windows  
**Signet:** local 0.5.4+ (Tauri `build_command` + Windows `cmd /C` for pnpm)

## Goal

Complete Sign → Prove → Check on a real installable app (v1.0 gate).

## Commands used

```bash
cd /e/Web\ Projects/miro-workspace\miro
signet scan
signet doctor
signet build --no-sign          # pnpm desktop:release → NSIS + MSI + exe
signet sums-key create          # once
signet build --skip-build       # Authenticode + SHA256SUMS.minisig
signet verify
signet inspect --file "apps/miro-desktop/src-tauri/target/release/bundle/nsis/Miro Desktop_0.2.0_x64-setup.exe"
```

## Friction found → fixes

| Issue | Fix |
|-------|-----|
| `frontendDist` `../miro-web/out` resolved under `miro-desktop/` | Miro: `../../miro-web/out` in `tauri.conf.json` |
| Signet ignored Tauri `build_command` | Signet: run monorepo script from project root |
| Windows `program not found` for `pnpm` | Signet: spawn via `cmd /C` |
| Corrupt cargo `cc` registry | Delete `~/.cargo/registry/src/.../cc-1.2.46` and rebuild |

## Outcome

| Step | Result |
|------|--------|
| Scan / init | `framework=tauri`, `tauri_root=apps/miro-desktop` |
| Build | NSIS + MSI + `miro-desktop.exe` discovered |
| Prove | `SHA256SUMS` + `.minisig` |
| Sign | `signtool` reported success on all three (`signtool+timestamp`) |
| Check | minisign OK; `inspect /pa` may report unsigned for self-signed (expected without Trusted Root) |

## Artifacts (Windows)

- `…/bundle/nsis/Miro Desktop_0.2.0_x64-setup.exe`
- `…/bundle/msi/Miro Desktop_0.2.0_x64_en-US.msi`
- `…/release/miro-desktop.exe`

## Time-to-first-useful check

~5–10+ min for first `desktop:release` (Rust compile); sign-only afterward is seconds.
