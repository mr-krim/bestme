#!/bin/bash

# Build script for Linux distributions
set -e

echo "Building BestMe for Linux..."

# Build the app
cd ui
npm run build
cd ../src-tauri
npm run tauri build

# Create AppImage if available
if command -v appimagetool &> /dev/null; then
    echo "Creating AppImage..."
    npm run tauri build -- --bundles appimage
fi

# Create Snap package if snapcraft is available
if command -v snapcraft &> /dev/null; then
    echo "Creating Snap package..."
    npm run tauri build -- --bundles snap
fi

echo "Linux builds complete!"
echo "Binaries are in src-tauri/target/release/bundle/"
ls -la src-tauri/target/release/bundle/