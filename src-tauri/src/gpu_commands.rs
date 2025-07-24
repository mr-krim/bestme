//! GPU-related Tauri commands

use serde::{Deserialize, Serialize};
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub available: bool,
    pub backend: Option<String>,
    pub device_name: Option<String>,
    pub memory_mb: Option<usize>,
    pub driver_version: Option<String>,
}

/// Get GPU information
#[tauri::command]
pub async fn get_gpu_info() -> Result<GpuInfo, String> {
    info!("Getting GPU information");
    
    #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal", feature = "gpu-vulkan", feature = "gpu-hipblas"))]
    {
        use bestme::audio::gpu_config;
        
        let status = gpu_config::get_gpu_status();
        
        Ok(GpuInfo {
            available: status.gpu_available,
            backend: status.backend,
            device_name: status.device_name,
            memory_mb: status.memory_mb,
            driver_version: status.driver_version,
        })
    }
    
    #[cfg(not(any(feature = "gpu-cuda", feature = "gpu-metal", feature = "gpu-vulkan", feature = "gpu-hipblas")))]
    {
        Ok(GpuInfo {
            available: false,
            backend: None,
            device_name: None,
            memory_mb: None,
            driver_version: None,
        })
    }
}

/// Check if GPU acceleration is enabled
#[tauri::command]
pub async fn is_gpu_enabled() -> bool {
    #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal", feature = "gpu-vulkan", feature = "gpu-hipblas"))]
    {
        true
    }
    
    #[cfg(not(any(feature = "gpu-cuda", feature = "gpu-metal", feature = "gpu-vulkan", feature = "gpu-hipblas")))]
    {
        false
    }
}

/// Get available GPU backends
#[tauri::command]
pub async fn get_available_gpu_backends() -> Vec<String> {
    let mut backends = Vec::new();
    
    #[cfg(feature = "gpu-cuda")]
    backends.push("CUDA".to_string());
    
    #[cfg(feature = "gpu-metal")]
    backends.push("Metal".to_string());
    
    #[cfg(feature = "gpu-vulkan")]
    backends.push("Vulkan".to_string());
    
    #[cfg(feature = "gpu-hipblas")]
    backends.push("HIP/ROCm".to_string());
    
    backends
}

/// Run a quick GPU benchmark
#[tauri::command]
pub async fn run_gpu_benchmark() -> Result<String, String> {
    info!("Running GPU benchmark");
    
    #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal", feature = "gpu-vulkan", feature = "gpu-hipblas"))]
    {
        use bestme::audio::gpu_benchmark;
        
        match gpu_benchmark::run_quick_benchmark().await {
            Ok(report) => Ok(report),
            Err(e) => Err(format!("Benchmark failed: {}", e))
        }
    }
    
    #[cfg(not(any(feature = "gpu-cuda", feature = "gpu-metal", feature = "gpu-vulkan", feature = "gpu-hipblas")))]
    {
        Err("GPU features not enabled in this build".to_string())
    }
}