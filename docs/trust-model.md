# Trust model

Signet separates **integrity** (what we can prove about bits and publisher keys) from **reputation** (what operating systems and stores decide to trust).

```text
Integrity (Signet)     →  does not imply  →  Reputation (OS / store)
fingerprint, sums,                        SmartScreen, Gatekeeper,
host sign, signed sums                    Play App Signing, Apple programs
```

## Trust tiers

| Tier id | Integrity meaning | Does **not** mean |
|---------|-------------------|-------------------|
| `checksum_only` | SHA256SUMS present | Host crypto signature |
| `self_signed_host` | Signed with Signet/imported self-issued identity | CA / notarized / store trust |
| `community_signed_sums` | `SHA256SUMS` attested (minisign/GPG) | Authenticode or Gatekeeper pass |
| `ca_authenticode` | Windows signature chains to public CA | Instant SmartScreen silence |
| `apple_notarized` | Developer ID + Apple notarization | Available without Apple program |
| `play_managed` | Play App Signing for distribution | Local keystore is the Play signing key |
| `unknown` | Not enough evidence | Safe to install |

Primary tier is reported in `TRUST.md`, doctor, and (planned) `signet verify`. See [`specs/backend/trust-tiers-and-verify-design.md`](../specs/backend/trust-tiers-and-verify-design.md).

## Platform honesty

| Platform | Self-sign reality | Public / store path |
|----------|-------------------|---------------------|
| Windows | SmartScreen often blocks or warns | OV / Azure Artifact Signing / Store |
| macOS | Ad-hoc/self-issued ≠ Gatekeeper for quarantined apps | Developer ID + notarization |
| Linux | Checksums + GPG/minisign are the community default | Distro repos / Flatpak remotes |
| Android | Local keystore required for sideload | Play App Signing for new Play apps |
| iOS | Free provisioning is short-lived | Paid Apple program / store / regional rules |

## Anti-patterns (forbidden in Signet UX)

1. **Never** tell end users to install your certificate into **Trusted Root** / Root store.
2. **Never** claim self-signing removes SmartScreen or Gatekeeper warnings.
3. **Never** put private keys, PFX passwords, or keystore passwords in `TRUST.md` or git.
4. **Never** mark a framework “supported” in README until build/sign for that path exists.

## What users should do

1. Compare the **SHA-256 fingerprint** in `TRUST.md` with `signet identity show`.
2. Verify downloads with `SHA256SUMS` via `signet verify` (and signed sums when Phase 8 ships).
3. Treat OS warnings as expected for self-signed desktop apps unless you graduate to CA/notarization (`signet graduate` — [graduation.md](graduation.md)).

## Related

- [`product.md`](product.md) — thesis and surfaces
- [`identity.md`](identity.md) — identity commands
- [`signing.md`](signing.md) — host signing
- [`graduation.md`](graduation.md) — OV / Azure / notarization helpers
- [`roadmap.md`](roadmap.md) — phased delivery
