#!/bin/bash
# BestMe Feature Testing Script

echo "BestMe Feature Testing"
echo "====================="

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Set library path
export LD_LIBRARY_PATH="/home/rd/bestme/target/release:$LD_LIBRARY_PATH"

echo -e "\n${YELLOW}1. Checking Application Status${NC}"
if pgrep -f bestme-tauri > /dev/null; then
    echo -e "${GREEN}✓ Application is running${NC}"
    PID=$(pgrep -f bestme-tauri)
    echo "  PID: $PID"
else
    echo -e "${RED}✗ Application is not running${NC}"
    echo "  Starting application..."
    ./target/release/bestme-tauri &
    sleep 3
fi

echo -e "\n${YELLOW}2. Checking Configuration${NC}"
CONFIG_FILE="$HOME/.config/bestme/config.json"
if [ -f "$CONFIG_FILE" ]; then
    echo -e "${GREEN}✓ Configuration file exists${NC}"
    echo "  Location: $CONFIG_FILE"
    # Show current settings
    echo "  Current Whisper model: $(jq -r '.audio.speech.model' "$CONFIG_FILE" 2>/dev/null || echo 'base.en')"
    echo "  Language: $(jq -r '.audio.speech.language' "$CONFIG_FILE" 2>/dev/null || echo 'en')"
else
    echo -e "${YELLOW}! Configuration file not found, will be created on first run${NC}"
fi

echo -e "\n${YELLOW}3. Checking Audio Devices${NC}"
# Check if we can access audio devices
if command -v arecord >/dev/null 2>&1; then
    DEVICES=$(arecord -l 2>/dev/null | grep "card" | wc -l)
    if [ "$DEVICES" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $DEVICES audio input device(s)${NC}"
        arecord -l | grep "card" | head -3
    else
        echo -e "${RED}✗ No audio input devices found${NC}"
    fi
else
    echo -e "${YELLOW}! arecord not found, skipping audio device check${NC}"
fi

echo -e "\n${YELLOW}4. Checking Model Directory${NC}"
MODEL_DIR="$HOME/.local/share/bestme/models"
if [ -d "$MODEL_DIR" ]; then
    echo -e "${GREEN}✓ Model directory exists${NC}"
    echo "  Location: $MODEL_DIR"
    
    # Check for Whisper models
    if [ -d "$MODEL_DIR/whisper" ]; then
        echo "  Whisper models:"
        ls -la "$MODEL_DIR/whisper" 2>/dev/null | grep ".bin" | awk '{print "    - " $9}'
    else
        echo -e "${YELLOW}  ! No Whisper models downloaded yet${NC}"
        echo "    Models will download automatically on first use"
    fi
    
    # Check for AI models
    if [ -d "$MODEL_DIR/ai" ]; then
        echo "  AI models:"
        ls -la "$MODEL_DIR/ai" 2>/dev/null | grep ".onnx" | awk '{print "    - " $9}'
    else
        echo -e "${YELLOW}  ! No AI models downloaded yet${NC}"
    fi
else
    echo -e "${YELLOW}! Model directory doesn't exist yet${NC}"
    echo "  Will be created at: $MODEL_DIR"
fi

echo -e "\n${YELLOW}5. Checking System Tray${NC}"
# This is harder to check programmatically, so we'll ask the user
echo "Please check your system tray (usually bottom-right or top-right)"
echo "Do you see a BestMe icon? (It might be a microphone or 'BM' icon)"

echo -e "\n${YELLOW}6. Testing Window Management${NC}"
echo "The app should have opened a window. Can you see it?"
echo "Try these actions:"
echo "  - Click the microphone button to start recording"
echo "  - Speak something and watch for transcription"
echo "  - Click settings to change options"
echo "  - Try minimize/maximize/close buttons"

echo -e "\n${YELLOW}7. Feature Test Checklist${NC}"
cat << EOF
Please manually test these features:

[ ] Main window opens
[ ] System tray icon appears
[ ] Click microphone to start recording
[ ] Audio visualization shows when speaking
[ ] Transcription appears after speaking
[ ] Stop recording works
[ ] Settings dialog opens
[ ] Theme switching works (light/dark)
[ ] Window can be minimized to tray
[ ] Application closes properly

Advanced Features (if models are downloaded):
[ ] AI text enhancement works
[ ] Voice commands respond
[ ] Multiple languages work
[ ] Export transcription works
EOF

echo -e "\n${YELLOW}8. Logs and Debugging${NC}"
echo "To see detailed logs, run:"
echo "  RUST_LOG=debug ./target/release/bestme-tauri"
echo ""
echo "Log file location: $HOME/.local/share/bestme/logs/"

echo -e "\n${GREEN}Testing Complete!${NC}"
echo "================================="
echo ""
echo "Next steps:"
echo "1. If everything works, take screenshots"
echo "2. Record a demo video showing transcription"
echo "3. Package the app with ./scripts/package-appimage.sh"
echo "4. Create a GitHub release"