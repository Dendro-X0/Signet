# Design: trust tiers and `signet verify`

**Phase:** 6 (trust clarity) + 7 (verify)  
**Status:** implemented (Phases 6–7; Phase 8 `--require-sig` wired)  
**Owners:** `crates/signet/src/trust_kit.rs`, new `commands/verify.rs`, `sign/checksum.rs` (read/verify), `commands/doctor.rs` (labels)  
**Plan alignment:** integrity-first roadmap; no Electron/Android work in this band.

## Problem

Users and agents cannot distinguish **integrity** (fingerprint, checksums, host signatures) from **reputation** (SmartScreen, Gatekeeper, Play, Apple). `TRUST.md` already warns, but lacks typed tiers and a single verify entry point.

## Goals

1. Name trust tiers in docs, `TRUST.md`, and doctor output.
2. Add `signet verify` that checks fingerprints + checksums (+ optional community sig once Phase 8 lands).
3. Never instruct end users to install a self-signed certificate into Root / Trusted Root.

## Non-goals

- Silencing SmartScreen / Gatekeeper.
- Detecting CA Authenticode or Apple notarization with perfect accuracy on every OS (best-effort detection only).
- Implementing minisign/GPG (Phase 8); verify must accept hooks for them.

---

## Trust tiers

Declared or detected labels. Multiple may apply; Signet reports the **primary integrity tier** plus optional reputation notes.

| Tier id | Meaning | Typical Signet path |
|---------|---------|---------------------|
| `checksum_only` | Artifacts have SHA256SUMS; no host crypto sign | `--no-sign` build |
| `self_signed_host` | Host-signed with Signet (or imported) self-issued identity | default `signet build` |
| `community_signed_sums` | `SHA256SUMS` has minisign/GPG signature | Phase 8 |
| `ca_authenticode` | Windows signature chains to public CA (declared or detected) | graduation helper (later) |
| `apple_notarized` | Developer ID + notarization (declared) | graduation helper (later) |
| `play_managed` | Play App Signing for Android distribution | external; document only |
| `unknown` | Insufficient evidence | missing TRUST / sums |

### Invariants

1. `self_signed_host` **does not imply** `ca_authenticode` or `apple_notarized`.
2. Tier strings in `TRUST.md` and CLI JSON must match the table above (snake_case).
3. Anti-pattern text is mandatory in generated TRUST: never install publisher cert into OS Root for end-user machines.

### Config (optional, Phase 6+)

```toml
[trust]
# Declared reputation / graduation intent (optional; does not change signing behavior)
declared_tier = "self_signed_host"
# Comma-free list of additional notes for TRUST.md
notes = []
```

If absent, Signet infers `self_signed_host` when an active identity exists and host signing ran; else `checksum_only` when sums exist; else `unknown`.

---

## Phase 6 — Trust clarity (no new command required)

### TRUST.md template changes (`trust_kit.rs`)

Add sections (order):

1. Existing publisher identity table.
2. **Trust tier** — primary tier id + one-sentence meaning + explicit “does not imply OS reputation.”
3. Verify steps — prefer `signet verify` once Phase 7 ships; until then keep manual steps.
4. Platform notes (existing) — add one line: “Do not install this certificate into Trusted Root on end-user PCs.”
5. What Signet will never put here (existing).

### Doctor

- Report inferred or declared tier as an informational check (`trust-tier`).
- If any help text mentions importing certs, limit to **developer machines** / enterprise MDM — never “tell your users to trust this root.”

### Acceptance (Phase 6)

- [x] Regenerated `TRUST.md` contains tier id and Root anti-pattern.
- [x] `docs/trust-model.md` matches tier table.
- [x] Doctor prints tier without failing the run solely for `self_signed_host`.

**Implemented:** `trust_tier.rs`, `[trust]` in config, `trust_kit` template, doctor `trust-tier` check.

---

## Phase 7 — `signet verify`

### CLI contract

```text
signet verify [OPTIONS]

Options:
  --config <PATH>           signet.toml (same resolution as other commands)
  --artifact <PATH>         repeatable; files to check against sums / identity
  --sums <PATH>             default: ./SHA256SUMS or beside artifacts
  --trust <PATH>            default: ./TRUST.md
  --require-sig             fail if community signature on sums missing/invalid
                            (Phase 8; until then: exit 3 with clear “not implemented” OR
                             skip if no Phase 8 binary feature — prefer soft warn in 7.0,
                             hard --require-sig only after Phase 8)
  --json                    machine-readable report
  --fingerprint <HEX>       override expected SHA-256 fingerprint (else parse TRUST.md)
```

**Default behavior (no `--artifact`):**

1. Load config + TRUST.md fingerprint if present.
2. If `SHA256SUMS` exists in cwd (or config project root), verify all listed files that exist on disk.
3. Print summary: tier, fingerprint source, checksum results, host-sign hint (best-effort).

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | All requested checks passed |
| 1 | Checksum or fingerprint mismatch |
| 2 | Missing inputs / unreadable files / parse error |
| 3 | Policy failure (`--require-sig` unmet) after Phase 8 |

### Report shape (human + JSON)

```json
{
  "tier": "self_signed_host",
  "fingerprint_expected": "...",
  "fingerprint_source": "TRUST.md",
  "checksums": [
    {"file": "app.exe", "ok": true, "expected": "...", "actual": "..."}
  ],
  "sums_signature": {"present": false, "ok": null, "scheme": null},
  "warnings": ["SmartScreen/Gatekeeper not evaluated"]
}
```

### Module ownership

| Concern | Module |
|---------|--------|
| Parse fingerprint from TRUST.md | `trust_kit` (new `parse_fingerprint`) |
| Verify SHA256SUMS lines | `sign/checksum` (new `verify_sha256sums`) |
| CLI / exit codes | `commands/verify.rs` |
| clap wiring | `cli.rs` |
| TUI optional later | Phase 7.x — not required for exit |

### Sequence

```text
resolve paths → load expected fingerprint → load SHA256SUMS
  → for each artifact: hash and compare
  → (Phase 8) verify sums signature if present or --require-sig
  → print report → exit
```

### Acceptance (Phase 7)

- [x] `signet verify` on a fixture with matching sums exits 0.
- [x] Tampered file exits 1.
- [x] Missing TRUST + missing sums with no artifacts exits 2.
- [x] Unit tests for `verify_sha256sums` and fingerprint parse.
- [x] Help text does not suggest installing Root certs.

**Implemented:** `commands/verify.rs`, checksum verify helpers, TRUST fingerprint parse. Phase 8 wires hard `--require-sig` (exit 3) + minisign/GPG.

### Proof plan

| Layer | Command / evidence |
|-------|-------------------|
| L1 | `cargo test -p signet checksum::` / trust_kit parse tests |
| L2 | `cargo test -p signet` |
| L3 | Manual: create temp dir with file + SHA256SUMS + TRUST.md; `cargo run -p signet -- verify` |

---

## Subtraction

- Do not fork a second trust document format; extend `trust_kit::render_trust_md`.
- Do not duplicate hashing outside `sign/checksum`.

## Open questions (non-blocking)

- Best-effort Windows thumbprint inspection via `signtool` / PowerShell — defer to Phase 7.1 if noisy.
- JSON schema version field — add `schema_version: 1` in first JSON emit.
