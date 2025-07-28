#!/bin/bash
echo "Testing BestMe application launch..."

# Kill any existing instances
pkill -f bestme-tauri 2>/dev/null

# Set environment variables
export RUST_LOG=info
export RUST_BACKTRACE=1

# Try to run the application
echo "Starting application..."
cd /home/rd/bestme

# First build the library with the fix
echo "Building library..."
cargo build -p bestme

# Then build the Tauri app
echo "Building Tauri app..."
cd src-tauri
cargo build

# Run the application
echo "Running application..."
cargo run