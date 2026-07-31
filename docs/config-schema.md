# signet.toml schema

Non-secret project configuration. Written by `signet init`. Legacy `selfsign.toml` is still loaded if `signet.toml` is absent.

```toml
# Signet project config (safe to commit)
# Private keys live under secrets_dir — see docs/secrets-layout.md

[project]
name = "my-app"
tauri_root = "."          # directory containing src-tauri (relative to this file)

[platforms]
windows = true
macos = true
linux = true

[release]
github = true
repo = ""                 # optional owner/name; else git remote / --repo
attach_trust = true

secrets_dir = ".signet"
```

| Field | Meaning |
|-------|---------|
| `project.name` | Display / release name |
| `project.tauri_root` | Tauri app root relative to the config file (Tauri adapter) |
| `platforms.*` | Which OS targets this project intends to ship |
| `release.github` | Enable GitHub Releases in `signet release` |
| `release.repo` | Optional `owner/name` override |
| `release.attach_trust` | Attach `TRUST.md` when present |
| `secrets_dir` | Relative path for private material (gitignored) |
