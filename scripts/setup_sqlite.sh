#!/bin/bash
set -e

# Directory for extension assets
ASSETS_DIR="extension/assets"
mkdir -p "$ASSETS_DIR"

echo "Downloading SQLite WASM artifacts..."

# Download official SQLite WASM build (using specific version known to be stable)
SQLITE_VERSION="3450100" # 3.45.1
SQLITE_YEAR="2024"
ZIP_NAME="sqlite-wasm-${SQLITE_VERSION}.zip"
URL="https://www.sqlite.org/${SQLITE_YEAR}/${ZIP_NAME}"

# Capture project root
PROJECT_ROOT=$(pwd)

# Create temp dir
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo "Fetching from $URL..."
curl -L -o "$ZIP_NAME" "$URL"
unzip -q "$ZIP_NAME"

# Copy essential files to assets directory
echo "Copying to $ASSETS_DIR..."
cp "sqlite-wasm-${SQLITE_VERSION}/jswasm/sqlite3.js" "$PROJECT_ROOT/$ASSETS_DIR/"
cp "sqlite-wasm-${SQLITE_VERSION}/jswasm/sqlite3.wasm" "$PROJECT_ROOT/$ASSETS_DIR/"
cp "sqlite-wasm-${SQLITE_VERSION}/jswasm/sqlite3-opfs-async-proxy.js" "$PROJECT_ROOT/$ASSETS_DIR/"

# Cleanup
cd - > /dev/null
rm -rf "$TMP_DIR"

echo "✅ SQLite WASM artifacts installed in $ASSETS_DIR"
echo "   - sqlite3.js"
echo "   - sqlite3.wasm"
echo "   - sqlite3-opfs-async-proxy.js"
