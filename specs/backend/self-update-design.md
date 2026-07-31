# Design: CLI self-install, update, uninstall

**Phase:** Distribution (parallel to integrity Phase 8)  
**Status:** implemented  
**Owners:** `self_manage/`, `commands/self_cmd.rs`, TUI hub, `installers/`, `.github/workflows/release-cli.yml`

## Goals

1. **One-command global install** (shell + PowerShell) onto PATH.
2. **`signet self update` / `uninstall` / `status`** for installer-managed copies.
3. **TUI actions** that call the same engines (Update Signet / Uninstall Signet).
4. Never touch per-app `.signet/` project secrets when uninstalling the CLI.

## Non-goals

- Replacing package managers (Homebrew/Scoop) as sole channel (optional later).
- Auto-updating without user action.
- Signing the Signet CLI with Authenticode/notarization in this band (document honesty).

---

## Layout

| OS | Install root | Binary |
|----|--------------|--------|
| Windows | `%LOCALAPPDATA%\Signet\` | `bin\signet.exe` |
| Unix | `$HOME/.signet-cli/` | `bin/signet` |

Receipt (committed by installer / `self update`):

```toml
# ~/.signet-cli/install.toml  or  %LOCALAPPDATA%\Signet\install.toml
method = "installer"          # only this enables self update/uninstall
repo = "Dendro-X0/Signet"
installed_version = "0.2.0"
binary_path = "…"             # absolute path to binary
updated_at = "…"              # RFC3339 optional
```

**Managed** = receipt `method = "installer"` and current exe resolves to `binary_path` (or lives under install root).  
**Unmanaged** = `cargo install`, `cargo run`, IDE builds → `self update` explains how to upgrade (cargo / re-run installer), TUI still shows status.

## Release assets (CI)

On tag `v*`, GitHub Actions builds and uploads:

| Asset | Host |
|-------|------|
| `signet-x86_64-pc-windows-msvc.exe` | Windows |
| `signet-x86_64-unknown-linux-gnu` | Linux |
| `signet-x86_64-apple-darwin` | macOS Intel |
| `signet-aarch64-apple-darwin` | macOS ARM |
| `SHA256SUMS` | all |
| `install.sh` / `install.ps1` | installers (copied from repo) |

Target triple selection uses `std::env::consts` + compile-time `TARGET` or runtime OS/ARCH map.

## CLI

```text
signet self status
signet self update [--check] [--force]
signet self uninstall [--yes]
```

### Update

1. Require managed install (unless `--force` with explicit warning for in-place replace of current exe when writable).
2. GET `https://api.github.com/repos/Dendro-X0/Signet/releases/latest` (User-Agent: signet).
3. Pick asset for host triple; download to temp.
4. Verify SHA-256 against release `SHA256SUMS` when present.
5. Replace binary (Windows: rename-running-exe trick; Unix: atomic rename).
6. Update receipt version.

### Uninstall

1. Require managed install.
2. Confirm unless `--yes`.
3. Remove binary + receipt + empty install dirs.
4. Print PATH cleanup hint (do not silently edit shell rc without documenting).
5. Windows: if exe locked after rename attempt, write deferred delete script and exit.

## TUI

Hub items (near Quit):

- **Update Signet** → `self update`
- **Uninstall Signet** → confirm → `self uninstall --yes` after confirm prompt

## Installers

```bash
curl -LsSf https://github.com/Dendro-X0/Signet/releases/latest/download/install.sh | sh
```

```powershell
irm https://github.com/Dendro-X0/Signet/releases/latest/download/install.ps1 | iex
```

Installers: download matching binary + write receipt + ensure install `bin` on PATH.

## Acceptance

- [x] `self status` reports managed vs unmanaged.
- [x] Managed `self update --check` hits GitHub API (when network + release assets exist).
- [x] Uninstall removes receipt + binary under install root; leaves project `.signet/` alone.
- [x] TUI dispatches to same command modules.
- [x] README documents one-liner install.
- [x] Unit tests: target asset name mapping, receipt round-trip.

## Proof

| Layer | Evidence |
|-------|----------|
| L1 | Unit tests for asset name + receipt |
| L2 | `cargo test -p signet` |
| L3 | `signet self status`; dry install path write in temp |
