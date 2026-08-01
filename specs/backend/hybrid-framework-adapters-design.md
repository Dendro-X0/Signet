# Design: Flutter / React Native / Expo / Capacitor adapters

**Band:** Later (multi-framework)  
**Status:** implemented  
**Depends on:** Phase 9 artifact contract; Electron/Android/iOS adapter patterns  
**Owners:** `artifact/flutter.rs`, `react_native.rs`, `expo.rs`, `capacitor.rs`, `walk_outputs.rs`, `scan/`, `docs/frameworks.md`  
**Plan alignment:** plug second-wave frameworks into discover → host_sign → sums → release without forking pipelines. Honesty: mobile store trust remains external.

## Problem

`signet scan` barely sees hybrid stacks. Maintainers using Flutter, React Native, Expo, or Capacitor need the same post-bundle Signet path Electron already has: discover installers/APKs, optional build wrap, host/Android signing where applicable.

## Goals

1. `framework = "flutter" | "react-native" | "expo" | "capacitor"` selects adapters.
2. Discover shippable artifacts under each stack’s common output dirs.
3. Build: **require** `[project].build_command` (no wrong-target guessing) unless `--skip-build`.
4. Scan detects project kinds + suggests framework id.
5. Doctor optional tool hints (`flutter`, `npm`/`npx`) when that framework is selected.
6. Public doc `docs/frameworks.md`.

## Non-goals

- Injecting signing into Gradle/Xcode/EAS cloud credentials.
- Claiming Play / App Store / TestFlight trust.
- Full EAS cloud orchestration (document `eas build` as external; local outputs only).
- .NET / other stacks (separate band).
- Refactoring Electron onto shared walk (optional later; new adapters use shared helper).

---

## Config

```toml
[project]
name = "my-app"
framework = "flutter"          # or react-native | expo | capacitor
tauri_root = "."               # app root (legacy field name)
build_command = "flutter build apk"   # REQUIRED for build (examples in docs)
```

Aliases: `rn` → `react-native`.

---

## Discover roots (depth-capped, skip node_modules/.git/.signet)

| Framework | Directories (relative to app root) |
|-----------|--------------------------------------|
| Flutter | `build/windows`, `build/macos`, `build/linux`, `build/app/outputs`, `build/ios`, `build/ios/iphoneos`, `dist`, `release` |
| React Native | `android/app/build/outputs`, `ios/build`, `android/app/build/outputs/apk`, `dist`, `release` |
| Expo | same as RN + `dist/` (EAS local / export) |
| Capacitor | `android/app/build/outputs`, `ios/App/build`, `ios/build`, `dist`, `www` (skip non-installable web-only unless classified) |

Collect via `ArtifactKind::classify_*` (exe, msi, dmg, app, appimage, deb, rpm, apk, aab, ipa, zip).

Host sign applies only to host-signable kinds on the current OS; Android APKs use existing android path when `signet android sign` / android adapter patterns — hybrid adapters **discover** APKs; signing still via host sign for desktop artifacts and `signet android` for APK when configured. For v1: hybrid adapters only discover + build wrap; `signet build` host_sign skips APK the same way Tauri/Electron do unless android framework — **APKs are discovered and checksummed; Android signing remains `signet android sign` or framework=android**. Document this.

---

## Build

When not `--skip-build`:

1. If `build_command` empty → error with examples for that framework.
2. Split argv (Electron-style quotes); run in app root; append `extra_args`.
3. Fail on non-zero status.

---

## Scan detection

| Kind | Marker |
|------|--------|
| Flutter | `pubspec.yaml` contains `flutter:` SDK / dependency |
| Expo | `package.json` contains `"expo"` (prefer over RN) |
| React Native | `package.json` contains `react-native` and not classified as Expo |
| Capacitor | `@capacitor/core` in package.json or `capacitor.config.*` |

---

## Acceptance

- [x] Design ready → implemented.
- [x] Four framework ids selectable; fixtures discover apk/exe under listed dirs.
- [x] Empty `build_command` fails clearly without `--skip-build`.
- [x] Scan reports new project kinds.
- [x] `docs/frameworks.md` + config-schema + roadmap.
- [x] `cargo test -p signet` + clippy `-D warnings`.

## Proof

| Layer | Command |
|-------|---------|
| L1 | Unit discover + select_adapter |
| L2 | `cargo test -p signet` + clippy |
| L3 | Optional dogfood on a real Flutter/RN tree |
