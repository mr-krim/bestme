#!/bin/bash
# Cross-compile BestMe for Windows from Linux

set -e

echo "Cross-compiling BestMe for Windows..."

# Check if mingw-w64 is installed
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "mingw-w64 is not installed. Please install it first:"
    echo "sudo apt-get install mingw-w64"
    exit 1
fi

# Add Windows target if not already added
rustup target add x86_64-pc-windows-gnu

# Set up environment for cross-compilation
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc

# Update dependencies
echo "Updating dependencies..."
cargo update

# Build frontend
echo "Building frontend..."
cd ui
npm install
npm run build
cd ..

# Cross-compile for Windows
echo "Cross-compiling for Windows x64..."
cd src-tauri
cargo build --release --target x86_64-pc-windows-gnu

echo "Build complete!"
echo "Binary location: src-tauri/target/x86_64-pc-windows-gnu/release/bestme-tauri.exe"

# Create a simple ZIP package
echo "Creating distribution package..."
cd ..
mkdir -p dist/windows-cross
cp src-tauri/target/x86_64-pc-windows-gnu/release/bestme-tauri.exe dist/windows-cross/BestMe.exe
cp -r ui/dist dist/windows-cross/
cp LICENSE.txt dist/windows-cross/

cd dist/windows-cross
zip -r ../BestMe-Windows-x64.zip *
cd ../..

echo "Distribution package created: dist/BestMe-Windows-x64.zip"
echo ""
echo "Note: This is a cross-compiled build. For a proper installer with:"
echo "- NSIS/MSI installer"
echo "- Code signing"
echo "- Auto-update support"
echo "Please build on a Windows machine using 'npm run tauri build'"