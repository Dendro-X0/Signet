# CLI install / self-update

Signet can be installed globally with a **one-liner**, then **updated or uninstalled from the TUI** (or `signet self …`).

Design: [`specs/backend/self-update-design.md`](../specs/backend/self-update-design.md)

## Install

```bash
curl -LsSf https://github.com/Dendro-X0/Signet/releases/latest/download/install.sh | sh
```

```powershell
irm https://github.com/Dendro-X0/Signet/releases/latest/download/install.ps1 | iex
```

Install root: `~/.signet-cli/` (Unix) or `%LOCALAPPDATA%\Signet\` (Windows). A receipt `install.toml` marks the install as managed.

## Manage

```bash
signet self status
signet self update
signet self update --check
signet self uninstall --yes
```

TUI hub (run `signet` with no args): **CLI status**, **Update Signet**, **Uninstall Signet**.

Uninstall removes only the CLI install root — never per-app project `.signet/` directories.

## Releases

Tag `v*` runs [`.github/workflows/release-cli.yml`](../.github/workflows/release-cli.yml), which uploads platform binaries, `SHA256SUMS`, and the installers.
