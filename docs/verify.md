# Verify

`signet verify` checks integrity for a project tree or explicit artifacts: TRUST.md fingerprint (informational / CLI cross-check), `SHA256SUMS` hashes, and community signatures (`SHA256SUMS.minisig` / `.asc`).

It does **not** evaluate SmartScreen, Gatekeeper, or store reputation. It never recommends installing certificates into Trusted Root.

## Usage

```bash
signet verify
signet verify --json
signet verify --sums ./SHA256SUMS --trust ./TRUST.md
signet verify --artifact path/to/setup.exe
signet verify --fingerprint AA:BB:...   # override / cross-check TRUST.md
signet verify --require-sig             # exit 3 if minisig/asc missing or invalid
signet verify --minisign-pub ./minisign.pub
```

Public key resolution for minisign (first match): `--minisign-pub`, then TRUST.md minisign section, then local `.signet/sums/minisign.pub`.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Checks passed |
| 1 | Checksum, fingerprint, or signature mismatch |
| 2 | Missing inputs / nothing to verify |
| 3 | `--require-sig` unmet (signature missing or invalid) |

## Related

- [trust-model.md](trust-model.md)
- [signing.md](signing.md)
- [secrets-layout.md](secrets-layout.md) — `signet sums-key`
- Spec: [`specs/backend/trust-tiers-and-verify-design.md`](../specs/backend/trust-tiers-and-verify-design.md)
- Spec: [`specs/backend/checksum-signing-design.md`](../specs/backend/checksum-signing-design.md)
