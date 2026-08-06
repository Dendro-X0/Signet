# Ship CI secrets (band S)

## Plan alignment

- **Parent:** CI secrets roadmap (post-0.5.16) — Clavis Android failure at keystore restore
- **Band:** 0.5.17
- **Slices:** S1 `ship secrets`, S2 doctor/plan CI readiness, S3 `ship --ci` preflight + restore
- **PAUSED/CANCELLED:** none
- **Out of scope:** Play/App Store upload; Apple team automation; OIDC provider setup (docs only)

## Secret names (generic `SIGNET_*`)

| Secret | When required | Source |
|--------|---------------|--------|
| `SIGNET_ANDROID_KEYSTORE_BASE64` | android commitment | base64 of `.signet/android/release.jks` |
| `SIGNET_ANDROID_META_BASE64` | android commitment | base64 of `.signet/android/meta.toml` |
| `SIGNET_ANDROID_STORE_PASS` | android commitment | env `SIGNET_ANDROID_STORE_PASS` (never toml) |
| `SIGNET_IDENTITY_BUNDLE_BASE64` | any desktop host-sign in CI | base64 tar of `.signet/identity/` |
| `SIGNET_SUMS_KEY_BASE64` | minisign checksum signing in CI | base64 of `.signet/sums/minisign.key` (optional if sums unsigned) |

iOS: no secret push in S1 — report `gap.ios.codesign` honesty only.

## S1 — `signet ship secrets`

```bash
signet ship secrets                 # assess + print matrix
signet ship secrets --push          # dry-run: print gh secret set recipe
signet ship secrets --push --apply  # gh secret set via stdin (requires gh auth)
```

- Detect needs from `[platforms]` + mobile targets (same as coverage).
- Local checks: identity active, android keystore+meta, sums key when minisign enabled.
- Do not create keystores/identities automatically in non-TTY; note `signet android keystore create` / `identity create`.
- `--apply` without `--push` → error.
- Never print store pass or key material; only names + presence.

## S2 — CI readiness

`assess_ci_readiness` → gaps with stable IDs:

- `gap.android.ci_secrets`
- `gap.android.keystore_local`
- `gap.desktop.identity_local`
- `gap.desktop.ci_identity`
- `gap.ios.codesign` (informational)
- `gap.github.auth` (cannot list/set secrets)

Doctor + `ship --plan` print section; next command `signet ship secrets --push`.

Remote presence via `gh secret list` when auth ready; else local-only.

## S3 — Workflow template

- Job `ship-preflight` (ubuntu-latest): checkout + check required secrets env/presence with `::error::` + hint to `signet ship secrets --push --apply`; fail &lt;30s before NDK.
- Android job: Signet-owned restore step writing keystore from secrets; `::error::` if missing.
- Desktop matrix: optional restore identity from `SIGNET_IDENTITY_BUNDLE_BASE64`.
- Comments: soft-fail / `--allow-partial` on release.

## Acceptance

- [x] AC1 — dry-run lists required secrets without applying
- [x] AC2 — `--apply` requires `--push` + gh ready
- [x] AC3 — doctor/plan show CI readiness + gap IDs
- [x] AC4 — CI YAML contains `ship-preflight` and restore `::error::`
- [x] AC5 — unit tests for required names + readiness gaps

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet` |
| L1 | `cargo clippy -p signet -- -D warnings` |
