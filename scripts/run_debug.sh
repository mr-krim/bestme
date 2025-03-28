#!/bin/bash
# Script to build and run BestMe in debug mode with verbose logging

# Enable voice commands
export ENABLE_VOICE_COMMANDS=true

# Set environment variables for verbose logging
export RUST_LOG=debug,voice_commands=trace,transcribe=trace

# Build and run in development mode
echo "Building and running BestMe in debug mode..."
echo "Logging level: $RUST_LOG"
echo "Voice commands: ${ENABLE_VOICE_COMMANDS:-disabled}"

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "Error: npm is not installed or not in the PATH"
    echo "Please install Node.js and npm before running this script"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed or not in the PATH"
    echo "Please install Rust and Cargo before running this script"
    exit 1
fi

# Navigate to project root
cd "$(dirname "$0")/.." || { echo "Failed to navigate to project root"; exit 1; }

# Install dependencies if needed
if [ ! -d "ui/node_modules" ]; then
    echo "Installing npm dependencies..."
    cd ui && npm install && cd ..
fi

# Run with tauri
echo "Starting Tauri development server in debug mode..."
cd ui && npm run tauri dev -- --features verbose

# If Tauri fails, try to run with cargo directly
if [ $? -ne 0 ]; then
    echo "Tauri run failed, trying with cargo directly..."
    cd ../src-tauri
    cargo run --features dev,verbose
fi
