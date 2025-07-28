//! GPU acceleration for Whisper transcription
//! 
//! This module provides GPU backend implementations for accelerated
//! speech-to-text processing on consumer GPUs.

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(feature = "gpu-cuda")]
mod cuda;
#[cfg(feature = "gpu-metal")]
mod metal;
// OpenCL and ROCm modules removed - not supported by whisper-rs
// #[cfg(feature = "gpu-opencl")]
// mod opencl;
// #[cfg(feature = "gpu-rocm")]
// mod rocm;

// Re-exports
#[cfg(feature = "gpu-cuda")]
pub use cuda::CudaBackend;
#[cfg(feature = "gpu-metal")]
pub use metal::MetalBackend;
// OpenCL and ROCm exports removed
// #[cfg(feature = "gpu-opencl")]
// pub use opencl::OpenCLBackend;
// #[cfg(feature = "gpu-rocm")]
// pub use rocm::RocmBackend;

/// Information about a GPU device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device name (e.g., "NVIDIA GeForce RTX 3080")
    pub name: String,
    /// Total memory in MB
    pub memory_mb: usize,
    /// Available memory in MB
    pub available_memory_mb: usize,
    /// Backend type (CUDA, Metal, etc.)
    pub backend: String,
    /// Whether this is a discrete GPU
    pub is_discrete: bool,
    /// Compute capability (for CUDA)
    pub compute_capability: Option<(u32, u32)>,
    /// Device index
    pub device_index: usize,
}

/// GPU backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuBackend {
    Cuda,
    Metal,
    Cpu, // Fallback
}

impl GpuBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            GpuBackend::Cuda => "CUDA",
            GpuBackend::Metal => "Metal",
            GpuBackend::Cpu => "CPU",
        }
    }
}

/// Trait for GPU-accelerated transcription
pub trait GpuAccelerator: Send + Sync {
    /// Check if this backend is available on the system
    fn is_available(&self) -> bool;
    
    /// Get information about the GPU device
    fn get_device_info(&self) -> DeviceInfo;
    
    /// Get the backend type
    fn get_backend_type(&self) -> GpuBackend;
    
    /// Initialize the GPU backend
    fn initialize(&mut self) -> Result<()>;
    
    /// Load a Whisper model onto the GPU
    fn load_model(&mut self, model_path: &Path) -> Result<()>;
    
    /// Transcribe audio using GPU acceleration
    fn transcribe(&self, audio: &[f32], sample_rate: u32) -> Result<crate::audio::transcribe::Transcript>;
    
    /// Get current GPU memory usage
    fn get_memory_usage(&self) -> Result<(usize, usize)>; // (used, total)
    
    /// Unload model from GPU memory
    fn unload_model(&mut self) -> Result<()>;
}

/// GPU manager for handling multiple backends
pub struct GpuManager {
    backends: Vec<Box<dyn GpuAccelerator>>,
    selected_backend: Option<GpuBackend>,
    config: GpuConfig,
}

/// GPU configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// Enable GPU acceleration
    pub enabled: bool,
    /// Preferred backend (None for auto-select)
    pub preferred_backend: Option<String>,
    /// GPU device index
    pub device_index: Option<usize>,
    /// Maximum GPU memory to use (MB)
    pub max_memory_mb: Option<usize>,
    /// Enable fallback to CPU
    pub fallback_to_cpu: bool,
    /// Batch size for GPU processing
    pub batch_size: usize,
    /// Use FP16 precision
    pub fp16_precision: bool,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preferred_backend: None,
            device_index: Some(0),
            max_memory_mb: Some(4096),
            fallback_to_cpu: true,
            batch_size: 4,
            fp16_precision: true,
        }
    }
}

impl GpuManager {
    /// Create a new GPU manager
    pub fn new(config: GpuConfig) -> Result<Self> {
        let mut backends: Vec<Box<dyn GpuAccelerator>> = Vec::new();
        
        // Try to initialize available backends
        #[cfg(feature = "gpu-cuda")]
        {
            match CudaBackend::new(config.device_index.unwrap_or(0)) {
                Ok(cuda) => {
                    info!("CUDA backend available: {}", cuda.get_device_info().name);
                    backends.push(Box::new(cuda));
                }
                Err(e) => {
                    debug!("CUDA backend not available: {}", e);
                }
            }
        }
        
        #[cfg(feature = "gpu-metal")]
        {
            match MetalBackend::new() {
                Ok(metal) => {
                    info!("Metal backend available");
                    backends.push(Box::new(metal));
                }
                Err(e) => {
                    debug!("Metal backend not available: {}", e);
                }
            }
        }
        
        // ROCm and OpenCL backends removed - not supported by whisper-rs
        
        // Select best backend
        let selected_backend = Self::select_best_backend(&backends, &config);
        
        if let Some(backend_type) = selected_backend {
            info!("Selected GPU backend: {:?}", backend_type);
        } else if config.fallback_to_cpu {
            warn!("No GPU backend available, will use CPU fallback");
        } else {
            return Err(anyhow::anyhow!("No GPU backend available and CPU fallback disabled"));
        }
        
        Ok(Self {
            backends,
            selected_backend,
            config,
        })
    }
    
