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

Prefer `gh` logged in, or set `GH_TOKEN` / `GITHUB_TOKEN`.

`signet doctor` reports `github-auth`.

## Repo resolution

1. `--repo owner/name`
2. `[release] repo` in `signet.toml`
3. `git remote get-url origin` parsed as GitHub
