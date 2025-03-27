#!/bin/bash
# Script to build and run BestMe with debug logging enabled

# Set environment variables for logging
export RUST_LOG=debug

# Build and run in development mode
echo "Building and running BestMe with debug logging..."
echo "Logging level: $RUST_LOG"

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

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing npm dependencies..."
    npm install
fi

# Run with tauri
echo "Starting Tauri development server..."
npm run tauri dev

# If Tauri fails, try to run with cargo directly
if [ $? -ne 0 ]; then
    echo "Tauri run failed, trying with cargo directly..."
    cd src-tauri
    cargo run --features dev
fi 
