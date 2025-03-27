# Voice Command System Debugging Guide

This document provides guidance for debugging the voice command system in BestMe.

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
