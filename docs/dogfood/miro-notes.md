# Dogfood notes — Miro Desktop (+ mobile surface)

**Status:** Sign → Prove → Check green on Windows for **v0.3.0** (self-signed; SmartScreen still warns)  
**App:** Miro (`E:\Web Projects\miro-workspace\miro`) — Tauri v2 desktop + Expo mobile in-repo  
**Framework:** `tauri` · `app_root = apps/miro-desktop` · `build_command = pnpm desktop:release`  
**Date:** 2026-08-03 (Signet **0.5.6** fresh pass after Miro `0.3.0` tag)  
**Host:** Windows x86_64  
**Signet:** 0.5.6 (`%LOCALAPPDATA%\Signet\bin` / `~/bin`)

## Goal

Complete Sign → Prove → Check on a real installable app; note cross-platform / monorepo friction for Tauri + Expo.

## Commands used (this pass)

```bash
cd /e/Web\ Projects/miro-workspace/miro
signet doctor
signet scan
# After setting build_command = "pnpm desktop:release" in signet.toml:
signet build --require-sums-sign   # ~5 min cold-ish; icons + sidecar + Next export + cargo + NSIS/MSI + Authenticode + minisig
signet verify
signet inspect --file "apps/miro-desktop/src-tauri/target/release/bundle/nsis/Miro Desktop_0.3.0_x64-setup.exe"
signet release --tag v0.3.0 --dry-run
```

Config that made monorepo build work:

```toml
[project]
name = "Miro Desktop"
app_root = "apps/miro-desktop"
framework = "tauri"
build_command = "pnpm desktop:release"

[platforms]
windows = true
macos = true
linux = true

[release]
github = true
repo = "Dendro-X0/Miro"
attach_trust = true
```

## Outcome

| Step | Result |
|------|--------|
| Doctor | Host tooling OK; `github-auth` missing (`gh` / `GH_TOKEN`) — blocks live `signet release` only |
| Scan | Detects **tauri** + nested **rust_cli** + **expo**; after build, 3 Windows installers |
| Build | NSIS + MSI + exe discovered; `pnpm desktop:release` honored from repo root |
| Prove | `SHA256SUMS` + `.minisig`; verify resolves basenames to on-disk paths |
| Sign | Authenticode `signtool+timestamp` on all three |
| Check | `inspect` → `status=signed` with honest “untrusted root /pa” detail |
| Release dry-run | Would attach 6 assets to `Dendro-X0/Miro` tag `v0.3.0` (needs `gh`) |

## Experience (human / agent)

**What worked well**

1. **One command end-to-end** once `build_command` is set — no manual `signtool` / PFX juggling.
2. **Honesty in inspect/verify** — self-signed is labeled clearly; SmartScreen not oversold.
3. **Monorepo recovery** — without `build_command`, `--skip-build` fails with a useful hint; with it, Signet runs from the `signet.toml` directory (correct for `pnpm desktop:release`).
4. **Release dry-run notes** — fingerprint + verify steps are paste-ready for GitHub Release body.
5. **Doctor** — quick confidence that signtool/OpenSSL/minisign keys exist before a 5-minute build.

**Friction**

1. **Stale tree before rebuild** — leftover `SHA256SUMS` for `0.2.0` basenames + empty `target/release/bundle` → `verify` said “no listed files found on disk.” Easy to misread as a Signet bug; it’s an artifact lifecycle issue.
2. **Scan root display** shows `root = ./miro` inside the Miro repo (cosmetic confusion).
3. **Triple project detection** (tauri + rust_cli under src-tauri + expo) without `[[targets]]` guidance in the default next-steps — agent may wonder which to ship.
4. **`platforms.macos/linux = true` in toml** vs scan suggestion `macos=false linux=false` on a Windows host — intent vs host capability not explained in one line.
5. **Cross-platform “self-sign Miro” on one machine** only covers Windows Authenticode. macOS/Linux codesign paths are documented but not exercised here; Expo is discovered but not part of `signet build` Tauri flow.
6. **`signet identity status`** — docs/habits may say “status”; CLI wants `identity` list / `identity show` (subcommand mismatch).
7. **Release blocked** without `gh` / `GH_TOKEN` even when git remote + tag already exist locally — dry-run is fine; publish needs an extra install.
8. **SHA256SUMS still basename-only** after 0.5.6 relative-path work — verify still succeeds (path resolution), but committing basenames is fragile if two folders share a filename; prefer always writing relative paths into the file.
9. **First useful check ~5 minutes** — dominated by Tauri/cargo, not Signet; still, a progress/ETA for “framework build vs sign phase” would help agents decide `--skip-build`.

## Cross-platform / multi-surface gaps (Miro-specific)

| Surface | Signet today | Gap |
|---------|--------------|-----|
| Windows desktop | Full Sign→Prove→Check | SmartScreen reputation (graduation) |
| macOS / Linux desktop | Config flags + docs | Need macOS/Linux CI hosts; not dogfooded this session |
| Expo mobile | Scan detects | No default `[[targets]]` recipe; APK/IPA ≠ Authenticode — needs android/ios helpers + honesty |

## Suggested improvements (Signet)

### P0 — correctness / trust kit

1. ~~**Always rewrite `SHA256SUMS` after Authenticode** and log `wrote SHA256SUMS` again~~ — **0.5.7** logs `wrote … (post-sign)`.
2. ~~**Persist relative paths** / document flat release~~ — build relative (0.5.6); release basenames documented in `docs/signing.md` (0.5.7).
3. ~~**Stale sums warn**~~ — **0.5.7** `signet verify` warns; `--fail-stale` hard-fails.

### P1 — monorepo UX

4. ~~**Scan next-steps** for `[[targets]]`~~ — **0.5.7**.
5. ~~Clarify **platforms.\*** vs host~~ — **0.5.7** scan note.
6. ~~Fix scan `root = ./miro` when cwd is app root~~ — **0.5.7** shows `.`.

### P2 — agent / CLI polish

7. ~~Alias `signet identity status`~~ — **0.5.7**.
8. **`signet build` phase banner**: `framework-build | discover | checksum | host-sign | sums-sign` with timings.
9. Optional `signet build --target desktop` example in Miro-shaped template when Expo is co-detected.
10. Doctor: if `release.repo` set but `gh` missing, print the exact Windows install one-liner (already in install.md — link it).

### P3 — graduation / reputation

11. Keep OV/Azure/notarize out of default build (good); add a single `signet graduate status` that reads Miro’s `[graduation]` stubs and says “not configured.”

## Time-to-first-useful check

~5 min for `signet build --require-sums-sign` on this machine after config fix (icons + sidecar + Next export + Rust release + NSIS/MSI + sign).  
`--skip-build` afterward is seconds.

## Related Miro tree notes

- `frontendDist` must remain `../../miro-web/out` (relative to `src-tauri`).
- Prefer committing updated `signet.toml` (`build_command`, `release.repo`) with the trust kit; keep `.signet/` gitignored.
- Local tag `v0.3.0` exists; `signet release` still needs GitHub auth to publish assets.
