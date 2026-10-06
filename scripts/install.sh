#!/usr/bin/env bash
# Build release binary + desktop bundle, then install permanently for the current user.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
APP_ID="ai-workload-monitor"
APP_NAME="AI Workload Monitor"
INSTALL_DIR="${HOME}/.local/share/${APP_ID}"
BIN_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/128x128/apps"

cd "$PROJECT_DIR"
source "${HOME}/.cargo/env" 2>/dev/null || true
export CARGO_TARGET_DIR="${PROJECT_DIR}/src-tauri/target"

echo "==> Regenerating icons..."
python3 "$SCRIPT_DIR/generate-icons.py"

echo "==> Building release (this may take a few minutes)..."
npm run tauri build

RELEASE_BIN="${CARGO_TARGET_DIR}/release/${APP_ID}"
DEB_FILE="$(find "${CARGO_TARGET_DIR}/release/bundle/deb" -name '*.deb' 2>/dev/null | head -1 || true)"

install_binary() {
  local dest="${INSTALL_DIR}/${APP_ID}"
  local staging="${dest}.new"

  cp "$RELEASE_BIN" "$staging"
  chmod +x "$staging"
  # mv renames in place without truncating a running binary (cp fails with "Text file busy")
  mv -f "$staging" "$dest"
}

install_user_layout() {
  echo "==> Installing to ${INSTALL_DIR}..."
  mkdir -p "$INSTALL_DIR" "$BIN_DIR" "$DESKTOP_DIR" "$ICON_DIR"

  if pgrep -f "${INSTALL_DIR}/${APP_ID}" >/dev/null 2>&1 || pgrep -f "/${APP_ID}" >/dev/null 2>&1; then
    echo "==> ${APP_NAME} is running — updating binary in place (restart app to load new version)."
  fi

  install_binary

  ln -sf "${INSTALL_DIR}/${APP_ID}" "${BIN_DIR}/${APP_ID}"

  cp "${PROJECT_DIR}/src-tauri/icons/128x128.png" "${ICON_DIR}/${APP_ID}.png"

  cat > "${DESKTOP_DIR}/${APP_ID}.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=${APP_NAME}
Comment=Monitor CPU, GPU, NPU, memory, network and AI workloads
Exec=${INSTALL_DIR}/${APP_ID}
Icon=${ICON_DIR}/${APP_ID}.png
Terminal=false
Categories=System;Monitor;Utility;
Keywords=monitor;cpu;gpu;npu;ai;system;performance;
StartupNotify=true
StartupWMClass=ai-workload-monitor
EOF

  chmod +x "${DESKTOP_DIR}/${APP_ID}.desktop"

  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
  fi

  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
  fi
}

if [[ -n "$DEB_FILE" ]] && command -v dpkg >/dev/null 2>&1; then
  echo "==> Found .deb package: $(basename "$DEB_FILE")"
  if sudo -n true 2>/dev/null; then
    echo "==> Installing system-wide via dpkg..."
    sudo dpkg -i "$DEB_FILE" || sudo apt-get install -f -y
    echo ""
    echo "Installed system-wide. Launch from application menu: ${APP_NAME}"
    exit 0
  else
    echo "==> No passwordless sudo — installing for current user only."
  fi
fi

if [[ ! -x "$RELEASE_BIN" ]]; then
  echo "ERROR: Release binary not found at ${RELEASE_BIN}"
  exit 1
fi

install_user_layout

echo ""
echo "============================================"
echo "  ${APP_NAME} installed successfully!"
echo "============================================"
echo ""
echo "  Launch from terminal:  ${APP_ID}"
echo "  Or search in app menu: ${APP_NAME}"
echo "  Binary location:       ${INSTALL_DIR}/${APP_ID}"
echo ""
echo "  Optional autostart:"
echo "    ${SCRIPT_DIR}/install-autostart.sh"
echo ""
