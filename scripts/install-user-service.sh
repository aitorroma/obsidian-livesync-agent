#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"

CONFIG_DIR="$HOME/.livesync-agent"
CONFIG_FILE="$CONFIG_DIR/config.toml"
BIN_DIR="$HOME/.local/bin"
BIN_PATH="$BIN_DIR/livesync-agent"
SERVICE_DIR="$HOME/.config/systemd/user"
SERVICE_FILE="$SERVICE_DIR/livesync-agent.service"

INTERVAL_SECONDS="30"
FORCE_CONFIG="0"
VAULT_PATH=""
BASE_URL=""
DATABASE=""
USERNAME=""
PASSWORD=""

usage() {
  cat <<USAGE
Install livesync-agent as a systemd --user service.

Usage:
  $0 --vault-path <path> --base-url <url> --database <name> [options]

Options:
  --vault-path <path>        Local vault path to sync (required)
  --base-url <url>           CouchDB base URL, e.g. https://couch.example.com (required)
  --database <name>          CouchDB database name (required)
  --username <user>          CouchDB username (optional)
  --password <pass>          CouchDB password (optional; visible in shell history)
  --password-stdin           Read CouchDB password from stdin (safer)
  --interval-seconds <n>     Daemon sync interval (default: 30)
  --force-config             Overwrite existing ~/.livesync-agent/config.toml
  -h, --help                 Show this help

Example:
  $0 \\
    --vault-path "$HOME/Obsidian" \\
    --base-url "https://data.example.com" \\
    --database "obsidian" \\
    --username "admin" \\
    --password "secret"
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --vault-path)
      VAULT_PATH="$2"; shift 2 ;;
    --base-url)
      BASE_URL="$2"; shift 2 ;;
    --database)
      DATABASE="$2"; shift 2 ;;
    --username)
      USERNAME="$2"; shift 2 ;;
    --password)
      PASSWORD="$2"; shift 2 ;;
    --password-stdin)
      read -r -s PASSWORD
      shift ;;
    --interval-seconds)
      INTERVAL_SECONDS="$2"; shift 2 ;;
    --force-config)
      FORCE_CONFIG="1"; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1 ;;
  esac
done

if [[ -z "$VAULT_PATH" || -z "$BASE_URL" || -z "$DATABASE" ]]; then
  echo "Error: --vault-path, --base-url and --database are required." >&2
  usage
  exit 1
fi

if [[ -n "$USERNAME" && -z "$PASSWORD" ]]; then
  echo -n "CouchDB password for ${USERNAME}: " >&2
  read -r -s PASSWORD
  echo >&2
fi

if [[ ! -d "$VAULT_PATH" ]]; then
  echo "Error: vault path does not exist: $VAULT_PATH" >&2
  exit 1
fi

echo "[1/5] Building release binary..."
(cd "$PROJECT_DIR" && cargo build --release)

mkdir -p "$BIN_DIR"
install -m 0755 "$PROJECT_DIR/target/release/livesync-agent" "$BIN_PATH"
echo "Installed binary: $BIN_PATH"

mkdir -p "$CONFIG_DIR"
if [[ -f "$CONFIG_FILE" && "$FORCE_CONFIG" != "1" ]]; then
  echo "Config already exists: $CONFIG_FILE"
  echo "Keeping existing config (use --force-config to overwrite)."
else
  cat > "$CONFIG_FILE" <<CFG
vault_path = "${VAULT_PATH}"
state_path = "${VAULT_PATH}/.livesync-agent/state.json"
ignore_prefixes = [
    ".git/",
    ".livesync-agent/",
]

[couchdb]
base_url = "${BASE_URL}"
database = "${DATABASE}"
username = "${USERNAME}"
password = "${PASSWORD}"
CFG
  chmod 600 "$CONFIG_FILE"
  echo "Wrote config: $CONFIG_FILE"
fi

mkdir -p "$SERVICE_DIR"
cat > "$SERVICE_FILE" <<UNIT
[Unit]
Description=LiveSync Agent (user service)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=${BIN_PATH} --config ${CONFIG_FILE} daemon --interval-seconds ${INTERVAL_SECONDS}
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
UNIT

echo "[4/5] Reloading user systemd..."
if command -v systemctl >/dev/null 2>&1; then
  systemctl --user daemon-reload
  systemctl --user enable --now livesync-agent.service
  echo "[5/5] Service started."
  systemctl --user --no-pager --full status livesync-agent.service || true
else
  echo "Warning: systemctl not found. Service file created at: $SERVICE_FILE"
fi

echo
echo "Done."
echo "Config:  $CONFIG_FILE"
echo "Binary:  $BIN_PATH"
echo "Service: $SERVICE_FILE"
echo
echo "Useful commands:"
echo "  systemctl --user status livesync-agent.service"
echo "  journalctl --user -u livesync-agent.service -f"
echo "  systemctl --user restart livesync-agent.service"
