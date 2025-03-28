# Windows 11 Testing Guide for BestMe

This guide outlines the steps for testing the BestMe application on Windows 11 following the Tauri 2.0 migration.

## Prerequisites

1. **Windows 11** operating system
2. **Rust and Cargo** installed (latest stable version recommended)
3. **Node.js and npm** for the frontend components
4. **Git** for version control
5. **Audio hardware** - A working microphone is required for audio capture testing

## System Dependencies

Install the following Windows-specific dependencies:

```powershell
# Install Visual Studio C++ build tools if not already installed
winget install Microsoft.VisualStudio.2022.BuildTools

# Install WebView2 Runtime (if not already installed)
winget install Microsoft.EdgeWebView2Runtime
```

## Setting Up the Development Environment

1. Clone the repository:
   ```bash
   git clone https://your-repo-url/bestme.git
   cd bestme
   ```

2. Build the application:
   ```bash
   cargo build
   ```

3. Run the application:
   ```bash
   cargo run
   ```

## Testing Protocol

### 1. Basic Application Launch

- [ ] Verify the application starts without errors
- [ ] Check that the console interface initializes correctly
- [ ] Verify configuration loading from the appropriate Windows paths

### 2. Audio Device Detection

- [ ] List audio devices and confirm your Windows microphone appears
- [ ] Verify the default audio device is correctly identified
- [ ] Test switching between multiple audio devices (if available)

### 3. Audio Recording

- [ ] Start audio capture with the default device
- [ ] Verify recording starts successfully
- [ ] Check that audio levels are detected
- [ ] Stop recording and verify graceful shutdown

### 4. Speech Recognition

- [ ] Configure Whisper settings
- [ ] Start recording and speak into the microphone
- [ ] Verify speech is transcribed correctly
- [ ] Test different languages if supported

### 5. Voice Commands

- [ ] Test basic voice commands
- [ ] Verify command detection accuracy
- [ ] Check that commands trigger the correct actions

### 6. UI Integration (if applicable)

- [ ] Verify the UI renders correctly
- [ ] Test UI interactions with the backend API
- [ ] Check responsive design across different window sizes

## Reporting Issues

Document any issues encountered during testing, including:

1. Exact steps to reproduce
2. Error messages or unexpected behavior
3. Windows version and hardware specs
4. Screenshots if applicable

## Windows-Specific Notes

- Audio permission requests may appear - grant necessary permissions
- Firewall alerts may appear if the application attempts network access
- The application should automatically use Windows-specific paths for configuration and data storage

## Success Criteria

The test is considered successful if:

1. The application builds and runs without errors
2. Audio devices are correctly detected and usable
3. Recording and transcription work as expected
4. No platform-specific crashes or issues occur

## Logging and Diagnostics

Run with enhanced logging for troubleshooting:

```bash
$env:RUST_LOG="debug"; cargo run
```

Review logs at: `%APPDATA%\bestme\logs\` 
