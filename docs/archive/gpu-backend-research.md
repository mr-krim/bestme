# GPU Backend Research for Whisper

## Current State
- Using whisper-rs v0.11.1 with CPU-only support
- No GPU features enabled in Cargo.toml

## Available GPU Backend Options

### 1. whisper-rs Native GPU Support

**Current Status**: whisper-rs 0.11.1 has experimental GPU support

**Features Available**:
```toml
whisper-rs = { version = "0.11.1", features = ["cuda", "opencl", "metal", "coreml"] }
```

**Pros**:
- Direct integration with current codebase
- Minimal changes required
- Supports multiple backends

**Cons**:
- GPU support may be experimental/incomplete
- Limited documentation
- May not support all consumer GPUs optimally

### 2. whisper.cpp Backend via whisper-rs-sys

**Implementation**: whisper-rs uses whisper.cpp under the hood via whisper-rs-sys

**GPU Support in whisper.cpp**:
- CUDA: Mature support for NVIDIA GPUs
- Metal: Good support for Apple Silicon
- OpenCL: Cross-platform but slower
- GGML: CPU optimizations that also benefit GPU
- Vulkan: Experimental support

**How to Enable**:
```toml
[dependencies]
whisper-rs = "0.11.1"
whisper-rs-sys = { version = "0.8", features = ["cuda", "metal"] }

[build-dependencies]
cmake = "0.1"
cc = "1.0"
```

### 3. candle-transformers Alternative

**Overview**: Rust-native ML framework with GPU support

**Implementation**:
```toml
[dependencies]
candle-core = { version = "0.3", features = ["cuda", "metal"] }
candle-nn = "0.3"
candle-transformers = "0.3"
```

**Pros**:
- Pure Rust implementation
- Better integration with Rust ecosystem
- Active development
- Supports quantized models

**Cons**:
- Would require rewriting transcription logic
- Different API from whisper-rs
- Less mature than whisper.cpp

### 4. ONNX Runtime Approach

**Overview**: Convert Whisper to ONNX and use ort (ONNX Runtime)

**Implementation**:
```toml
[dependencies]
ort = { version = "2.0", features = ["cuda", "tensorrt", "directml", "coreml"] }
```

**Pros**:
- Excellent cross-platform GPU support
- Supports all major GPU vendors
- Production-ready
- Great performance with TensorRT

**Cons**:
- Requires model conversion
- Larger binary size
- More complex deployment

### 5. tch (PyTorch) Bindings

**Overview**: Use PyTorch C++ API via Rust bindings

**Implementation**:
```toml
[dependencies]
tch = { version = "0.13", features = ["cuda-11.8"] }
```

**Pros**:
- Mature GPU support
- Can use PyTorch Whisper models directly
- Excellent performance

**Cons**:
- Heavy dependency (PyTorch)
- Complex deployment
- Large binary size

## Recommended Approach

### Phase 1: Enhance whisper-rs with GPU features
1. Enable GPU features in whisper-rs
2. Build with appropriate flags for each platform
3. Test on consumer GPUs

### Phase 2: Implement Multi-Backend Support
```rust
pub enum WhisperBackend {
    WhisperRs(WhisperRsBackend),  // Current implementation
    Candle(CandleBackend),         // Rust-native alternative
    Onnx(OnnxBackend),             // Cross-platform optimized
}
```

## Build Configuration for GPU Support

### NVIDIA CUDA
```toml
[target.'cfg(target_os = "linux")'.dependencies]
whisper-rs = { version = "0.11.1", features = ["cuda"] }

[target.'cfg(target_os = "windows")'.dependencies]
whisper-rs = { version = "0.11.1", features = ["cuda"] }
```

### AMD ROCm
```toml
# Currently no direct ROCm support in whisper-rs
# Options:
# 1. Use OpenCL feature as fallback
# 2. Implement custom ROCm backend
# 3. Use ONNX Runtime with ROCm EP
```

