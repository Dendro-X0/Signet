# Dogfood notes — Miro Desktop

**Status:** Sign → Prove → Check green on Windows (self-signed; SmartScreen still warns)  
**App:** Miro (`E:\Web Projects\miro-workspace\miro`) — Tauri v2 desktop  
**Framework:** `tauri` · `app_root = apps/miro-desktop` · `build_command = pnpm desktop:release`  
**Date:** 2026-08-03 (updated for Signet 0.5.6 Check honesty)  
**Host:** Windows  
**Signet:** ≥0.5.6 recommended (inspect honesty + relative SHA256SUMS + optional `[[targets]]`)

## Goal

Complete Sign → Prove → Check on a real installable app (v1.0 gate).

## Commands used

```bash
cd /e/Web\ Projects/miro-workspace/miro
signet scan
signet doctor
signet build --no-sign          # pnpm desktop:release → NSIS + MSI + exe
signet sums-key create          # once
signet build --skip-build       # Authenticode + SHA256SUMS.minisig
signet verify                   # finds relative paths under SHA256SUMS
signet inspect --file "apps/miro-desktop/src-tauri/target/release/bundle/nsis/Miro Desktop_0.2.0_x64-setup.exe"
# Expected: status=signed (self-signed / untrusted root detail OK) — not false "unsigned"
```

Optional monorepo shape (0.5.6+):

```toml
[[targets]]
id = "desktop"
framework = "tauri"
app_root = "apps/miro-desktop"
build_command = "pnpm desktop:release"
```

## Friction found → fixes

| Issue | Fix |
|-------|-----|
| `frontendDist` wrong relative path | Miro: `../../miro-web/out` |
| Signet ignored Tauri `build_command` | Signet 0.5.5: project-root script |
| Windows `program not found` for `pnpm` | Signet 0.5.5: `cmd /C` |
| `inspect` reported unsigned after sign | Signet 0.5.6: signature presence vs `/pa` trust |
| `verify` skipped all sums (basenames only) | Signet 0.5.6: relative paths in SHA256SUMS |

## Outcome

| Step | Result |
|------|--------|
| Build | NSIS + MSI + exe discovered |
| Prove | `SHA256SUMS` (+ `.minisig`) with relative paths |
| Sign | Authenticode `signtool+timestamp` |
| Check | minisign OK; inspect **signed** for self-signed (honest detail) |

## Time-to-first-useful check

~5–10+ min first `desktop:release`; sign-only afterward is seconds.
