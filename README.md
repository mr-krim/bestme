# BestMe - Modern Speech-to-Text Application

BestMe is a cross-platform speech-to-text application powered by AI, designed to work on Windows, macOS, and Linux.

## Features

- Real-time speech transcription
- Multiple language support
- Voice command capabilities
- System tray integration
- Configurable speech recognition settings

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

### Troubleshooting

If you encounter issues with voice commands:

1. Check the console logs for error messages
2. Ensure your microphone is working correctly
3. Try simple commands first (like "computer, period")
4. Adjust the sensitivity in the voice command settings
5. See the debugging guide in `docs/debugging.md` for more details 
