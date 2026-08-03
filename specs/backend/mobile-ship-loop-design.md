# Mobile through ship loop (slice G)

## Plan alignment

- **Handoff:** User chose fix order **#6** (skip #5 graduate for this session). Shortcoming item 4; parent design slice G.
- **Band:** 0.5.13
- **PAUSED/CANCELLED:** none
- **In scope:** Android-first (APK) + iOS honesty in coverage / CI / collect / release; `[platforms].android|ios`; mobile gap in release gate
- **Out of scope:** Play/App Store upload; EAS cloud; graduate mobile; full Expo recipe defaults

## Contracts

### Config

```toml
[platforms]
windows = true
macos = true
linux = true
android = false   # NEW — default false
ios = false       # NEW — default false
```

**Declared mobile** = `platforms.android|ios` **OR** any `[[targets]]` / project framework in `{android,expo,react-native,flutter,capacitor}` → android; `{ios,expo,...}` → ios for Expo/Capacitor/RN/Flutter also imply android (and ios for expo/rn/capacitor/flutter/ios).

### Coverage

- Track `present.android` / `present.ios` from scan, discover, staging, SHA256SUMS (`.apk`/`.aab` / `.ipa`)
- `gap` includes `android` / `ios` when declared and missing
- `summary_line` and `print_human` show mobile when any mobile declared
- Release gate uses full `has_gap()` (desktop + mobile)

### CI (`ship --ci`)

- Keep desktop matrix
- If android declared: add `ship-android` job (`ubuntu-latest`, upload `**/*.apk`, `**/*.aab`)
- If ios declared: add `ship-ios` job (`macos-latest`, upload `**/*.ipa`, note Xcode / `signet ios package`)

### Collect / release

- Collect already accepts apk/ipa — keep
- `classify_kind`: `.apk`/`.aab` → `android`; `.ipa` → `ios`
- Upload artifact globs include apk/aab/ipa

### Check honesty

- Ship plan notes: Android keystore ≠ Play; iOS free provisioning / no App Store via Signet

## Acceptance

- [x] AC1 — platforms.android/ios round-trip in config
- [x] AC2 — expo target alone declares android (+ ios) commitment; gap until APK/IPA present
- [x] AC3 — CI template includes ship-android when android declared
- [x] AC4 — release classifies apk as android; gap fails without --allow-partial
- [x] AC5 — unit tests for mobile commitment + classify

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet` |
| L1 | `cargo clippy -p signet -- -D warnings` |
