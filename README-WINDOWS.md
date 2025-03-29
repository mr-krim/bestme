# BestMe - Windows 11 Testing

## Overview

This document provides an overview of the Windows 11 testing process for the BestMe application after the Tauri 2.0 migration. For detailed testing steps, please refer to the [Windows Testing Guide](docs/windows-testing-guide.md).

## Quick Start

1. **Clone this repository**:
   ```
   git clone https://your-repo-url/bestme.git
   cd bestme
   ```

2. **Install Dependencies**:
   Make sure you have:
   - Rust and Cargo (latest stable)
   - Visual Studio Build Tools 
   - WebView2 Runtime

3. **Build and Run**:
   ```powershell
   # Using the build script
   .\scripts\build-windows.ps1
   
   # Or manually
   cargo run
   ```

## Running in GUI Mode

To run the application in GUI mode:

1. **Using the included scripts**:
   ```powershell
   # Using the batch file
   .\run-gui-mode.bat
   
   # Using PowerShell
   .\run-gui-mode.ps1
   ```

2. **Or with command-line parameters**:
   ```powershell
   # Basic GUI mode
   .\bestme.exe --gui
   
   # With verbose logging
   .\bestme.exe --gui --verbose
   ```

## Windows-Specific Features

- Enhanced audio device detection for Windows systems
- Native Windows path handling for configuration files
- Windows Media API integration for audio capture
- Windows notification system integration

## What We're Testing

1. **Core Migration Verification**:
   - Confirm that Tauri 2.0 initializes correctly on Windows
   - Verify all plugins load properly
   - Check for any Windows-specific compatibility issues

2. **Audio Functionality**:
   - Audio device detection and selection
   - Audio capture with correct sample rates
   - Transcription accuracy with Windows audio devices

3. **UI/UX Integration**:
   - Windows-native UI elements
   - Proper rendering of application window
   - Responsive design on various screen sizes

## Reporting Issues

When reporting Windows-specific issues:
- Use the issue template in `docs/ISSUE_TEMPLATE.md`
- Include "Windows 11" in the issue title
- Attach Windows Event Viewer logs if relevant

## Known Windows-Specific Limitations

- Audio device permissions may need to be granted manually
- First-time launches may be slower due to WebView2 initialization
- Windows Defender may ask for confirmation to run the application

## Building for Distribution

For packaging a Windows release build:

```powershell
# Use Tauri CLI to build a bundled Windows application
cargo tauri build
```

Output will be located in `target\release\bundle\`.

## Troubleshooting GUI Mode

If the application's GUI window doesn't appear or closes immediately:

1. Run with verbose logging: `.\bestme.exe --gui --verbose`
2. Check the logs in: `%APPDATA%\bestme\logs\`
3. Try running the application as administrator
4. Make sure your system meets the minimum requirements
5. Verify that WebView2 is properly installed

## Contact

For urgent Windows-specific issues during testing, please contact:
- Your Name: your.email@example.com 
