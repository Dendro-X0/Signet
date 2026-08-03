# Graduate profile on ship plan (slice F)

## Plan alignment

- **Handoff:** Fix order **#5** — Graduate on same ship plan (shortcoming item 3)
- **Parent:** [`multi-platform-ship-design.md`](multi-platform-ship-design.md) slice F
- **Band:** 0.5.14
- **PAUSED/CANCELLED:** none
- **In scope:** `[ship].path` (`self` \| `graduate`); plan shows per-OS Sign backend; `graduate apply` discovers host installers + runs configured helper; CI template wires apply when path=graduate
- **Out of scope:** Buying certs; Play/App Store; changing self-sign identity; Linux CA packaging

## Contracts

### Config

```toml
[ship]
path = "self"       # default — Signet identity / host codesign
# path = "graduate" # OV / Azure / Apple notarize via [graduation] + env
```

`[graduation]` stays secret-free public ids (existing). Env overrides unchanged (`SIGNET_AZURE_*`, `SIGNET_OV_*`, `SIGNET_NOTARY_PROFILE`).

### Path resolution (when `path = "graduate"`)

| Platform | Backend chosen | Ready when |
|----------|----------------|------------|
| Windows | Prefer **azure** if dlib+metadata (config or env); else **ov** if thumbprint/PFX | else `missing` |
| macOS | **notarize** (+ staple) | keychain profile config or env |
| Linux | integrity-first (sums / self) | always — note only |
| Android/iOS | unchanged honesty | Play/App Store external |

When `path = "self"`: every desktop platform reports `self`.

### Commands

- `signet ship --plan` — print coverage **and** Sign path / per-OS graduate readiness
- `signet graduate apply` — discover host-signable release installers (exe/msi or dmg/app) and run the resolved graduate backend; no-op with clear message on Linux / when path=self
- `signet ship --ci` — if path=graduate, after build add `signet graduate apply` (Windows + macOS runners); comment that Azure/OV/Apple secrets must be restored via Actions secrets

### Honesty

- Graduate never falls back to Signet self-signed identity
- Plan/CI notes: SmartScreen/Gatekeeper still not guaranteed by helpers alone

## Acceptance

- [x] AC1 — `[ship].path` round-trips; default `self`
- [x] AC2 — plan shows azure/ov/notarize vs missing when path=graduate
- [x] AC3 — CI YAML includes `graduate apply` iff path=graduate
- [x] AC4 — `graduate apply` refuses empty credential (unit/argv level) and discovers files in test fixture
- [x] AC5 — docs/ship.md + graduation.md + CHANGELOG

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet` |
| L1 | `cargo clippy -p signet -- -D warnings` |
