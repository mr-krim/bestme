# AI Model Integration - Execution Plan

## Overview
This document outlines the detailed plan for integrating real AI models into BestMe, focusing on Phi-3-mini and Llama-3.2 models with GPU acceleration and performance optimization.

## Phase 1: Model Acquisition and Conversion (Day 1-2)

### 1.1 Model Download Strategy
```bash
# Models to download:
# 1. Microsoft Phi-3-mini (3.8B parameters)
#    - Best for: Grammar correction, general text enhancement
#    - Size: ~7.5GB (FP16), ~3.8GB (Int8)
#    - URL: https://huggingface.co/microsoft/Phi-3-mini-4k-instruct

# 2. Meta Llama-3.2-1B
#    - Best for: Fast inference, basic corrections
#    - Size: ~2GB (FP16), ~1GB (Int8)
#    - URL: https://huggingface.co/meta-llama/Llama-3.2-1B
```

### 1.2 Model Conversion Pipeline
1. **Download original models** in PyTorch/Safetensors format
2. **Convert to ONNX** using optimum library:
   ```python
   from optimum.onnxruntime import ORTModelForCausalLM
   model = ORTModelForCausalLM.from_pretrained("microsoft/Phi-3-mini-4k-instruct", export=True)
   model.save_pretrained("./models/phi3-mini-onnx")
   ```
3. **Optimize ONNX models** for inference:
   - Graph optimization
   - Operator fusion
   - Precision reduction (FP16)

### 1.3 Model Registry Implementation
```rust
// src/ai/models/registry.rs
pub struct ModelRegistry {
    models: HashMap<String, ModelMetadata>,
    download_manager: ModelDownloadManager,
}

pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub format: ModelFormat,
    pub capabilities: Vec<Capability>,
    pub performance: PerformanceProfile,
}
```

## Phase 2: GPU Acceleration Integration (Day 2-3)

### 2.1 Backend Selection Strategy
```rust
// src/ai/gpu/ai_backend.rs
pub enum AIBackend {
    CPU,
    CUDA(CUDAConfig),
    Metal(MetalConfig),
    Vulkan(VulkanConfig),
}

impl AIBackend {
    pub fn auto_select() -> Self {
        if cfg!(target_os = "macos") && metal::is_available() {
            AIBackend::Metal(MetalConfig::default())
        } else if cuda::is_available() {
            AIBackend::CUDA(CUDAConfig::default())
        } else if vulkan::is_available() {
            AIBackend::Vulkan(VulkanConfig::default())
        } else {
            AIBackend::CPU
        }
    }
}
```

### 2.2 ONNX Runtime GPU Providers
1. **CUDA Provider** (NVIDIA GPUs)
   ```rust
   let cuda_options = CUDAExecutionProviderOptions {
       device_id: 0,
       gpu_mem_limit: 2 * 1024 * 1024 * 1024, // 2GB
       arena_extend_strategy: ArenaExtendStrategy::NextPowerOfTwo,
       ..Default::default()
   };
   ```

2. **CoreML Provider** (Apple Silicon)
   ```rust
   let coreml_options = CoreMLExecutionProviderOptions {
       use_cpu_only: false,
       enable_on_subgraph: true,
       ..Default::default()
   };
   ```

3. **DirectML Provider** (Windows/Cross-platform)
   ```rust
   let directml_options = DirectMLExecutionProviderOptions {
       device_id: 0,
       ..Default::default()
   };
   ```

### 2.3 Memory Management
```rust
pub struct GPUMemoryManager {
    total_vram: u64,
    allocated: u64,
    model_cache: LruCache<String, Arc<LoadedModel>>,
}

impl GPUMemoryManager {
    pub fn can_load_model(&self, model_size: u64) -> bool {
        self.available_memory() >= model_size * 1.2 // 20% overhead
    }
    
    pub fn evict_lru_model(&mut self) -> Result<()> {
        // Evict least recently used model to free memory
    }
}
```

## Phase 3: Performance Optimization (Day 3-4)

### 3.1 Model Quantization
```rust
pub enum QuantizationMode {
    None,
    Dynamic,      // Quantize weights on-the-fly
    Static(u8),   // Pre-quantized to Int8/Int4
}

pub async fn quantize_model(
    model_path: &Path,
    mode: QuantizationMode,
) -> Result<PathBuf> {
    match mode {
        QuantizationMode::Static(bits) => {
            // Use ONNX Runtime quantization tools
            let output_path = model_path.with_extension(format!("int{}.onnx", bits));
            quantize_static(model_path, &output_path, bits).await?;
            Ok(output_path)
        }
        _ => Ok(model_path.to_path_buf()),
    }
}
```

### 3.2 Inference Pipeline Optimization
```rust
pub struct OptimizedInferencePipeline {
    model: Arc<ort::Session>,
    tokenizer: Arc<Tokenizer>,
    batch_size: usize,
    max_sequence_length: usize,
    use_kv_cache: bool,
}

impl OptimizedInferencePipeline {
    pub async fn process_batch(&self, texts: Vec<String>) -> Result<Vec<EnhancedText>> {
        // 1. Batch tokenization
        let encodings = self.batch_tokenize(&texts).await?;
        
        // 2. Pad sequences for batching
        let padded = self.pad_sequences(encodings);
        
        // 3. Run inference
        let outputs = self.batch_inference(padded).await?;
        
        // 4. Decode and post-process
        self.decode_outputs(outputs, texts).await
    }
}
```

