#!/usr/bin/env bash
set -euo pipefail

REPO="${LIVESYNC_AGENT_REPO:-aitorroma/obsidian-livesync}"
BIN_NAME="livesync-agent"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="latest"

usage() {
  cat <<USAGE
Install livesync-agent from GitHub Releases.

Usage:
  $0 [options]

Options:
  --version <tag>      Release tag to install (default: latest)
  --install-dir <dir>  Install directory (default: ~/.local/bin)
  --repo <owner/name>  GitHub repo (default: aitorroma/obsidian-livesync)
  -h, --help           Show this help

Examples:
  curl -fsSL https://raw.githubusercontent.com/aitorroma/obsidian-livesync/main/scripts/install.sh | bash
  $0 --version v0.1.0
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="$2"; shift 2 ;;
    --install-dir)
      INSTALL_DIR="$2"; shift 2 ;;
    --repo)
      REPO="$2"; shift 2 ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1 ;;
  esac
done

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"

case "$arch" in
  x86_64|amd64) arch="x86_64" ;;
  arm64|aarch64) arch="aarch64" ;;
  *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
esac

case "$os" in
  linux)
    target="${arch}-unknown-linux-gnu"
    archive_ext="tar.gz"
    bin_file="$BIN_NAME"
    ;;
  darwin)
    target="${arch}-apple-darwin"
    archive_ext="tar.gz"
    bin_file="$BIN_NAME"
    ;;
  msys*|mingw*|cygwin*)
    target="x86_64-pc-windows-msvc"
    archive_ext="zip"
    bin_file="${BIN_NAME}.exe"
    ;;
  *)
    echo "Unsupported OS: $os" >&2
    exit 1
    ;;
esac

if [[ "$VERSION" == "latest" ]]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)"
  if [[ -z "$VERSION" ]]; then
    echo "Failed to resolve latest release tag from ${REPO}" >&2
    exit 1
  fi
fi

asset="${BIN_NAME}-${VERSION}-${target}.${archive_ext}"
base_url="https://github.com/${REPO}/releases/download/${VERSION}"
asset_url="${base_url}/${asset}"
checksums_url="${base_url}/SHA256SUMS"

tmp_dir="$(mktemp -d)"
cleanup() { rm -rf "$tmp_dir"; }
trap cleanup EXIT

echo "Downloading ${asset} ..."
curl -fL "$asset_url" -o "$tmp_dir/$asset"

echo "Downloading SHA256SUMS ..."
curl -fL "$checksums_url" -o "$tmp_dir/SHA256SUMS"

expected_line="$(grep "  ${asset}$" "$tmp_dir/SHA256SUMS" || true)"
if [[ -z "$expected_line" ]]; then
  echo "Checksum entry not found for ${asset}" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  (cd "$tmp_dir" && echo "$expected_line" | sha256sum -c -)
elif command -v shasum >/dev/null 2>&1; then
  expected_hash="$(echo "$expected_line" | awk '{print $1}')"
  actual_hash="$(shasum -a 256 "$tmp_dir/$asset" | awk '{print $1}')"
  [[ "$expected_hash" == "$actual_hash" ]] || { echo "Checksum mismatch" >&2; exit 1; }
else
  echo "Neither sha256sum nor shasum found for checksum validation" >&2
  exit 1
fi

mkdir -p "$tmp_dir/unpack"
if [[ "$archive_ext" == "zip" ]]; then
  if ! command -v unzip >/dev/null 2>&1; then
    echo "unzip is required to install zip archives" >&2
    exit 1
  fi
  unzip -q "$tmp_dir/$asset" -d "$tmp_dir/unpack"
else
  tar -xzf "$tmp_dir/$asset" -C "$tmp_dir/unpack"
fi

mkdir -p "$INSTALL_DIR"
install -m 0755 "$tmp_dir/unpack/$bin_file" "$INSTALL_DIR/$bin_file"

echo "Installed: $INSTALL_DIR/$bin_file"
if [[ "$os" != msys* && "$os" != mingw* && "$os" != cygwin* ]]; then
  echo "Run: $INSTALL_DIR/$bin_file --help"
fi
