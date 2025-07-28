#!/bin/bash
# Quick deploy to Windows for testing

echo "Deploying to Windows for testing..."

# Build the app
echo "Building release version..."
cargo tauri build

# Copy to shared folder
SHARED="/mnt/c/BestMe-Shared"
mkdir -p "$SHARED/latest-build"

# Copy executable and resources
cp -v target/release/bestme-tauri "$SHARED/latest-build/BestMe.exe" 2>/dev/null || \
cp -v src-tauri/target/release/bestme-tauri.exe "$SHARED/latest-build/BestMe.exe"

cp -r config "$SHARED/latest-build/"

# Create run script for Windows
cat > "$SHARED/latest-build/run-debug.bat" << 'BAT'
@echo off
echo Starting BestMe in debug mode...
set RUST_LOG=debug
BestMe.exe
pause
BAT

echo "✓ Deployed to: C:\BestMe-Shared\latest-build\"
echo "Run BestMe.exe on Windows to test!"
