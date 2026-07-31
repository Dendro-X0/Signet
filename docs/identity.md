# Identity + trust

Signet stores self-signed **code-signing** X.509 certificates (ECDSA P-256 by default) under the gitignored `.signet/identity/` tree.

| Action | Command |
|--------|---------|
| Create | `signet identity create` |
| Import PEM | `signet identity import --cert … --key …` |
| List | `signet identity list` (`*` = active) |
| Show | `signet identity show` |
| Switch active | `signet identity use <name>` |

## TRUST.md

`signet trust` writes `TRUST.md` (default: project root) with:

- Certificate subject / validity (public)
- Fingerprint for user verification
- Honest install notes (SmartScreen / Gatekeeper)

Never include private keys.

Identity material feeds `signet build` signing — see [`signing.md`](signing.md).
