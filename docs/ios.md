# iOS packaging with Signet

Signet helps turn an existing **`.app`** into an **`.ipa`** (zip with `Payload/`) and documents Apple’s trust model honestly.

It does **not** grant App Store, TestFlight, or notarization trust.

## Channels

| Channel | What Signet does | What Signet does **not** do |
|---------|------------------|------------------------------|
| Free Apple ID / development | Document **~7-day** provisioning; package IPA from an already-built `.app` | Extend free provisioning; install on arbitrary devices forever |
| Ad Hoc (paid Developer Program) | Document only | Manage provisioning profiles for you |
| App Store / TestFlight | Point you at Apple’s programs | Upload, notarize, or claim store trust |

## Free provisioning (7 days)

With a free Apple ID, development builds typically stop launching after about **seven days** until you re-sign/re-provision in Xcode. That is Apple’s policy — Signet cannot change it. Plan demos and CI accordingly.

## Package an IPA

```bash
# After Xcode / `tauri ios build` (or similar) produced Demo.app:
signet ios package --app path/to/Demo.app
# → writes Demo.ipa beside the .app (or --out path.ipa)

signet ios notes   # short honesty summary
```

An IPA is only a zip:

```text
Payload/
  Demo.app/
    …
```

## `framework = "ios"`

```toml
[project]
framework = "ios"
tauri_root = "."
# Required for `signet build` without --skip-build:
# build_command = "xcodebuild -scheme MyApp -configuration Release …"
```

- **Discover** looks under `build/`, `dist/`, `release/`, Tauri `gen/apple` / `gen/ios`, etc.
- **Build** does **not** guess schemes — set `build_command` or use `--skip-build`.
- **Sign** is not performed by Signet for iOS — use Xcode / Apple tooling, then package.

## Doctor

On macOS, `signet doctor` reports `codesign` / `xcodebuild` when useful. Missing tools are optional unless you set `framework = "ios"`.

## Related

- [trust-model.md](trust-model.md)
- [android.md](android.md) (Play honesty counterpart)
- Spec: [`specs/backend/ios-signing-design.md`](../specs/backend/ios-signing-design.md)
