# Current session handoff

**Updated:** 2026-07-31  
**Band:** Phases 6–7 + CLI distribution implemented; Phase 8 next

## Next atomic step

Implement **Phase 8 — Checksum signing** from [`specs/backend/checksum-signing-design.md`](../../specs/backend/checksum-signing-design.md):

1. `signet sums-key create/show` + minisign under `.signet/sums/`
2. Sign `SHA256SUMS` → `SHA256SUMS.minisig` from build/release
3. Wire `signet verify` hard `--require-sig` (exit 3)

**PAUSED / CANCELLED:** none  
**Blocked for coding:** Phases 9–12 stubs

## Canonical owners

| Work | Owner |
|------|--------|
| CLI self-update | `self_manage/`, `commands/self_cmd.rs`, `installers/` |
| Checksum signing | Phase 8 |

## Distribution note

CLI one-liner install requires a tagged release so `release-cli.yml` uploads binaries. Until the next `v*` tag, use `cargo install --path crates/signet` for local use; `signet self status` reports unmanaged.

## Recently completed

- Phase 6–7 integrity
- CLI installers + `signet self` + TUI update/uninstall
