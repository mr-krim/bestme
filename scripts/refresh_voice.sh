#!/bin/bash
# Script to watch for changes and rebuild when voice command files are modified

# Default directories to watch
VOICE_COMMANDS_DIR="src-tauri/src/plugin/voice_commands.rs"
TRANSCRIBE_DIR="src-tauri/src/plugin/transcribe.rs"
CONFIG_FILES="src-tauri/src/config.rs"
MAIN_FILE="src-tauri/src/main.rs"

# Check if inotify-tools is installed
if ! command -v inotifywait &> /dev/null; then
    echo "Error: inotify-tools is not installed"
    echo "Please install it with: sudo apt-get install inotify-tools"
    exit 1
fi

# Function to restart application
restart_app() {
    echo "Changes detected, rebuilding..."
    
    # Kill any running instances of the app
    pkill -f "cargo run" || true
    pkill -f "npm run tauri" || true
    
    # Run the voice command script
    ./run_voice.sh &
    
    echo "Restart complete, watching for changes..."
}

# Initial start
restart_app

# Watch for changes
echo "Watching for changes in voice command files..."
echo "Press Ctrl+C to stop watching"

while true; do
    inotifywait -e modify -e create -e delete -e move \
        $VOICE_COMMANDS_DIR $TRANSCRIBE_DIR $CONFIG_FILES $MAIN_FILE
    
    # Wait a bit to avoid multiple restarts
    sleep 1
    
    restart_app
done 
