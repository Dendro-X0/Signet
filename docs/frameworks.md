# Hybrid frameworks (Flutter / React Native / Expo / Capacitor)

Signet’s artifact contract plugs these stacks into **discover → checksum → host sign (desktop) → release**. Mobile store trust stays external — use [`android.md`](android.md), [`ios.md`](ios.md), and [`graduation.md`](graduation.md) as needed.

## Config

```toml
[project]
name = "my-app"
framework = "flutter"   # flutter | react-native | rn | expo | capacitor
tauri_root = "."        # app root (legacy field name)
build_command = "flutter build apk"   # REQUIRED for `signet build` (no target guess)
```

If `framework` is **omitted**, Signet resolves it via `signet scan` (same preference as Init). Prefer an explicit value in committed configs.

```bash
signet build --skip-build          # discover + sums (+ host sign desktop artifacts)
signet android sign --apk path.apk # APK crypto sign
signet ios package --app App.app   # IPA zip helper
signet inspect --file path.apk
```

## Framework notes

| Id | Discover roots (common) | Build |
|----|-------------------------|-------|
| `flutter` | `build/windows`, `build/macos`, `build/linux`, `build/app/outputs`, `build/ios`, `dist` | e.g. `flutter build apk` / `windows` / `macos` |
| `react-native` (`rn`) | `android/app/build/outputs`, `ios/build`, `dist` | your RN release script |
| `expo` | `dist`, android/ios outputs | local EAS / export; **cloud EAS is external** — download then `--skip-build` |
| `capacitor` | `android/app/build/outputs`, `ios/App/build`, `dist` | `npx cap sync` + native pack scripts |

## Rust CLI / binary (`framework = "cli"`)

For Cargo workspaces and binary crates that are **not** installable UI apps (this Signet repo is an example):

```toml
[project]
name = "signet"
framework = "cli"
tauri_root = "."          # workspace or package root
# build_command = ""      # optional — defaults to cargo build --release
```

`signet scan` / guided init prefer a root Rust workspace over nested `demo/` / `fixture` Electron samples. Discover looks under `target/<profile>/` (including the workspace `target/` when `tauri_root` is a member crate).

## Honesty

- Setting `framework` does **not** enable Play App Signing or App Store upload.
- APKs discovered by hybrid adapters are checksummed; sign them with `signet android` (or ship unsigned for sideload tooling you control).
- IPAs: package/sign with Apple tooling; Signet packages from `.app` only.

## Related

- [signing.md](signing.md)
- Spec: [`specs/backend/hybrid-framework-adapters-design.md`](../specs/backend/hybrid-framework-adapters-design.md)
