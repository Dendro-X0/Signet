# signet.toml schema

Non-secret project configuration. Written by `signet init`. Legacy `selfsign.toml` is still loaded if `signet.toml` is absent.

```toml
# Signet project config (safe to commit)
# Private keys live under secrets_dir — see docs/secrets-layout.md

[project]
name = "my-app"
app_root = "."            # app root (legacy alias: tauri_root)
framework = "tauri"       # tauri | electron | android | ios | flutter | react-native | expo | capacitor | cli
# build_command = ""      # required for flutter/rn/expo/capacitor/ios build (see docs/frameworks.md)
                          # cli defaults to: cargo build --release
                          # tauri monorepo: e.g. pnpm desktop:release

# Optional monorepo ship targets (omit → single target from [project])
# [[targets]]
# id = "desktop"
# framework = "tauri"
# app_root = "apps/miro-desktop"
# build_command = "pnpm desktop:release"


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
| `project.app_root` | App root relative to config (legacy key: `tauri_root`) |
| `project.framework` | Adapter id: `tauri`, `electron`, `android`, `ios`, `flutter`, `react-native`/`rn`, `expo`, `capacitor`, `cli` |
| `project.build_command` | Optional build argv (required for hybrid/iOS; Tauri monorepo scripts OK) |
| `targets[]` | Optional `[[targets]]` (`id`, `framework`, `app_root`, `build_command`); `signet build --target` |
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
