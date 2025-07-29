#!/bin/bash

# Build script for macOS
set -e

echo "Building BestMe for macOS..."

# Check if we're on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    # Build the app
    cd ui
    npm run build
    cd ../src-tauri
    npm run tauri build
    
    # Sign the app if certificates are available
    if security find-identity -p codesigning &> /dev/null; then
        echo "Code signing certificates found. The build will be signed."
    else
        echo "No code signing certificates found. The app will be unsigned."
    fi
    
    echo "macOS build complete!"
    echo "App bundle: src-tauri/target/release/bundle/macos/BestMe.app"
    echo "DMG: src-tauri/target/release/bundle/dmg/"
else
    echo "This script must be run on macOS"
    echo "For cross-compilation from Linux, additional setup is required"
fi