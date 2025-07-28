# BestMe Testing Guide

This guide provides comprehensive testing procedures for all BestMe features, including unit tests, integration tests, performance benchmarks, GPU acceleration, and real-world usage scenarios.

## General Testing Prerequisites

- Working microphone for audio capture testing
- Latest version of BestMe built from source
- Platform-specific development environment set up (see [INSTALLATION.md](INSTALLATION.md))
- GPU drivers installed (for GPU testing)

## Test Categories

### 1. Unit Tests

Run all unit tests:
```bash
# Run all Rust unit tests
cargo test --lib

# Run frontend tests
cd ui
npm test
```

Specific test categories:
```bash
# Audio tests
cargo test audio::

# Transcription tests
cargo test transcribe::

# VAD tests
cargo test vad::

# Voice command tests
cargo test voice_commands::

# GPU tests (requires GPU features)
cargo test --features gpu-cuda gpu::
```

### 2. Integration Tests

Run all integration tests:
```bash
cargo test --test '*'
```

Specific integration tests:
```bash
# Streaming pipeline tests
cargo test streaming_integration_tests

# GPU integration tests
cargo test --features gpu-cuda gpu_integration_tests

# Performance tests
cargo test --test performance_tests

# Real-world usage tests
cargo test --test real_world_tests
```

### 3. GPU Acceleration Testing

#### Prerequisites
- **NVIDIA**: CUDA Toolkit 11.8+ and driver 525.60+
- **AMD**: ROCm 5.7+ and AMDGPU-PRO driver
- **Apple**: macOS with Metal support
- **Intel**: OpenVINO 2023.2+ and Level Zero

#### Build with GPU support:
```bash
# NVIDIA GPUs (RTX 3080, etc.)
cargo build --release --features gpu-cuda

# AMD GPUs (RX 6000/7000 series)
cargo build --release --features gpu-hipblas

# Apple Silicon
cargo build --release --features gpu-metal

# Intel Arc GPUs
cargo build --release --features gpu-vulkan

# All GPU backends
cargo build --release --features gpu-all
```

#### Run GPU tests:
```bash
# Test GPU compilation
./scripts/test_gpu_build.sh

# Run GPU unit tests
cargo test --features gpu-cuda gpu::tests

# Run GPU integration tests
cargo test --features gpu-cuda gpu_integration_tests

# Run GPU benchmarks
cargo test --features gpu-cuda --release test_gpu_tiny_model
cargo test --features gpu-cuda --release test_gpu_small_model
cargo test --features gpu-cuda --release --ignored test_all_models_comprehensive
```

#### GPU Performance Benchmarks

##### Expected Performance Targets (Consumer GPUs)

| GPU Type | Model | Tiny | Base | Small | Medium |
|----------|-------|------|------|-------|--------|
| NVIDIA RTX 3080 | RTF | <0.2x | <0.3x | <0.5x | <0.8x |
| AMD RX 6900 XT | RTF | <0.3x | <0.4x | <0.7x | <1.0x |
| Apple M1 Max | RTF | <0.2x | <0.3x | <0.5x | <0.7x |
| Intel Arc A770 | RTF | <0.4x | <0.6x | <0.9x | <1.2x |

*RTF = Real-Time Factor (lower is better, <1.0 means faster than real-time)*

##### Benchmarking Script
```bash
# Run comprehensive GPU benchmark
cargo run --release --features gpu-cuda --bin benchmark -- --gpu

# Benchmark specific model
cargo run --release --features gpu-cuda --bin benchmark -- --gpu --model small

# Compare CPU vs GPU
cargo run --release --features gpu-cuda --bin benchmark -- --compare
```

##### Memory Usage Guidelines

| Model Size | VRAM Required | System RAM |
|------------|---------------|------------|
| Tiny | 400 MB | 1 GB |
| Base | 600 MB | 1.5 GB |
| Small | 1.5 GB | 3 GB |
| Medium | 3.0 GB | 6 GB |
| Large | 6.0 GB | 12 GB |

#### GPU Troubleshooting

