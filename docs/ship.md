# Ship (multi-platform)

`[platforms]` is a **commitment** (desktop + optional mobile). Local `signet build` only covers this host; `signet ship` orchestrates the rest.

`[ship] path` chooses the Sign backend on the **same** plan: `self` (default) or `graduate` (OV / Azure / notarize).

## Commands

```bash
signet ship --plan                 # coverage + Sign path + CI secrets readiness
signet ship secrets                # assess required GitHub Actions secrets (local)
signet ship secrets --push         # print gh secret set recipe (dry-run)
signet ship secrets --push --apply # push via `gh secret set` (stdin; needs gh auth)
signet ship --ci                   # write .github/workflows/signet-ship.yml (+ preflight)
signet ship --ci --force           # overwrite workflow
signet ship --collect ./artifacts  # merge CI downloads → dist/signet-ship/ + SHA256SUMS
signet release --tag vX.Y.Z        # fails if coverage gap (unless --allow-partial)
signet graduate apply              # when path=graduate: discover host installers + official Sign
```

### CI secrets

Push local `.signet/` material before expecting green Actions (never commit secrets):

| Secret | Material |
|--------|----------|
| `SIGNET_IDENTITY_BUNDLE_BASE64` | zip of `.signet/identity/` |
| `SIGNET_SUMS_KEY_BASE64` | `.signet/sums/minisign.key` (if minisign on) |
| `SIGNET_ANDROID_KEYSTORE_BASE64` | `.signet/android/release.jks` |
| `SIGNET_ANDROID_META_BASE64` | `.signet/android/meta.toml` |
| `SIGNET_ANDROID_STORE_PASS` | from env (never `signet.toml`) |

Workflow preflight fails fast with `::error::… signet ship secrets --push --apply` when secrets are missing.

## Flow

1. Declare `windows` / `macos` / `linux` and optionally `android` / `ios` in `signet.toml` (mobile frameworks / `[[targets]]` also imply android/ios).
2. Optional: `[ship] path = "graduate"` + `[graduation]` / env credentials.
3. `signet ship --ci` → commit workflow (desktop matrix + mobile jobs + `graduate apply` when configured) → push tag / `workflow_dispatch`.
4. Download job artifacts → `signet ship --collect DIR`.
5. `signet ship --plan` should show no gap → `signet release`.

Restore `.signet/identity` in CI via secrets for the self path. For graduate, restore Azure/OV/Apple secrets (`SIGNET_AZURE_*`, `SIGNET_OV_*`, `SIGNET_NOTARY_PROFILE`) — never commit them.

**Honesty:** Android local keystore ≠ Play App Signing; iOS free provisioning / IPA packaging ≠ App Store — see `docs/android.md` / `docs/ios.md`. Graduate helpers do not buy certs or silence SmartScreen/Gatekeeper by magic — see `docs/graduation.md`.

## Specs

- [`specs/backend/multi-platform-ship-design.md`](../specs/backend/multi-platform-ship-design.md)
- [`specs/backend/ship-ci-collect-design.md`](../specs/backend/ship-ci-collect-design.md)
- [`specs/backend/mobile-ship-loop-design.md`](../specs/backend/mobile-ship-loop-design.md)
- [`specs/backend/ship-graduate-profile-design.md`](../specs/backend/ship-graduate-profile-design.md)
