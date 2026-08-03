# Ship (multi-platform)

`[platforms]` is a **commitment**. Local `signet build` only covers this host; `signet ship` orchestrates the rest.

## Commands

```bash
signet ship --plan                 # declared vs present + gap
signet ship --ci                   # write .github/workflows/signet-ship.yml
signet ship --ci --force           # overwrite workflow
signet ship --collect ./artifacts  # merge CI downloads → dist/signet-ship/ + SHA256SUMS
signet release --tag vX.Y.Z        # fails if coverage gap (unless --allow-partial)
```

## Flow

1. Declare `windows` / `macos` / `linux` in `signet.toml`.
2. `signet ship --ci` → commit workflow → push tag / `workflow_dispatch`.
3. Download matrix artifacts → `signet ship --collect DIR`.
4. `signet ship --plan` should show no gap → `signet release`.

Restore `.signet/identity` in CI via secrets if you need host signatures (not checksums alone).

## Specs

- [`specs/backend/multi-platform-ship-design.md`](../specs/backend/multi-platform-ship-design.md)
- [`specs/backend/ship-ci-collect-design.md`](../specs/backend/ship-ci-collect-design.md)
