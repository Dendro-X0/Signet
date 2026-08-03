# Design: Platform coverage report (ship slice A)

## Plan alignment

- **Handoff:** `docs/dogfood/signet-shortcomings.md` fix order #1 (items 1–2, 7, 11–12)
- **Parent:** `specs/backend/multi-platform-ship-design.md` **Slice A**
- **Band:** 0.5.9 (honesty before CI/collect)
- **PAUSED/CANCELLED:** none
- **In scope:** declared-vs-present coverage report; print on doctor / build start / guided; `signet ship --plan`
- **Out of scope:** fail-closed release gate (slice E); soft-fail targets (slice B); dry-run read-only (slice C); CI template (slice D)

## Contracts

### Coverage model

```text
declared_desktop = platforms.windows|macos|linux (true flags)
host_can_sign    = current OS desktop platform only (this slice)
present_desktop  = artifact kinds found under project (discover + common paths)
gap              = declared − present
```

| Platform | Present when any of |
|----------|---------------------|
| windows | `.exe` installer / `.msi` under discover or walk |
| macos | `.dmg` / `.app` |
| linux | `.AppImage` / `.deb` / `.rpm` |

Mobile (android/ios) noted as **debt** when installable mobile targets exist or scan would detect — not part of `[platforms]` today; list under “targets / mobile” notes only.

### CLI

```bash
signet ship --plan          # print coverage + host slice + next steps
signet doctor               # includes coverage section (warn if gap)
signet build                # prints coverage banner before build
```

Guided Check completion prints the same gap one-liner when declared platforms exceed present.

### Honesty copy

- Never claim other OS will be signed on this host.
- Gap line example: `ship coverage: windows=present macos=MISSING linux=MISSING (host=windows can sign windows only)`
- Doctor: coverage gap is a **warning** check (does not fail doctor exit unless we already fail on other things — keep doctor exit = tooling only; coverage is informational warn).

## Ownership

| Piece | Module |
|-------|--------|
| Assess coverage | `ship/coverage.rs` |
| CLI | `commands/ship.rs` |
| Doctor section | `commands/doctor.rs` |
| Build banner | `commands/build.rs` |
| Guided | `tui/flows.rs` |

## Acceptance

- [ ] AC1 — unit: config windows+macos+linux + only windows artifacts → gap macos, linux
- [ ] AC2 — `ship --plan` prints declared / present / gap / host capability
- [ ] AC3 — `build` prints coverage before framework build
- [ ] AC4 — `doctor` shows coverage warn when gap non-empty
- [ ] AC5 — guided surfaces gap after verify path when gap non-empty

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p signet ship::` |
| L1 | `cargo clippy -p signet -- -D warnings` |
