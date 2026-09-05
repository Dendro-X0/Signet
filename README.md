# Signet

**Signet** is a CLI + TUI for **app signing and verification**: **Sign → Prove → Check** across desktop and mobile frameworks.

- **Sign** — self-signed identity, or facilitate official paths (OV / Azure Trusted Signing / Apple notarize)
- **Prove** — `TRUST.md`, checksums, optional minisign
- **Check** — `signet verify` + `signet inspect`

Self-signing is **not** paid Apple / Microsoft / Google trust. Signet optimizes for indie and OSS: repeatable local signing, honest install docs, agent-friendly automation.

**Phase 1 launch (Install Trust pack):** [docs/launch/START-HERE.md](docs/launch/START-HERE.md) — free demo/CLI; optional paid one-OS ritual pack. Not a security bypass product.

**Repo:** [github.com/Dendro-X0/Signet](https://github.com/Dendro-X0/Signet)

## Install

**Windows (PowerShell)**

```powershell
irm https://github.com/Dendro-X0/Signet/releases/latest/download/install.ps1 | iex
```

**macOS / Linux**

```bash
curl -LsSf https://github.com/Dendro-X0/Signet/releases/latest/download/install.sh | sh
```

Then: `signet --version` · `signet self update` · full notes in [docs/install.md](docs/install.md).

## Two-minute start

**Scripted demo kit** (fixed fixture — best for local smoke / GIF prep):

```bash
./demo/scripts/happy-path.sh
# Windows: pwsh ./demo/scripts/happy-path.ps1
```

Guide: [docs/demo.md](docs/demo.md).

**Interactive:**

```bash
signet doctor
signet                 # TUI → Guided setup (Sign → Prove → Check)
```

**CLI sketch:**

```bash
signet scan
signet init --name my-app
signet identity create
signet trust
signet build           # set project.framework in signet.toml (see docs/frameworks.md)
signet verify
signet inspect --file path/to/artifact
```

## Two paths

| Path | When | Start here |
|------|------|------------|
| **Self-signed** (default) | Indie / OSS sideload; expect OS warnings | `identity` → `build` → `trust` / sums |
| **Official / paid** | OV/EV, Azure Trusted Signing, Apple notarize | [docs/graduation.md](docs/graduation.md) · `signet graduate notes` |

Integrity vs reputation: [docs/trust-model.md](docs/trust-model.md).  
Check downloads: [docs/verify.md](docs/verify.md) (`verify` + `inspect`).

## What Signet never does

- Tell end users to install your cert into **Trusted Root**
- Claim self-sign removes SmartScreen or Gatekeeper
- Put private keys or passwords in `TRUST.md` / git
- Pretend Play App Signing or App Store upload is done by packaging helpers alone

## Frameworks

Adapters discover + wrap build when you set `project.framework` (and `build_command` when required):

Tauri · Electron · Android · iOS · Flutter · React Native · Expo · Capacitor — [docs/frameworks.md](docs/frameworks.md), [docs/signing.md](docs/signing.md).

.NET and others: [docs/roadmap.md](docs/roadmap.md) (Beyond).

## Commands

| Job | Commands |
|-----|----------|
| Sign | `identity`, `build`, `android`, `ios`, `graduate` |
| Prove | `trust`, `sums-key`, `release` |
| Check | `verify`, `inspect` |
| Project | `init`, `scan`, `doctor`, `self`, TUI (`signet`) |

## Docs

| Doc | Topic |
|-----|--------|
| [docs/product.md](docs/product.md) | Thesis + surfaces |
| [docs/trust-model.md](docs/trust-model.md) | Tiers + anti-patterns |
| [docs/verify.md](docs/verify.md) | Verify + inspect |
| [docs/graduation.md](docs/graduation.md) | Official signing helpers |
| [docs/frameworks.md](docs/frameworks.md) | Hybrid adapters |
| [docs/demo.md](docs/demo.md) | Demo kit / recording happy path |
| [docs/android.md](docs/android.md) / [ios.md](docs/ios.md) | Mobile honesty |
| [docs/roadmap.md](docs/roadmap.md) | Phases 13–16 public cut |
| [START-HERE.md](START-HERE.md) | Contributor boot |
| [CHANGELOG.md](CHANGELOG.md) | Releases |

More: [install](docs/install.md) · [identity](docs/identity.md) · [signing](docs/signing.md) · [release](docs/release.md) · [scan](docs/scan.md) · [tui](docs/tui.md) · [specs](specs/backend/README.md)

## Development

```bash
cargo test -p signet
cargo run -p signet -- --help
```

## License

MIT OR Apache-2.0
