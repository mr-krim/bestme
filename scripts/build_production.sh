#!/bin/bash

# Build script for BestMe production builds
set -e

echo "Building BestMe for production..."

# Update dependencies to latest versions
echo "1. Updating Rust dependencies..."
cargo update

echo "2. Updating UI dependencies..."
cd ui
npm update
cd ..

# Build the UI first
echo "3. Building UI..."
cd ui
npm run build
cd ..

# Build the Tauri app for production
echo "4. Building Tauri app..."
cd src-tauri
npm run tauri build

echo "Build complete! Binaries are in src-tauri/target/release/bundle/"