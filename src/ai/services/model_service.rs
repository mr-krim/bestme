use crate::ai::{Result, AIError, AIProvider, EnhancementOptions, EnhancedText};
use crate::ai::models::{ModelMetadata, ModelFormat};
use crate::ai::models::registry::ModelRegistry;
use crate::ai::local::onnx_runtime::{OnnxRuntimeModel, OnnxModelConfig, OnnxModelType};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ModelService {
    registry: Arc<ModelRegistry>,
    loaded_models: Arc<RwLock<HashMap<String, Arc<dyn AIModel>>>>,
    gpu_manager: Arc<GPUMemoryManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedModelInfo {
    pub model_id: String,
    pub memory_usage_mb: u64,
    pub backend: String,
    pub load_time_ms: u64,
}

pub trait AIModel: Send + Sync {
    fn model_id(&self) -> &str;
    fn memory_usage(&self) -> u64;
    fn enhance_text(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> impl std::future::Future<Output = Result<EnhancedText>> + Send;
}

#[derive(Debug)]
pub struct GPUMemoryManager {
    total_vram: u64,
    allocated: u64,
    model_cache: Arc<RwLock<Vec<(String, u64)>>>, // (model_id, size)
}

impl ModelService {
    pub async fn new() -> Result<Self> {
        let registry = Arc::new(ModelRegistry::new().await?);
        let gpu_manager = Arc::new(GPUMemoryManager::new()?);
        
        Ok(Self {
            registry,
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
            gpu_manager,
        })
    }

    pub async fn load_model(&self, model_id: &str) -> Result<Arc<dyn AIModel>> {
        // Check if already loaded
        {
            let models = self.loaded_models.read().await;
            if let Some(model) = models.get(model_id) {
                log::info!("Model {} already loaded", model_id);
                return Ok(model.clone());
            }
        }

        // Get model metadata
        let metadata = self.registry.get_metadata(model_id).await?;
        
        // Check if model is downloaded
        let model_path = self.registry.get_model_path(model_id);
        if !model_path.exists() {
            log::info!("Model {} not found locally, downloading...", model_id);
            self.registry.download_model(model_id).await?;
        }
        
        // Check if optimized version exists
        let optimized_path = model_path.with_extension("optimized.onnx");
        let final_model_path = if optimized_path.exists() {
            log::info!("Using optimized model at {:?}", optimized_path);
            optimized_path
        } else if metadata.performance.avg_latency_ms > 100.0 {
            // Optimize models with high latency
            log::info!("Optimizing model for better performance...");
            self.optimize_model(&model_path, &optimized_path).await?;
            optimized_path
        } else {
            model_path.clone()
        };

        // Check GPU memory
        if !self.gpu_manager.can_load_model(metadata.size_bytes).await? {
            log::info!("Insufficient GPU memory, attempting to free space...");
            self.gpu_manager.evict_lru_model().await?;
        }

        // Load model based on format
        use crate::ai::telemetry::{metrics, tracing};
        
        let tracer = tracing::get_tracer();
        let mut load_span = tracer.start_model_load_span(model_id);
        
        let start_time = std::time::Instant::now();
        let model = match self.load_model_from_path(&metadata, &final_model_path).await {
            Ok(m) => m,
            Err(e) => {
                load_span.record_error(&e.to_string());
                return Err(e);
            }
        };
        let load_time_ms = start_time.elapsed().as_millis() as u64;

        log::info!(
            "Loaded model {} in {}ms, memory usage: {} MB",
            model_id,
            load_time_ms,
            model.memory_usage() / (1024 * 1024)
        );

        // Record model load metrics
        let meter = opentelemetry::global::meter("bestme-ai");
        let aggregator = metrics::get_aggregator(&meter);
        let metrics_collector = aggregator.get_model_collector(model_id, &meter);
        metrics_collector.record_model_loaded(start_time.elapsed());
        
        // Record optimization if applied
        if optimized_path.exists() {
            load_span.record_optimization("dynamic");
        }

        // Wrap model with caching if enabled
        let final_model = if std::env::var("BESTME_ENABLE_MODEL_CACHE").unwrap_or_else(|_| "true".to_string()) == "true" {
            use crate::ai::local::model_warmup::{CachedModel, WarmupConfig};
            
            log::info!("Enabling model caching and warmup");
            let cached_model = CachedModel::new(model.clone(), WarmupConfig::default()).await?;
            Arc::new(cached_model) as Arc<dyn AIModel>
        } else {
            model
        };

        // Cache the loaded model
        {
            let mut models = self.loaded_models.write().await;
            models.insert(model_id.to_string(), final_model.clone());
        }

        // Update GPU memory tracking
        self.gpu_manager.allocate_memory(model_id, final_model.memory_usage()).await?;
        
        // Complete load span
        let memory_mb = final_model.memory_usage() as f64 / (1024.0 * 1024.0);
        load_span.record_model_size(memory_mb);
        load_span.complete(memory_mb);

        Ok(final_model)
    }

    async fn load_model_from_path(
        &self,
        metadata: &ModelMetadata,
        path: &PathBuf,
    ) -> Result<Arc<dyn AIModel>> {
        match metadata.format {
            ModelFormat::ONNX => {
                log::info!("Loading ONNX model from {:?}", path);
                
                // Determine model type based on capabilities
                let model_type = if metadata.name.contains("t5") || metadata.name.contains("T5") {
                    OnnxModelType::Seq2Seq
                } else if metadata.capabilities.contains(&crate::ai::models::Capability::GrammarCorrection) {
                    OnnxModelType::Seq2Seq
                } else {
                    OnnxModelType::CausalLM
                };
                
                // Configure based on GPU availability
                let config = OnnxModelConfig {
                    max_length: 512,
                    batch_size: metadata.performance.max_batch_size,
                    use_gpu: self.gpu_manager.available_memory() > 0,
                    device_id: 0,
                    num_threads: num_cpus::get() as i16,
                    memory_pattern: true,
                    model_type,
                };
                
                // Look for tokenizer file
                let tokenizer_path = path.with_extension("json");
                if !tokenizer_path.exists() {
                    // Try in the same directory with tokenizer.json name
                    let tokenizer_path = path.parent().unwrap().join("tokenizer.json");
                    if !tokenizer_path.exists() {
                        return Err(AIError::ConfigError(
                            format!("Tokenizer not found for model {}", metadata.id)
                        ));
                    }
                }
                
                let model = OnnxRuntimeModel::new(
                    metadata.id.clone(),
                    path,
                    &tokenizer_path,
                    config,
                ).await?;
                
                Ok(Arc::new(model) as Arc<dyn AIModel>)
            }
            ModelFormat::Safetensors => {
                // TODO: Implement Candle model loading
                Err(AIError::ConfigError("Safetensors loading not yet implemented".to_string()))
            }
            _ => {
                Err(AIError::ConfigError(format!("Unsupported model format: {:?}", metadata.format)))
            }
        }
    }

    pub async fn unload_model(&self, model_id: &str) -> Result<()> {
        let mut models = self.loaded_models.write().await;
        if let Some(model) = models.remove(model_id) {
            let memory = model.memory_usage();
            drop(model); // Explicitly drop to free resources
            self.gpu_manager.free_memory(model_id, memory).await?;
            log::info!("Unloaded model {}", model_id);
        }
        Ok(())
    }

    pub async fn list_loaded_models(&self) -> Vec<LoadedModelInfo> {
        let models = self.loaded_models.read().await;
        models.iter().map(|(id, model)| LoadedModelInfo {
            model_id: id.clone(),
            memory_usage_mb: model.memory_usage() / (1024 * 1024),
            backend: "ONNX".to_string(), // TODO: Get actual backend
            load_time_ms: 0, // TODO: Track actual load time
        }).collect()
    }

    pub fn get_registry(&self) -> Arc<ModelRegistry> {
        self.registry.clone()
    }
    
    pub fn get_gpu_manager(&self) -> Arc<GPUMemoryManager> {
        self.gpu_manager.clone()
    }
    
    pub async fn get_gpu_info(&self) -> Vec<crate::ai::gpu::AIGpuInfo> {
        let detector = crate::ai::gpu::get_gpu_detector();
        detector.detect_backends()
    }
    
    async fn optimize_model(&self, input_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
        use crate::ai::local::model_optimizer::{ModelOptimizer, OptimizationPresets};
        
        // Select optimization config based on GPU availability
        let config = if self.gpu_manager.available_memory() > 0 {
            OptimizationPresets::gpu_performance()
        } else {
            OptimizationPresets::cpu_fast()
        };
        
        let optimizer = ModelOptimizer::new(config);
        let result = optimizer.optimize_model(input_path, output_path).await?;
        
        log::info!(
            "Model optimization complete: {:.1}% size reduction, optimizations: {:?}",
            result.size_reduction_percent,
            result.optimizations_applied
        );
        
        Ok(())
    }
}

impl GPUMemoryManager {
    pub fn new() -> Result<Self> {
        // Detect available GPU memory using the new GPU detector
        let gpu_detector = crate::ai::gpu::get_gpu_detector();
        let backends = gpu_detector.detect_backends();
        
        // Find the best GPU backend
        let best_gpu = backends.iter()
            .filter(|b| b.backend.is_gpu())
            .max_by_key(|b| b.available_memory_mb);
        
        let total_vram = if let Some(gpu) = best_gpu {
            log::info!("Using {} with {}MB VRAM", gpu.device_name, gpu.memory_mb);
            gpu.memory_mb * 1024 * 1024 // Convert to bytes
        } else {
            log::warn!("No GPU detected, using CPU memory limits");
            2 * 1024 * 1024 * 1024 // 2GB limit for CPU
        };
        
        Ok(Self {
            total_vram,
            allocated: 0,
            model_cache: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn can_load_model(&self, model_size: u64) -> Result<bool> {
        let required = (model_size as f64 * 1.2) as u64; // 20% overhead
        Ok(self.available_memory() >= required)
    }

    pub fn available_memory(&self) -> u64 {
        self.total_vram.saturating_sub(self.allocated)
    }

    pub async fn allocate_memory(&self, model_id: &str, size: u64) -> Result<()> {
        let mut cache = self.model_cache.write().await;
        cache.push((model_id.to_string(), size));
        // Note: In a real implementation, we'd update self.allocated atomically
        Ok(())
    }

    pub async fn free_memory(&self, model_id: &str, _size: u64) -> Result<()> {
        let mut cache = self.model_cache.write().await;
        cache.retain(|(id, _)| id != model_id);
        // Note: In a real implementation, we'd update self.allocated atomically
        Ok(())
    }

    pub async fn evict_lru_model(&self) -> Result<()> {
        let cache = self.model_cache.read().await;
        if let Some((model_id, _)) = cache.first() {
            let model_id = model_id.clone();
            drop(cache); // Release read lock
            
            log::info!("Evicting LRU model: {}", model_id);
            // TODO: Actually evict the model through ModelService
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_service_creation() {
        let service = ModelService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_gpu_memory_manager() {
        let manager = GPUMemoryManager::new().unwrap();
        assert!(manager.total_vram > 0);
        assert_eq!(manager.available_memory(), manager.total_vram);
    }
}