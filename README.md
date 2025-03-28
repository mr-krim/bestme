# BestMe - Voice-Enabled Transcription App

BestMe is a cross-platform desktop application for real-time audio transcription with voice command support. 
The app uses Whisper AI for highly accurate transcription and includes a voice command system for hands-free control.

## Features

- **Real-time transcription** using OpenAI's Whisper model (offline)
- **Voice command system** for hands-free control
- **Multi-language support** with over 30 languages
- **Translation capabilities** to convert non-English speech to English
- **Customizable settings** for transcription accuracy and performance

## Quick Start

### Linux/macOS

```bash
# Run with default settings (voice commands disabled)
./scripts/run_default.sh

# Run with voice commands enabled
./scripts/run_voice.sh

# Run in debug mode with verbose logging
./scripts/run_debug.sh
```

### Windows

```batch
# Run with default settings (voice commands disabled)
scripts\run_default.bat

# Run with voice commands enabled
scripts\run_voice.bat

# Run in debug mode with verbose logging
scripts\run_debug.bat
```

## Testing Voice Commands

### Linux/macOS

```bash
# Interactive test utility for voice commands
./scripts/test_voice_commands.sh
```

### Windows

```batch
# Interactive test utility for voice commands
scripts\test_voice_commands.bat
```

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v14 or later)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/installation)

### Development Scripts

For developers working on the voice command system, we provide helpful scripts:

- **scripts/refresh_voice.sh** - Automatically rebuilds when voice command files change (Linux/macOS only)
- **scripts/run_debug.sh/bat** - Runs with verbose logging for debugging

See [docs/SCRIPTS.md](./docs/SCRIPTS.md) for detailed information on all available scripts.

## Voice Commands

The following voice commands are currently supported:

- "BestMe, start recording" - Starts audio recording and transcription
- "BestMe, stop recording" - Stops recording
- "BestMe, save transcript" - Saves the current transcript to a file
- "BestMe, clear transcript" - Clears the current transcript
- "BestMe, switch to dark mode" - Toggles dark/light theme

Voice commands can be customized in the settings panel.

## Project Status

This project is actively under development. See [docs/implementation-status.md](./docs/implementation-status.md) for detailed progress information.

## Tauri 2.0 Migration

BestMe has been migrated to Tauri 2.0, bringing significant improvements and modernizations:

### Key Improvements

- **Simplified Architecture**: Cleaner code structure with removal of conditional compilation
- **Async Commands**: All plugin commands now use async/await for better performance
- **Improved Plugin System**: More consistent plugin initialization and state management
- **Enhanced Error Handling**: Better error context and propagation

### Migration Resources

- **Migration Summary**: See [docs/migration-summary.md](./docs/migration-summary.md) for migration details
- **Testing Plan**: See [docs/testing-plan.md](./docs/testing-plan.md) for validation approach
- **Optimization Tips**: See [docs/optimization-tips.md](./docs/optimization-tips.md) for performance recommendations

### Linux Development Setup

For Linux development, use the setup script to install required dependencies:

```bash
./scripts/setup-linux-deps.sh
```

## License

[MIT](LICENSE)

## Acknowledgments

- [Tauri](https://tauri.app/) for the cross-platform framework
- [Rust](https://www.rust-lang.org/) for the powerful and safe language
- [Svelte](https://svelte.dev/) for the reactive UI framework
- [Whisper](https://github.com/openai/whisper) for the speech recognition technology 

## Running the App with Voice Commands

To run BestMe with the voice command system:

1. Make sure you have Rust and Node.js installed
2. Install Tauri CLI: `cargo install tauri-cli`
3. Enable debug logging to see voice command detection:
   ```bash
   RUST_LOG=debug cargo tauri dev
   ```

### Testing Voice Commands

1. Start recording by clicking the microphone button
2. Enable voice commands with the toggle switch
3. Speak a command with the prefix (e.g., "computer, period")
4. Check the command history display to see detected commands
5. View real-time feedback in the notification that appears when commands are detected

### Troubleshooting

If you encounter issues with voice commands:

1. Check the console logs for error messages
2. Ensure your microphone is working correctly
3. Try simple commands first (like "computer, period")
4. Adjust the sensitivity in the voice command settings
5. See the debugging guide in `docs/debugging.md` for more details 
