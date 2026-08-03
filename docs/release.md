# Release

`signet release` gathers signed (or built) artifacts, refreshes `SHA256SUMS`, optionally attaches `TRUST.md`, and publishes a GitHub Release for a tag.

`--dry-run` lists assets and notes only — it does **not** rewrite or create `SHA256SUMS` / signatures. Live release still flattens sums to asset basenames for GitHub.

## Usage

```bash
signet release --tag v1.0.0
signet release --tag v1.0.0 --dry-run
signet release --tag v1.0.0 --repo owner/name
signet release --tag v1.0.0 --draft --prerelease
signet release --tag v1.0.0 --artifact path/to/setup.exe
signet release --tag v1.0.0 --no-trust
signet release --tag v1.0.0 --no-clobber
```

## Auth

Live publish needs **ready** GitHub auth. `signet doctor` reports `github-auth`; dry-run prints `auth: …` without failing.

### Preferred: GitHub CLI

1. Install [GitHub CLI](https://cli.github.com/) (`winget install GitHub.cli`, `brew install gh`, …).
2. `gh auth login` (browser or device flow).
3. Confirm with `gh auth status`.

Having `gh` on `PATH` without a login is **not** enough — Signet treats that as not ready and prints setup steps.

### Alternative: token

Set `GH_TOKEN` or `GITHUB_TOKEN` to a classic PAT with **`repo`** scope (or a fine-grained token with Contents + Metadata write on the target repo). Prefer env / CI secrets — never commit tokens into `signet.toml`.

### Surfaces

| Command | Behavior |
|---------|----------|
| `signet doctor` | Status + setup guide when not ready |
| `signet release --dry-run` | Shows `auth:` line |
| `signet release` (live) | PrefLights; refuses with the same guide if not ready |
| Guided Release | Blocks publish until auth is ready |

## Repo resolution

1. `--repo owner/name`
2. `[release] repo` in `signet.toml`
3. `git remote get-url origin` parsed as GitHub
