# Verify

`signet verify` checks integrity for a project tree or explicit artifacts: TRUST.md fingerprint (informational / CLI cross-check) and `SHA256SUMS` hashes.

It does **not** evaluate SmartScreen, Gatekeeper, or store reputation. It never recommends installing certificates into Trusted Root.

## Usage

```bash
signet verify
signet verify --json
signet verify --sums ./SHA256SUMS --trust ./TRUST.md
signet verify --artifact path/to/setup.exe
signet verify --fingerprint AA:BB:...   # override / cross-check TRUST.md
signet verify --require-sig             # soft-warn until Phase 8 checksum signing
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Checks passed |
| 1 | Checksum or fingerprint mismatch |
| 2 | Missing inputs / nothing to verify |
| 3 | Reserved for `--require-sig` policy after Phase 8 |

## Related

- [trust-model.md](trust-model.md)
- [signing.md](signing.md)
- Spec: [`specs/backend/trust-tiers-and-verify-design.md`](../specs/backend/trust-tiers-and-verify-design.md)
