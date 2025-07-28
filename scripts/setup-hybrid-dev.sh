#!/bin/bash
# Setup hybrid development workflow between WSL and Windows

echo "Setting up WSL-Windows hybrid development..."

# Create shared directory for easy file transfer
SHARED_DIR="/mnt/c/BestMe-Shared"
mkdir -p "$SHARED_DIR"

# Create quick deploy script
cat > scripts/deploy-to-windows.sh << 'EOF'
#!/bin/bash
# Quick deploy to Windows for testing

echo "Deploying to Windows for testing..."

# Build the app
echo "Building release version..."
cargo tauri build

# Copy to shared folder
SHARED="/mnt/c/BestMe-Shared"
mkdir -p "$SHARED/latest-build"

# Copy executable and resources
cp -v target/release/bestme-tauri "$SHARED/latest-build/BestMe.exe" 2>/dev/null || \
cp -v src-tauri/target/release/bestme-tauri.exe "$SHARED/latest-build/BestMe.exe"

cp -r config "$SHARED/latest-build/"

# Create run script for Windows
cat > "$SHARED/latest-build/run-debug.bat" << 'BAT'
@echo off
echo Starting BestMe in debug mode...
set RUST_LOG=debug
BestMe.exe
pause
BAT

echo "✓ Deployed to: C:\BestMe-Shared\latest-build\"
echo "Run BestMe.exe on Windows to test!"
EOF

chmod +x scripts/deploy-to-windows.sh

# Create feedback collection script
cat > scripts/collect-feedback.sh << 'EOF'
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
EOF

chmod +x scripts/collect-feedback.sh

echo "✅ Hybrid development setup complete!"
echo ""
echo "Workflow:"
echo "1. Develop in WSL: cargo tauri dev"
echo "2. Deploy to Windows: ./scripts/deploy-to-windows.sh"
echo "3. Test on Windows: C:\BestMe-Shared\latest-build\BestMe.exe"
echo "4. Collect feedback: ./scripts/collect-feedback.sh"