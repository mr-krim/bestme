# Voice Command System Debugging Guide

This document provides guidance for debugging the voice command system in BestMe.

## Using Debug Scripts

For easier debugging, use the provided scripts in the `scripts/` directory:

1. **Debug Mode with Verbose Logging:**
   ```bash
   # Linux/macOS
   ./scripts/run_debug.sh
   
   # Windows
   scripts\run_debug.bat
   ```
   This sets RUST_LOG=debug and enables voice commands automatically.

2. **Development Mode with Auto-Refresh:**
   ```bash
   # Linux/macOS
   ./scripts/refresh_voice.sh
   ```
   This watches voice command files and automatically rebuilds when changes are detected.

3. **Interactive Testing:**
   ```bash
   # Linux/macOS
   ./scripts/test_voice_commands.sh
   
   # Windows
   scripts\test_voice_commands.bat
   ```
   Provides an interactive shell for testing voice command functionality.

## Configuration File Troubleshooting

BestMe uses two configuration files:

1. **~/.config/bestme/config.json**: The primary configuration file.
2. **settings.cfg**: A secondary configuration file in the application directory.

### Common Configuration Issues

1. **Missing Fields in config.json:**
   - The application requires specific fields to be present in the config.json file.
   - If you see errors like `missing field 'auto_punctuate'`, check your config.json.
   - All fields defined in the structs must be present in the config file.

2. **Invalid Voice Command Format:**
   - The `custom_commands` field must be a vector of tuples: `["command", {"Custom": "ActionName"}]`.
   - Using a different format will result in parsing errors.

3. **TOML Parsing Errors with settings.cfg:**
   - The settings.cfg file must be valid TOML.
   - Avoid using array-of-tables format (`[[audio.voice_commands.custom_commands]]`) as this is not properly handled.
   - Custom commands should be configured in config.json instead.

### Debugging Configuration Issues

Enable verbose logging to see configuration loading details:

```bash
RUST_LOG=info cargo run
```

Look for logs like:

```
[INFO bestme::config] Config directory: "/home/user/.config/bestme"
[INFO bestme::config] Loading existing configuration from: "/home/user/.config/bestme/config.json"
```

If you see errors like:

```
[ERROR bestme::config] Failed to parse configuration file: missing field `translate_to_english`
```

This means your config.json is missing required fields.

### Fix for Missing Configuration

If your config.json is missing or invalid, you can create a new one with all required fields:

```json
{
  "version": "0.1.0",
  "general": {
    "theme": "system",
    "auto_start": false,
    "minimize_to_tray": true
  },
  "audio": {
    "input_device": null,
    "input_volume": 1.0,
    "speech": {
      "model_size": "Small",
      "model_path": null,
      "language": "auto",
      "auto_punctuate": true,
      "translate_to_english": false,
      "context_formatting": true,
      "segment_duration": 3.0,
      "save_transcription": true,
      "output_format": "txt",
      "buffer_size": 3.0
    },
    "voice_commands": {
      "enabled": true,
      "command_prefix": "hey computer",
      "require_prefix": true,
      "sensitivity": 0.7,
      "custom_commands": [
        ["Open Settings", {"Custom": "OpenSettings"}],
        ["Toggle Recording", {"Custom": "ToggleRecording"}]
      ]
    }
  }
}
```

### Configuration Fixes Implemented

The following improvements have been made to enhance configuration handling:

1. **Enhanced Logging in ConfigManager:**
   - Added detailed logging for configuration loading process
   - Improved error messages with specific failure information
   - Added path reporting for file operations

2. **Fixed Settings Schema:**
   - Updated `settings.cfg` to be strictly TOML-compatible
   - Removed unsupported array-of-tables format from voice commands
   - Simplified custom commands configuration

3. **Robust Configuration Loading:**
   - Fixed `apply_settings_from_file` to properly navigate nested configuration
   - Added support for voice command settings in the configuration merger
   - Improved error handling for missing or invalid configuration files

4. **Default Configuration:**
   - Added a comprehensive default configuration in code
   - Ensured all required fields are included in the default configuration
   - Added example JSON configuration in documentation

