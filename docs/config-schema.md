# signet.toml schema

Non-secret project configuration. Written by `signet init`. Legacy `selfsign.toml` is still loaded if `signet.toml` is absent.

```toml
# Signet project config (safe to commit)
# Private keys live under secrets_dir — see docs/secrets-layout.md

[project]
name = "my-app"
tauri_root = "."          # app root (src-tauri parent for Tauri; package.json dir for Electron)
framework = "tauri"       # tauri | electron | android | ios | flutter | react-native | expo | capacitor
# build_command = ""      # required for flutter/rn/expo/capacitor/ios build (see docs/frameworks.md)


[platforms]
windows = true
macos = true
linux = true

[release]
github = true
repo = ""                 # optional owner/name; else git remote / --repo
attach_trust = true

# Optional — does not change host PE/codesign behavior
# [trust]
# declared_tier = "self_signed_host"
# notes = ["Beta channel only"]
#
# [trust.checksum_signing]
# minisign = true          # default
# gpg = false              # opt-in → SHA256SUMS.asc
# gpg_key_id = ""

# Optional — OV / Azure / Apple notarization helpers (no secrets)
# [graduation]
# ov_thumbprint = ""
# timestamp_url = "http://timestamp.digicert.com"
# [graduation.azure]
# dlib = ""
# metadata = ""
# timestamp_url = "http://timestamp.acs.microsoft.com"
# [graduation.apple]
# keychain_profile = ""

secrets_dir = ".signet"
```

| Field | Meaning |
|-------|---------|
| `project.name` | Display / release name |
| `project.tauri_root` | App root relative to config (Tauri: dir with `src-tauri`; Electron: `package.json` dir) |
| `project.framework` | Adapter id: `tauri`, `electron`, `android`, `ios`, `flutter`, `react-native`/`rn`, `expo`, `capacitor` |
| `project.build_command` | Optional build argv (required for iOS `signet build` without `--skip-build`) |
| `platforms.*` | Which OS targets this project intends to ship |
| `release.github` | Enable GitHub Releases in `signet release` |
| `release.repo` | Optional `owner/name` override |
| `release.attach_trust` | Attach `TRUST.md` when present |
| `trust.declared_tier` | Optional tier id override (see [trust-model.md](trust-model.md)) |
| `trust.notes` | Extra notes in the TRUST.md Trust tier section |
| `trust.checksum_signing.minisign` | Sign `SHA256SUMS` with `.signet/sums/` minisign key (default true) |
| `trust.checksum_signing.gpg` | Opt-in GPG detach-sign |
| `trust.checksum_signing.gpg_key_id` | Optional GPG key id |
| `graduation.*` | OV thumbprint / Azure dlib+metadata / Apple Keychain profile (see [graduation.md](graduation.md)) |
| `secrets_dir` | Relative path for private material (gitignored) |
