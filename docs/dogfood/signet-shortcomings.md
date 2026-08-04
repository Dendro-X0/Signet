# Signet — current shortcomings

**Source:** Miro dogfood; last re-trial **Signet 0.5.15** (PATH installer, 2026-08-03).  
**Use:** Address remaining items one at a time.  
**Related:** [`specs/backend/multi-platform-ship-design.md`](../../specs/backend/multi-platform-ship-design.md) · [`miro-notes.md`](miro-notes.md)

---

## Addressed (0.5.9 → 0.5.15) — re-verified on Miro

| # | Was | Status |
|---|-----|--------|
| 1–2, 7, 11–12 | Coverage / platforms commitment / upfront gap / guided honesty | **Done 0.5.9** — `ship --plan`, doctor `ship-coverage`, build gap line |
| 5–6 | Sibling unpaid target aborts build | **Done 0.5.10** — soft-skip + debt; `--strict-targets` |
| 8 | `release --dry-run` mutates sums | **Done 0.5.11** — read-only |
| 1, 9 | No CI / collect / release coverage gate | **Done 0.5.12** — `ship --ci`, `--collect`, fail-closed release |
| 4 | Mobile second-class in ship loop | **Done 0.5.13** — android/ios platforms, CI jobs, classify/collect |
| 3 | Graduate not on same ship plan | **Done 0.5.14** — `[ship] path`, plan backends, CI `graduate apply` |
| 10 | Release auth unclear | **Done 0.5.15** — assessor + doctor/release guide |

---

## Remaining / residual

1. **CI template is generic, not app-ready** — Emitted workflow installs Signet via `cargo install` and calls `signet build`, but Miro still needs Node/pnpm/Tauri (and mobile) steps. Maintainer must hand-edit before matrix is useful.

2. **Identity restore in CI is documented, not automated** — Self-sign on runners still requires wiring `.signet/identity` (and graduate secrets) via Actions secrets; no helper to emit the secret-upload recipe.

3. **Expo unpaid recipe is debt by choice** — Soft-fail works; closing android/ios coverage still needs a real `build_command` / EAS-or-local export path.

4. **End-to-end multi-OS release not dogfooded live** — Collect smoke used placeholder `.dmg`/`.AppImage`. Real macOS/Linux runners + live `signet release` (with auth) still pending as an ops proof, not a missing command.

5. **`--target` unpaid-only fails hard** — `signet build --target miro-mobile` exits error (nothing to produce). Soft-fail applies when siblings discover artifacts. Acceptable; could be clearer messaging.

---

## Out of scope (constraints)

- No cross-compile/sign of macOS/Linux GUI installers on Windows.
- No fake SmartScreen / Gatekeeper / Play / App Store trust.
- Store upload (Play / App Store Connect / full EAS) remains external.

---

## Suggested next (optional)

| Priority | Item |
|----------|------|
| P2 | Framework-aware CI snippets (Tauri/pnpm) in `ship --ci` or docs recipe for Miro |
| P2 | Optional `ship --ci` comments/checklist for identity + graduate secret names |
| P3 | Live Miro matrix → collect → release dogfood when `gh` auth available |
| P3 | Softer copy when `--target` alone is unpaid |
