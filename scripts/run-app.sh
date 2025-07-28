#!/bin/bash
# Simple script to run BestMe with proper library path

set -e

# Set environment
export RUST_LOG=info
export RUST_BACKTRACE=1
export LD_LIBRARY_PATH="/home/rd/bestme/target/debug:$LD_LIBRARY_PATH"

# Change to project directory
cd /home/rd/bestme

echo "Starting BestMe application..."
echo "================================="
echo "Library path: $LD_LIBRARY_PATH"
echo ""

# Run the application
exec target/debug/bestme-tauri