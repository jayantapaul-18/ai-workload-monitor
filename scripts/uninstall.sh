#!/usr/bin/env bash
# Remove user-level install and autostart entry.
set -euo pipefail

APP_ID="ai-workload-monitor"
INSTALL_DIR="${HOME}/.local/share/${APP_ID}"
BIN_LINK="${HOME}/.local/bin/${APP_ID}"
DESKTOP_FILE="${HOME}/.local/share/applications/${APP_ID}.desktop"
ICON_FILE="${HOME}/.local/share/icons/hicolor/128x128/apps/${APP_ID}.png"
AUTOSTART_FILE="${HOME}/.config/autostart/${APP_ID}.desktop"

rm -f "$BIN_LINK" "$DESKTOP_FILE" "$ICON_FILE" "$AUTOSTART_FILE"
rm -rf "$INSTALL_DIR"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
fi

echo "AI Workload Monitor removed from user install."
echo "If you installed via .deb, run: sudo apt remove ai-workload-monitor"
