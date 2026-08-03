# Fault tolerance — design (0.5.7)

## Plan alignment

- **Handoff atomic step:** Maintainer pivot from v1.0.0 gate → Miro dogfood fault tolerance (`docs/dogfood/miro-notes.md` P0/P1).
- **Band:** 0.5.7
- **PAUSED/CANCELLED:** none
- **In scope:** stale SHA256SUMS detection on verify; always log post-sign sums rewrite; scan multi-app `[[targets]]` guidance; platforms intent vs host capability; `identity status` alias; clarify release basenames vs build relative paths.
- **Out of scope:** phase banners / ETA; `graduate status`; cross-host macOS/Linux signing; Expo APK pipeline.

## Meta

Honest Check should not look like a Signet bug when the tree is stale, silent, or multi-app. Prefer **warnings + clear next action** over hard fails; opt into hard fail with `--fail-stale`.

## Ownership & data flow

```
signet build  → write_sha256sums (pre) → host/android sign → write_sha256sums (post) + log
signet verify → verify_sha256sums + assess_sums_freshness → warnings [/ --fail-stale]
signet scan   → finalize_report notes + next_steps ([[targets]], platforms/host)
```

| Concern | Owner |
|---------|--------|
| Freshness heuristics | `sign/checksum.rs` |
| CLI warn / fail | `commands/verify.rs` |
| Post-sign log | `commands/build.rs` |
| Multi-target + platforms copy | `scan/report.rs` |
| Identity alias | `commands/identity.rs` |

## Behavior

### Stale sums (`assess_sums_freshness`)

Given `SHA256SUMS` + search roots + optional project version:

1. Parse all listed names; resolve each; count **listed** / **found** / **missing**.
2. Extract semver-like `X.Y.Z` tokens from listed basenames (unique, sorted).
3. **Empty disk:** `listed > 0 && found == 0` → stale (rebuild or prune).
4. **Version mismatch:** project version known, sums versions non-empty, and **no** sums version equals project version (normalized, strip `v`) → warn mismatch.
5. Partial missing (`0 < found < listed`) → warn with counts (not necessarily version-stale).

`signet verify`:

- Always append human-readable warnings (and JSON `warnings`).
- `--fail-stale`: exit Failure when empty-disk or version-mismatch (not for partial missing alone).

### Post-sign sums rewrite log

After every successful `write_sha256sums` in build (pre-sign, post-host-sign, post-android-sign), print `wrote <path>` (and note `(post-sign)` on the rewrite after host/android sign).

### Relative vs basename paths

- **Build / local Prove:** keep relative paths vs project `SHA256SUMS` dir (0.5.6).
- **Release collect:** keep **flat asset basenames** (GitHub Release layout). Document: consumers download into one dir; `sha256sum -c` works on basenames.

### Scan UX

- When ≥2 **installable** projects (exclude nested `cli` under a Tauri/`src-tauri` tree when a desktop app is preferred): note + next-step pointing at `[[targets]]` / `signet scan --apply`.
- One-liner: `[platforms]` = shipping intent; suggested flags / host OS = what this machine can sign today.
- `root` display: `.` when scan root equals cwd.

### Identity

- `signet identity status` → same as `identity show` (active fingerprint).

## Acceptance criteria

- [ ] AC1 — verify warns when sums list files but none exist; message mentions rebuild/prune
- [ ] AC2 — verify warns when sums filenames carry a version ≠ project version
- [ ] AC3 — `--fail-stale` exits non-zero for AC1/AC2 cases
- [ ] AC4 — build prints `wrote …SHA256SUMS` after post-sign rewrite
- [ ] AC5 — multi-installable scan emits `[[targets]]` guidance in next steps/notes
- [ ] AC6 — platforms note distinguishes intent vs host
- [ ] AC7 — `identity status` works
- [ ] AC8 — unit tests for freshness heuristics

## Proof plan

| Criterion | Layer | Command |
|-----------|-------|---------|
| AC1–AC3, AC8 | L1 | `cargo test -p signet checksum::` / freshness tests |
| AC4–AC7 | L1 | `cargo test -p signet` + targeted scan/identity if present |
| Clippy | L1 | `cargo clippy -p signet -- -D warnings` |

## Risks

- Semver extraction may false-positive on dates / tool versions in paths — limit to basename only; require `\d+\.\d+\.\d+`.
- Nested rust_cli under Tauri must not alone trigger multi-target spam.
