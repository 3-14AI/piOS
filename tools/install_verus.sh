#!/bin/bash
set -e

VERUS_DIR="tools/verus"
# Ensure VERUS_DIR exists
mkdir -p "$VERUS_DIR"

# Cleanup previous installation
rm -rf "$VERUS_DIR"
mkdir -p "$VERUS_DIR"

# PINNED Verus release compatible with Rust 1.94.0
PINNED_TAG="0.2026.04.19.6f7d4de"

echo "Using pinned Verus release: $PINNED_TAG"

FULL_DOWNLOAD_URL="https://github.com/verus-lang/verus/releases/download/release/${PINNED_TAG}/verus-${PINNED_TAG}-x86-linux.zip"
echo "Downloading Verus binary from $FULL_DOWNLOAD_URL..."

curl --retry 5 --retry-delay 2 --retry-max-time 30 -L -o verus.zip "$FULL_DOWNLOAD_URL"

echo "Extracting binary..."
unzip -q -o verus.zip -d "$VERUS_DIR"
rm verus.zip

# Download source code for dependencies
SOURCE_URL="https://github.com/verus-lang/verus/archive/refs/tags/release/${PINNED_TAG}.zip"

echo "Downloading source code from $SOURCE_URL..."
curl --retry 5 --retry-delay 2 --retry-max-time 30 -L -o verus-source.zip "$SOURCE_URL"

echo "Extracting source..."
unzip -q -o verus-source.zip

# Find the extracted directory
SOURCE_DIR=$(ls -d verus-* | grep -v zip | head -n 1)
echo "Source extracted to $SOURCE_DIR"

if [ -d "$SOURCE_DIR/dependencies" ]; then
    echo "Found dependencies in $SOURCE_DIR/dependencies"
    cp -r "$SOURCE_DIR/dependencies" "$VERUS_DIR/"
else
    echo "Could not find dependencies in extracted source root."
    ls -F "$SOURCE_DIR"
fi

# Patch Cargo.toml files to remove [workspace] to avoid conflicts
echo "Patching dependencies Cargo.toml..."
find "$VERUS_DIR/dependencies" -name "Cargo.toml" -exec sed -i 's/^\[workspace\]/# [workspace]/' {} \;
find "$VERUS_DIR/dependencies" -name "Cargo.toml" -exec sed -i 's/^members =/# members =/' {} \;
find "$VERUS_DIR" -name "Cargo.toml" -exec sed -i 's/^\[lints\]/# \[lints\]/g' {} \;
find "$VERUS_DIR" -name "Cargo.toml" -exec sed -i 's/^workspace = true/# workspace = true/g' {} \;

rm verus-source.zip
rm -rf "$SOURCE_DIR"

# Check contents
echo "Verus installed in $VERUS_DIR"
ls -F "$VERUS_DIR"
ls -F "$VERUS_DIR/dependencies" || true
