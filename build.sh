#!/bin/bash

# RustFS Launcher Build Script
# Downloads required binary files for current platform before building

set -e

BINARIES_DIR="src-tauri/binaries"
TEMP_DIR="temp_downloads"

# Create directories
mkdir -p "$BINARIES_DIR"
mkdir -p "$TEMP_DIR"

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

echo "Detected platform: $OS $ARCH"
echo "Downloading RustFS binary for current platform..."

echo "Resolving latest RustFS version..."
LATEST_JSON=$(curl -fsSL "https://version.rustfs.com/latest.json")
RUSTFS_RELEASE_TAG=$(printf '%s' "$LATEST_JSON" | jq -r '.tag // .version // empty')

if [ -z "$RUSTFS_RELEASE_TAG" ] || [ "$RUSTFS_RELEASE_TAG" = "null" ]; then
    echo "✗ Error: Failed to resolve RustFS version from latest.json"
    exit 1
fi

RUSTFS_ASSET_VERSION="$RUSTFS_RELEASE_TAG"
case "$RUSTFS_ASSET_VERSION" in
    v*) ;;
    *) RUSTFS_ASSET_VERSION="v$RUSTFS_ASSET_VERSION" ;;
esac

echo "Latest RustFS version: $RUSTFS_RELEASE_TAG"

# Function to download and extract binary
download_binary() {
    local url=$1
    local filename=$2
    local target_name=$3
    
    echo "Downloading $filename..."
    
    if curl -fL --retry 3 --retry-delay 5 -H "Cache-Control: no-cache" -o "$TEMP_DIR/$filename.zip" "$url"; then
        echo "Extracting $filename..."
        unzip -o -q "$TEMP_DIR/$filename.zip" -d "$TEMP_DIR/$filename"
        
        # The extracted binary is named 'rustfs' or 'rustfs.exe'
        local extracted_binary=""
        if [ -f "$TEMP_DIR/$filename/rustfs.exe" ]; then
            extracted_binary="rustfs.exe"
        elif [ -f "$TEMP_DIR/$filename/rustfs" ]; then
            extracted_binary="rustfs"
        else
            echo "✗ Error: Binary not found in extracted files"
            ls -la "$TEMP_DIR/$filename/"
            exit 1
        fi
        
        cp "$TEMP_DIR/$filename/$extracted_binary" "$BINARIES_DIR/$target_name"
        chmod +x "$BINARIES_DIR/$target_name"
        echo "✓ $target_name downloaded and extracted successfully"
    else
        echo "✗ Error: Failed to download $filename"
        exit 1
    fi
}

# Determine which binary to download based on platform
case "$OS" in
    "Darwin")
        case "$ARCH" in
            "arm64")
                echo "Downloading for macOS Apple Silicon (aarch64)..."
                download_binary "https://github.com/rustfs/rustfs/releases/download/${RUSTFS_RELEASE_TAG}/rustfs-macos-aarch64-${RUSTFS_ASSET_VERSION}.zip" "rustfs-macos-aarch64" "rustfs-macos-aarch64"
                ;;
            "x86_64")
                echo "Downloading for macOS Intel (x86_64)..."
                download_binary "https://github.com/rustfs/rustfs/releases/download/${RUSTFS_RELEASE_TAG}/rustfs-macos-x86_64-${RUSTFS_ASSET_VERSION}.zip" "rustfs-macos-x86_64" "rustfs-macos-x86_64"
                ;;
            *)
                echo "✗ Error: Unsupported macOS architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    "Linux")
        echo "✗ Error: Linux is not a supported launcher target"
        echo "RustFS Launcher ships Windows and macOS installers only."
        echo "On Linux, run the RustFS binary from https://github.com/rustfs/rustfs/releases"
        exit 1
        ;;
    *)
        echo "✗ Error: Unsupported operating system: $OS"
        echo "Please use build.bat for Windows or download binaries manually"
        exit 1
        ;;
esac

# Clean up temporary files
echo "Cleaning up temporary files..."
rm -rf "$TEMP_DIR"

echo "Binary downloaded successfully for $OS $ARCH!"
echo "You can now run: cargo tauri build"