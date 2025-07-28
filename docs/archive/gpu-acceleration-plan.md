# GPU Acceleration Plan for Whisper

## Overview
This plan outlines the implementation of GPU acceleration for Whisper transcription in BestMe, targeting significant performance improvements across NVIDIA, AMD, Intel, and Apple Silicon platforms.

## Current State
- CPU-only Whisper implementation using whisper-rs
- Good performance for small models (tiny, base)
- Slower performance for larger models (medium, large)
- No GPU utilization currently

## Goals
1. **Performance**: Achieve 5-10x speedup for transcription
2. **Compatibility**: Support consumer GPUs:
   - NVIDIA: GTX 1060/1070/1080, RTX 2060/2070/2080, RTX 3060/3070/3080/3090, RTX 4060/4070/4080/4090
   - AMD: RX 5700/6600/6700/6800/6900 XT, RX 7600/7700/7800/7900 XT
   - Intel: Arc A380/A580/A750/A770, Integrated Xe Graphics
   - Apple Silicon: M1/M2/M3/M4 (all variants)
3. **Fallback**: Graceful fallback to CPU when GPU unavailable
4. **User Experience**: Automatic GPU detection and selection

## Implementation Strategy

### Phase 1: Research & Setup (2 days)

#### Day 1: GPU Backend Investigation
**Tasks:**
1. **Investigate whisper-rs GPU support**
   ```toml
   whisper-rs = { version = "0.11", features = ["cuda", "coreml", "openvino"] }
   ```

2. **Research alternative implementations**
   - whisper.cpp GPU backends
   - candle-transformers for Rust-native GPU
   - ONNX Runtime for cross-platform GPU

3. **Benchmark baseline performance**
   - Test all model sizes on CPU
   - Document transcription speeds
   - Measure memory usage

#### Day 2: Development Environment Setup
**Tasks:**
1. **CUDA Setup (NVIDIA)**
   - Install CUDA Toolkit 12.x
   - Set up cuDNN
   - Configure build environment

2. **Metal Setup (macOS)**
   - Verify Metal Performance Shaders
   - Set up CoreML tools
   - Test compilation flags

3. **OpenVINO Setup (Intel)**
   - Install OpenVINO toolkit
   - Configure for Intel Arc GPUs
   - Test on integrated Xe graphics

4. **ROCm Setup (AMD)**
   - Install ROCm for AMD GPUs
   - Configure HIP/rocBLAS
   - Test on RX 6000/7000 series

### Phase 2: Core Implementation (3-4 days)

