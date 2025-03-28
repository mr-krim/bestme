#!/bin/bash
# Script to automatically refresh the application when voice command files change

# Check for inotify-tools (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if ! command -v inotifywait &> /dev/null; then
        echo "ERROR: inotify-tools not found. Please install it with:"
        echo "  sudo apt-get install inotify-tools  # For Debian/Ubuntu"
        echo "  sudo yum install inotify-tools      # For CentOS/RHEL"
        exit 1
    fi
fi

# Navigate to project root
cd "$(dirname "$0")/.." || { echo "Failed to navigate to project root"; exit 1; }

# Source files to monitor
TAURI_FILES="src-tauri/src/plugin/voice_commands.rs"
LIB_FILES="src/audio/voice_commands.rs"

echo "Starting voice command file watcher..."
echo "Watching for changes in voice command files..."
echo "Press Ctrl+C to stop."

# Start the app initially
./scripts/run_voice.sh &
APP_PID=$!

# Function to restart the app
restart_app() {
    echo "Restarting application..."
    if [[ -n "$APP_PID" ]]; then
        kill $APP_PID 2>/dev/null
        wait $APP_PID 2>/dev/null
    fi
    ./scripts/run_voice.sh &
    APP_PID=$!
}

# Watch for file changes
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux version using inotifywait
    while true; do
        inotifywait -q -e modify $TAURI_FILES $LIB_FILES
        restart_app
    done
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS version using fswatch
    if ! command -v fswatch &> /dev/null; then
        echo "fswatch not found. Please install it with:"
        echo "  brew install fswatch"
        exit 1
    fi
    
    fswatch -o $TAURI_FILES $LIB_FILES | while read -r _; do
        restart_app
    done
else
    echo "File watching not supported on this platform."
    echo "Press Ctrl+C to stop the application."
    wait $APP_PID
fi 
