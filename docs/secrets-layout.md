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
```

Legacy `.selfsign/` layouts are still detected for status/scan.

## Config pointer

`signet.toml` includes:

```toml
secrets_dir = ".signet"
```

## Rules

1. Never write private keys into `signet.toml`, `TRUST.md`, or release notes.
2. Public fingerprints and cert subjects may appear in trust docs and commits.
3. CI should inject secrets via the runner’s secret store / env, not the git tree.
4. `signet doctor` may report a missing identity; it must not print key contents.

## Commands

```bash
signet identity create [--name default] [--cn "..."] [--org "..."] [--days 825]
signet identity import --cert cert.pem --key key.pem [--name imported]
signet identity list
signet identity show
signet identity use <name>
signet trust [--out TRUST.md]
```