#### Day 3-4: GPU Backend Abstraction
**Implementation:**
```rust
// src/audio/gpu/mod.rs
pub enum GpuBackend {
    Cuda(CudaBackend),        // NVIDIA GPUs
    Rocm(RocmBackend),        // AMD GPUs  
    Metal(MetalBackend),      // Apple Silicon
    OpenVino(OpenVinoBackend), // Intel Arc & Xe
    Vulkan(VulkanBackend),    // Cross-platform fallback
    Cpu,                      // Final fallback
}

pub trait GpuAccelerator: Send + Sync {
    fn is_available(&self) -> bool;
    fn get_device_info(&self) -> DeviceInfo;
    fn load_model(&mut self, model_path: &Path) -> Result<()>;
    fn transcribe(&self, audio: &[f32]) -> Result<Transcript>;
}

pub struct GpuManager {
    backends: Vec<Box<dyn GpuAccelerator>>,
    selected_backend: Option<GpuBackend>,
}

impl GpuManager {
    pub fn new() -> Self {
        let mut backends = Vec::new();
        
        #[cfg(feature = "cuda")]
        if let Ok(cuda) = CudaBackend::new() {
            backends.push(Box::new(cuda));
        }
        
        #[cfg(feature = "rocm")]
        if let Ok(rocm) = RocmBackend::new() {
            backends.push(Box::new(rocm));
        }
        
        #[cfg(target_os = "macos")]
        if let Ok(metal) = MetalBackend::new() {
            backends.push(Box::new(metal));
        }
        
        #[cfg(feature = "openvino")]
        if let Ok(openvino) = OpenVinoBackend::new() {
            backends.push(Box::new(openvino));
        }
        
        // Vulkan as cross-platform fallback
        if let Ok(vulkan) = VulkanBackend::new() {
            backends.push(Box::new(vulkan));
        }
        
        // Auto-select best available backend
        let selected = Self::select_best_backend(&backends);
        
        Self { backends, selected_backend: selected }
    }
    
    fn select_best_backend(backends: &[Box<dyn GpuAccelerator>]) -> Option<GpuBackend> {
        // Priority order for consumer GPUs:
        // 1. Discrete GPU over integrated
        // 2. Higher VRAM capacity
        // 3. Native backend over compatibility layer
        
        let mut best_backend = None;
        let mut best_score = 0;
        
        for backend in backends {
            let info = backend.get_device_info();
            let mut score = 0;
            
            // Prefer discrete GPUs
            if info.is_discrete {
                score += 1000;
            }
            
            // Score by VRAM (important for larger models)
            score += (info.memory_mb / 1024) * 100; // Points per GB
            
            // Native backend bonus
            match info.backend.as_str() {
                "CUDA" | "Metal" => score += 500,
                "ROCm" | "OpenVINO" => score += 400,
                "Vulkan" => score += 100,
                _ => {}
            }
            
            if score > best_score && backend.is_available() {
                best_score = score;
                best_backend = Some(backend.get_backend_type());
            }
        }
        
        best_backend
    }
}
```

#### Day 5: Platform-Specific Implementations

**CUDA Implementation:**
```rust
// src/audio/gpu/cuda.rs
pub struct CudaBackend {
    device: CudaDevice,
    model: Option<WhisperCudaModel>,
}

impl GpuAccelerator for CudaBackend {
    fn is_available(&self) -> bool {
        // Check CUDA availability - support GTX 10 series and newer
        cuda::is_available() && self.device.compute_capability() >= (6, 1)
    }
    
    fn transcribe(&self, audio: &[f32]) -> Result<Transcript> {
        // GPU memory transfer
        let d_audio = self.device.alloc_and_copy(audio)?;
        
        // Run inference on GPU
        let d_output = self.model.forward(&d_audio)?;
        
        // Copy back to host
        let output = d_output.to_host()?;
        
        // Decode tokens
        self.decode_output(output)
    }
}
```

**Metal Implementation:**
```rust
// src/audio/gpu/metal.rs
pub struct MetalBackend {
    device: metal::Device,
    model: CoreMLModel,
    command_queue: CommandQueue,
}

impl GpuAccelerator for MetalBackend {
    fn transcribe(&self, audio: &[f32]) -> Result<Transcript> {
        // Create Metal buffer
        let buffer = self.device.new_buffer_with_data(
            audio.as_ptr() as *const _,
            audio.len() * size_of::<f32>(),
            MTLResourceOptions::StorageModeShared,
        );
        
        // Run through CoreML
        let output = self.model.predict(&buffer)?;
        
        self.decode_output(output)
    }
}
```

**AMD ROCm Implementation:**
```rust
// src/audio/gpu/rocm.rs
pub struct RocmBackend {
    device: HipDevice,
    model: Option<WhisperRocmModel>,
    memory_pool: HipMemPool,
}

impl GpuAccelerator for RocmBackend {
    fn is_available(&self) -> bool {
        // Check ROCm availability for RX 5000 series and newer
        rocm::is_available() && 
        self.device.get_name().contains_any(&["RX 5", "RX 6", "RX 7"])
    }
    
    fn transcribe(&self, audio: &[f32]) -> Result<Transcript> {
        // Allocate from memory pool for better performance
        let d_audio = self.memory_pool.alloc(audio.len() * 4)?;
        hip::memcpy_htod(&d_audio, audio)?;
        
        // Run inference on AMD GPU
        let d_output = self.model.forward(&d_audio)?;
        
        // Copy back to host
        let output = vec![0f32; d_output.size()];
        hip::memcpy_dtoh(&mut output, &d_output)?;
        
        self.decode_output(output)
    }
}
```

