# selfsign.toml schema

Non-secret project configuration. Written by `selfsign init`.

```toml
# selfsign project config (safe to commit)
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

secrets_dir = ".selfsign"
```

| Field | Meaning |
|-------|---------|
| `project.name` | Display / release name |
| `project.tauri_root` | Tauri app root relative to the config file |
| `platforms.*` | Which OS targets this project intends to ship |
| `release.github` | Enable GitHub Releases in `selfsign release` |
| `release.repo` | Optional `owner/name` override |
| `release.attach_trust` | Attach `TRUST.md` when present |
| `secrets_dir` | Relative path for private material (gitignored) |
