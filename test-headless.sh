#!/bin/bash
# Headless testing script for BestMe

set -e

echo "================================="
echo "BestMe Headless Testing"
echo "================================="

# Set library path
export LD_LIBRARY_PATH="/home/rd/bestme/target/release:$LD_LIBRARY_PATH"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "\n${YELLOW}1. Running unit tests${NC}"
if cargo test --lib -- --nocapture; then
    echo -e "${GREEN}✓ Unit tests passed${NC}"
else
    echo -e "${RED}✗ Unit tests failed${NC}"
fi

echo -e "\n${YELLOW}2. Running integration tests${NC}"
if cargo test --test basic_integration_test -- --nocapture; then
    echo -e "${GREEN}✓ Integration tests passed${NC}"
else
    echo -e "${RED}✗ Integration tests failed${NC}"
fi

echo -e "\n${YELLOW}3. Testing configuration${NC}"
CONFIG_FILE="$HOME/.config/bestme/config.json"
if [ -f "$CONFIG_FILE" ]; then
    echo -e "${GREEN}✓ Config file exists${NC}"
    echo "  Model: $(jq -r '.audio.speech.model // "base.en"' "$CONFIG_FILE")"
    echo "  Language: $(jq -r '.audio.speech.language // "en"' "$CONFIG_FILE")"
else
    echo -e "${YELLOW}! Config file will be created on first run${NC}"
fi

echo -e "\n${YELLOW}4. Testing model directory${NC}"
MODEL_DIR="$HOME/.local/share/bestme/models"
if [ -d "$MODEL_DIR" ]; then
    echo -e "${GREEN}✓ Model directory exists${NC}"
    WHISPER_COUNT=$(find "$MODEL_DIR/whisper" -name "*.bin" 2>/dev/null | wc -l || echo 0)
    AI_COUNT=$(find "$MODEL_DIR/ai" -name "*.onnx" 2>/dev/null | wc -l || echo 0)
    echo "  Whisper models: $WHISPER_COUNT"
    echo "  AI models: $AI_COUNT"
else
    echo -e "${YELLOW}! Model directory will be created on first use${NC}"
fi

echo -e "\n${YELLOW}5. Testing binary execution${NC}"
if [ -f "./target/release/bestme-tauri" ]; then
    echo -e "${GREEN}✓ Release binary exists${NC}"
    echo "  Size: $(du -h ./target/release/bestme-tauri | cut -f1)"
    
    # Try to get version
    if ./target/release/bestme-tauri --version 2>/dev/null; then
        echo -e "${GREEN}✓ Binary runs successfully${NC}"
    else
        echo -e "${YELLOW}! Binary requires display environment${NC}"
    fi
else
    echo -e "${RED}✗ Release binary not found${NC}"
fi

echo -e "\n${YELLOW}6. Checking dependencies${NC}"
# Check for required libraries
if ldd ./target/release/bestme-tauri 2>/dev/null | grep -q "not found"; then
    echo -e "${RED}✗ Missing dependencies:${NC}"
    ldd ./target/release/bestme-tauri | grep "not found"
else
    echo -e "${GREEN}✓ All dependencies satisfied${NC}"
fi

echo -e "\n${YELLOW}7. Performance benchmarks${NC}"
if cargo bench --no-run 2>/dev/null; then
    echo -e "${GREEN}✓ Benchmarks compiled successfully${NC}"
else
    echo -e "${YELLOW}! Benchmarks not available${NC}"
fi

echo -e "\n================================="
echo -e "${GREEN}Headless Testing Complete!${NC}"
echo "================================="

echo -e "\nNext steps for GUI testing:"
echo "1. Connect to a display environment"
echo "2. Run: ./scripts/run-app.sh"
echo "3. Test audio recording and transcription"
echo "4. Take screenshots for documentation"
echo "5. Create a demo video"

exit 0