##### NVIDIA CUDA Issues
```bash
# Check CUDA installation
nvidia-smi
nvcc --version

# Test CUDA availability
cargo test --features gpu-cuda detect_cuda_devices

# Debug CUDA errors
export CUDA_LAUNCH_BLOCKING=1
export RUST_LOG=bestme::audio::gpu=debug
cargo run --features gpu-cuda
```

##### AMD ROCm Issues
```bash
# Check ROCm installation
rocm-smi
rocminfo

# Test HIP availability
cargo test --features gpu-hipblas detect_rocm_devices

# Debug ROCm errors
export HSA_ENABLE_SDMA=0
export RUST_LOG=bestme::audio::gpu=debug
cargo run --features gpu-hipblas
```

##### Apple Metal Issues
```bash
# Check Metal support
system_profiler SPDisplaysDataType

# Test Metal availability
cargo test --features gpu-metal detect_metal_devices

# Debug Metal errors
export RUST_LOG=bestme::audio::gpu=debug
export MTL_CAPTURE_ENABLED=1
cargo run --features gpu-metal
```

### 4. Performance Testing

#### Quick performance test:
```bash
cargo test --release test_all_features_performance
```

#### Memory usage test:
```bash
cargo test --release --ignored test_memory_usage
```

#### Full benchmark suite:
```bash
cargo bench
```

### 5. Real-World Usage Tests

Test realistic scenarios:
```bash
# Conversation scenario
cargo test test_real_world_conversation

# Presentation scenario
cargo test test_real_world_presentation

# Noisy environment
cargo test test_real_world_noisy_environment

# Vocabulary enhancement
cargo test test_vocabulary_enhancement

# Multi-language
cargo test test_multi_language_real_world
```

## Platform-Specific Testing

### Windows Testing

#### Prerequisites
- Windows 10 version 1803+ or Windows 11
- Visual Studio Build Tools
- WebView2 Runtime
- Administrator privileges for certain tests

#### Test Protocol
1. **Application Launch**
   - [ ] Verify clean startup without errors
   - [ ] Check system tray icon appears
   - [ ] Confirm WebView2 initializes properly

2. **Audio System**
   - [ ] List all Windows audio devices
   - [ ] Test microphone selection
   - [ ] Verify audio level visualization
   - [ ] Test with USB and Bluetooth microphones

3. **Permissions**
   - [ ] Grant microphone access when prompted
   - [ ] Verify Windows Defender doesn't block the app
   - [ ] Check firewall permissions if needed

#### Windows-Specific Commands
```powershell
# Run with debug logging
$env:RUST_LOG="debug"; cargo run

# Check logs
Get-Content "$env:APPDATA\bestme\logs\bestme.log"
```

### macOS Testing

#### Prerequisites
- macOS 10.15 (Catalina) or later
- Xcode Command Line Tools
- Microphone permissions granted

#### Test Protocol
1. **Application Launch**
   - [ ] Verify app launches without security warnings
   - [ ] Check menu bar integration
   - [ ] Confirm proper app signing (for distribution)

2. **Audio System**
   - [ ] Test with built-in microphone
   - [ ] Test with external audio devices
   - [ ] Verify Core Audio integration

3. **Permissions**
   - [ ] Grant microphone access in System Preferences
   - [ ] Test accessibility permissions for global hotkeys

#### macOS-Specific Commands
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Check logs
tail -f ~/Library/Application\ Support/bestme/logs/bestme.log

# Test code signing
codesign --verify --deep --verbose=2 target/release/bundle/macos/BestMe.app
```

### Linux Testing

#### Prerequisites
- Modern Linux distribution (Ubuntu 20.04+, Fedora 35+, etc.)
- All system dependencies installed (see [INSTALLATION.md](INSTALLATION.md))
- PulseAudio or PipeWire audio system

#### Test Protocol
1. **Application Launch**
   - [ ] Verify GTK integration
   - [ ] Check system tray support (varies by DE)
   - [ ] Confirm WebKit2GTK loads properly

2. **Audio System**
   - [ ] Test with PulseAudio
   - [ ] Test with PipeWire (if available)
   - [ ] Verify ALSA fallback works

3. **Desktop Integration**
   - [ ] Test on GNOME
   - [ ] Test on KDE Plasma
   - [ ] Test on other DEs (XFCE, etc.)

#### Linux-Specific Commands
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Check audio devices
pactl list sources

# Monitor D-Bus for tray integration
dbus-monitor --session "interface='org.kde.StatusNotifierItem'"
```

