#!/usr/bin/env bash
set -euo pipefail

# Install Tauri 2 prerequisites on Ubuntu/Debian
PACKAGES=(
  pkg-config
  libdbus-1-dev
  libwebkit2gtk-4.1-dev
  libayatana-appindicator3-dev
  librsvg2-dev
  libssl-dev
  build-essential
  curl
)

echo "Installing AI Workload Monitor dependencies..."
sudo apt-get update
sudo apt-get install -y "${PACKAGES[@]}"

echo ""
echo "Dependencies installed. Next steps:"
echo "  cd $(dirname "$0")/.."
echo "  npm install"
echo "  npm run tauri dev"
