# Signet

**Signet** is a CLI + TUI for **identity, signing, trust docs, and release** of self-signed apps — desktop and mobile, across frameworks.

Today the deepest path is **Tauri** (Windows / macOS / Linux). Scanning already recognizes Electron, Android, and iOS markers; framework-native build/sign adapters expand from there.

Self-signing is **not** a substitute for paid Apple / Microsoft / Google developer programs. Signet optimizes for independent and non-profit OSS: repeatable local trust, honest install guidance, and agent-friendly automation.

**Repo:** [github.com/Dendro-X0/Signet](https://github.com/Dendro-X0/Signet)

## Install

Pick your OS. After install, open the TUI with `signet` to **Update Signet** or **Uninstall Signet**, or use `signet self update` / `signet self uninstall --yes`.

### Windows

```powershell
irm https://github.com/Dendro-X0/Signet/releases/latest/download/install.ps1 | iex
```

Installs to `%LOCALAPPDATA%\Signet\bin` and adds it to your user `PATH`.

Direct binary (optional): [signet-x86_64-pc-windows-msvc.exe](https://github.com/Dendro-X0/Signet/releases/latest/download/signet-x86_64-pc-windows-msvc.exe)

### macOS

```bash
curl -LsSf https://github.com/Dendro-X0/Signet/releases/latest/download/install.sh | sh
```

Works on Apple Silicon and Intel. Add `~/.signet-cli/bin` to your `PATH` if the installer prints that hint.

Direct binaries (optional):

- Apple Silicon: [signet-aarch64-apple-darwin](https://github.com/Dendro-X0/Signet/releases/latest/download/signet-aarch64-apple-darwin)
- Intel: [signet-x86_64-apple-darwin](https://github.com/Dendro-X0/Signet/releases/latest/download/signet-x86_64-apple-darwin)

### Linux

```bash
curl -LsSf https://github.com/Dendro-X0/Signet/releases/latest/download/install.sh | sh
```

Supports x86_64 and aarch64. Add `~/.signet-cli/bin` to your `PATH` if prompted.

Direct binaries (optional):

- x86_64: [signet-x86_64-unknown-linux-gnu](https://github.com/Dendro-X0/Signet/releases/latest/download/signet-x86_64-unknown-linux-gnu)
- aarch64: [signet-aarch64-unknown-linux-gnu](https://github.com/Dendro-X0/Signet/releases/latest/download/signet-aarch64-unknown-linux-gnu)

### Manage

```bash
signet --version
signet self status
signet self update
signet self uninstall --yes
```

### From source (contributors)

```bash
cargo install --path crates/signet
# or:
cargo build --release -p signet
```

Requires a Rust toolchain. Platform **app** signing (not the CLI installer) needs host tools (`signtool` + OpenSSL on Windows, `codesign` on macOS, OpenSSL on Linux). Run `signet doctor` to check.

## Quick start

```bash
signet doctor
signet                  # interactive hub (TTY)
signet scan             # find apps/installers + suggest config
signet init --name my-app
signet identity create
signet trust
signet build            # Tauri build + sign (today)
signet release --tag v0.2.0 --dry-run
```

## Scope

| Surface | Status |
|---------|--------|
| Identity + TRUST.md | Ready |
| Windows / macOS / Linux signing | Ready (desktop artifacts) |
| Tauri `build` wrap | Ready |
| Repo `scan` (Tauri, Electron, Android, iOS, …) | Ready |
| Electron / Flutter / RN / Capacitor build adapters | Roadmap |
| Mobile store signing helpers | Roadmap (honest about Play / Apple gates) |

## Commands

| Command | Purpose |
|---------|---------|
| `signet` | TUI hub + guided setup |
| `scan` | Repo self-check: projects, installers, suggested config |
| `doctor` | Host tooling / auth checks |
| `init` | Write `signet.toml` + `.signet/` |
| `identity` | Create / import / list / show signing identity |
| `sums-key` | Create / show minisign key for `SHA256SUMS` |
| `android` | Android keystore create/import/show + APK sign |
| `ios` | Package `.app` → `.ipa` + honesty notes |
| `trust` | Emit `TRUST.md` (safe to commit) |
| `verify` | Fingerprint + SHA256SUMS + minisign/GPG checks |
| `self` | CLI install status / update / uninstall |
| `release` | Checksums + GitHub Release publish |

Legacy `selfsign.toml` / `.selfsign/` are still detected if present.

## Honesty

- **Windows:** SmartScreen may still warn for self-signed / low-reputation certs.
- **macOS:** Gatekeeper may block; notarization requires an Apple Developer account.
- **Android / iOS:** Listed by `scan`; store signing stays with Play / Apple tooling until Signet adds explicit helpers.

Private keys live under gitignored `.signet/` — never in `TRUST.md` or git.

## Docs

| Doc | Topic |
|-----|--------|
| [START-HERE.md](START-HERE.md) | Agent / contributor boot |
| [docs/product.md](docs/product.md) | Product thesis |
| [docs/trust-model.md](docs/trust-model.md) | Integrity vs reputation, tiers |
| [docs/verify.md](docs/verify.md) | `signet verify` |
| [docs/install.md](docs/install.md) | One-command install + self update |
| [docs/roadmap.md](docs/roadmap.md) | Phases + spec gate |
| [specs/backend/README.md](specs/backend/README.md) | Design contracts (Phases 6+) |
| [docs/scan.md](docs/scan.md) | Repo scan |
| [docs/tui.md](docs/tui.md) | Hub & guided flows |
| [docs/identity.md](docs/identity.md) | Identity + trust kit |
| [docs/signing.md](docs/signing.md) | Platform signing |
| [docs/android.md](docs/android.md) | Android keystore + Play honesty |
| [docs/ios.md](docs/ios.md) | iOS IPA package + free-provisioning honesty |
| [docs/release.md](docs/release.md) | GitHub Releases |
| [CHANGELOG.md](CHANGELOG.md) | Releases |

## Development

```bash
cargo test -p signet
cargo run -p signet -- --help
```

## License

MIT OR Apache-2.0
