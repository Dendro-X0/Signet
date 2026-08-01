# Design: checksum signing (minisign + optional GPG)

**Phase:** 8  
**Status:** implemented  
**Depends on:** Phase 6–7 designs (verify hooks for `sums_signature`)  
**Owners:** `sign/sums_sig.rs`, `sign/checksum.rs`, `commands/sums_key.rs`, `commands/build.rs`, `commands/release.rs`, `commands/doctor.rs`, `trust_kit.rs`  
**Plan alignment:** community verify layer; does **not** replace Authenticode / codesign.

## Problem

OSS users often verify releases with **signed checksum files** (`SHA256SUMS` + `.minisig` / `.asc`), especially on Linux and for Windows builds that never buy a CA cert. Signet already writes `SHA256SUMS` but does not attest them.

## Goals

1. Sign `SHA256SUMS` by default with **minisign** (Signet-managed key under `.signet/`).
2. Optionally detach-sign with **GPG** when configured / `gpg` available.
3. Publish public key material via `TRUST.md` and release assets.
4. Extend `signet verify` to validate signatures (complete Phase 7 `--require-sig`).

## Non-goals

- Replacing host PE/Mach-O signing.
- Embedding AppImage GPG signatures inside the AppImage (future optional).
- Sigstore/cosign as default (may be a later optional scheme).

---

## Default scheme: minisign

### Why minisign first

- Small keys, simple UX, common in Rust/OSS tooling.
- Implementable in-process (Rust crate) without requiring `gpg` on Windows CI.
- Clear file naming: `SHA256SUMS.minisig`.

### Key layout (gitignored)

```text
.signet/
  sums/
    minisign.key          # private — never commit
    minisign.pub          # public — also embedded/copied for TRUST
    meta.toml             # created_at, algorithm note
```

Commands:

```text
signet identity sums-key create     # or: signet sums-key create
signet identity sums-key show       # print public key / path
```

**Decision (frozen):** use dedicated subcommand group under identity-adjacent surface:

```text
signet sums-key create [--force]
signet sums-key show
```

Private key only under `.signet/sums/`. Public key written to `.signet/sums/minisign.pub` and quoted in `TRUST.md`.

### Artifact layout

| File | Role | Commit? | Release attach? |
|------|------|---------|-----------------|
| `SHA256SUMS` | GNU checksums | optional | yes |
| `SHA256SUMS.minisig` | minisign over `SHA256SUMS` bytes | optional | yes |
| `SHA256SUMS.asc` | optional GPG armor detach | optional | yes if produced |
| public key in `TRUST.md` | human verify | yes | via TRUST |

Signature covers the **checksum file contents**, not each binary individually (standard community pattern).

---

## Optional scheme: GPG

```toml
[trust.checksum_signing]
minisign = true          # default true once Phase 8 ships
gpg = false              # opt-in
gpg_key_id = ""          # optional; else gpg default key
```

When `gpg = true`:

1. Require `gpg` on PATH (`doctor` reports).
2. Run equivalent of `gpg --detach-sign --armor -o SHA256SUMS.asc SHA256SUMS`.
3. Passphrase only via agent / env (`SIGNET_GPG_PASSPHRASE` or gpg-agent) — never config file.

If GPG fails and minisign succeeded, build/release **warns** but does not fail unless `--require-gpg`.

---

## Wiring

### `signet build`

After writing `SHA256SUMS`:

1. If minisign enabled and key exists → write `SHA256SUMS.minisig`.
2. If minisign enabled and key missing → warn: run `signet sums-key create` (do not fail build by default).
3. If `gpg` enabled → attempt `.asc`.

Flags:

```text
--no-sums-sign          # skip checksum signing
--require-sums-sign     # fail if minisign (default scheme) cannot sign
```

### `signet release`

Attach `SHA256SUMS`, `SHA256SUMS.minisig`, and `SHA256SUMS.asc` when present (same as TRUST attach policy).

### `signet trust`

Include:

- Minisign public key (raw or path instruction).
- Verify example: `signet verify` and/or `minisign -Vm SHA256SUMS -p …`.

### `signet verify` (Phase 8 completion)

- Detect `.minisig` beside sums; verify with embedded/TRUST public key or `--minisign-pub PATH`.
- Detect `.asc`; verify with `gpg --verify` when available.
- `--require-sig`: exit 3 if no valid community signature.

### Doctor

| Check | Severity |
|-------|----------|
| `sums-minisign-key` | optional — missing key |
| `gpg` | optional — only if `gpg = true` in config |

---

## Errors and invariants

1. Never write private minisign/GPG material into `TRUST.md` or release notes.
2. Signing checksums does not upgrade trust tier to `ca_authenticode` / `apple_notarized`.
3. Tier becomes `community_signed_sums` when a valid sums signature is present (in addition to host tier notes).
4. Re-signing: if `SHA256SUMS` changes, signatures must be regenerated in the same command.

## Acceptance

- [x] `sums-key create` produces pub/priv under `.signet/sums/`; priv gitignored.
- [x] Build with key present emits `SHA256SUMS.minisig`.
- [x] `signet verify` validates matching sig; fails on tampered `SHA256SUMS`.
- [x] `--require-sums-sign` fails build when key missing.
- [x] GPG path documented + tested only when `gpg` available (ignore or skip on hosts without it).
- [x] Unit tests for sign/verify round-trip with temp keys.

**Status:** implemented (2026-07-31)

## Proof plan

| Layer | Evidence |
|-------|----------|
| L1 | Unit tests: sign sums → verify; tamper → fail (`sign/sums_sig.rs`) |
| L2 | `cargo test -p signet` |
| L3 | End-to-end: identity + sums-key + write fixture sums + verify |

## Subtraction

- Do not invent a custom signature format; minisign and OpenPGP armor only.
- Do not store passphrase in `signet.toml`.

## Do not implement until

~~Phases 6–7 landed enough that `verify` exists~~ — done.
