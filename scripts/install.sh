#!/usr/bin/env bash
set -euo pipefail

REPO="${LIVESYNC_AGENT_REPO:-aitorroma/obsidian-livesync-agent}"
BIN_NAME="livesync-agent"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="latest"

usage() {
  cat <<USAGE
Install livesync-agent from GitHub Releases (Linux x86_64).

Usage:
  $0 [options]

Options:
  --version <tag>      Release tag to install (default: latest)
  --install-dir <dir>  Install directory (default: ~/.local/bin)
  --repo <owner/name>  GitHub repo (default: aitorroma/obsidian-livesync-agent)
  -h, --help           Show this help

Examples:
  curl -fsSL https://raw.githubusercontent.com/aitorroma/obsidian-livesync-agent/main/scripts/install.sh | bash
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

if [[ "$os" != "linux" ]]; then
  echo "This installer currently supports Linux only." >&2
  echo "On macOS use Homebrew (build from source): brew install livesync-agent" >&2
  exit 1
fi

if [[ "$arch" != "x86_64" && "$arch" != "amd64" ]]; then
  echo "This installer currently supports Linux x86_64 only." >&2
  exit 1
fi

target="x86_64-unknown-linux-gnu"
archive_ext="tar.gz"
bin_file="$BIN_NAME"

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

(cd "$tmp_dir" && echo "$expected_line" | sha256sum -c -)

mkdir -p "$tmp_dir/unpack"
tar -xzf "$tmp_dir/$asset" -C "$tmp_dir/unpack"

mkdir -p "$INSTALL_DIR"
install -m 0755 "$tmp_dir/unpack/$bin_file" "$INSTALL_DIR/$bin_file"

echo "Installed: $INSTALL_DIR/$bin_file"
echo "Run: $INSTALL_DIR/$bin_file --help"
