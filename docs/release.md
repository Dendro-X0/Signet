# Release (Phase 4)

`selfsign release` gathers signed (or built) artifacts, refreshes `SHA256SUMS`, optionally attaches `TRUST.md`, and publishes a GitHub Release for a tag.

## Command

```bash
selfsign release --tag v1.0.0
selfsign release --tag v1.0.0 --dry-run
selfsign release --tag v1.0.0 --repo owner/name
selfsign release --tag v1.0.0 --draft --prerelease
selfsign release --tag v1.0.0 --artifact path/to/setup.exe
selfsign release --tag v1.0.0 --no-trust
selfsign release --tag v1.0.0 --no-clobber
```

## Auth

Either:

1. [GitHub CLI](https://cli.github.com/) on PATH (`gh auth login`), or
2. `GH_TOKEN` / `GITHUB_TOKEN` with `repo` scope (REST upload API)

`selfsign doctor` reports `github-auth`.

## Repo detection

1. `--repo owner/name`
2. `[release] repo` in `selfsign.toml`
3. `git remote get-url origin` (GitHub SSH/HTTPS)

## Assets included

- Discovered bundle files under `src-tauri/target/{profile}/bundle` (files only; not `.app` dirs)
- Sidecar `*.sig` when present
- `SHA256SUMS` (regenerated to match asset names)
- `TRUST.md` when present and `attach_trust` is enabled

## Config

```toml
[release]
github = true
repo = ""                 # optional owner/name
attach_trust = true
```

## Notes body

Release notes include the app name, self-signed honesty blurb, active identity fingerprint (if available), and verify steps.
