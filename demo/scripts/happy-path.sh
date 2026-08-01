#!/usr/bin/env bash
# Signet demo kit — CLI happy path (Sign → Prove → Check) against demo/fixture.
# Prereqs: `signet` on PATH, or set SIGNET (e.g. SIGNET="cargo run -q -p signet --").
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIXTURE="$ROOT/fixture"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
SIGNET_BIN="${SIGNET:-signet}"

run() {
  # shellcheck disable=SC2086
  $SIGNET_BIN "$@"
}

echo "==> demo fixture: $FIXTURE"
echo "==> using: $SIGNET_BIN"
cd "$FIXTURE"

if ! run --version >/dev/null 2>&1; then
  echo "error: cannot run Signet. Install from README, or from repo root:"
  echo "  export SIGNET=\"cargo run -q -p signet --\""
  echo "  ./demo/scripts/happy-path.sh"
  exit 1
fi

echo ""
echo "==> Doctor"
run doctor || true

echo ""
echo "==> Identity (Sign)"
if [[ -f .signet/identity/active ]]; then
  run identity show || true
else
  run identity create --name default --cn "HelloSignet Demo" --org "Signet Demo" --days 825
fi

echo ""
echo "==> Trust (Prove)"
run trust

echo ""
echo "==> Build --skip-build --no-sign (Prove checksums; fake PE is not host-signed)"
run build --skip-build --no-sign --no-sums-sign

echo ""
echo "==> Verify (Check)"
run verify || true

echo ""
echo "==> Inspect (Check)"
run inspect --file dist/HelloSignet.exe || true
run inspect --file dist/HelloSignet.AppImage || true

echo ""
echo "==> Graduate notes (official path hint)"
run graduate notes

echo ""
echo "OK — CLI happy path finished."
echo "Visual:  cd \"$FIXTURE\" && signet   # TUI → Guided setup"
echo "Docs:    $REPO_ROOT/docs/demo.md"
echo "Release: curl -fsSL https://github.com/Dendro-X0/Signet/releases/latest/download/SHA256SUMS | head"
