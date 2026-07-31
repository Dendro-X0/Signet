# Design stub: artifact contract

**Phase:** 9  
**Status:** stub — **do not implement** until Phases 6–8 are done  
**Depends on:** trust tiers, verify, checksum signing  

## Problem

`signet build` is Tauri-shaped (discover under `src-tauri` bundles). Release, checksums, host sign, and trust should stay framework-agnostic so Electron/Android adapters do not fork those pipelines.

## In scope (when implemented)

- Shared types: `Artifact { path, platform, kind, name_for_sums }`.
- Pipeline stages: `discover → (optional build) → host_sign → write_sums → sums_sign → collect_release`.
- Trait or module boundary: `FrameworkAdapter` with `discover(root, cfg) -> Vec<Artifact>` and optional `build(...)`.
- Move Tauri discovery behind the contract without changing CLI UX.

## Out of scope

- New frameworks’ full build orchestration details (those live in adapter specs).
- Changing GitHub release auth.

## Owner modules (intended)

| Stage | Module |
|-------|--------|
| Contract types | `crates/signet/src/artifact/` (new) |
| Tauri adapter | `sign/discover.rs` (refactor behind trait) |
| Host sign | `sign/mod.rs` |
| Sums + sig | `sign/checksum.rs`, Phase 8 sums_sig |
| Release collect | `release/collect.rs` |

## Do not implement until

- Phase 7 `signet verify` exists.
- Phase 8 minisign path exists (or explicitly waived by maintainer).

## Open questions

- Should `kind` enumerate `nsis | msi | dmg | appimage | exe | apk | ipa | zip` in v1 of the contract?
- Dry-run JSON for adapters for agent consumption?