### 3.3 Streaming Inference
```rust
pub struct StreamingInference {
    pipeline: Arc<OptimizedInferencePipeline>,
    buffer: String,
    min_chunk_size: usize,
}

impl StreamingInference {
    pub async fn process_chunk(&mut self, text: &str) -> Option<String> {
        self.buffer.push_str(text);
        
        if self.buffer.len() >= self.min_chunk_size {
            let result = self.pipeline.process_single(&self.buffer).await.ok()?;
            self.buffer.clear();
            Some(result.enhanced)
        } else {
            None
        }
    }
}
```

## Phase 4: Integration and Testing (Day 4-5)

### 4.1 Model Loading Service
```rust
// src/ai/services/model_service.rs
pub struct ModelService {
    registry: Arc<ModelRegistry>,
    gpu_manager: Arc<GPUMemoryManager>,
    loaded_models: Arc<RwLock<HashMap<String, Arc<dyn AIModel>>>>,
}

impl ModelService {
    pub async fn load_model(&self, model_id: &str) -> Result<Arc<dyn AIModel>> {
        // Check if already loaded
        if let Some(model) = self.get_loaded_model(model_id).await {
            return Ok(model);
        }
        
        // Check GPU memory
        let metadata = self.registry.get_metadata(model_id)?;
        if !self.gpu_manager.can_load_model(metadata.size_bytes) {
            self.gpu_manager.evict_lru_model().await?;
        }
        
        // Load model with appropriate backend
        let backend = AIBackend::auto_select();
        let model = self.load_with_backend(model_id, backend).await?;
        
        // Cache and return
        self.cache_model(model_id, model.clone()).await;
        Ok(model)
    }
}
```

### 4.2 Performance Benchmarks
```rust
// src/ai/benchmarks/mod.rs
pub struct AIBenchmark {
    pub model: String,
    pub backend: String,
    pub quantization: String,
    pub batch_size: usize,
    pub avg_latency_ms: f64,
    pub throughput_tps: f64, // tokens per second
    pub memory_usage_mb: u64,
}

pub async fn run_benchmarks() -> Vec<AIBenchmark> {
    let test_texts = load_test_dataset();
    let mut results = Vec::new();
    
    for model in ["phi-3-mini", "llama-3.2-1b"] {
        for backend in [AIBackend::CPU, AIBackend::auto_select()] {
            for quant in [QuantizationMode::None, QuantizationMode::Static(8)] {
                let result = benchmark_configuration(model, backend, quant, &test_texts).await;
                results.push(result);
            }
        }
    }
    
    results
}
```

### 4.3 Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_model_loading_and_inference() {
        let service = ModelService::new().await.unwrap();
        
        // Load Phi-3 model
        let model = service.load_model("phi-3-mini").await.unwrap();
        
        // Test inference
        let result = model.enhance_text(
            "this is a test sentense with erors",
            &EnhancementOptions::default()
        ).await.unwrap();
        
        assert_eq!(result.enhanced, "This is a test sentence with errors.");
        assert!(result.confidence > 0.8);
    }
    
    #[tokio::test]
    async fn test_gpu_acceleration() {
        let backend = AIBackend::auto_select();
        assert_ne!(backend, AIBackend::CPU, "GPU should be available");
        
        // Benchmark GPU vs CPU
        let gpu_time = benchmark_with_backend(AIBackend::auto_select()).await;
        let cpu_time = benchmark_with_backend(AIBackend::CPU).await;
        
        assert!(gpu_time < cpu_time * 0.5, "GPU should be at least 2x faster");
    }
}
```

## Implementation Timeline

### Day 1: Model Download and Conversion
- [ ] Set up Hugging Face CLI and authentication
- [ ] Download Phi-3-mini and Llama-3.2 models
- [ ] Convert models to ONNX format
- [ ] Optimize ONNX graphs

### Day 2: GPU Backend Integration
- [ ] Implement AIBackend enum and auto-selection
- [ ] Add CUDA execution provider
- [ ] Add CoreML execution provider
- [ ] Add DirectML execution provider

### Day 3: Performance Optimization
- [ ] Implement model quantization
- [ ] Create batched inference pipeline
- [ ] Add streaming inference support
- [ ] Implement memory management

### Day 4: Service Integration
- [ ] Create ModelService with registry
- [ ] Implement model caching
- [ ] Add performance monitoring
- [ ] Create benchmark suite

### Day 5: Testing and Validation
- [ ] Run comprehensive benchmarks
- [ ] Integration tests with real models
- [ ] Performance profiling
- [ ] Documentation updates

## Success Metrics

1. **Performance**
   - Grammar correction latency < 50ms (90th percentile)
   - GPU acceleration provides >2x speedup
   - Memory usage < 2GB per model

2. **Accuracy**
   - Grammar correction accuracy > 95%
   - Intent detection accuracy > 90%
   - No hallucinations in corrections

3. **Reliability**
   - Model loading success rate > 99.9%
   - Graceful fallback to CPU when GPU unavailable
   - No memory leaks during extended use

## Resources

- [Phi-3 Model Card](https://huggingface.co/microsoft/Phi-3-mini-4k-instruct)
- [Llama 3.2 Release](https://ai.meta.com/blog/llama-3-2-connect-2024-vision-edge-mobile-devices/)
- [ONNX Runtime GPU Docs](https://onnxruntime.ai/docs/execution-providers/)
- [Optimum Library](https://huggingface.co/docs/optimum/index)