**Intel Arc/Xe Implementation:**
```rust
// src/audio/gpu/openvino.rs
pub struct OpenVinoBackend {
    core: Core,
    device_name: String, // "GPU" for Arc, "GPU.0" for integrated
    compiled_model: CompiledModel,
}

impl GpuAccelerator for OpenVinoBackend {
    fn is_available(&self) -> bool {
        let devices = self.core.get_available_devices();
        devices.iter().any(|d| d.starts_with("GPU"))
    }
    
    fn get_device_info(&self) -> DeviceInfo {
        let name = self.core.get_property(&self.device_name, "FULL_DEVICE_NAME");
        let memory = self.core.get_property(&self.device_name, "GPU_DEVICE_TOTAL_MEM_SIZE");
        
        DeviceInfo {
            name,
            memory_mb: memory / 1024 / 1024,
            backend: "OpenVINO",
            is_discrete: name.contains("Arc"),
        }
    }
}
```

### Phase 3: Integration & Optimization (2 days)

#### Day 6: Tauri Plugin Integration
**Tasks:**
1. **Update TranscriptionProcessor**
   ```rust
   pub struct EnhancedTranscriptionProcessor {
       whisper: WhisperContext,
       gpu_manager: Option<GpuManager>,
       config: WhisperConfig,
   }
   
   impl EnhancedTranscriptionProcessor {
       pub fn new(config: WhisperConfig) -> Result<Self> {
           let gpu_manager = if config.enable_gpu {
               GpuManager::new().ok()
           } else {
               None
           };
           
           Ok(Self {
               whisper: WhisperContext::new(&config)?,
               gpu_manager,
               config,
           })
       }
       
       pub async fn transcribe(&mut self, audio: &[f32]) -> Result<Transcript> {
           if let Some(gpu) = &self.gpu_manager {
               gpu.transcribe(audio).or_else(|e| {
                   warn!("GPU transcription failed: {}, falling back to CPU", e);
                   self.whisper.transcribe(audio)
               })
           } else {
               self.whisper.transcribe(audio)
           }
       }
   }
   ```

2. **Add GPU configuration options**
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct GpuConfig {
       pub enable_gpu: bool,
       pub preferred_backend: Option<String>,
       pub gpu_device_index: Option<usize>,
       pub max_gpu_memory_mb: Option<usize>,
       pub fallback_to_cpu: bool,
   }
   ```

#### Day 7: Performance Optimization
**Tasks:**
1. **Batch processing for efficiency**
   ```rust
   pub async fn transcribe_batch(&mut self, chunks: Vec<AudioChunk>) -> Vec<Transcript> {
       // Process multiple chunks in single GPU call
       let batch = self.prepare_batch(chunks);
       self.gpu_manager.transcribe_batch(batch)
   }
   ```

2. **Memory management**
   - Implement GPU memory pooling
   - Add memory usage monitoring
   - Automatic model unloading

3. **Dynamic backend switching**
   - Monitor GPU temperature/usage
   - Switch to CPU if GPU overloaded
   - Load balancing for multi-GPU

### Phase 4: UI Integration & Testing (2 days)

#### Day 8: UI Components
**Create GPU settings UI:**
```svelte
<!-- ui/src/components/GpuSettings.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  
  let gpuEnabled = false;
  let availableBackends = [];
  let selectedBackend = 'auto';
  let gpuInfo = {
    name: '',
    memory: 0,
    utilization: 0,
  };
  
  onMount(async () => {
    gpuInfo = await invoke('get_gpu_info');
    availableBackends = await invoke('get_available_gpu_backends');
  });
  
  async function toggleGpu() {
    await invoke('set_gpu_enabled', { enabled: !gpuEnabled });
    gpuEnabled = !gpuEnabled;
  }
</script>

