# BestMe Launch Scripts

This directory contains scripts to help you run BestMe with different configurations.

## Installation Scripts

### Linux/macOS

#### scripts/scripts_install.sh
- Checks for required dependencies (Node.js, Rust, Cargo)
- Installs inotify-tools if needed and available
- Sets up script permissions automatically
- Usage: `./scripts/scripts_install.sh`

### Windows

#### scripts/scripts_install.bat
- Checks for required dependencies (Node.js, Rust, Cargo)
- Installs Tauri CLI and other development tools
- Sets up batch file attributes
- Usage: `scripts/scripts_install.bat`

## Available Scripts

### Linux/macOS Scripts

#### scripts/run_default.sh
- Runs BestMe with default settings
- Voice commands are disabled
- Standard info logging level
- Usage: `./scripts/run_default.sh`

#### scripts/run_voice.sh
- Runs BestMe with voice commands enabled
- Standard info logging level
- Usage: `./scripts/run_voice.sh`

#### scripts/run_debug.sh
- Runs BestMe in debug mode with verbose logging
- Voice commands are enabled by default (can be disabled by commenting the export line)
- Detailed debug and trace logging for voice commands and transcription
- Usage: `./scripts/run_debug.sh`

#### scripts/run_dev.sh
- Main development script (may be customized from original setup)
- Usage: `./scripts/run_dev.sh`

#### scripts/refresh_voice.sh
- Watches for changes in voice command related files and automatically rebuilds
- Requires inotify-tools (Linux) or fswatch (macOS) to be installed
- Great for active development of voice command features
- Usage: `./scripts/refresh_voice.sh`

#### scripts/test_voice_commands.sh
- Interactive test utility for voice command functionality
- Provides options to run unit tests or simulate command detection
- Helpful for debugging voice command detection logic
- Usage: `./scripts/test_voice_commands.sh`

### Windows Scripts

#### scripts/run_default.bat
- Windows equivalent of run_default.sh
- Usage: `scripts/run_default.bat`

#### scripts/run_voice.bat
- Windows equivalent of run_voice.sh
- Usage: `scripts/run_voice.bat`

#### scripts/run_debug.bat
- Windows equivalent of run_debug.sh
- Usage: `scripts/run_debug.bat`

#### scripts/run_dev.bat
- Windows equivalent of run_dev.sh
- Usage: `scripts/run_dev.bat`

#### scripts/test_voice_commands.bat
- Windows equivalent of test_voice_commands.sh
- Usage: `scripts/test_voice_commands.bat`

## Setting Up Permissions

### Linux/macOS
Before running the scripts, you need to set executable permissions:

```bash
chmod +x scripts/run_default.sh scripts/run_voice.sh scripts/run_debug.sh scripts/run_dev.sh scripts/refresh_voice.sh scripts/test_voice_commands.sh
```

Or run the installation script which will set permissions automatically:

```bash
chmod +x scripts/scripts_install.sh
./scripts/scripts_install.sh
```

### Windows
Windows batch files (.bat) don't require special permissions to run. However, you may need to:

1. Right-click on the script and select "Run as administrator" if needed
2. If you receive security warnings, you may need to unblock the files:
   - Right-click on the script file
   - Select Properties
   - Check "Unblock" if it appears at the bottom of the General tab
   - Click Apply and OK

## Dependencies

- For `refresh_voice.sh`, you need to install:
  - On Linux: inotify-tools
    ```bash
    sudo apt-get install inotify-tools
    ```
  - On macOS: fswatch
    ```bash
    brew install fswatch
    ```

## Troubleshooting

If the scripts fail to run with Tauri, they will automatically attempt to run using Cargo directly from the src-tauri directory.

Common issues:
1. Missing Node.js/npm - Install Node.js and npm
2. Missing Rust/Cargo - Install Rust and Cargo
3. Permission denied (Linux/macOS) - Run the chmod command above
4. For `refresh_voice.sh`: 
   - Linux: "Command not found: inotifywait" - Install inotify-tools
   - macOS: "Command not found: fswatch" - Install fswatch

## Environment Variables

You can customize the behavior by setting environment variables before running:

### Linux/macOS
```bash
export RUST_LOG=debug     # Sets logging level
export ENABLE_VOICE_COMMANDS=true  # Enables voice commands
```

### Windows
```batch
set RUST_LOG=debug
set ENABLE_VOICE_COMMANDS=true
```
