# Current session handoff

**Updated:** 2026-07-31  
**Band:** Phase 6 implemented; Phase 7 next

## Next atomic step

Implement **Phase 7 — `signet verify`** from [`specs/backend/trust-tiers-and-verify-design.md`](../../specs/backend/trust-tiers-and-verify-design.md):

1. Add `commands/verify.rs` + clap wiring.
2. Parse fingerprint from TRUST.md; verify SHA256SUMS via `sign/checksum`.
3. Exit codes 0/1/2; soft-warn on `--require-sig` until Phase 8.
4. Do **not** start Electron/Android adapters or Phase 8 unless asked.

**PAUSED / CANCELLED:** none  
**Blocked for coding:** Phases 9–12 stubs; Phase 10–11 blocked on earlier phases.

## Canonical owners

| Work | Owner |
|------|--------|
| TRUST template / tiers | `crates/signet/src/trust_kit.rs`, `trust_tier.rs` |
| Verify CLI | `crates/signet/src/commands/verify.rs` (Phase 7) |
| Checksums | `crates/signet/src/sign/checksum.rs` |

## Phase 6 proof (done)

- L1: `trust_kit` / `trust_tier` unit tests (tier + Root anti-pattern)
- L2: `cargo test -p signet`
- L3: `cargo run -p signet -- trust` / `doctor` (trust-tier check)

## Recently completed

- Phase 6: trust tiers in TRUST.md, `[trust]` config, doctor `trust-tier`
- Specs + roadmap for Phases 6–12
