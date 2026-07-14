#!/bin/bash
set -e

PKG_NAME="clicense-server"
PKG_VERSION="0.1.0"
PACKAGE="${PKG_NAME}_${PKG_VERSION}_amd64.deb"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEB_ROOT="${SCRIPT_DIR}/deb"

echo "=== Building ${PACKAGE} ==="
echo "  Project root: ${PROJECT_ROOT}"
echo ""

# 1. Verify Linux release binaries exist
SERVER_BIN="${PROJECT_ROOT}/target/release/clicense-server"
CLI_BIN="${PROJECT_ROOT}/target/release/clicense"

if [ ! -f "$SERVER_BIN" ]; then
    echo "Error: Server binary not found at $SERVER_BIN"
    echo "Build it first on Linux: cargo build --release -p clicense-server"
    exit 1
fi
if [ ! -f "$CLI_BIN" ]; then
    echo "Error: CLI binary not found at $CLI_BIN"
    echo "Build it first on Linux: cargo build --release -p cnt-license"
    exit 1
fi

echo "[1/4] Stripping and copying binaries..."
strip "$SERVER_BIN" 2>/dev/null || true
strip "$CLI_BIN" 2>/dev/null || true
cp "$SERVER_BIN" "${DEB_ROOT}/usr/bin/clicense-server"
cp "$CLI_BIN" "${DEB_ROOT}/usr/bin/clicense"
chmod 755 "${DEB_ROOT}/usr/bin/clicense-server"
chmod 755 "${DEB_ROOT}/usr/bin/clicense"

echo "[2/4] Setting permissions..."
chmod 644 "${DEB_ROOT}/etc/clicense-server/config.yml"
chmod 644 "${DEB_ROOT}/etc/systemd/system/clicense-server.service"

echo "[3/4] Building .deb package..."
dpkg-deb --root-owner-group --build "${DEB_ROOT}" "${PACKAGE}"

echo "[4/4] Verifying..."
dpkg-deb --info "${PACKAGE}"
echo ""
echo "=== Package created: ${PACKAGE} ==="
ls -lh "${PACKAGE}"

echo ""
echo "To install:"
echo "  sudo dpkg -i ${PACKAGE}"
echo ""
echo "Package contents:"
dpkg-deb --contents "${PACKAGE}" | head -30
