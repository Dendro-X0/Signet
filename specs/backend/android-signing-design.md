# Design stub: Android signing helpers

**Phase:** 11  
**Status:** stub — **blocked on Phases 6–9**  
**Depends on:** trust model honesty, verify, artifact contract  

## Problem

Android **requires** a signature for every installable APK/AAB. Indie/OSS self-signing (local keystore) is the normal sideload/F-Droid path. Play distribution uses **Play App Signing** (upload key ≠ app signing key). Signet must help the first path and document the second without conflating them.

## In scope (when implemented)

- Create/import a release keystore under `.signet/android/` (gitignored).
- Sign APK (and document AAB upload-key flow) via `apksigner` / `jarsigner` when SDK tools exist.
- Emit fingerprint / cert digests into `TRUST.md` Android section.
- `signet doctor` checks for `apksigner` / Android SDK hints.
- Scan → suggest Android platforms when Gradle/manifest found.

## Out of scope

- Replacing Play App Signing or storing Google Play upload credentials in Signet cloud.
- Claiming Play Store install trust from a local keystore.
- iOS (Phase 12).

## Trust honesty (required in UX copy)

| Channel | Signet role |
|---------|-------------|
| Sideload / F-Droid-style | Manage developer keystore; verify checksums + cert digest |
| Google Play (new apps) | Document upload key vs app signing key; do not pretend local key is the Play distribution key |

Tier note: local APK sign → integrity; `play_managed` only when declared for Play releases.

## Do not implement until

- Phases 6–8 (trust + verify + checksum signing) are available so Android artifacts participate in the same verify story.
- Phase 9 artifact contract can represent `apk` / `aab` kinds.

## Open questions

- Default to `apksigner` (API 30+ alignment) vs older `jarsigner` fallback?
- One keystore per app vs shared Signet identity mapping?
