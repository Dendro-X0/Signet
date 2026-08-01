# Backend design specs

Implementation contracts for Signet. **Public narrative** lives under [`docs/`](../../docs/); **coding contracts** live here.

## Coding gate

1. Read [`docs/roadmap.md`](../../docs/roadmap.md) — confirm the phase is open.
2. Read the design for that phase — status must be `ready` or `approved`.
3. Do **not** implement a later phase before earlier integrity phases (6 → 7 → 8) unless the maintainer explicitly overrides the handoff.
4. After implementation, update this index status and [`docs/handoffs/current-session.md`](../../docs/handoffs/current-session.md).

## Index

| Spec | Phase | Depth | Status |
|------|-------|-------|--------|
| [trust-tiers-and-verify-design.md](trust-tiers-and-verify-design.md) | 6–7 | Deep | implemented |
| [checksum-signing-design.md](checksum-signing-design.md) | 8 | Deep | implemented |
| [artifact-contract-design.md](artifact-contract-design.md) | 9 | Deep | implemented |
| [electron-adapter-design.md](electron-adapter-design.md) | 10 | Deep | implemented |
| [android-signing-design.md](android-signing-design.md) | 11 | Deep | implemented |
| [ios-signing-design.md](ios-signing-design.md) | 12 | Deep | implemented |
| [self-update-design.md](self-update-design.md) | Dist | Deep | implemented |

## Status legend

| Status | Meaning |
|--------|---------|
| `stub` | Sequencing only — **do not implement** |
| `ready` | Contracts frozen enough to implement |
| `approved` | Maintainer signed off (optional tighter gate) |
| `implemented` | Code matches spec; proof recorded in handoff |

## Proof language

Use L1–L3 as named in each design. Do not claim “fixed” or “verified” without running the listed commands.
