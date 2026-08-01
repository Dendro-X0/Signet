# Secrets layout

Private key material for **Signet** **must not** be committed. Non-secret project settings live in `signet.toml` (safe to commit). `TRUST.md` is also safe to commit (fingerprint only).

## On disk (per app project)

```text
your-app/
  signet.toml                # committed
  TRUST.md                   # committed — from `signet trust`
  .gitignore                 # must include .signet/
  .signet/                   # NOT committed
    README.md
    identity/
      active                 # TOML: name = "default"
      default/
        meta.toml            # name, CN, fingerprint, validity (no private key)
        cert.pem             # public certificate
        key.pem              # PRIVATE KEY — never commit, never put in TRUST.md
    sums/
      minisign.key           # PRIVATE — SHA256SUMS signing key
      minisign.pub           # public (also quoted in TRUST.md)
      meta.toml
    android/
      release.jks            # PRIVATE — Android release/upload keystore
      meta.toml              # alias + cert SHA-256 (no passwords)
```

Legacy `.selfsign/` layouts are still detected for status/scan.

## Config pointer

`signet.toml` includes:

```toml
secrets_dir = ".signet"

# Optional Phase 8 defaults (minisign on by default once configured):
# [trust.checksum_signing]
# minisign = true
# gpg = false
# gpg_key_id = ""
```

Android passwords: `SIGNET_ANDROID_STORE_PASS` / optional `SIGNET_ANDROID_KEY_PASS` — never in config.
## Rules

1. Never write private keys into `signet.toml`, `TRUST.md`, or release notes.
2. Public fingerprints, minisign public keys, and cert subjects may appear in trust docs and commits.
3. CI should inject secrets via the runner’s secret store / env, not the git tree.
4. `signet doctor` may report a missing identity or sums key; it must not print key contents.
5. Optional: `SIGNET_MINISIGN_PASSWORD` / `SIGNET_GPG_PASSPHRASE` for encrypted keys — never in config files.

## Commands

```bash
signet identity create [--name default] [--cn "..."] [--org "..."] [--days 825]
signet identity import --cert cert.pem --key key.pem [--name imported]
signet identity list
signet identity show
signet identity use <name>
signet sums-key create [--force]
signet sums-key show
signet android keystore create [--force]
signet android keystore show
signet android sign --apk path.apk
signet trust [--out TRUST.md]
```
