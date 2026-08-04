# Dogfood notes — Miro Desktop (+ mobile surface)

**Status:** Signet **0.5.15** (PATH installer) — ship stack re-verified on Miro  
**App:** Miro (`E:\Web Projects\miro-workspace\miro`) — Tauri v2 + Expo in-repo  
**Date:** 2026-08-03 (re-run after v0.5.15 release)  
**Host:** Windows x86_64  
**Signet:** **0.5.15** (`signet self status` → installer-managed)  
**North star:** [`specs/backend/multi-platform-ship-design.md`](../../specs/backend/multi-platform-ship-design.md)  
**Shortcomings backlog:** [`signet-shortcomings.md`](signet-shortcomings.md)

## Goal

Confirm 0.5.9–0.5.15 fixes on a real monorepo: coverage commitment, soft-fail unpaid Expo, dry-run readonly, CI emit, collect, graduate plan, auth guide — without live GitHub publish.

## Commands

```bash
cd /e/Web\ Projects/miro-workspace/miro
signet --version   # 0.5.15
signet doctor
signet ship --plan
signet build --require-sums-sign          # desktop build + soft-skip Expo
signet verify
signet inspect --file "apps/miro-desktop/.../Miro Desktop_0.3.0_x64-setup.exe"
signet release --tag v0.3.0 --dry-run
# CI emit in a temp project copy (not committed to Miro):
signet ship --ci
# Collect smoke (real Win installers + placeholder .dmg/.AppImage):
signet ship --collect ./artifacts-in
```

## Results vs shortcomings / fix order

| Item | Ver. | Trial |
|------|------|--------|
| Coverage plan + doctor `ship-coverage` + build upfront gap | 0.5.9 | **Pass** — gap `macos, linux, android, ios`; windows present |
| Soft-fail unpaid `[[targets]]` | 0.5.10 | **Pass** — Expo skipped with debt note; desktop signed; exit 0 |
| `release --dry-run` read-only | 0.5.11 | **Pass** — SHA256SUMS hash unchanged; prints read-only note |
| `ship --ci` + `--collect` + release coverage gate | 0.5.12 | **Pass** — matrix win/mac/linux; collect filled mac/linux from staged files; dry-run warns live release needs `--allow-partial` |
| Mobile platforms via Expo target | 0.5.13 | **Pass** — android/ios declared; CI emits `ship-android` / `ship-ios`; gap until APK/IPA |
| `[ship] path=graduate` on same plan | 0.5.14 | **Pass** — windows/macos `graduate:MISSING` until Azure/OV/Apple configured; linux integrity |
| Release auth guided path | 0.5.15 | **Pass** — doctor numbered setup; dry-run `auth: NOT READY` + guide |

### Host Windows Sign → Prove → Check

- Full `signet build`: NSIS + MSI + exe, Authenticode + timestamp
- Soft-fail: `warning: target miro-mobile … skipped — unpaid recipe` then continues to sign
- `signet verify`: checksums **ok**, minisig **ok**
- Inspect setup.exe: **signed** (untrusted root — expected)
- Dry-run: 6 assets → `Dendro-X0/Miro` `v0.3.0`; coverage + auth warnings; sums untouched

### Still expected on this laptop alone

- Real macOS/Linux/APK/IPA need GitHub `ship --ci` (or other hosts) + `--collect`
- Live release blocked by coverage gap (without `--allow-partial`) and missing `gh` / `GH_TOKEN`
- Expo still unpaid (`build_command` empty) — intentional debt
- CI template is generic (add Node/pnpm/Tauri steps for Miro before relying on matrix)

## Config

```toml
[platforms]
windows = true
macos = true
linux = true
# android/ios implied by expo [[targets]]

[[targets]]
id = "miro-desktop"
framework = "tauri"
app_root = "apps/miro-desktop"
build_command = "pnpm desktop:release"

[[targets]]
id = "miro-mobile"
framework = "expo"
app_root = "apps/miro-mobile"
# build_command TBD
```

## Next (optional ops)

1. Commit `signet ship --ci` into Miro; restore `.signet/identity` via Actions secrets.
2. Run matrix → `--collect` → close desktop gap; add Expo/Android recipe when ready.
3. `gh auth login` (or `GH_TOKEN`) then live `signet release` (or intentional `--allow-partial`).