<div class="gpu-settings">
  <h3>GPU Acceleration</h3>
  
  <label>
    <input type="checkbox" bind:checked={gpuEnabled} on:change={toggleGpu}>
    Enable GPU Acceleration
  </label>
  
  {#if gpuEnabled && gpuInfo.name}
    <div class="gpu-info">
      <p>GPU: {gpuInfo.name}</p>
      <p>Memory: {gpuInfo.memory} MB</p>
      <p>Current Usage: {gpuInfo.utilization}%</p>
    </div>
    
    <label>
      Backend:
      <select bind:value={selectedBackend}>
        <option value="auto">Auto-detect</option>
        {#each availableBackends as backend}
          <option value={backend}>{backend}</option>
        {/each}
      </select>
    </label>
  {/if}
</div>
```

#### Day 9: Testing & Benchmarking
**Comprehensive testing plan:**

1. **Unit Tests**
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_gpu_detection() {
           let manager = GpuManager::new();
           assert!(manager.has_gpu_available());
       }
       
       #[test]
       fn test_fallback_to_cpu() {
           let mut processor = create_test_processor();
           processor.force_cpu_fallback();
           let result = processor.transcribe(test_audio());
           assert!(result.is_ok());
       }
   }
   ```

2. **Performance Benchmarks**
   ```rust
   #[bench]
   fn bench_gpu_vs_cpu(b: &mut Bencher) {
       let audio = load_test_audio();
       let gpu_processor = create_gpu_processor();
       let cpu_processor = create_cpu_processor();
       
       println!("GPU Benchmark:");
       b.iter(|| gpu_processor.transcribe(&audio));
       
       println!("CPU Benchmark:");
       b.iter(|| cpu_processor.transcribe(&audio));
   }
   ```

3. **Integration Tests**
   - Test with all model sizes
   - Verify memory usage stays within limits
   - Test GPU failure scenarios
   - Verify correct fallback behavior

## Expected Outcomes

### Performance Targets by GPU

#### NVIDIA Consumer GPUs
| GPU Model | Model Size | CPU Time | GPU Time | Speedup | Power Usage |
|-----------|------------|----------|----------|---------|-------------|
| RTX 3080  | Medium     | 45s      | 3.5s     | 12.9x   | ~180W       |
| RTX 3070  | Medium     | 45s      | 4.5s     | 10x     | ~150W       |
| RTX 3060  | Medium     | 45s      | 6s       | 7.5x    | ~120W       |
| RTX 4070  | Medium     | 45s      | 3s       | 15x     | ~140W       |
| GTX 1080  | Small      | 15s      | 2.5s     | 6x      | ~150W       |

#### AMD Consumer GPUs
| GPU Model  | Model Size | CPU Time | GPU Time | Speedup | Power Usage |
|------------|------------|----------|----------|---------|-------------|
| RX 7900 XT | Medium     | 45s      | 4s       | 11.3x   | ~200W       |
| RX 6800 XT | Medium     | 45s      | 5s       | 9x      | ~180W       |
| RX 6700 XT | Medium     | 45s      | 6.5s     | 6.9x    | ~150W       |
| RX 5700 XT | Small      | 15s      | 3s       | 5x      | ~160W       |

#### Intel Arc GPUs
| GPU Model | Model Size | CPU Time | GPU Time | Speedup | Power Usage |
|-----------|------------|----------|----------|---------|-------------|
| Arc A770  | Medium     | 45s      | 5.5s     | 8.2x    | ~150W       |
| Arc A750  | Medium     | 45s      | 6.5s     | 6.9x    | ~130W       |
| Arc A380  | Small      | 15s      | 3.5s     | 4.3x    | ~75W        |
| Xe (i7)   | Tiny       | 2s       | 0.6s     | 3.3x    | ~30W        |

#### Apple Silicon
| GPU Model  | Model Size | CPU Time | GPU Time | Speedup | Power Usage |
|------------|------------|----------|----------|---------|-------------|
| M3 Max     | Large      | 120s     | 8s       | 15x     | ~40W        |
| M2 Pro     | Medium     | 45s      | 5s       | 9x      | ~30W        |
| M1         | Small      | 15s      | 2s       | 7.5x    | ~20W        |

### Memory Requirements by Model Size

| Whisper Model | VRAM Required | Recommended GPUs |
|---------------|---------------|------------------|
| Tiny (39M)    | ~1 GB        | Any modern GPU   |
| Base (74M)    | ~1.5 GB      | GTX 1650+, RX 5500+ |
| Small (244M)  | ~2 GB        | GTX 1660+, RX 5600+ |
| Medium (769M) | ~3 GB        | RTX 3060+, RX 6600+ |
| Large (1550M) | ~5 GB        | RTX 3070+, RX 6700+ |

**Consumer GPU Memory Considerations:**
- RTX 3060: 12GB (can run all models comfortably)
- RTX 3070/3070Ti: 8GB (all models, but large needs optimization)
- RTX 3080: 10GB/12GB variants (all models with headroom)
- RX 6600: 8GB (up to medium comfortably)
- RX 6700 XT: 12GB (all models)
- Arc A750: 8GB (up to medium/large with optimization)

**Memory Optimization Strategies:**
- Dynamic model loading based on available VRAM
- Automatic downgrade to smaller model if VRAM insufficient
- Model unloading after 5 minutes of inactivity
- Shared memory between CPU/GPU for hybrid processing

### Platform Support
- ✅ NVIDIA GPUs (CUDA 11.x/12.x) - GTX 10 series and newer
- ✅ AMD GPUs (ROCm 5.x/6.x) - RX 5000 series and newer
- ✅ Intel GPUs (OpenVINO) - Arc discrete & Xe integrated
- ✅ Apple Silicon (Metal) - All M-series chips
- ✅ Vulkan fallback for older/unsupported GPUs

## Configuration Options

```json
{
  "audio": {
    "speech": {
      "gpu": {
        "enabled": true,
        "backend": "auto",
        "device_index": 0,
        "max_memory_mb": 4096,
        "fallback_to_cpu": true,
        "batch_size": 4,
        "fp16_precision": true
      }
    }
  }
}
```

## Success Criteria
1. [ ] GPU acceleration working on 3+ platforms
2. [ ] 5x+ performance improvement for medium/large models
3. [ ] Automatic GPU detection and selection
4. [ ] Graceful fallback to CPU
5. [ ] Memory usage within specified limits
6. [ ] No degradation in transcription quality
7. [ ] UI shows GPU status and controls

## Risks & Mitigations

### Technical Risks
1. **GPU driver compatibility**
   - Mitigation: Support multiple CUDA versions, provide clear error messages

2. **Memory limitations**
   - Mitigation: Dynamic model loading, batch size adjustment

3. **Platform-specific bugs**
   - Mitigation: Extensive testing, gradual rollout

### Dependencies & Driver Requirements

**NVIDIA:**
- Driver: 525.60+ (Linux), 527.41+ (Windows)
- CUDA Toolkit: 11.8+ or 12.x
- cuDNN: 8.6+

**AMD:**
- Driver: AMDGPU-PRO 23.20+ or ROCm 5.7+
- ROCm Runtime: 5.7+ (Linux), HIP SDK (Windows)
- MIOpen: 2.20+

**Intel:**
- Driver: Intel Graphics Driver 31.0.101.4255+
- OpenVINO: 2023.2+
- Level Zero: 1.3+

**Consumer GPU Specific Notes:**
- Laptop GPUs (mobile variants) typically have 10-20% lower performance
- Ensure adequate cooling for sustained workloads
- Power limit adjustments may improve performance/efficiency balance
- Multi-GPU setups can be used for batch processing

## Timeline Summary
- **Days 1-2**: Research and setup
- **Days 3-5**: Core implementation
- **Days 6-7**: Integration and optimization
- **Days 8-9**: UI and testing
- **Total**: 9 working days

## Next Steps After GPU Acceleration
1. Performance profiling of entire application
2. Real-world usage testing
3. Documentation update
4. Consider cloud GPU options for web version