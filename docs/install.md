# CLI install / self-update

Signet can be installed globally with a **one-liner**, then **updated or uninstalled from the TUI** (or `signet self …`).

Design: [`specs/backend/self-update-design.md`](../specs/backend/self-update-design.md)

## Windows

```powershell
irm https://github.com/Dendro-X0/Signet/releases/latest/download/install.ps1 | iex
```

- Install root: `%LOCALAPPDATA%\Signet\`
- Binary: `bin\signet.exe` (user PATH updated by the installer)

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