    /// Select the best available backend
    fn select_best_backend(
        backends: &[Box<dyn GpuAccelerator>],
        config: &GpuConfig,
    ) -> Option<GpuBackend> {
        if backends.is_empty() {
            return None;
        }
        
        // If user specified a preference, try to use it
        if let Some(preferred) = &config.preferred_backend {
            for backend in backends {
                if backend.get_backend_type().as_str() == preferred {
                    return Some(backend.get_backend_type());
                }
            }
        }
        
        // Otherwise, score backends and pick the best
        let mut best_backend = None;
        let mut best_score = 0;
        
        for backend in backends {
            if !backend.is_available() {
                continue;
            }
            
            let info = backend.get_device_info();
            let mut score = 0;
            
            // Prefer discrete GPUs
            if info.is_discrete {
                score += 1000;
            }
            
            // Score by available VRAM
            score += (info.available_memory_mb / 1024) * 100; // Points per GB
            
            // Backend preference (native > compatibility)
            match backend.get_backend_type() {
                GpuBackend::Cuda | GpuBackend::Metal => score += 500,
                GpuBackend::Cpu => score += 0,
            }
            
            // Check memory requirements
            if let Some(max_mem) = config.max_memory_mb {
                if info.available_memory_mb < max_mem {
                    score /= 2; // Penalize if insufficient memory
                }
            }
            
            if score > best_score {
                best_score = score;
                best_backend = Some(backend.get_backend_type());
            }
        }
        
        best_backend
    }
    
    /// Get the selected backend
    pub fn get_selected_backend(&self) -> Option<GpuBackend> {
        self.selected_backend
    }
    
    /// Get information about all available GPUs
    pub fn get_all_devices(&self) -> Vec<DeviceInfo> {
        self.backends
            .iter()
            .filter(|b| b.is_available())
            .map(|b| b.get_device_info())
            .collect()
    }
    
    /// Get the active GPU accelerator
    pub fn get_accelerator(&mut self) -> Option<&mut Box<dyn GpuAccelerator>> {
        if let Some(backend_type) = self.selected_backend {
            self.backends
                .iter_mut()
                .find(|b| b.get_backend_type() == backend_type)
        } else {
            None
        }
    }
    
    /// Initialize the selected backend
    pub fn initialize(&mut self) -> Result<()> {
        if let Some(accelerator) = self.get_accelerator() {
            accelerator.initialize()
                .context("Failed to initialize GPU backend")?;
        }
        Ok(())
    }
    
    /// Load a model onto the GPU
    pub fn load_model(&mut self, model_path: &Path) -> Result<()> {
        if let Some(accelerator) = self.get_accelerator() {
            accelerator.load_model(model_path)
                .context("Failed to load model on GPU")?;
        }
        Ok(())
    }
    
    /// Transcribe audio using GPU acceleration
    pub fn transcribe(&mut self, audio: &[f32], sample_rate: u32) -> Result<crate::audio::transcribe::Transcript> {
        if let Some(accelerator) = self.get_accelerator() {
            accelerator.transcribe(audio, sample_rate)
                .context("GPU transcription failed")
        } else if self.config.fallback_to_cpu {
            // CPU fallback will be handled by the caller
            Err(anyhow::anyhow!("No GPU backend available"))
        } else {
            Err(anyhow::anyhow!("GPU acceleration required but not available"))
        }
    }
    
    /// Get current GPU memory usage
    pub fn get_memory_usage(&self) -> Result<(usize, usize)> {
        if let Some(backend_type) = self.selected_backend {
            if let Some(backend) = self.backends.iter().find(|b| b.get_backend_type() == backend_type) {
                return backend.get_memory_usage();
            }
        }
        Ok((0, 0))
    }
}

/// Detect available GPU backends on the system
pub fn detect_available_backends() -> Vec<GpuBackend> {
    let mut backends = vec![];
    
    #[cfg(feature = "gpu-cuda")]
    {
        if let Ok(cuda) = CudaBackend::new(0) {
            if cuda.is_available() {
                backends.push(GpuBackend::Cuda);
            }
        }
    }
    
    #[cfg(feature = "gpu-metal")]
    {
        if let Ok(metal) = MetalBackend::new() {
            if metal.is_available() {
                backends.push(GpuBackend::Metal);
            }
        }
    }
    
    // ROCm and OpenCL detection removed
    
    backends
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod gpu_integration_tests;

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_gpu_config_default() {
        let config = GpuConfig::default();
        assert!(config.enabled);
        assert!(config.fallback_to_cpu);
        assert_eq!(config.batch_size, 4);
    }
    
    #[test]
    fn test_backend_detection() {
        let backends = detect_available_backends();
        println!("Available GPU backends: {:?}", backends);
        // At least CPU should always be available as fallback
    }
}