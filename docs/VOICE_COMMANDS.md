# Voice Command System

The BestMe application includes a powerful voice command system that allows users to control the application using natural speech commands. This document explains how to use voice commands, how they work, and how to customize them.

## Voice Command Basics

Voice commands allow you to perform actions within the BestMe application by simply speaking commands. The system is designed to:

1. Listen to your transcribed speech in real-time
2. Detect specific command phrases
3. Execute corresponding actions 

## Enabling Voice Commands

To enable voice commands:

1. Open the BestMe application
2. Go to the Settings tab
3. Enable the "Voice Commands" toggle
4. Configure your preferred command prefix (default is "computer")
5. Choose whether to require the prefix for all commands

## Available Commands

The following voice commands are supported by default:

| Command | Example Phrase | Action |
|---------|---------------|--------|
| Delete | "delete that" or "delete last 3 words" | Removes the last few words from the transcription |
| Undo | "undo that" | Reverts the last change |
| Redo | "redo that" | Redoes the last undone change |
| Capitalize | "capitalize that" | Capitalizes the last word |
| Lowercase | "make that lowercase" | Converts the last word to lowercase |
| New Line | "new line" | Adds a line break |
| New Paragraph | "new paragraph" | Adds a paragraph break |
| Period | "period" | Adds a period |
| Comma | "comma" | Adds a comma |
| Question Mark | "question mark" | Adds a question mark |
| Exclamation Mark | "exclamation mark" | Adds an exclamation mark |
| Pause | "pause recording" | Pauses the recording |
| Resume | "resume recording" | Resumes the recording |
| Stop | "stop recording" | Stops the recording |

## Command Prefix

By default, commands require a prefix word to distinguish them from normal speech. The default prefix is "computer", so you would say "computer delete that" to use the delete command.

You can:
- Change the prefix to any word you prefer
- Disable the prefix requirement if you want to use commands without saying the prefix first

## Voice Command History

BestMe keeps a history of the commands you've issued, which can be useful for:
- Reviewing what commands have been executed
- Debugging if a command didn't work as expected
- Understanding patterns in your voice command usage

To view command history:
1. Click the "Show History" button in the transcription area
2. The command history panel will display all recent commands with their:
   - Command type
   - Trigger text (what you said)
   - Timestamp

## How Voice Commands Work

Behind the scenes, the voice command system:

1. Continuously analyzes your transcribed speech
2. Applies pattern matching to detect command phrases
3. Extracts any parameters (like "delete last 3 words" where "3" is a parameter)
4. Executes the associated action
5. Provides visual feedback when a command is detected

## Troubleshooting

If voice commands aren't working as expected:

1. **Check if voice commands are enabled** - Ensure the feature is turned on in Settings
2. **Verify your microphone** - Make sure your microphone is working properly
3. **Try using the prefix** - If you're not using a prefix, try adding "computer" before your command
4. **Speak clearly** - Enunciate your commands clearly for better recognition
5. **Check command history** - Look at the command history to see if commands are being detected
6. **Review logs** - Advanced users can check application logs for detailed information

## Advanced Configuration

Advanced users can modify the source code to:
- Add custom commands
- Change command detection patterns
- Modify command behaviors
- Adjust sensitivity settings

The voice command system is designed to be extensible for future enhancements.

## Future Enhancements

Planned improvements to the voice command system include:
- Custom user-defined commands
- Contextual commands based on application state
- Enhanced natural language understanding
- Support for command sequences and macros 

## Tauri 2.0 Migration Guide

### Overview
The BestMe voice command system is being migrated to Tauri 2.0, which includes several improvements to state management and command handling. This guide outlines the key changes and migration steps.

### State Management Changes

