# CLI install / self-update

Signet can be installed globally with a **one-liner**, then **updated or uninstalled from the TUI** (or `signet self …`).

Design: [`specs/backend/self-update-design.md`](../specs/backend/self-update-design.md)

## Windows

```powershell
irm https://github.com/Dendro-X0/Signet/releases/latest/download/install.ps1 | iex
```

If download fails with `unexpected EOF` / transport errors (flaky `Invoke-WebRequest`), retry or install via curl:

```powershell
curl.exe -fL --retry 3 -o "$env:TEMP\signet.exe" `
  https://github.com/Dendro-X0/Signet/releases/latest/download/signet-x86_64-pc-windows-msvc.exe
New-Item -Force -ItemType Directory "$env:LOCALAPPDATA\Signet\bin" | Out-Null
Copy-Item -Force "$env:TEMP\signet.exe" "$env:LOCALAPPDATA\Signet\bin\signet.exe"
Copy-Item -Force "$env:TEMP\signet.exe" "$env:USERPROFILE\bin\signet.exe"
& "$env:LOCALAPPDATA\Signet\bin\signet.exe" --version
```

- Install root: `%LOCALAPPDATA%\Signet\`
- Binary: `bin\signet.exe` (user PATH updated by the installer — **prepended** so it can beat `~\.cargo\bin`)
- Also mirrors to `%USERPROFILE%\bin\signet.exe` so **Git Bash / Cursor** terminals that already have `~/bin` on PATH work **without restarting the IDE**
- Installer prefers `curl.exe` with retries (falls back to `Invoke-WebRequest`)

### “command not found” after install (Windows + bash)

The installer wrote the binary and updated the **User** PATH registry, but **already-open Cursor / Git Bash** processes keep a stale `PATH` until you fully quit and reopen the app.

**Immediate fix (this terminal):**

```bash
cp "$LOCALAPPDATA/Signet/bin/signet.exe" "$HOME/bin/signet.exe"
hash -r
signet --version
```

Or:

```bash
export PATH="$LOCALAPPDATA/Signet/bin:$HOME/bin:$PATH"
signet --version
```

**Durable:** re-run the installer (v0.5.2+) which mirrors into `~/bin`, or fully quit Cursor and open a new window.

## Cargo vs installer (PATH shadow)

If you previously ran `cargo install --path …` / `cargo install --git …`, **`~/.cargo/bin/signet` often wins** over the official installer. Symptoms:

```text
# installer downloaded v0.5.0 …
signet --version
signet 0.4.0          # still the cargo binary
```

Check:

```powershell
Get-Command signet | Format-List Source
& "$env:LOCALAPPDATA\Signet\bin\signet.exe" --version
```

```bash
command -v signet
~/.signet-cli/bin/signet --version   # Unix installer root
```

Fix (pick one):

```bash
cargo uninstall signet
# then open a new terminal and re-check: signet --version
```

Or remove/rename the cargo `signet` binary. Re-run the official installer afterward if needed — it warns when PATH still resolves elsewhere.

`signet self status` reports this shadow when an installer receipt exists but the running process is from cargo.

## macOS

```bash
curl -LsSf https://github.com/Dendro-X0/Signet/releases/latest/download/install.sh | sh
```

- Apple Silicon and Intel
- Install root: `~/.signet-cli/`

## Linux

```bash
curl -LsSf https://github.com/Dendro-X0/Signet/releases/latest/download/install.sh | sh
```

- x86_64 and aarch64
- Install root: `~/.signet-cli/`

## Manage

```bash
signet self status
signet self update
signet self update --check
signet self uninstall --yes
```

TUI hub (run `signet` with no args): **CLI status**, **Update Signet**, **Uninstall Signet**.

Uninstall removes only the CLI install root — never per-app project `.signet/` directories.

## Releases & CI

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| [`ci.yml`](../.github/workflows/ci.yml) | push/PR to `main` | `cargo test` (+ clippy on Linux) |
| [`release-cli.yml`](../.github/workflows/release-cli.yml) | tag `v*` | Multi-OS binaries, installers, `SHA256SUMS`, GitHub Release |

Publish a release by tagging, e.g. `git tag -a v0.3.0 -m "…" && git push origin v0.3.0`.
