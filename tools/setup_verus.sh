#!/bin/bash
echo "Setting up Verus..."

# Check if verus is in PATH
if command -v verus &> /dev/null; then
    echo "Verus is already installed."
    exit 0
fi

TARGET_DIR="tools/verus"
if [ -d "$TARGET_DIR" ]; then
    echo "Verus directory exists at $TARGET_DIR."
    echo "Please ensure it is in your PATH or use it directly."
    exit 0
fi

echo "Verus not found. Please download the latest release from https://github.com/verus-lang/verus/releases"
echo "and extract it to $TARGET_DIR or add it to your PATH."
