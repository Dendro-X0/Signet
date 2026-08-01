#!/usr/bin/env sh
# Signet CLI installer — one command:
#   curl -LsSf https://github.com/Dendro-X0/Signet/releases/latest/download/install.sh | sh
set -eu

REPO="Dendro-X0/Signet"
BASE="https://github.com/${REPO}/releases/latest/download"
ROOT="${HOME}/.signet-cli"
BIN_DIR="${ROOT}/bin"
BIN="${BIN_DIR}/signet"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "${OS}-${ARCH}" in
  linux-x86_64|linux-amd64) ASSET="signet-x86_64-unknown-linux-gnu" ;;
  linux-aarch64|linux-arm64) ASSET="signet-aarch64-unknown-linux-gnu" ;;
  darwin-x86_64) ASSET="signet-x86_64-apple-darwin" ;;
  darwin-arm64) ASSET="signet-aarch64-apple-darwin" ;;
  *)
    echo "error: unsupported platform ${OS}-${ARCH}" >&2
    exit 1
    ;;
esac

VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)
VERSION_NUM=$(echo "$VERSION" | sed 's/^v//')

mkdir -p "$BIN_DIR"
TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT

echo "Downloading ${ASSET} (${VERSION})…"
curl -fsSL "${BASE}/${ASSET}" -o "$TMP"
chmod +x "$TMP"
mv "$TMP" "$BIN"

cat > "${ROOT}/install.toml" <<EOF
# Signet CLI install receipt — do not edit
method = "installer"
repo = "${REPO}"
installed_version = "${VERSION_NUM}"
binary_path = "${BIN}"
EOF

echo "Installed to ${BIN}"
if [ -x "${BIN}" ]; then
  echo "Managed binary: $(${BIN} --version 2>/dev/null || true)"
fi

case ":${PATH}:" in
  *":${BIN_DIR}:"*) ;;
  *)
    echo ""
    echo "Add Signet to your PATH (for this shell and future ones):"
    echo "  export PATH=\"${BIN_DIR}:\$PATH\""
    if [ -n "${SHELL:-}" ]; then
      case "$SHELL" in
        */zsh) RC="${HOME}/.zshrc" ;;
        */bash) RC="${HOME}/.bashrc" ;;
        *) RC="${HOME}/.profile" ;;
      esac
      echo "  echo 'export PATH=\"${BIN_DIR}:\$PATH\"' >> ${RC}"
    fi
    ;;
esac

# Prefer installer over cargo when both are present.
export PATH="${BIN_DIR}:${PATH}"
RESOLVED=$(command -v signet 2>/dev/null || true)
if [ -n "${RESOLVED}" ]; then
  # Compare real paths when possible
  OURS="${BIN}"
  if [ "$(uname -s)" = "Darwin" ] || [ "$(uname -s)" = "Linux" ]; then
    OURS=$(cd "$(dirname "${BIN}")" && pwd)/$(basename "${BIN}")
  fi
  if [ "${RESOLVED}" != "${BIN}" ] && [ "${RESOLVED}" != "${OURS}" ]; then
    echo ""
    echo "WARNING: \`signet\` on PATH is not the installer binary:"
    echo "  PATH resolves to: ${RESOLVED}"
    echo "  Installer binary: ${BIN}"
    case "${RESOLVED}" in
      */.cargo/bin/*)
        echo "  This looks like a cargo install. Fix with:"
        echo "    cargo uninstall signet"
        echo "  Or remove that file, then open a new terminal."
        ;;
      *)
        echo "  Remove or rename the shadowed binary, or put ${BIN_DIR} earlier on PATH."
        ;;
    esac
    echo "  Verify managed build:  ${BIN} --version"
  else
    echo ""
    echo "Then run:  signet --version"
  fi
else
  echo ""
  echo "Then run:  signet --version   (after updating PATH)"
fi

echo "Update:    signet self update"
echo "Uninstall: signet self uninstall --yes"
echo "Or open:   signet   (TUI → Update / Uninstall Signet)"
