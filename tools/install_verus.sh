#!/bin/bash
set -e

VERUS_DIR="tools/verus"
# Ensure VERUS_DIR exists
mkdir -p "$VERUS_DIR"

# Cleanup previous installation
rm -rf "$VERUS_DIR"
mkdir -p "$VERUS_DIR"

echo "Fetching Verus release info..."

# 1. Get the latest release page
curl -sL -o release_page.html https://github.com/verus-lang/verus/releases/latest

# 2. Extract the expanded_assets URL
ASSETS_FRAGMENT_URL=$(grep -o 'src="[^"]*expanded_assets[^"]*"' release_page.html | cut -d '"' -f 2)

if [ -z "$ASSETS_FRAGMENT_URL" ]; then
    echo "Could not find assets fragment URL."
    exit 1
fi

echo "Fetching assets from $ASSETS_FRAGMENT_URL..."
curl -sL -o assets_page.html "$ASSETS_FRAGMENT_URL"

# 3. Find the download URL for linux
DOWNLOAD_PATH=$(grep -o 'href="[^"]*linux.zip"' assets_page.html | head -n 1 | cut -d '"' -f 2)

if [ -z "$DOWNLOAD_PATH" ]; then
    echo "Could not find linux zip in assets."
    exit 1
fi

FULL_DOWNLOAD_URL="https://github.com$DOWNLOAD_PATH"
echo "Downloading Verus binary from $FULL_DOWNLOAD_URL..."

curl -L -o verus.zip "$FULL_DOWNLOAD_URL"

echo "Extracting binary..."
unzip -q -o verus.zip -d "$VERUS_DIR"
rm verus.zip

# 4. Download source code for dependencies
TAG=$(echo "$DOWNLOAD_PATH" | cut -d '/' -f 6)
DECODED_TAG="${TAG//%2F//}"
SOURCE_URL="https://github.com/verus-lang/verus/archive/refs/tags/${DECODED_TAG}.zip"

echo "Downloading source code from $SOURCE_URL..."
curl -L -o verus-source.zip "$SOURCE_URL"

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

rm verus-source.zip
rm -rf "$SOURCE_DIR"
rm release_page.html assets_page.html

# Check contents
echo "Verus installed in $VERUS_DIR"
ls -F "$VERUS_DIR"
ls -F "$VERUS_DIR/dependencies" || true