#### Current Approach (Tauri 1.x)
```rust
// State definition with generic parameters
pub struct VoiceCommandState<R: Runtime = tauri_runtime_wry::Wry<tauri::Wry>> {
    manager: Arc<Mutex<Option<TauriVoiceCommandManager>>>,
    transcribe_state: Option<Arc<TranscribeState<R>>>,
    // ...other fields
}

// State initialization
app.manage(Arc::new(Mutex::new(VoiceCommandState::new())));

// State access in commands
#[tauri::command]
fn get_last_command<R: Runtime>(
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState<R>>>>
) -> Option<CommandData> {
    let state = state.inner().lock();
    state.get_last_command_data()
}
```

#### New Approach (Tauri 2.0)
```rust
// Simplified state without generic parameters
pub struct VoiceCommandState {
    manager: Option<TauriVoiceCommandManager>,
    app_handle: Option<AppHandle>,
    // ...other fields
}

// Direct state management
app.manage(VoiceCommandState::new());

// Simplified state access in commands
#[tauri::command]
async fn get_last_command(
    state: State<'_, VoiceCommandState>
) -> Option<CommandData> {
    state.get_last_command_data()
}
```

### Command Handler Migration

#### Current Handlers
```rust
#[tauri::command]
fn toggle_voice_commands<R: Runtime>(
    enabled: bool,
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState<R>>>>
) -> Result<(), String> {
    let mut state = state.inner().lock();
    
    if enabled {
        state.enable()
    } else {
        state.disable()
    }
}
```

#### Tauri 2.0 Handlers
```rust
#[tauri::command]
async fn toggle_voice_commands(
    enabled: bool,
    state: State<'_, VoiceCommandState>
) -> Result<(), String> {
    if enabled {
        state.enable().await
    } else {
        state.disable().await
    }
}
```

### Event System Migration

#### Current Approach
```rust
// Using tokio channels
let (sender, receiver) = mpsc::channel(100);

// Spawning tasks to process events
tokio::spawn(async move {
    while let Some(event) = receiver.recv().await {
        // Process event
    }
});

// Sending events
let _ = sender.send(VoiceCommandEvent::CommandDetected(command)).await;
```

#### Tauri 2.0 Approach
```rust
// Using Tauri's built-in event system
app.listen_global("voice-command", move |event| {
    // Process event
});

// Emitting events
app.emit_all("voice-command", 
    CommandDetected { 
        command_type: "Delete".to_string(),
        text: "delete last word".to_string()
    }
).unwrap();
```

### Text Editing Commands Interface

Tauri 2.0 introduces a cleaner interface for text editing commands:

```rust
// Delete operation
#[tauri::command]
async fn delete_text(
    scope: String,
    state: State<'_, VoiceCommandState>
) -> Result<String, String> {
    state.apply_delete_operation(&scope).await
}

// Undo operation
#[tauri::command]
async fn undo(
    state: State<'_, VoiceCommandState>
) -> Result<String, String> {
    state.undo().await
}

// Redo operation
#[tauri::command]
async fn redo(
    state: State<'_, VoiceCommandState>
) -> Result<String, String> {
    state.redo().await
}
```

### Frontend Migration

#### Current Frontend Code
```javascript
// Invoking command handlers
const result = await invoke('plugin:voice_commands:get_last_command');

// Event listeners
await listen('plugin:voice_commands/command-detected', (event) => {
  // Handle command
});
```

#### Tauri 2.0 Frontend Code
```javascript
// Simplified command invocation
const result = await invoke('get_last_command');

// Event listeners with typed payload
await listen('voice-command', (event: VoiceCommandEvent) => {
  // Handle command with typed data
});
```

### Migration Steps

1. Update state structures to remove generic parameters
2. Convert Arc<Mutex<>> wrappers to direct state management
3. Update command handlers to use the new state access pattern
4. Migrate event handling to use Tauri's built-in event system
5. Update frontend code to use the new API
6. Update the plugin registration to use Tauri 2.0 patterns

### Testing

When migrating voice commands to Tauri 2.0, use these testing commands:

```bash
# Test basic command detection
./scripts/test_voice_commands.sh

# Test with Tauri 2.0 syntax
cargo test --features="tauri-2" --package bestme --lib plugin::voice_commands::tests::test_tauri2_voice_command_detection
``` 
