pub mod backend_selector;

use crate::ai::{Result, AIError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// GPU backend information for AI inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIGpuInfo {
    pub backend: AIGpuBackend,
    pub device_name: String,
    pub device_index: i32,
    pub memory_mb: u64,
    pub available_memory_mb: u64,
    pub compute_capability: Option<(u32, u32)>,
}

/// Supported GPU backends for AI inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIGpuBackend {
    CUDA,
    Metal,
    DirectML,
    CPU,
}

impl AIGpuBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            AIGpuBackend::CUDA => "CUDA",
            AIGpuBackend::Metal => "Metal",
            AIGpuBackend::DirectML => "DirectML",
            AIGpuBackend::CPU => "CPU",
        }
    }
    
    pub fn is_gpu(&self) -> bool {
        !matches!(self, AIGpuBackend::CPU)
    }
}

/// Trait for GPU detection and information
pub trait GpuDetector: Send + Sync {
    /// Detect available GPU backends
    fn detect_backends(&self) -> Vec<AIGpuInfo>;
    
    /// Select the best backend based on criteria
    fn select_best_backend(&self, prefer_gpu: bool) -> AIGpuBackend;
    
    /// Get information about a specific backend
    fn get_backend_info(&self, backend: AIGpuBackend) -> Option<AIGpuInfo>;
}

/// Get the default GPU detector implementation
pub fn get_gpu_detector() -> Arc<dyn GpuDetector> {
    Arc::new(backend_selector::DefaultGpuDetector::new())
}