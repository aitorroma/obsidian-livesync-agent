#!/usr/bin/env bash
set -euo pipefail

SERVICE_FILE="$HOME/.config/systemd/user/livesync-agent.service"
BIN_PATH="$HOME/.local/bin/livesync-agent"

if command -v systemctl >/dev/null 2>&1; then
  systemctl --user disable --now livesync-agent.service 2>/dev/null || true
  systemctl --user daemon-reload || true
fi

rm -f "$SERVICE_FILE"
rm -f "$BIN_PATH"

echo "Uninstalled livesync-agent service and binary."
echo "Config remains at ~/.livesync-agent/config.toml"