These changes make the application more resilient to configuration issues and provide clearer error messages when problems do occur.

## Common Issues and Solutions

### 1. VoiceCommandConfig Mismatch

**Issue:** The `VoiceCommandConfig` structure in the plugin doesn't match the one in the library.

**Solution:**
- Check fields in both `bestme/src/audio/voice_commands.rs` and `bestme/src-tauri/src/plugin/voice_commands.rs`
- Ensure field names match (command_prefix vs prefix, sensitivity vs confidence)
- Use consistent field types (Option<String> vs String)

### 2. Tokio Integration

**Issue:** Missing tokio dependency or incorrect feature flags.

**Solution:**
- Verify tokio is in Cargo.toml with the "full" features
- Ensure async functions are properly marked with async/await
- Check for missing awaits on futures

### 3. Voice Command Process Transcription

**Issue:** String handling in `process_transcription` causes ownership issues.

**Solution:**
- Modify function to accept `&str` instead of String where appropriate
- Remove unnecessary `.to_string()` calls that cause moves
- Use cloning where appropriate to avoid ownership issues

### 4. Manager Trait Import

**Issue:** Missing Manager trait imports for Tauri.

**Solution:**
- Add `use tauri::Manager;` to files that use app.manage(), app.emit_all(), etc.
- Use proper methods for system tray: `tray_handle()` instead of `system_tray_handle()`
- Use `on_tray_event` instead of `on_system_tray_event`

### 5. Thread Safety Issues

**Issue:** CaptureManager and related structures don't implement Send/Sync.

**Solution:**
- Add `unsafe impl Send for AudioData {}` where needed
- Use tokio::sync::Mutex instead of std::sync::Mutex for async contexts 
- Ensure proper locking patterns to avoid deadlocks

## Debugging Steps

1. **Enable Verbose Logging:**
   ```rust
   RUST_LOG=debug cargo tauri dev
   ```
   Or use the debug script: `./scripts/run_debug.sh`

2. **Check Command Detection:**
   - Look for logs with "Processing transcription for commands" 
   - Verify "Voice command detected" messages appear when commands are spoken

3. **Test Command Execution:**
   - Enable one command at a time to isolate issues
   - Use simple commands like "computer, period" first
   - Check command history to see if commands are being detected

4. **Frontend Integration:**
   - Open browser dev tools to see console errors
   - Add console.log for command detection events
   - Verify invoke calls are properly structured

## Advanced Debugging

### Command History Debugging

To debug command history:
```javascript
// In browser console
await window.__TAURI__.invoke('plugin:voice_commands:get_command_history')
```

### Configuration Debugging

To debug voice command configuration:
```javascript
// In browser console
await window.__TAURI__.invoke('plugin:voice_commands:get_voice_command_config')
```

### Pattern Matching Debug

Add this to `process_transcription` to debug pattern matching:
```rust
debug!("Matching text: '{}' with pattern: '{}'", command_text, pattern);
debug!("Match confidence: {}", confidence);
```

## Log File Analysis

When using `run_debug.sh` or `run_debug.bat`, logs are more verbose. Key patterns to look for:

1. **Voice command initialization:**
   ```
   [DEBUG voice_commands] Initializing voice command system
   [DEBUG voice_commands] Voice command config loaded: { ... }
   ```

2. **Command detection:**
   ```
   [DEBUG voice_commands] Processing transcription: "computer, do something"
   [DEBUG voice_commands] Command detected: DoSomething (confidence: 0.92)
   ```

3. **Error patterns:**
   ```
   [ERROR voice_commands] Failed to process command
   [WARN voice_commands] Low confidence command detected (0.45): ignored
   ```

## Common Error Messages and Solutions

1. **"Cannot borrow as mutable"**
   - Check if you're trying to mutate an immutable reference
   - Add `mut` to variable declarations where needed

2. **"Value used after move"**
   - Clone values before moving them
   - Use references instead of ownership where possible

3. **"Method not found in type"**
   - Verify trait imports (use tauri::Manager)
   - Check if you're using the right type or version of an API

4. **"tokio runtime not found"**
   - Ensure the app is running with a tokio runtime
   - Check tokio dependency in Cargo.toml
