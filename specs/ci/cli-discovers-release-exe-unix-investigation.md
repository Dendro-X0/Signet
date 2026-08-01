# CI investigation: cli discovers_release_exe (unix)

**Run / jobs:** `ci / test (ubuntu-latest)`, `ci / test (macos-latest)` — fail; `windows-latest` — pass  
**Commit class:** `artifact::cli::tests::discovers_release_exe`  
**Primary failure class:** unit test / platform fixture

## Exact error

```text
assertion `left == right` failed
  left: 0
  right: 1
```

at `crates/signet/src/artifact/cli.rs` discover test.

## Root cause

`collect_bins_in_dir` on Unix requires the executable bit (`mode & 0o111`). The test writes `mytool.exe` with `fs::write` only — no `+x` — so Unix discovers **0** binaries. Windows path only accepts `.exe` and does not check `+x`, so it passes.

## Fix (this iteration only)

1. Unix: treat `.exe` as a discoverable PE (cross-compile / fixture) without requiring `+x`; extensionless bins still need `+x`.
2. Test: create a host-shaped binary per OS (`mytool.exe` on Windows; `mytool` + `+x` on Unix) and still assert junk (`.pdb`, `deps/`) is skipped.

## Local proof

```bash
cargo test -p signet artifact::cli::tests::discovers_release_exe
```
