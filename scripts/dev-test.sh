#!/bin/bash
# Development test script for BestMe

set -e

echo "BestMe Development Test Script"
echo "=============================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Kill any existing instances
echo -e "${YELLOW}Cleaning up existing instances...${NC}"
pkill -f bestme-tauri 2>/dev/null || true

# Set environment
export RUST_LOG=info
export RUST_BACKTRACE=1

# Set ONNX Runtime library path
export LD_LIBRARY_PATH="/home/rd/bestme/target/debug:$LD_LIBRARY_PATH"

# Change to project directory
cd /home/rd/bestme

# Check if debug binary exists
if [ -f "target/debug/bestme-tauri" ]; then
    echo -e "${GREEN}Debug binary found!${NC}"
    BINARY="target/debug/bestme-tauri"
else
    echo -e "${YELLOW}Debug binary not found, building...${NC}"
    cargo build -p bestme-tauri || {
        echo -e "${RED}Build failed!${NC}"
        exit 1
    }
    BINARY="target/debug/bestme-tauri"
fi

# Run tests
echo -e "\n${GREEN}Running BestMe Application Tests${NC}"
echo "=================================="

# Test 1: Basic startup
echo -e "\n${YELLOW}Test 1: Application Startup${NC}"
timeout 5s $BINARY 2>&1 | head -20 || {
    if [ $? -eq 124 ]; then
        echo -e "${GREEN}✓ Application started successfully (timed out after 5s as expected)${NC}"
    else
        echo -e "${RED}✗ Application failed to start${NC}"
        exit 1
    fi
}

# Test 2: Configuration
echo -e "\n${YELLOW}Test 2: Configuration Check${NC}"
CONFIG_FILE="$HOME/.config/bestme/config.json"
if [ -f "$CONFIG_FILE" ]; then
    echo -e "${GREEN}✓ Configuration file exists${NC}"
    echo "Location: $CONFIG_FILE"
else
    echo -e "${YELLOW}⚠ No configuration file found (will be created on first run)${NC}"
fi

# Test 3: Model directory
echo -e "\n${YELLOW}Test 3: Model Directory Check${NC}"
MODEL_DIR="$HOME/.local/share/bestme/models"
if [ -d "$MODEL_DIR" ]; then
    echo -e "${GREEN}✓ Model directory exists${NC}"
    echo "Contents:"
    ls -la "$MODEL_DIR" 2>/dev/null | head -5 || echo "  (empty)"
else
    echo -e "${YELLOW}⚠ Model directory not found (will be created when needed)${NC}"
fi

# Test 4: Audio devices
echo -e "\n${YELLOW}Test 4: Audio Device Detection${NC}"
$BINARY 2>&1 | grep -i "Found.*input device" | head -5 || {
    echo -e "${YELLOW}⚠ Could not verify audio device detection${NC}"
}

echo -e "\n${GREEN}Basic tests completed!${NC}"
echo "====================="

# Instructions
echo -e "\n${YELLOW}Next Steps:${NC}"
echo "1. Run the application: $BINARY"
echo "2. Test audio recording by clicking the microphone button"
echo "3. Test transcription by speaking"
echo "4. Test AI enhancement by selecting text and right-clicking"

# Optional: Run in dev mode
echo -e "\n${YELLOW}Run in development mode? (y/n)${NC}"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    echo -e "${GREEN}Starting BestMe in development mode...${NC}"
    exec $BINARY
fi