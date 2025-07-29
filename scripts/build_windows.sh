#!/bin/bash

# Build script for Windows target
set -e

echo "Building BestMe for Windows..."

# Check if we're on Linux and can cross-compile
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Setting up Windows cross-compilation..."
    
    # Install Windows target if not already installed
    rustup target add x86_64-pc-windows-gnu
    
    # Build for Windows
    cd ui
    npm run build
    cd ../src-tauri
    
    # Cross-compile for Windows
    cargo build --release --target x86_64-pc-windows-gnu
    
    echo "Windows build complete!"
    echo "Binary: src-tauri/target/x86_64-pc-windows-gnu/release/bestme-tauri.exe"
else
    echo "For cross-compilation, run this on Linux with mingw-w64 installed"
    echo "On Windows, use 'npm run tauri build' directly"
fi