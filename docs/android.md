# Android signing with Signet

Signet helps **local / sideload** APK signing with a project keystore under `.signet/android/`.  
It does **not** replace Google Play App Signing.

## Channels

| Channel | What Signet does | What Signet does **not** do |
|---------|------------------|------------------------------|
| Sideload / F-Droid-style | Create/import keystore; sign APKs (`apksigner` / `jarsigner`); publish cert digest in `TRUST.md` | Claim OS/store “trusted publisher” status |
| Google Play | Document upload key vs app signing key; optional local upload-key keystore | Hold Play Console secrets; pretend the local key is the Play **app signing** key |

## Quick start (sideload)

```bash
export SIGNET_ANDROID_STORE_PASS='…'   # never put this in signet.toml
# optional: export SIGNET_ANDROID_KEY_PASS='…'

signet init --name my-app
# in signet.toml: framework = "android"

signet android keystore create
signet android sign --apk path/to/app-release-unsigned.apk
# or: signet build --skip-build   # discovers APKs + signs when framework=android

signet trust   # embeds Android cert digest when keystore meta exists
```

## Play App Signing (honesty)

For new Play apps, Google manages the **app signing key**. You keep an **upload key**:

1. Create or import an upload keystore with Signet (or Android Studio).
2. Register the upload certificate in Play Console.
3. Upload an AAB signed with the **upload** key (Gradle / Play’s documented flow).
4. Play re-signs with the **app signing** key for distribution.

Signet will **skip auto-signing `.aab`** as if it were the Play distribution key. Use Play/Gradle for AAB upload signing, and keep `docs` / `TRUST.md` clear that the digest is the upload (or sideload) cert.

Declare `play_managed` in `[trust].declared_tier` only when you actually ship via Play App Signing.

## Tools

- `keytool` (JDK) — keystore create/import/show  
- `apksigner` (Android SDK build-tools) — preferred APK sign  
- `jarsigner` — fallback if apksigner is missing  
- Env: `ANDROID_HOME` / `ANDROID_SDK_ROOT` help locate build-tools  

`signet doctor` reports these when `framework = "android"`.

## Related

- [secrets-layout.md](secrets-layout.md)  
- [trust-model.md](trust-model.md)  
- Spec: [`specs/backend/android-signing-design.md`](../specs/backend/android-signing-design.md)
