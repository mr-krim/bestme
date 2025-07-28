# BestMe Testing Summary

## Testing Infrastructure Overview

The BestMe project has a comprehensive testing infrastructure covering all major features and components. This document summarizes the testing approach, available tests, and how to run them.

## Test Categories

### 1. Unit Tests
- **Location**: Throughout the codebase in `#[cfg(test)]` modules
- **Coverage**: Individual functions and modules
- **Key Areas**:
  - Audio processing (`src/audio/`)
  - Voice Activity Detection (`src/audio/vad.rs`)
  - Vocabulary management (`src/audio/vocabulary.rs`)
  - Multi-pass processing (`src/audio/multi_pass.rs`)
  - GPU acceleration (`src/audio/gpu/`)

### 2. Integration Tests
- **Location**: `src/audio/tests/` and `tests/` directories
- **Key Files**:
  - `streaming_integration_tests.rs` - Streaming pipeline tests
  - `gpu_integration_tests.rs` - GPU acceleration tests
  - `performance_tests.rs` - Performance benchmarks
  - `real_world_tests.rs` - Real-world usage scenarios

### 3. GPU Acceleration Tests
- **Build Script**: `scripts/test_gpu_build.sh`
- **Integration Tests**: `src/audio/gpu/gpu_integration_tests.rs`
- **Backends Tested**:
  - CUDA (NVIDIA)
  - Metal (Apple)
  - ROCm (AMD)
  - Vulkan (Cross-platform)

### 4. Performance Tests
- **Benchmark Tool**: `cargo run --release --bin benchmark`
- **Test File**: `tests/performance_tests.rs`
- **Features Tested**:
  - VAD performance
  - Vocabulary operations
  - Multi-pass processing
  - Streaming pipeline
  - Full transcription pipeline

### 5. Real-World Tests
- **Test File**: `tests/real_world_tests.rs`
- **Scenarios**:
  - Conversation transcription
  - Presentation recording
  - Noisy environment handling
  - Vocabulary enhancement
  - Multi-language support

## Quick Test Commands

### Run All Tests
```bash
# All unit tests
cargo test --lib

# All integration tests
cargo test --test '*'

# All tests with GPU features
cargo test --features gpu-cuda

# All tests in release mode
cargo test --release
```

### Specific Feature Tests
```bash
# Audio tests only
cargo test audio::

# VAD tests
cargo test vad::

# GPU tests
cargo test --features gpu-cuda gpu::

# Streaming tests
cargo test streaming_

# Performance tests
cargo test --release test_all_features_performance
```

### Benchmarking
```bash
# Run benchmark tool
cargo run --release --bin benchmark

# Compare CPU vs GPU
cargo run --release --features gpu-cuda --bin benchmark -- --compare

# Benchmark specific model
cargo run --release --features gpu-cuda --bin benchmark -- --model small

# JSON output
cargo run --release --bin benchmark -- --format json
```

## Test Coverage

### Current Coverage Status
- **Core Audio**: ~90% coverage
- **Transcription Pipeline**: ~85% coverage  
- **Voice Commands**: ~80% coverage
- **GPU Acceleration**: ~75% coverage
- **UI Components**: ~70% coverage

### Generate Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

## Platform-Specific Testing

### Windows
```powershell
# Set debug logging
$env:RUST_LOG="debug"
cargo test

# Test with specific audio device
$env:AUDIO_DEVICE="Microphone Array"
cargo test audio::
```

### macOS
```bash
# Test with Metal GPU
cargo test --features gpu-metal

# Test system integration
cargo test --features tauri-2 system_integration
```

### Linux
```bash
# Test with different audio systems
# PulseAudio
cargo test audio::

# PipeWire
pw-jack cargo test audio::

# Test GPU backends
cargo test --features gpu-cuda  # NVIDIA
cargo test --features gpu-hipblas  # AMD
```

## CI/CD Testing

The project uses GitHub Actions for continuous integration:

### Workflows
- **test.yml**: Main test workflow
- **Platforms**: Windows, macOS, Linux
- **Features**: All feature combinations
- **GPU**: Compilation tests for all backends

### Local CI Testing
```bash
# Install act
brew install act  # macOS
sudo snap install act  # Linux

# Run CI locally
act -j test

# Run with specific event
act pull_request
```

## Testing Best Practices

### Before Committing
1. Run unit tests: `cargo test --lib`
2. Run integration tests: `cargo test --test '*'`
3. Check formatting: `cargo fmt -- --check`
4. Run clippy: `cargo clippy -- -D warnings`
5. Test your specific changes thoroughly

### Adding New Tests
1. **Unit Tests**: Add in the same file as the code
2. **Integration Tests**: Add to appropriate test file
3. **Performance Tests**: Update `performance_tests.rs`
4. **Document**: Update this summary when adding test categories

### Test Naming Convention
- Unit tests: `test_<function_name>_<scenario>`
- Integration tests: `test_<feature>_<integration_point>`
- Performance tests: `test_<feature>_performance`
- GPU tests: `test_gpu_<backend>_<scenario>`

## Troubleshooting Test Failures

### Common Issues

1. **Model Download Failures**
   ```bash
   # Clear model cache
   rm -rf ~/.cache/whisper
   
   # Re-run with debug logging
   RUST_LOG=debug cargo test
   ```

2. **Audio Device Issues**
   ```bash
   # List available devices
   cargo run --example list_audio_devices
   
   # Test with specific device
   AUDIO_DEVICE="Device Name" cargo test
   ```

3. **GPU Test Failures**
   ```bash
   # Check GPU detection
   nvidia-smi  # NVIDIA
   rocm-smi   # AMD
   
   # Test GPU compilation only
   cargo check --features gpu-cuda
   ```

4. **Memory Issues**
   ```bash
   # Run with memory limit
   ulimit -v 4000000  # 4GB limit
   cargo test
   
   # Run tests sequentially
   cargo test -- --test-threads=1
   ```

## Test Data

Test data is organized in:
- `test_data/audio_samples/` - Audio test files
- `test_data/commands/` - Voice command samples  
- `test_data/languages/` - Multi-language samples

## Future Testing Improvements

1. **Automated Performance Regression Testing**
2. **Visual Regression Testing for UI**
3. **Fuzz Testing for Audio Processing**
4. **Extended Multi-Language Test Suite**
5. **Automated GPU Performance Tracking**

## Resources

- [TESTING.md](TESTING.md) - Detailed testing guide
- [gpu-acceleration-plan.md](gpu-acceleration-plan.md) - GPU testing details
- [DEVELOPMENT.md](DEVELOPMENT.md) - Development workflow
- [Benchmark Tool](../src/bin/benchmark.rs) - Performance testing utility