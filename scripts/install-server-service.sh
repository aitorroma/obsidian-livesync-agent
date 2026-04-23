#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"

TARGET_USER=""
VAULT_PATH=""
BASE_URL=""
DATABASE=""
USERNAME=""
PASSWORD=""
INTERVAL_SECONDS="30"
FORCE_CONFIG="0"
ENABLE_LINGER="1"

CONFIG_DIR="/etc/livesync-agent"
SERVICE_TEMPLATE="/etc/systemd/system/livesync-agent@.service"

usage() {
  cat <<USAGE
Install livesync-agent as a system-wide service for a specific user.

Usage:
  $0 --user <linux-user> --vault-path <path> --base-url <url> --database <name> [options]

Options:
  --user <name>             Linux user that will run the sync process (required)
  --vault-path <path>       Vault path owned by target user (required)
  --base-url <url>          CouchDB base URL (required)
  --database <name>         CouchDB database (required)
  --username <name>         CouchDB username (optional)
  --password <pass>         CouchDB password (optional; visible in shell history)
  --password-stdin          Read CouchDB password from stdin (safer)
  --interval-seconds <n>    Sync interval in daemon mode (default: 30)
  --force-config            Overwrite existing /etc/livesync-agent/<user>.toml
  --disable-linger          Do not call loginctl enable-linger
  -h, --help                Show help
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --user)
      TARGET_USER="$2"; shift 2 ;;
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
    --disable-linger)
      ENABLE_LINGER="0"; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1 ;;
  esac
done

if [[ "${EUID}" -ne 0 ]]; then
  echo "This script must run as root (use sudo)." >&2
  exit 1
fi

if [[ -z "$TARGET_USER" || -z "$VAULT_PATH" || -z "$BASE_URL" || -z "$DATABASE" ]]; then
  echo "Error: --user, --vault-path, --base-url, --database are required." >&2
  usage
  exit 1
fi

if ! id "$TARGET_USER" >/dev/null 2>&1; then
  echo "Error: user does not exist: $TARGET_USER" >&2
  exit 1
fi

if [[ ! -d "$VAULT_PATH" ]]; then
  echo "Error: vault path does not exist: $VAULT_PATH" >&2
  exit 1
fi

if [[ -n "$USERNAME" && -z "$PASSWORD" ]]; then
  echo -n "CouchDB password for ${USERNAME}: " >&2
  read -r -s PASSWORD
  echo >&2
fi

echo "[1/6] Building release binary..."
(cd "$PROJECT_DIR" && cargo build --release)

install -m 0755 "$PROJECT_DIR/target/release/livesync-agent" /usr/local/bin/livesync-agent
echo "Installed /usr/local/bin/livesync-agent"

mkdir -p "$CONFIG_DIR"
CONFIG_FILE="$CONFIG_DIR/${TARGET_USER}.toml"
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
  chown "$TARGET_USER":"$TARGET_USER" "$CONFIG_FILE"
  chmod 600 "$CONFIG_FILE"
  echo "Wrote config: $CONFIG_FILE"
fi

cat > "$SERVICE_TEMPLATE" <<UNIT
[Unit]
Description=LiveSync Agent (%i)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=%i
Group=%i
ExecStart=/usr/local/bin/livesync-agent --config /etc/livesync-agent/%i.toml daemon --interval-seconds ${INTERVAL_SECONDS}
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT

echo "[4/6] Reloading systemd..."
systemctl daemon-reload

if [[ "$ENABLE_LINGER" == "1" ]] && command -v loginctl >/dev/null 2>&1; then
  echo "[5/6] Enabling linger for $TARGET_USER ..."
  loginctl enable-linger "$TARGET_USER" || true
fi

echo "[6/6] Enabling and starting service..."
systemctl enable --now "livesync-agent@${TARGET_USER}.service"
systemctl --no-pager --full status "livesync-agent@${TARGET_USER}.service" || true

echo
echo "Done."
echo "Config:  $CONFIG_FILE"
echo "Service: livesync-agent@${TARGET_USER}.service"
echo
echo "Useful commands:"
echo "  systemctl status livesync-agent@${TARGET_USER}.service"
echo "  journalctl -u livesync-agent@${TARGET_USER}.service -f"
echo "  systemctl restart livesync-agent@${TARGET_USER}.service"
