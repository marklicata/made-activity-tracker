#!/bin/bash
# Fast Python venv setup - only creates if missing

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_PATH="$SCRIPT_DIR/../src-tauri/amplifier-tools/.venv"
INSTALL_MARKER="$VENV_PATH/.installed"

cd "$SCRIPT_DIR/../src-tauri/amplifier-tools"

if [ ! -d "$VENV_PATH" ]; then
    echo "Creating Python venv..."
    python3 -m venv .venv
    NEED_INSTALL=1
else
    echo "✓ Python venv exists"
    NEED_INSTALL=0
fi

# Check if dependencies need installing
if [ ! -f "$INSTALL_MARKER" ] || [ "$NEED_INSTALL" = "1" ]; then
    echo "Installing dependencies..."
    source .venv/bin/activate
    pip install -e . --quiet
    touch "$INSTALL_MARKER"
    echo "✓ Dependencies installed"
else
    echo "✓ Dependencies already installed"
fi

echo "✓ Python setup complete"
