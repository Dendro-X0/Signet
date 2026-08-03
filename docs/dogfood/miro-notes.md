# Dogfood notes — Miro Desktop (+ mobile surface)

**Status:** Windows Sign → Prove → Check green for **v0.3.0** with Signet **0.5.8**; **multi-platform product gap confirmed** (see design below)  
**App:** Miro (`E:\Web Projects\miro-workspace\miro`) — Tauri v2 + Expo in-repo  
**Date:** 2026-08-03  
**Host:** Windows x86_64  
**Signet:** 0.5.8  
**North star:** [`specs/backend/multi-platform-ship-design.md`](../../specs/backend/multi-platform-ship-design.md)  
**Shortcomings backlog:** [`signet-shortcomings.md`](signet-shortcomings.md) (fix one by one)

## Goal

From the **user/business** bar: rapid signing across **all declared platforms**, self-sign **and** assisted official paths, desktop **and** mobile — not “whatever this laptop can crypto-sign.”

Attempt `signet build` once with `[platforms] windows/macos/linux = true` and both `[[targets]]`, and record the gap.

## One-go attempt (all platforms / all targets)

```bash
cd /e/Web\ Projects/miro-workspace/miro
signet build --require-sums-sign
```

| Step | Result |
|------|--------|
| `miro-desktop` Tauri build | **Pass** (~2.5 min) — NSIS + MSI + exe on this host |
| `miro-mobile` Expo build | **Fail** — `[project].build_command is required` (no Expo recipe) |
| macOS / Linux installers | **Not produced** — Tauri only bundles the host OS; no DMG/deb/AppImage on disk |
| Host crypto sign for mac/linux | **N/A** — Signet only host-signs matching OS (`codesign` / openssl detached need macOS / Linux hosts) |

Scan today: *“other OS assets need a matching CI/host.”* That is an **engine constraint**, not the product ceiling — Signet must **own** that CI/host path (ship plan → matrix → collect → one trust kit).

### Completed Windows path (after targeting desktop)

```bash
signet build --target miro-desktop --skip-build --require-sums-sign
signet verify
signet trust
signet release --tag v0.3.0 --dry-run
```

→ 3 Authenticode-signed artifacts + minisig sums + TRUST.md; dry-run lists 6 release assets (windows only).

### Friction found this pass

1. **Naive one-go fails mid-pipeline** — desktop build succeeds, then Expo aborts the whole command (desktop artifacts exist but Signet never reaches the sign step in that invocation). Prefer `signet build --target miro-desktop` until Expo has a `build_command`, or use `--skip-build` for sign-only.
2. **`[platforms] macos/linux=true` does not trigger remote/CI builds** — no orchestrator; maintainers need a matrix (3 hosts or GH Actions) each running `signet build` then merging assets into one release.
3. **`signet release --dry-run` rewrote `SHA256SUMS` to basenames** — verify still resolves via tree walk, but dry-run mutating on-disk sums is surprising (should be no-op for files).

## Outcome vs prior asks (0.5.7 → 0.5.8)

| Prior ask | 0.5.8 |
|-----------|--------|
| Don’t shrink `[platforms]` without `--force` | **Pass** |
| Draft `[[targets]]` for tauri+expo | **Pass** |
| Skip “identity create” when identity exists | **Pass** |
| Basename-only sums resolve | **Pass** |
| All-OS facilitation as a product | **Fail today** — host limits explain *how*, not *whether* Signet must own the job |

## Product verdict (maintainer)

Windows-only local success is **insufficient**. If Signet cannot facilitate the declared multi-platform release (orchestrate matching runners, collect, one trust kit, dual-path graduate), it does not earn a dedicated tool. Track work in **multi-platform-ship-design** — not as a footnote on `doctor`.

## Suggested Signet follow-ups (map to design slices)

| Priority | Item | Slice |
|----------|------|-------|
| P0 | Ship plan: declared platforms = coverage commitment; doctor/ship report gaps | A |
| P0 | CI template + collect + release coverage gate | D–E |
| P1 | Soft-fail targets missing `build_command`; don’t abort sibling desktop sign | B |
| P1 | Graduate profile on same ship plan (Azure / notarize) | F |
| P2 | `release --dry-run` read-only | C |
| P2 | Mobile rows in ship plan | G |

## Config (shipping intent)

```toml
[platforms]
windows = true
macos = true
linux = true

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

## Time

Full desktop build+sign on Windows: ~3 min. All-OS one-shot: not achievable on this machine alone.
