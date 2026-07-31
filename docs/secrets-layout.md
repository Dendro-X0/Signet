# Secrets layout

Private key material for `selfsign` **must not** be committed. Non-secret project settings live in `selfsign.toml` (safe to commit). `TRUST.md` is also safe to commit (fingerprint only).

## On disk (per app project)

```text
your-tauri-app/
  selfsign.toml              # committed
  TRUST.md                   # committed — from `selfsign trust`
  .gitignore                 # must include .selfsign/
  .selfsign/                 # NOT committed
    README.md
    identity/
      active                 # TOML: name = "default"
      default/
        meta.toml            # name, CN, fingerprint, validity (no private key)
        cert.pem             # public certificate
        key.pem              # PRIVATE KEY — never commit, never put in TRUST.md
```

## Config pointer

`selfsign.toml` includes:

```toml
secrets_dir = ".selfsign"
```

## Rules

1. Never write private keys into `selfsign.toml`, `TRUST.md`, or release notes.
2. Public fingerprints and cert subjects may appear in trust docs and commits.
3. CI should inject secrets via the runner’s secret store / env, not the git tree.
4. `selfsign doctor` may report a missing identity; it must not print key contents.

## Commands

```bash
selfsign identity create [--name default] [--cn "..."] [--org "..."] [--days 825]
selfsign identity import --cert cert.pem --key key.pem [--name imported]
selfsign identity list
selfsign identity show
selfsign identity use <name>
selfsign trust [--out TRUST.md]
```
