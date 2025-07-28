#!/bin/bash

echo "Starting BestMe in test mode (without AI features)..."

# Set environment variables
export RUST_LOG=warn,bestme=info
export BESTME_DISABLE_AI=true

# Run with core features only
echo "Building and running with core features..."
cd ui && npm install && cd ..
cargo tauri dev

echo "Test run complete."