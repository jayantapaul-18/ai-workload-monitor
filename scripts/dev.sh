#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"
source "$HOME/.cargo/env" 2>/dev/null || true

# Ensure tray/window/install icons match public/logo assets
python3 "$SCRIPT_DIR/generate-icons.py"

exec npm run tauri dev
