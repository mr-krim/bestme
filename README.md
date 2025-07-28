# BestMe - AI-Powered Speech Transcription

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=flat&logo=tauri&logoColor=%23FFFFFF)](https://tauri.app/)

BestMe is a modern, cross-platform speech-to-text application with AI enhancement capabilities. Built with Rust and Tauri 2.0, it offers real-time transcription, voice commands, and intelligent text processing.

## 🌟 Features

- **Real-time Speech Transcription** - Powered by OpenAI's Whisper model
- **AI Text Enhancement** - Improve grammar, style, and clarity
- **Voice Commands** - Control your computer with voice
- **Cross-Platform** - Works on Windows, macOS, and Linux
- **GPU Acceleration** - CUDA and Metal support for faster processing
- **Multiple AI Providers** - Local ONNX models and cloud APIs (OpenAI, OpenRouter, Anthropic)
- **Privacy-First** - All transcription happens locally
- **System Tray Integration** - Runs in background
- **Custom Model Support** - Bring your own ONNX models

## Requirements

- Rust 1.70 or newer
- Cargo package manager
- Platform-specific dependencies (see below)

## Platform-Specific Setup

### Windows
- Windows 10 or newer
- Microsoft Visual C++ Build Tools
- Git for Windows

### macOS
- macOS 10.15 (Catalina) or newer
- Xcode Command Line Tools
- Homebrew (recommended for dependencies)

### Linux
- A modern Linux distribution (Ubuntu 20.04+, Fedora 36+, etc.)
- GCC or Clang
- X11 or Wayland development libraries
- PulseAudio or ALSA development libraries

## Building from Source

### Clone the repository
```bash
git clone https://github.com/your-organization/bestme.git
cd bestme
```

### Building and Running on Windows

```powershell
# Debug build and run
cargo run

# Or use the provided script
.\scripts\run-gui-mode.ps1

# Release build
cargo build --release
```

### Building and Running on macOS

```bash
# Install dependencies
brew install pkg-config

# Debug build and run
cargo run

# Release build
cargo build --release
```

### Building and Running on Linux

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install build-essential libgtk-3-dev libpulse-dev

# Install dependencies (Fedora)
sudo dnf install gcc-c++ gtk3-devel pulseaudio-libs-devel

# Debug build and run
cargo run

# Release build
cargo build --release
```

## Configuration

The application uses a configuration file located at:

- Windows: `%APPDATA%\bestme\BestMe\config\config.json`
- macOS: `~/Library/Application Support/bestme/BestMe/config/config.json`
- Linux: `~/.config/bestme/BestMe/config/config.json`

You can also place a `config.json` file in the `config` directory of the application or create a `settings.cfg` file in the application's root directory.

## AI Features

BestMe includes comprehensive AI capabilities for enhanced transcription accuracy and text processing. See the [AI Guide](docs/AI-GUIDE.md) for complete documentation.

### Key AI Capabilities

1. **Text Enhancement**: Automatic grammar correction, punctuation, and style improvements
2. **Multiple Providers**: Support for local (ONNX) and cloud providers (OpenRouter, OpenAI)
3. **Custom Models**: Import and manage your own ONNX models
4. **Intelligent Selection**: Automatic model selection based on text characteristics
5. **GPU Acceleration**: Support for CUDA, Metal, and DirectML

## Development Status

- Windows: Full native GUI implementation
- macOS: In progress
- Linux: In progress

## Tauri Integration

The application is in transition to using Tauri for cross-platform UI. The native UI implementations will remain available for platform-specific optimizations.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

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

### Documentation

- **[Development Guide](docs/DEVELOPMENT.md)** - Technical details and architecture
- **[AI Guide](docs/AI-GUIDE.md)** - Complete AI system documentation
- **[Installation Guide](docs/INSTALLATION.md)** - Platform-specific setup
- **[Testing Guide](docs/TESTING.md)** - Testing procedures
- **[Voice Commands](docs/VOICE_COMMANDS.md)** - Voice command reference
- **[Changelog](CHANGELOG.md)** - Version history and milestones

## Troubleshooting

If you encounter issues:

1. Check the console logs for error messages
2. Ensure your microphone is working correctly
3. Try simple commands first (like "computer, period")
4. Adjust the sensitivity in the voice command settings
5. See the [Development Guide](docs/DEVELOPMENT.md#debugging) for detailed debugging 
