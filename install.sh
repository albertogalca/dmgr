#!/bin/bash
set -e

REPO="albertogalca/dmgr"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  TARGET="x86_64-apple-darwin" ;;
    arm64)   TARGET="aarch64-apple-darwin" ;;
    *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Get latest version
VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')
if [ -z "$VERSION" ]; then
    echo "Failed to fetch latest version"
    exit 1
fi

echo "Installing dmgr v$VERSION for $TARGET..."

# Download and extract
TARBALL="dmgr-$VERSION-$TARGET.tar.gz"
URL="https://github.com/$REPO/releases/download/v$VERSION/$TARBALL"

mkdir -p "$INSTALL_DIR"
curl -sL "$URL" | tar xz -C "$INSTALL_DIR"

echo "Installed dmgr to $INSTALL_DIR/dmgr"

# Check if in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "Add to your PATH:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
fi
