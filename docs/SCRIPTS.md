# BestMe Launch Scripts

This directory contains scripts to help you run BestMe with different configurations.

## Installation Scripts

### Linux/macOS

#### scripts_install.sh
- Checks for required dependencies (Node.js, Rust, Cargo)
- Installs inotify-tools if needed and available
- Sets up script permissions automatically
- Usage: `./scripts_install.sh`

### Windows

#### scripts_install.bat
- Checks for required dependencies (Node.js, Rust, Cargo)
- Installs Tauri CLI and other development tools
- Sets up batch file attributes
- Usage: `scripts_install.bat`

## Available Scripts

### Linux/macOS Scripts

#### run_default.sh
- Runs BestMe with default settings
- Voice commands are disabled
- Standard info logging level
- Usage: `./run_default.sh`

#### run_voice.sh
- Runs BestMe with voice commands enabled
- Standard info logging level
- Usage: `./run_voice.sh`

#### run_debug.sh
- Runs BestMe in debug mode with verbose logging
- Voice commands are enabled by default (can be disabled by commenting the export line)
- Detailed debug and trace logging for voice commands and transcription
- Usage: `./run_debug.sh`

#### run_dev.sh
- Main development script (may be customized from original setup)
- Usage: `./run_dev.sh`

#### refresh_voice.sh
- Watches for changes in voice command related files and automatically rebuilds
- Requires inotify-tools to be installed
- Great for active development of voice command features
- Usage: `./refresh_voice.sh`

#### test_voice_commands.sh
- Interactive test utility for voice command functionality
- Provides options to run unit tests or simulate command detection
- Helpful for debugging voice command detection logic
- Usage: `./test_voice_commands.sh`

### Windows Scripts

#### run_default.bat
- Windows equivalent of run_default.sh
- Usage: `run_default.bat`

#### run_voice.bat
- Windows equivalent of run_voice.sh
- Usage: `run_voice.bat`

#### run_debug.bat
- Windows equivalent of run_debug.sh
- Usage: `run_debug.bat`

#### run_dev.bat
- Windows equivalent of run_dev.sh
- Usage: `run_dev.bat`

#### test_voice_commands.bat
- Windows equivalent of test_voice_commands.sh
- Usage: `test_voice_commands.bat`

## Setting Up Permissions

### Linux/macOS
Before running the scripts, you need to set executable permissions:

```bash
chmod +x run_default.sh run_voice.sh run_debug.sh run_dev.sh refresh_voice.sh test_voice_commands.sh
```

Or run the installation script which will set permissions automatically:

```bash
chmod +x scripts_install.sh
./scripts_install.sh
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

- For `refresh_voice.sh`, you need to install inotify-tools (Linux only):
  ```bash
  sudo apt-get install inotify-tools
  ```

## Troubleshooting

If the scripts fail to run with Tauri, they will automatically attempt to run using Cargo directly from the src-tauri directory.

Common issues:
1. Missing Node.js/npm - Install Node.js and npm
2. Missing Rust/Cargo - Install Rust and Cargo
3. Permission denied (Linux/macOS) - Run the chmod command above
4. For `refresh_voice.sh`: "Command not found: inotifywait" - Install inotify-tools

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
