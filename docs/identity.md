# Identity & trust (Phase 2)

## Identity

`selfsign` stores self-signed **code-signing** X.509 certificates (ECDSA P-256 by default) under the gitignored `.selfsign/identity/` tree.

| Action | Command |
|--------|---------|
| Create | `selfsign identity create` |
| Import PEM | `selfsign identity import --cert … --key …` |
| List | `selfsign identity list` (`*` = active) |
| Show | `selfsign identity show` |
| Switch active | `selfsign identity use <name>` |

Fingerprint = SHA-256 of the certificate DER, colon-separated uppercase hex.

## Trust kit

`selfsign trust` writes `TRUST.md` (default: project root) with:

- Publisher table (CN, org, fingerprint, validity)
- Verify steps
- Honest Windows / macOS / Linux warning notes (SmartScreen, Gatekeeper, checksums)

It refuses to embed private key material.

## Platform note

Phase 2 produces PEM identity material. Phase 3 (`selfsign build`) signs host artifacts with that identity — see [`signing.md`](signing.md).
