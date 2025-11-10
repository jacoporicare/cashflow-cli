#!/bin/bash

set -e  # Exit on error

SOURCE_DIR="$(dirname "$0")"

if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

cargo build --release

INSTALL_DIR="$HOME/.local/bin"
cp "$SOURCE_DIR/../target/release/cashflow" "$INSTALL_DIR/"

echo "✅ Installed cashflow to $INSTALL_DIR"
echo "💡 Make sure $INSTALL_DIR is in your PATH"