### Intel Arc/Xe
```toml
# Use OpenVINO through ONNX Runtime
ort = { version = "2.0", features = ["openvino"] }
```

### Apple Silicon
```toml
[target.'cfg(target_os = "macos")'.dependencies]
whisper-rs = { version = "0.11.1", features = ["metal", "coreml"] }
```

## Implementation Strategy

### Step 1: Test whisper-rs GPU features
```rust
// Check if GPU features compile and work
#[cfg(feature = "cuda")]
fn test_cuda_backend() -> Result<()> {
    let whisper = WhisperBuilder::new()
        .with_model_path("models/ggml-base.bin")
        .with_gpu(true)
        .build()?;
    
    // Test transcription
    let audio = load_test_audio();
    let result = whisper.transcribe(&audio)?;
    println!("GPU transcription: {:?}", result);
    Ok(())
}
```

### Step 2: Build System Changes
```toml
# Cargo.toml
[features]
gpu-cuda = ["whisper-rs/cuda"]
gpu-metal = ["whisper-rs/metal"]
gpu-opencl = ["whisper-rs/opencl"]
gpu-all = ["gpu-cuda", "gpu-metal", "gpu-opencl"]

# Platform-specific features
[target.'cfg(target_os = "windows")'.features]
default = ["gpu-cuda"]

[target.'cfg(target_os = "macos")'.features]
default = ["gpu-metal"]

[target.'cfg(target_os = "linux")'.features]
default = ["gpu-cuda", "gpu-opencl"]
```

### Step 3: Runtime GPU Detection
```rust
pub fn detect_available_backends() -> Vec<GpuBackend> {
    let mut backends = vec![];
    
    #[cfg(feature = "cuda")]
    if cuda::is_available() {
        backends.push(GpuBackend::Cuda);
    }
    
    #[cfg(feature = "metal")]
    if metal::is_available() {
        backends.push(GpuBackend::Metal);
    }
    
    #[cfg(feature = "opencl")]
    if opencl::get_platforms().is_ok() {
        backends.push(GpuBackend::OpenCL);
    }
    
    backends
}
```

## Next Steps

1. **Test whisper-rs GPU features**:
   - Create test branch
   - Enable cuda/metal features
   - Build and test on target hardware

2. **Evaluate alternatives if needed**:
   - If whisper-rs GPU support insufficient
   - Test candle-transformers performance
   - Consider ONNX Runtime for broader support

3. **Implement abstraction layer**:
   - Support multiple backends
   - Runtime backend selection
   - Graceful fallback

## Dependencies to Add

```toml
# For GPU support
[dependencies]
# Option 1: whisper-rs with GPU
whisper-rs = { version = "0.11.1", features = ["cuda", "metal"] }

# Option 2: Alternative backends
candle-core = { version = "0.3", optional = true }
ort = { version = "2.0", optional = true }

# GPU detection utilities
nvml-wrapper = { version = "0.9", optional = true }  # NVIDIA GPU info
ash = { version = "0.37", optional = true }          # Vulkan detection

[build-dependencies]
cmake = "0.1"
cc = "1.0"
cuda-sys = { version = "0.2", optional = true }
```

## Platform-Specific Notes

### Windows
- Requires CUDA Toolkit for NVIDIA
- Visual Studio 2019/2022 for compilation
- DirectML as alternative through ONNX

### Linux
- CUDA Toolkit for NVIDIA
- ROCm for AMD (through OpenCL or custom)
- Standard build tools (gcc, cmake)

### macOS
- Xcode command line tools
- Metal/CoreML work out of the box
- No CUDA support (NVIDIA GPUs unsupported)

## Conclusion

The most pragmatic approach is to:
1. Start with whisper-rs GPU features
2. Test thoroughly on consumer GPUs
3. Implement fallback mechanisms
4. Consider alternatives only if necessary

This minimizes code changes while providing GPU acceleration for most users.