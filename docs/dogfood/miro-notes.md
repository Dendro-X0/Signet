# Dogfood notes — Miro Desktop (+ mobile surface)

**Status:** Sign → Prove → Check green on Windows for **v0.3.0** with Signet **0.5.7**; remaining apply/basename friction addressed in **0.5.8** (pending re-dogfood)  
**App:** Miro (`E:\Web Projects\miro-workspace\miro`) — Tauri v2 desktop + Expo mobile in-repo  
**Framework:** `tauri` · `app_root = apps/miro-desktop` · `build_command = pnpm desktop:release`  
**Date:** 2026-08-03 (re-run after Signet 0.5.7)  
**Host:** Windows x86_64  
**Signet:** 0.5.7 (notes); workspace now **0.5.8**

## Goal

Re-dogfood Miro after Signet 0.5.7 (stale-sums, post-sign log, scan clarity, `identity status`).

## Commands (0.5.7 pass)

```bash
cd /e/Web\ Projects/miro-workspace/miro
signet --version          # 0.5.7
signet doctor
signet scan
signet build --skip-build --require-sums-sign
signet verify
signet verify --fail-stale
signet identity status    # alias works
signet inspect --file "apps/miro-desktop/src-tauri/target/release/bundle/nsis/Miro Desktop_0.3.0_x64-setup.exe"
signet release --tag v0.3.0 --dry-run
# optional:
# signet scan --apply     # see caveats below
```

## Outcome

| Step | 0.5.6 | 0.5.7 |
|------|-------|-------|
| Doctor | OK (no `gh`) | Same |
| Scan root display | `./miro` (confusing) | `.` (fixed) |
| Multi-app next steps | Generic build | Suggests `[[targets]]` / `scan --apply` |
| Platforms intent vs host | Unclear | Explicit one-liner note |
| Post-sign sums log | Only minisig | `wrote SHA256SUMS (post-sign)` |
| SHA256SUMS paths | Often basenames | **Relative monorepo paths** after this build |
| `identity status` | Missing subcommand | Works (alias → show) |
| `verify --fail-stale` | N/A | Passes when relative paths present |
| Inspect honesty | signed + untrusted root | Same (good) |
| Release dry-run | Needs `gh` to publish | Same |

## What improved (matches prior Miro feedback)

1. Post-sign checksum rewrite is **logged** and relative paths are written — `verify` + `--fail-stale` green after `--skip-build`.
2. Scan explains **shipping intent vs host can sign today**.
3. Multi-installable detection (tauri + expo) pushes toward `[[targets]]`.
4. `signet identity status` works.

## Remaining friction

1. ~~**`signet scan --apply` overwrote `[platforms]`**~~ — **0.5.8** never shrinks without `--force`.
2. ~~**`scan --apply` did not draft `[[targets]]`** / identity create spam~~ — **0.5.8** drafts on empty targets; skips identity hint when present.
3. ~~**Basename-only leftover sums**~~ — **0.5.8** bounded walk resolves basenames for verify + stale.
4. **`github-auth` still missing** — dry-run OK; live release blocked (expected).
5. **Expo** still not in the default build loop — intentional until Miro adds a mobile `[[targets]]` + android/ios recipe.

## Suggested follow-ups (0.5.9+)

| Priority | Item |
|----------|------|
| P2 | Optional `signet build` phase timings (framework vs sign) |
| P3 | Doctor: link `gh` install one-liner when `release.repo` set |

## Config kept on Miro

```toml
[project]
name = "Miro Desktop"
app_root = "apps/miro-desktop"
framework = "tauri"
build_command = "pnpm desktop:release"

[platforms]
windows = true
macos = true   # shipping intent (restored after scan --apply)
linux = true

[release]
repo = "Dendro-X0/Miro"
```

## Time

`--skip-build` re-sign + verify: ~10s. Full `pnpm desktop:release` via Signet still ~5 min when needed.
