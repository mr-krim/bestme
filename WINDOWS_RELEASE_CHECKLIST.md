# Windows Release Checklist for BestMe

## Pre-Release Requirements

### 1. Development Environment
- [ ] Windows 10/11 with latest updates
- [ ] Visual Studio 2022 with C++ build tools
- [ ] Node.js 20+ and npm
- [ ] Rust via rustup (stable channel)
- [ ] Git for Windows

### 2. Code Signing Certificate (Optional but Recommended)
- [ ] Obtain EV Code Signing Certificate
- [ ] Install certificate in Windows Certificate Store
- [ ] Update `certificateThumbprint` in tauri.conf.json

## Build Process

### 1. Prepare Release
```bash
# Update version numbers
./scripts/prepare_windows_release.sh

# Or manually:
# 1. Update version in Cargo.toml
# 2. Update version in src-tauri/Cargo.toml
# 3. Update version in ui/package.json
# 4. Update version in src-tauri/tauri.conf.json
```

### 2. Run Tests
```powershell
# Run Rust tests
cargo test

# Run UI tests
cd ui
npm test
```

### 3. Build Production Release
```powershell
cd ui
npm run tauri build
```

### 4. Output Files
After successful build, find installers in:
- `src-tauri/target/release/bundle/msi/` - MSI installer
- `src-tauri/target/release/bundle/nsis/` - NSIS installer (exe)

## Testing Checklist

### 1. Installation Testing
- [ ] Test on clean Windows 10 machine
- [ ] Test on clean Windows 11 machine
- [ ] Verify WebView2 auto-installation works
- [ ] Check UAC prompts are appropriate
- [ ] Verify installation directory is correct
- [ ] Check Start Menu shortcuts created
- [ ] Check Desktop shortcut created (if enabled)

### 2. Functionality Testing
- [ ] Application launches successfully
- [ ] System tray icon appears and works
- [ ] Audio device detection works
- [ ] Microphone permissions requested properly
- [ ] Transcription functionality works
- [ ] Voice commands work
- [ ] AI features work (both local and cloud)
- [ ] Settings are saved and persist
- [ ] Text injection works in other applications

### 3. Security Testing
- [ ] Windows Defender doesn't flag the app
- [ ] SmartScreen doesn't block installation
- [ ] Verify code signature is valid (if signed)

### 4. Performance Testing
- [ ] CPU usage is reasonable during idle
- [ ] Memory usage is acceptable
- [ ] App starts within 3 seconds
- [ ] No memory leaks during extended use

## Distribution

### 1. GitHub Release
```bash
# Tag the release
git tag -a v1.0.0-windows -m "Windows Release v1.0.0"
git push origin v1.0.0-windows

# Upload to GitHub Releases:
# - BestMe_1.0.0_x64_en-US.msi
# - BestMe_1.0.0_x64-setup.exe
```

### 2. Update Website
- [ ] Upload installer to download server
- [ ] Update download links
- [ ] Update installation instructions
- [ ] Add Windows-specific FAQ items

### 3. Windows Store (Optional)
- [ ] Create MSIX package
- [ ] Submit to Microsoft Store
- [ ] Wait for certification

## Post-Release

### 1. Monitor
- [ ] Check for crash reports
- [ ] Monitor user feedback
- [ ] Track download statistics

### 2. Known Issues to Document
- WebView2 requirement (auto-installed)
- Microphone permissions in Windows Settings
- Antivirus false positives (if any)

## Troubleshooting Common Issues

### Build Failures
1. **Missing Visual Studio Tools**
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```

2. **Rust Target Missing**
   ```powershell
   rustup target add x86_64-pc-windows-msvc
   ```

### Code Signing Issues
1. **Certificate Not Found**
   - Run `certutil -store My` to list certificates
   - Copy the thumbprint exactly as shown

2. **Timestamp Server Timeout**
   - Try alternate timestamp servers:
     - http://timestamp.digicert.com
     - http://timestamp.comodoca.com

### Runtime Issues
1. **WebView2 Not Found**
   - The installer should download it automatically
   - Manual install: https://go.microsoft.com/fwlink/p/?LinkId=2124703

2. **Audio Device Access Denied**
   - Check Windows Privacy Settings > Microphone
   - Ensure app has permission

## Windows-Specific Features to Highlight

1. **System Integration**
   - Jump list support (right-click on taskbar)
   - Native notifications
   - System tray integration

2. **Performance**
   - Hardware-accelerated audio processing
   - GPU acceleration for AI models (if NVIDIA GPU present)
   - Efficient memory management

3. **Security**
   - Local-only audio processing
   - Encrypted settings storage
   - No telemetry without consent