#!/usr/bin/env bash
# Add AI Workload Monitor to login autostart (user session).
set -euo pipefail

APP_ID="ai-workload-monitor"
APP_NAME="AI Workload Monitor"
INSTALL_DIR="${HOME}/.local/share/${APP_ID}"
AUTOSTART_DIR="${HOME}/.config/autostart"
ICON_DIR="${HOME}/.local/share/icons/hicolor/128x128/apps"

if [[ ! -x "${INSTALL_DIR}/${APP_ID}" ]]; then
  echo "App not installed. Run scripts/install.sh first."
  exit 1
fi

mkdir -p "$AUTOSTART_DIR"

cat > "${AUTOSTART_DIR}/${APP_ID}.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=${APP_NAME}
Comment=Monitor CPU, GPU, NPU, memory, network and AI workloads
Exec=${INSTALL_DIR}/${APP_ID}
Icon=${ICON_DIR}/${APP_ID}.png
Terminal=false
Categories=System;Monitor;
X-GNOME-Autostart-enabled=true
Hidden=false
EOF

chmod +x "${AUTOSTART_DIR}/${APP_ID}.desktop"
echo "Autostart enabled. App will launch on login."
