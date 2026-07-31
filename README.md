# selfsign

CLI + TUI to **identity, sign, explain, and release** self-signed [Tauri](https://tauri.app/) apps across Windows, macOS, and Linux.

Self-signing is **not** a substitute for paid Apple / Microsoft / Google developer programs. It is a practical path for independent and non-profit OSS: repeatable signing, honest install docs, and agent-friendly automation.

## Install

```bash
cargo install --path crates/selfsign
# or from a clone:
cargo build --release -p selfsign
```

Requires a Rust toolchain. Platform signing also needs host tools (`signtool` + OpenSSL on Windows, `codesign` on macOS, OpenSSL on Linux). Run `selfsign doctor` to check.

## Quick start

```bash
selfsign doctor
selfsign                  # interactive hub (TTY)
selfsign scan             # find installers + suggest config
selfsign init --name my-app
selfsign identity create
selfsign trust
selfsign build
selfsign release --tag v0.1.0 --dry-run
```

Typical flow for a Tauri app directory:

```bash
cd path/to/tauri-app
selfsign scan --apply
selfsign identity create
selfsign trust
selfsign build
selfsign release --tag v0.1.0   # needs `gh` or GH_TOKEN
```

## Commands

| Command | Purpose |
|---------|---------|
| `selfsign` | TUI hub + guided setup |
| `scan` | Repo self-check: projects, installers, suggested config |
| `doctor` | Host tooling / auth checks |
| `init` | Write `selfsign.toml` + `.selfsign/` |
| `identity` | Create / import / list / show signing identity |
| `trust` | Emit `TRUST.md` (safe to commit) |
| `build` | `tauri build` + sign host artifacts |
| `release` | Checksums + GitHub Release publish |

## Honesty

- **Windows:** SmartScreen may still warn for self-signed / low-reputation certs.
- **macOS:** Gatekeeper may block; notarization requires an Apple Developer account (not performed by selfsign).
- **Android / iOS:** Detected by `scan` for awareness; store signing stays with Play / Apple tooling.

Private keys live under gitignored `.selfsign/` — never in `TRUST.md` or git.

## Docs

| Doc | Topic |
|-----|--------|
| [START-HERE.md](START-HERE.md) | Agent / contributor boot |
| [docs/product.md](docs/product.md) | Product thesis |
| [docs/scan.md](docs/scan.md) | Repo scan |
| [docs/tui.md](docs/tui.md) | Hub & guided flows |
| [docs/identity.md](docs/identity.md) | Identity + trust kit |
| [docs/signing.md](docs/signing.md) | Platform signing |
| [docs/release.md](docs/release.md) | GitHub Releases |
| [docs/roadmap.md](docs/roadmap.md) | Phases |
| [CHANGELOG.md](CHANGELOG.md) | Releases |

## Development

```bash
cargo test -p selfsign
cargo run -p selfsign -- --help
```

Workspace layout: `crates/selfsign` binary, docs under `docs/`.

## License

MIT OR Apache-2.0
