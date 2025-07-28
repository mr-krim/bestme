# BestMe Development Guide

This guide contains comprehensive technical information for developers working on BestMe.

## Table of Contents

1. [Project Architecture](#project-architecture)
2. [Development Setup](#development-setup)
3. [Building & Testing](#building--testing)
4. [Key Components](#key-components)
5. [AI System](#ai-system)
6. [Voice Commands](#voice-commands)
7. [Platform-Specific Notes](#platform-specific-notes)
8. [Performance & Optimization](#performance--optimization)
9. [Debugging](#debugging)
10. [Release Process](#release-process)

## Project Architecture

### Technology Stack

- **Backend**: Rust with Tauri 2.0
- **Frontend**: Svelte with TypeScript
- **Audio**: Whisper AI for transcription
- **AI**: ONNX Runtime for local inference, multiple cloud providers
- **GPU**: CUDA, Metal, DirectML support

### Project Structure

```
bestme/
├── src/                    # Core Rust library
│   ├── audio/             # Audio capture and processing
│   ├── ai/                # AI integration (local/cloud)
│   ├── config/            # Configuration management
│   ├── gui/               # Native GUI implementations
│   └── lib.rs             # Library entry point
├── src-tauri/             # Tauri application
│   ├── src/
│   │   ├── main.rs        # Application entry
│   │   └── plugin/        # Custom Tauri plugins
│   └── tauri.conf.json    # Tauri configuration
├── ui/                    # Svelte frontend
│   ├── src/
│   │   ├── App.svelte     # Main component
│   │   ├── components/    # UI components
│   │   └── lib/          # Utilities
│   └── package.json      # Frontend dependencies
├── scripts/              # Development scripts
└── docs/                # Documentation
```

## Development Setup

### Prerequisites

1. **Rust**: 1.70+ (install via [rustup](https://rustup.rs/))
2. **Node.js**: 18+ (for frontend)
3. **Platform Tools**:
   - Windows: Visual Studio Build Tools
   - macOS: Xcode Command Line Tools
   - Linux: gcc, pkg-config, webkit2gtk-4.0

### Initial Setup

```bash
# Clone repository
git clone https://github.com/your-org/bestme.git
cd bestme

# Install Rust dependencies
cargo build

# Install frontend dependencies
cd ui
npm install
cd ..

# Install Tauri CLI
cargo install tauri-cli
```

### Running Development Mode

```bash
# Run with hot-reload
cargo tauri dev

# With debug logging
RUST_LOG=debug cargo tauri dev

# Run specific features
cargo tauri dev -- --features "gpu_acceleration"
```

## Building & Testing

### Testing Strategy

#### Unit Tests
```bash
# Run all Rust tests
cargo test

# Run specific module tests
cargo test audio::
cargo test ai::

# Run with output
cargo test -- --nocapture
```

#### Integration Tests
```bash
# Run integration tests
cargo test --test '*' --features integration-tests

# UI tests
cd ui && npm test
```

#### Coverage
```bash
# Generate coverage report
cargo tarpaulin --out Html

# Check coverage percentage
cargo tarpaulin --print-summary
```

### Building for Production

```bash
# Build optimized binary
cargo tauri build

# Platform-specific builds
cargo tauri build --target x86_64-pc-windows-msvc
cargo tauri build --target x86_64-apple-darwin
cargo tauri build --target x86_64-unknown-linux-gnu
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy -- -D warnings

# Fix common issues
cargo fix --lib -p bestme

# Check for security issues
cargo audit
```

## Key Components

### Audio System

The audio system handles capture and transcription:

```rust
// Core components
src/audio/
├── capture.rs          # Audio device management
├── transcribe.rs       # Whisper integration
├── streaming_transcribe.rs # Real-time processing
├── voice_commands.rs   # Command detection
└── ai_voice_commands.rs # AI-enhanced commands
```

Key features:
- Real-time audio capture with VAD
- GPU-accelerated Whisper transcription
- Streaming pipeline with <300ms latency
- Multi-device support

### AI System

See [AI-GUIDE.md](./AI-GUIDE.md) for comprehensive AI documentation.

Key modules:
- **Local inference**: ONNX Runtime with GPU support
- **Cloud providers**: OpenRouter, OpenAI, Requesty
- **Model management**: Download, update, custom models
- **Performance**: Caching, quantization, batch processing

### Configuration System

```rust
// Configuration structure
pub struct Config {
    pub ui: UIConfig,
    pub audio: AudioConfig,
    pub transcription: TranscriptionConfig,
    pub ai: AIConfig,
    pub storage: StorageConfig,
}
```

Configuration locations:
- Windows: `%APPDATA%\bestme\config.json`
- macOS: `~/Library/Application Support/bestme/config.json`
- Linux: `~/.config/bestme/config.json`

## Voice Commands

### Architecture

```
Voice Input → Whisper → Command Parser → Action Executor
                ↓
         AI Enhancement (optional)
```

### Command Format

```json
{
  "prefix": ["computer", "assistant"],
  "commands": {
    "punctuation": {
      "period": ".",
      "comma": ",",
      "question": "?"
    },
    "actions": {
      "new line": "\n",
      "new paragraph": "\n\n"
    }
  }
}
```

### AI Voice Commands

Enhanced commands with context understanding:

```rust
// Example: "computer, make this formal"
let enhanced = ai_commander.process_command(
    "make this formal",
    "hey whats up",
    CommandContext { /* ... */ }
).await?;
// Result: "Hello, how are you?"
```

## Platform-Specific Notes

### Windows

- **Audio**: WASAPI for low-latency capture
- **GPU**: DirectML for broad compatibility
- **Packaging**: MSI installer with auto-update

### macOS

- **Audio**: CoreAudio framework
- **GPU**: Metal Performance Shaders
- **Security**: Notarization required
- **Permissions**: Microphone access

### Linux

- **Audio**: PulseAudio/ALSA
- **GPU**: CUDA on NVIDIA systems
- **Desktop**: Works with X11 and Wayland
- **Packaging**: AppImage, deb, rpm

## Performance & Optimization

### Benchmarking

```bash
# Run benchmarks
cargo bench

# AI model benchmarks
cargo test --release -- --ignored benchmark_

# Profile with flamegraph
cargo flamegraph --bin bestme
```

### Optimization Tips

1. **Audio Pipeline**
   - Use appropriate buffer sizes (512-2048 samples)
   - Enable GPU for Whisper when available
   - Use streaming mode for real-time

2. **AI Performance**
   - Quantize models (INT8/FP16)
   - Enable batch processing
   - Use model caching
   - Implement warmup

3. **Memory Management**
   - Unload unused models
   - Clear transcription buffers
   - Use streaming iterators

### Performance Targets

- Audio latency: <50ms
- Transcription: <300ms for 5s audio
- AI inference: <50ms for text enhancement
- Memory usage: <500MB base, <4GB with models

## Debugging

### Logging

```bash
# Set log level
export RUST_LOG=debug
export RUST_LOG=bestme=debug,tauri=info

# Log to file
export RUST_LOG_FILE=/tmp/bestme.log
```

### Common Issues

1. **Audio Device Issues**
   ```bash
   # List devices
   cargo run -- --list-audio-devices
   
   # Test specific device
   cargo run -- --audio-device "Microphone Name"
   ```

2. **Model Loading**
   ```bash
   # Verify model path
   cargo run -- --verify-models
   
   # Clear model cache
   rm -rf ~/.local/share/bestme/models/
   ```

3. **GPU Detection**
   ```bash
   # Check GPU status
   cargo run -- --gpu-info
   
   # Force CPU mode
   cargo run -- --cpu-only
   ```

### Debug Tools

- **Tauri DevTools**: Ctrl+Shift+I in app
- **Performance Monitor**: Built-in metrics view
- **Audio Visualizer**: Real-time waveform
- **Command History**: Voice command debugging

## Release Process

### Version Management

Follow semantic versioning:
- MAJOR: Breaking changes
- MINOR: New features
- PATCH: Bug fixes

### Release Checklist

1. **Update Version**
   ```toml
   # Cargo.toml
   version = "X.Y.Z"
   
   # tauri.conf.json
   "package": { "version": "X.Y.Z" }
   
   # package.json
   "version": "X.Y.Z"
   ```

2. **Run Tests**
   ```bash
   ./scripts/pre-release-check.sh
   ```

3. **Build Release**
   ```bash
   cargo tauri build --release
   ```

4. **Platform Testing**
   - Test on Windows 10/11
   - Test on macOS 12+
   - Test on Ubuntu 20.04+

5. **Create Release**
   - Tag version: `git tag v.X.Y.Z`
   - Generate changelog
   - Upload artifacts
   - Update documentation

### Continuous Integration

GitHub Actions workflow:
```yaml
- Build on: Windows, macOS, Linux
- Run tests
- Generate artifacts
- Create draft release
```

## Contributing

### Code Style

- Follow Rust formatting (`cargo fmt`)
- Use clippy suggestions
- Write unit tests for new features
- Document public APIs
- Update relevant documentation

### Pull Request Process

1. Create feature branch
2. Write/update tests
3. Update documentation
4. Run full test suite
5. Submit PR with description
6. Address review feedback

### Development Philosophy

- **No shortcuts**: Implement features properly
- **Test everything**: Maintain >80% coverage
- **Performance matters**: This is real-time software
- **Cross-platform**: Must work on all platforms
- **Privacy first**: All processing local by default

---

For specific component details, refer to:
- [AI-GUIDE.md](./AI-GUIDE.md) - AI system documentation
- [CLAUDE.md](../CLAUDE.md) - Development context
- Source code documentation in `/src/`