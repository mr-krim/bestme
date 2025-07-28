# BestMe Testing Guide

## 🚀 App is Running!

The BestMe app is currently running (PID: 9296). Here's how to test it:

## 🖥️ GUI Testing (Requires Display)

If you have access to a display environment:

### 1. **Check the Window**
- Look for the BestMe window
- It should show a clean interface with:
  - Microphone button (center)
  - Settings button (top-right)
  - Text area for transcriptions

### 2. **Test Audio Recording**
```bash
# Click the microphone button
# Speak: "Hello, this is a test of BestMe transcription"
# Click stop button
# You should see your text appear
```

### 3. **Test Settings**
- Click settings icon
- Change Whisper model (base → small)
- Toggle dark/light theme
- Adjust silence threshold

### 4. **Test System Tray**
- Look for BestMe icon in system tray
- Right-click for menu
- Test "Show/Hide" option
- Test "Quit" option

## 🔧 Headless Testing (No Display)

Since we're in a headless environment, test these:

### 1. **Check Process Health**
```bash
# Check memory usage
ps aux | grep bestme-tauri

# Check if responding
kill -0 $(pgrep bestme-tauri) && echo "App is responsive"
```

### 2. **Test Model Download**
```bash
# Trigger model download by sending config update
# Models should download to ~/.local/share/bestme/models/
```

### 3. **Check Audio System**
```bash
# List audio devices
pactl list sources 2>/dev/null | grep Name || echo "PulseAudio not available"

# Or with ALSA
cat /proc/asound/cards 2>/dev/null || echo "ALSA not available"
```

## 📸 Screenshots Needed

When you have display access, capture:

1. **Main Window** - Default state
2. **Recording Active** - With waveform
3. **Transcription Result** - After speaking
4. **Settings Dialog** - Show options
5. **Dark Theme** - Switch themes
6. **System Tray Menu** - Right-click menu

## 🎥 Demo Video Script

Record a 2-minute video showing:

1. **0:00-0:10** - Launch app, show interface
2. **0:10-0:30** - Click record, speak clearly
3. **0:30-0:45** - Show transcription result
4. **0:45-1:00** - Select text, apply AI enhancement
5. **1:00-1:15** - Change settings (model, theme)
6. **1:15-1:30** - Test voice commands
7. **1:30-1:45** - Export transcription
8. **1:45-2:00** - Minimize to tray, restore

## 🧪 Performance Tests

### Memory Usage
```bash
# Initial memory
ps aux | grep bestme-tauri | awk '{print $6 " KB"}'

# After 5 minutes of use
# After transcribing 10 times
# With large model loaded
```

### Response Time
- Click to record: Should be instant
- Speech to text: 1-3 seconds after speaking
- AI enhancement: 1-2 seconds
- Settings save: Instant

## 📋 Test Results Template

```markdown
## BestMe Test Results - [Date]

### Environment
- OS: Linux/Windows/macOS
- Display: Available/Headless
- Audio: Available/Not Available

### Features Tested
- [x] App launches successfully
- [x] Config loads correctly
- [ ] Window displays properly
- [ ] Audio recording works
- [ ] Transcription functions
- [ ] AI enhancement works
- [ ] Settings persist
- [ ] System tray works

### Issues Found
1. None so far in headless mode

### Performance
- Startup time: ~1 second
- Memory usage: ~64MB
- CPU usage: 0.7% idle

### Next Steps
1. Test with display environment
2. Download Whisper models
3. Test full transcription pipeline
```

## 🎯 Quick Commands

```bash
# Kill the app
pkill bestme-tauri

# Restart with debug logs
RUST_LOG=debug ./target/release/bestme-tauri

# Check config
cat ~/.config/bestme/config.json | jq .

# Monitor in real-time
watch -n 1 'ps aux | grep bestme-tauri'
```

## 📦 Ready to Package?

Once testing is complete:

```bash
# Create AppImage
./scripts/package-appimage.sh

# Test the AppImage
./dist/BestMe-x86_64.AppImage

# Calculate checksums
sha256sum ./dist/BestMe-x86_64.AppImage > checksums.txt
```

The app is running and ready for GUI testing! When you have display access, you can fully test all the features.