#!/usr/bin/env sh
# Install create-rust-app from GitHub Releases.
#
#   curl -fsSL https://create-awesome-rust-app.vercel.app/install.sh | sh
#
# Installs `create-rust-app` and a `create-awesome-rust-app` alias into
# `~/.local/bin` (override with `CRA_INSTALL_DIR`). Pin a version with
# `CRA_VERSION=0.1.0`.
set -eu

REPO="Create-Rust-App/create-rust-app"
VERSION="${CRA_VERSION:-0.1.0}"
INSTALL_DIR="${CRA_INSTALL_DIR:-$HOME/.local/bin}"
TAG="create-rust-app@${VERSION}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac
case "$OS" in
  linux) OS="linux" ;;
  darwin) OS="darwin" ;;
  *) echo "Unsupported OS: $OS (Windows users: download the .exe from Releases)" >&2; exit 1 ;;
esac

ASSET="create-rust-app-${OS}-${ARCH}"
URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"

mkdir -p "$INSTALL_DIR"
TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT INT TERM
curl -fsSL -o "$TMP" "$URL"
chmod +x "$TMP"
mv "$TMP" "${INSTALL_DIR}/create-rust-app"
ln -sf "${INSTALL_DIR}/create-rust-app" "${INSTALL_DIR}/create-awesome-rust-app"
echo "Installed create-rust-app ${VERSION} to ${INSTALL_DIR}/create-rust-app"
