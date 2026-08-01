# Graduation ladder (OV / Azure / notarization)

Part of Signet’s **official / paid Sign path** (see [product.md](product.md) dual path).  
Self-signed Signet builds prove **integrity**. OS reputation (SmartScreen, Gatekeeper) usually requires a **paid or program-gated** path. Signet’s `graduate` helpers wrap those tools honestly — they do **not** buy certificates, silence warnings by magic, or tell end users to install publisher certs into Trusted Root.

## Ladder

| Step | Platform | What you need | Signet helper |
|------|----------|---------------|---------------|
| Self-signed | All | `.signet/identity` | `signet build` (default) |
| OV / EV Authenticode | Windows | CA-issued code-signing cert in store or PFX | `signet graduate ov-sign` |
| Azure Trusted Signing | Windows | Azure Trusted Signing account + dlib + metadata | `signet graduate azure-sign` |
| Developer ID + notarize | macOS | Apple Developer Program + notary credentials | `signet graduate notarize` / `staple` |

Declare reputation in config only when it is true:

```toml
[trust]
declared_tier = "ca_authenticode"   # or "apple_notarized"
```

## Commands

```bash
signet graduate notes
signet graduate ov-sign --file app.exe --thumbprint ABCDEF…
# or: --pfx path.pfx   + env SIGNET_OV_PFX_PASS
signet graduate azure-sign --file app.exe
signet graduate notarize --path App.app --profile MyNotaryProfile
signet graduate staple --path App.app
```

### Windows OV

Uses `signtool` with `/sha1` (thumbprint) or `/f` (PFX). **Never** falls back to Signet’s self-signed identity.

| Source | Thumbprint | PFX |
|--------|------------|-----|
| CLI | `--thumbprint` | `--pfx` / `--pfx-pass` |
| Env | `SIGNET_OV_THUMBPRINT` | `SIGNET_OV_PFX`, `SIGNET_OV_PFX_PASS` |
| Config | `[graduation].ov_thumbprint` | — |

### Azure Trusted Signing

Wraps:

```text
signtool sign /fd SHA256 /td SHA256 /tr http://timestamp.acs.microsoft.com \
  /dlib <Azure.CodeSigning.Dlib.dll> /dmdf <metadata.json> <file>
```

Set `[graduation.azure] dlib` + `metadata`, or `SIGNET_AZURE_DLIB` / `SIGNET_AZURE_METADATA`. Azure authentication stays with Microsoft’s dlib / Azure identity — Signet does not store those secrets.

### Apple notarization

1. Sign with a **Developer ID** certificate (Xcode / `codesign`) — not Signet’s ad-hoc/self-issued identity.
2. Store credentials: `xcrun notarytool store-credentials`.
3. `signet graduate notarize --path … --profile …` (runs `notarytool submit --wait`, then `stapler` unless `--no-staple`).

## Config sketch

```toml
[graduation]
ov_thumbprint = ""
timestamp_url = "http://timestamp.digicert.com"

[graduation.azure]
dlib = ""
metadata = ""
timestamp_url = "http://timestamp.acs.microsoft.com"

[graduation.apple]
keychain_profile = ""
```

## Related

- [trust-model.md](trust-model.md) — integrity vs reputation
- [signing.md](signing.md) — default self-sign build path
- [specs/backend/graduation-helpers-design.md](../specs/backend/graduation-helpers-design.md)
