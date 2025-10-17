#!/usr/bin/env bash

set -e

# Create dist directory
mkdir -p dist

# Determine platform and architecture
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture names
case "$ARCH" in
    x86_64) ARCH="amd64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Set target for cross-compilation if needed
case "$PLATFORM-$ARCH" in
    linux-amd64) TARGET="x86_64-unknown-linux-gnu" ;;
    windows-amd64) TARGET="x86_64-pc-windows-msvc" ;;
    *) echo "Unsupported platform: $PLATFORM-$ARCH"; exit 1 ;;
esac

# Install the target if it's not already installed
echo "Installing target: $TARGET"
rustup target add "$TARGET"

# Build the binary
echo "Building for $PLATFORM-$ARCH (target: $TARGET)"
cargo build --release --target "$TARGET"

# Copy binary to dist directory with GitHub CLI extension naming convention
BINARY_NAME="gh-slot"
OUTPUT_NAME="gh-slot-${PLATFORM}-${ARCH}"

if [ "$PLATFORM" = "windows" ]; then
    BINARY_NAME="gh-slot.exe"
    OUTPUT_NAME="gh-slot-${PLATFORM}-${ARCH}.exe"
fi

cp "target/$TARGET/release/$BINARY_NAME" "dist/$OUTPUT_NAME"

echo "Binary built successfully: dist/$OUTPUT_NAME"
