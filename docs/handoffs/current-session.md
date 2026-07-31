# Current session handoff

**Updated:** 2026-07-31  
**Band:** Phases 6–7 implemented; Phase 8 next

## Next atomic step

Implement **Phase 8 — Checksum signing** from [`specs/backend/checksum-signing-design.md`](../../specs/backend/checksum-signing-design.md):

1. `signet sums-key create/show` + minisign under `.signet/sums/`
2. Sign `SHA256SUMS` → `SHA256SUMS.minisig` from build/release
3. Wire `signet verify` hard `--require-sig` (exit 3)
4. Do **not** start Electron/Android unless asked

**PAUSED / CANCELLED:** none  
**Blocked for coding:** Phases 9–12 stubs

## Canonical owners

| Work | Owner |
|------|--------|
| TRUST / tiers | `trust_kit.rs`, `trust_tier.rs` |
| Verify | `commands/verify.rs`, `sign/checksum.rs` |
| Checksum signing | Phase 8 — `sums_sig` (new) |

## Phase 7 proof (done)

- L1: checksum + fingerprint parse unit tests
- L2: `cargo test -p signet` (23 ok)
- L3: fixture verify exit 0 / tamper exit 1 / empty exit 2

## Recently completed

- Phase 6 trust tiers
- Phase 7 `signet verify`
