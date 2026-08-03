# Release GitHub auth guided path (#7)

## Plan alignment

- **Handoff:** Fix order **#7** — Release auth guided path (shortcoming item 10)
- **Band:** 0.5.15
- **PAUSED/CANCELLED:** none
- **In scope:** Shared auth assessor; doctor detail + setup steps; dry-run auth line; live release preflight with guided error; guided release / docs
- **Out of scope:** OAuth device flow inside Signet; storing tokens; non-GitHub hosts

## Problem

Doctor only says “missing — install `gh` or set GH_TOKEN”. Live `signet release` may try `gh`, fail, then API-fail with a short credential message. Maintainers lack a single guided path (`gh auth login` vs token scopes vs install).

## Contracts

### `release/auth.rs`

```text
GithubAuthKind:
  GhLoggedIn | GhInstalledNotLoggedIn | TokenEnv { var } | Missing

assess_github_auth() → report
report.ready() → bool  (GhLoggedIn | TokenEnv)
report.summary_line() / doctor_detail() / setup_guide()
```

- Token env (`GH_TOKEN` / `GITHUB_TOKEN`) counts as ready even without `gh`
- `gh` present is **not** ready unless `gh auth status` succeeds (exit 0)
- `setup_guide()` is numbered, OS-agnostic: install `gh` → `gh auth login` → or classic PAT with `repo` scope as `GH_TOKEN`

### Call sites

| Surface | Behavior |
|---------|----------|
| `signet doctor` | Richer `github-auth` detail; when not ready, print setup guide section |
| `signet release --dry-run` | Print `auth: …` readiness (does not fail) |
| `signet release` (live) | Preflight: bail with setup guide if not ready |
| Guided release / setup | Before publish, show auth status; refuse publish if not ready (print guide) |
| `publish` API path | Reuse same wording in token error |

## Acceptance

- [x] AC1 — `gh` installed but not logged in → not ready
- [x] AC2 — token env alone → ready
- [x] AC3 — live release without auth prints setup guide (bail)
- [x] AC4 — doctor prints setup steps when github-auth fails
- [x] AC5 — docs/release.md Auth section expanded; CHANGELOG 0.5.15

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet` |
| L1 | `cargo clippy -p signet -- -D warnings` |
