#!/bin/bash
# Collect feedback from Windows testing

FEEDBACK_DIR="feedback/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$FEEDBACK_DIR"

echo "Collecting feedback from Windows testing..."

# Check for screenshots
SCREENSHOT_DIR="/mnt/c/BestMe-Shared/screenshots"
if [ -d "$SCREENSHOT_DIR" ]; then
    cp -r "$SCREENSHOT_DIR"/* "$FEEDBACK_DIR/" 2>/dev/null
    echo "✓ Collected screenshots"
fi

# Check for logs
LOG_FILE="/mnt/c/Users/$USER/AppData/Roaming/bestme/logs/bestme.log"
if [ -f "$LOG_FILE" ]; then
    cp "$LOG_FILE" "$FEEDBACK_DIR/windows-debug.log"
    echo "✓ Collected debug logs"
fi

# Create feedback report template
cat > "$FEEDBACK_DIR/feedback.md" << 'MD'
# Windows Testing Feedback

Date: $(date)

## Issues Found

1. **Issue**: 
   - **Steps to reproduce**: 
   - **Expected**: 
   - **Actual**: 
   - **Screenshot**: 

## Working Features
- [ ] Application starts
- [ ] Audio recording works
- [ ] Transcription appears
- [ ] Settings save
- [ ] System tray works

## Performance
- Startup time: 
- Memory usage: 
- Transcription speed: 

## Notes

MD

echo "✓ Created feedback template: $FEEDBACK_DIR/feedback.md"
echo "Edit this file with your testing results!"
