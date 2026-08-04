# GitHub auth: confirmed browser open (0.5.16)

## Plan alignment

- **User ask:** Next update — confirmed terminal step that opens the target GitHub URL in the browser so credentials can be configured without manual navigation.
- **Band:** 0.5.16
- **PAUSED/CANCELLED:** none
- **Owner:** `release/auth.rs`, doctor / release dry-run / guided release
- **Out of scope:** Automating `gh auth login` non-interactively; storing tokens

## Contracts

### URLs

| Auth kind | Browser URL |
|-----------|-------------|
| Missing (`gh` not installed) | `https://cli.github.com/` |
| `gh` installed, not logged in | `https://github.com/settings/tokens/new?scopes=repo&description=signet-release` |
| Ready | no offer |

Token page is the fastest credential path when `gh` exists but login is unfinished; guide still mentions `gh auth login`.

### Offer flow

1. Print setup guide (existing).
2. If not ready, TTY stdin, and not `--json` / non-interactive: prompt  
   `Open <url> in your browser now? [y/N]`
3. On yes: open URL via OS (`cmd /C start`, `open`, `xdg-open`); print note on failure.
4. Never open without confirmation. Skip when stdin is not a terminal (CI).

### Surfaces

- `signet doctor` (human mode)
- `signet release --dry-run` when auth not ready
- Guided release / guided setup when publish blocked on auth
- Live release preflight: offer once before bail (TTY only)

## Acceptance

- [x] AC1 — `setup_browser_url()` returns expected URLs per kind
- [x] AC2 — offer skipped when ready or non-TTY
- [x] AC3 — doctor/dry-run call offer after guide
- [x] AC4 — docs/release.md + CHANGELOG 0.5.16

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet` |
| L1 | `cargo clippy -p signet -- -D warnings` |
