# Design: artifact contract

**Phase:** 9  
**Status:** implemented  
**Depends on:** Phases 6–8 (trust, verify, checksum signing)  
**Owners:** `crates/signet/src/artifact/` (new), `sign/discover.rs` (Tauri behind adapter), `commands/build.rs`, `release/collect.rs`  
**Plan alignment:** unlock Phase 10 Electron without forking host sign / sums / release.

## Problem

`signet build` and release collect are Tauri-shaped (`src-tauri/target/.../bundle`). Host sign, `SHA256SUMS`, minisign, TRUST, and GitHub publish should stay framework-agnostic so a second adapter plugs in without copying those pipelines.

## Goals

1. Shared `Artifact` type used by discover → host_sign → sums → sums_sign → release collect.
2. `FrameworkAdapter` boundary: optional `build` + required `discover`.
3. Move Tauri discovery/build behind the contract **without changing CLI UX** (same flags, same default paths).
4. Config declares `project.framework` (default `"tauri"`); unknown frameworks fail with a clear Phase-N hint.

## Non-goals

- Implementing Electron / Android / iOS adapters (Phases 10–12).
- Changing GitHub auth or verify exit codes.
- Replacing host PE/Mach-O/OpenSSL signing backends.

---

## Types

```text
ArtifactKind  — snake enum ids stable for JSON / agents
Artifact      — path, kind, name_for_sums (release asset basename)
```

### `ArtifactKind` (v1)

Keep existing Signet kinds; reserve mobile/archive for later adapters (classify may return them; host_sign ignores until Phase 11–12):

| Variant | `as_str` | Host-sign today |
|---------|----------|-----------------|
| `WindowsExe` | `windows-exe` | yes (Windows) |
| `WindowsMsi` | `windows-msi` | yes |
| `MacApp` | `macos-app` | yes |
| `MacDmg` | `macos-dmg` | yes |
| `LinuxAppImage` | `linux-appimage` | yes |
| `LinuxDeb` | `linux-deb` | yes |
| `LinuxRpm` | `linux-rpm` | yes |
| `Apk` | `android-apk` | no (Phase 11) |
| `Ipa` | `ios-ipa` | no (Phase 12) |
| `Zip` | `zip` | no |
| `Other` | `other` | no |

**Decision (frozen):** enumerate these in v1; do not invent `nsis` as a separate kind (NSIS setup `.exe` is `WindowsExe`).

### `Artifact`

| Field | Meaning |
|-------|---------|
| `path` | Absolute or project-relative path on disk |
| `kind` | `ArtifactKind` |
| `name_for_sums` | Basename (or unique release asset name) used in `SHA256SUMS` / GitHub |

`DiscoveredArtifact { path, kind }` becomes a thin compatibility alias or `From` into `Artifact` (name defaults to file name).

---

## `FrameworkAdapter`

```text
id() -> &'static str
label_root(ctx) -> PathBuf          # printed as “tauri crate: …” today
build(ctx, BuildOpts) -> Result<()> # no-op / Err if unsupported; Tauri runs cargo-tauri
discover(ctx, profile) -> Vec<Artifact>
```

`BuildOpts`: `profile`, `extra_args`, `skip_build` (when true, `build` is not called by the pipeline).

### Selection

```toml
[project]
name = "my-app"
tauri_root = "."
framework = "tauri"   # default when absent
```

| Value | Adapter |
|-------|---------|
| `tauri` / absent | `TauriAdapter` |
| other | hard error: “framework X not supported yet (see Phase 10+)” |

Scan may later suggest `framework`; Phase 9 does not require scan changes.

---

## Pipeline (shared)

Stages owned **outside** adapters:

```text
1. select_adapter(config)
2. if !skip_build → adapter.build(...)
3. artifacts = explicit --artifact OR adapter.discover(...)
4. write SHA256SUMS (all file artifacts)
5. if !no_sign → host_sign(host_signable(artifacts))
6. refresh SHA256SUMS; maybe_sign_sums (Phase 8)
7. release collect reuses discover + sums + attach sigs/TRUST
```

Adapters must not write `SHA256SUMS` or call GitHub.

### Agent / dry-run JSON

Optional helper `artifacts_json(&[Artifact])` for future `--json` discover. Phase 9: unit-tested serializer; release dry-run may keep human listing (already prints assets). Full `signet build --json` is **not** required for exit.

---

## Module layout

| Path | Role |
|------|------|
| `artifact/mod.rs` | re-exports, `select_adapter` |
| `artifact/kind.rs` | `ArtifactKind` |
| `artifact/types.rs` | `Artifact` |
| `artifact/adapter.rs` | trait + `BuildOpts` |
| `artifact/pipeline.rs` | shared sums helpers used by build/release if useful |
| `artifact/tauri.rs` | `TauriAdapter` — wraps current discover + tauri CLI build |
| `sign/discover.rs` | keep low-level Tauri filesystem walk; called by `TauriAdapter` |
| `sign/mod.rs` | `sign_host_artifacts` accepts `&[Artifact]` (or Convert) |

---

## CLI / UX invariants

- Existing flags unchanged (`--skip-build`, `--no-sign`, `--artifact`, `--tauri-arg`, sums-sign flags).
- Default projects without `framework` behave exactly as today.
- Error when no artifacts: still mention Tauri bundle path when adapter is Tauri; generic message for others later.

---

## Acceptance

- [x] Design status `ready` then `implemented`.
- [x] `FrameworkAdapter` + `TauriAdapter` exist; build/release call them.
- [x] `cargo test -p signet` green; clippy `-D warnings`.
- [x] Explicit `--artifact` path still works without adapter discover.
- [x] Adding a stub second adapter would only need a new module + match arm (documented; not shipped).

**Status:** implemented (2026-07-31)

## Proof plan

| Layer | Evidence |
|-------|----------|
| L1 | Unit: kind ids, Artifact name_for_sums, adapter discover fixture (existing Tauri fixture via adapter) |
| L2 | `cargo test -p signet` + `cargo clippy -p signet -- -D warnings` |
| L3 | `signet build --skip-build --no-sign --artifact …` still writes sums (+ minisig if key) |

## Subtraction

- Do not duplicate checksum/sign/release logic inside adapters.
- Do not rename CLI commands or break `signet.toml` without defaulting `framework`.

## Open questions — resolved

| Question | Decision |
|----------|----------|
| Enumerate kind including apk/ipa/zip? | Yes, reserved variants; host_sign ignores until later phases |
| Dry-run JSON for adapters? | Helper + tests in Phase 9; full CLI JSON later |