## Feature Testing Checklist

### Core Features
- [ ] **Audio Capture**
  - Start/stop recording
  - Device switching
  - Level monitoring
  - Buffer management

- [ ] **Transcription**
  - Real-time transcription
  - Language detection
  - Model switching (base/small/medium)
  - Accuracy verification

- [ ] **Voice Commands**
  - Command recognition
  - Command history
  - Custom commands
  - Error handling

- [ ] **UI Components**
  - Main window rendering
  - Settings dialog
  - System tray menu
  - Dark/light theme switching

### Advanced Features
- [ ] **Translation**
  - Multi-language input
  - Translation accuracy
  - Performance impact

- [ ] **Export Functions**
  - Text export
  - Audio export
  - Settings export/import

## Performance Testing

### Benchmarks
```bash
# Run performance benchmarks
cargo bench

# Profile CPU usage
cargo run --release -- --profile
```

### Memory Testing
- Monitor RAM usage during:
  - Long recording sessions (30+ minutes)
  - Multiple model switches
  - Heavy UI interactions

### Stress Testing
1. Run continuous recording for 1+ hours
2. Rapidly switch between audio devices
3. Process multiple languages in sequence
4. Test with poor quality audio input

## Debugging Common Issues

### Audio Issues
```bash
# Enable audio debugging
RUST_LOG=bestme::audio=trace cargo run

# Test audio capture directly
cargo run --example test_audio
```

### Transcription Issues
```bash
# Enable whisper debugging
RUST_LOG=bestme::transcribe=debug cargo run

# Test model loading
cargo run --example test_whisper
```

### UI Issues
```bash
# Enable Tauri debugging
RUST_LOG=tauri=debug cargo run

# Open developer tools (in app)
# Press Ctrl+Shift+I (Windows/Linux) or Cmd+Opt+I (macOS)
```

## Automated Testing

### CI/CD Pipeline
The project uses GitHub Actions for automated testing:

```yaml
# See .github/workflows/test.yml
- Build on all platforms
- Run unit tests
- Run integration tests
- Package applications
```

### Local CI Testing
```bash
# Install act for local GitHub Actions testing
# Then run:
act -j test
```

## Test Data

Test files are located in `test_data/`:
- `audio_samples/` - Various audio formats and qualities
- `commands/` - Voice command test cases
- `languages/` - Multi-language test phrases

## Reporting Issues

When reporting test failures:

1. **Environment Info**
   ```bash
   cargo version
   rustc --version
   node --version
   # OS version and specs
   ```

2. **Steps to Reproduce**
   - Exact commands run
   - User actions performed
   - Expected vs actual behavior

3. **Logs and Screenshots**
   - Full error messages
   - Debug logs
   - Screenshots if UI-related

4. **System State**
   - Audio devices connected
   - CPU/Memory usage
   - Network conditions (if relevant)

## Success Criteria

A test run is considered successful when:
- All unit tests pass
- Application starts without errors
- Core features work as documented
- No memory leaks detected
- Performance meets requirements:
  - Transcription latency < 500ms
  - UI responsiveness < 100ms
  - Memory usage < 500MB baseline
  - GPU acceleration achieves RTF < 1.0 for real-time

## Continuous Integration Testing

### GitHub Actions Workflow
The project includes comprehensive CI testing:

```yaml
# .github/workflows/test.yml includes:
- Multi-platform builds (Windows, macOS, Linux)
- Unit and integration tests
- GPU feature compilation tests
- Performance benchmarks
- Code coverage reporting
```

### Running CI Tests Locally
```bash
# Install act for local GitHub Actions
brew install act  # macOS
# or
sudo snap install act  # Linux

# Run CI tests locally
act -j test

# Run specific workflow
act -j test-gpu
```

## Test Coverage Requirements

Minimum coverage targets:
- Core audio processing: 90%
- Transcription pipeline: 85%
- Voice commands: 80%
- GPU acceleration: 75%
- UI components: 70%

Generate coverage report:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```