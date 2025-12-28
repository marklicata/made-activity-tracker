#!/bin/bash
# Fast Python venv setup - only creates if missing

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_PATH="$SCRIPT_DIR/../src-tauri/amplifier-tools/.venv"

cd "$SCRIPT_DIR/../src-tauri/amplifier-tools"

if [ ! -d "$VENV_PATH" ]; then
    echo "Creating Python venv..."
    uv venv > /dev/null 2>&1
else
    echo "✓ Python venv exists"
fi

echo "Installing dependencies..."
uv pip install -e . --quiet > /dev/null 2>&1

echo "✓ Python setup complete"
