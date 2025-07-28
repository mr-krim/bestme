//! GPU configuration for Whisper transcription

use serde::{Deserialize, Serialize};
use log::{info, warn};

/// GPU configuration for transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// Enable GPU acceleration
    pub enabled: bool,
    
    /// Show GPU info in UI
    pub show_gpu_info: bool,
    
    /// Log GPU memory usage
    pub log_memory_usage: bool,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_gpu_info: true,
            log_memory_usage: false,
        }
    }
}

/// Check which GPU backend is available at compile time
pub fn get_available_gpu_backend() -> Option<&'static str> {
    #[cfg(feature = "gpu-cuda")]
    {
        return Some("CUDA");
    }
    
    #[cfg(feature = "gpu-metal")]
    {
        return Some("Metal");
    }
    
    None
}

/// Get GPU status information
pub fn get_gpu_status() -> GpuStatus {
    let backend = get_available_gpu_backend();
    
    let mut status = GpuStatus {
        gpu_available: backend.is_some(),
        backend: backend.map(|s| s.to_string()),
        device_name: None,
        memory_mb: None,
        driver_version: None,
    };
    
    // Try to get actual GPU info if available
    #[cfg(feature = "gpu-cuda")]
    {
        if let Some(info) = get_cuda_info() {
            status.device_name = Some(info.0);
            status.memory_mb = Some(info.1);
            status.driver_version = info.2;
        }
    }
    
    #[cfg(feature = "gpu-metal")]
    {
        if let Some(info) = get_metal_info() {
            status.device_name = Some(info.0);
            status.memory_mb = Some(info.1);
        }
    }
    
    status
}

/// GPU status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    /// Whether GPU acceleration is available
    pub gpu_available: bool,
    
    /// Backend name (CUDA, Metal, etc.)
    pub backend: Option<String>,
    
    /// GPU device name
    pub device_name: Option<String>,
    
    /// Total GPU memory in MB
    pub memory_mb: Option<usize>,
    
    /// Driver version
    pub driver_version: Option<String>,
}

#[cfg(feature = "gpu-cuda")]
fn get_cuda_info() -> Option<(String, usize, Option<String>)> {
    use nvml_wrapper::Nvml;
    
    match Nvml::init() {
        Ok(nvml) => {
            if let Ok(device) = nvml.device_by_index(0) {
                let name = device.name().ok()?;
                let memory = device.memory_info().ok()?.total / (1024 * 1024);
                let driver = nvml.sys_driver_version().ok();
                
                info!("CUDA GPU detected: {} ({}MB)", name, memory);
                Some((name, memory as usize, driver))
            } else {
                warn!("CUDA runtime available but no devices found");
                None
            }
        }
        Err(e) => {
            warn!("Failed to initialize NVML: {}", e);
            None
        }
    }
}

#[cfg(feature = "gpu-metal")]
fn get_metal_info() -> Option<(String, usize)> {
    // For Metal, we can't easily query device info without creating a device
    // Return generic info based on system
    use std::process::Command;
    
    let name = if let Ok(output) = Command::new("sysctl")
        .arg("-n")
        .arg("machdep.cpu.brand_string")
        .output()
    {
        let cpu_info = String::from_utf8_lossy(&output.stdout);
        if cpu_info.contains("M1") {
            "Apple M1 GPU"
        } else if cpu_info.contains("M2") {
            "Apple M2 GPU"
        } else if cpu_info.contains("M3") {
            "Apple M3 GPU"
        } else {
            "Apple Silicon GPU"
        }
    } else {
        "Metal GPU"
    };
    
    // Get total system memory as unified memory
    let memory = if let Ok(output) = Command::new("sysctl")
        .arg("-n")
        .arg("hw.memsize")
        .output()
    {
        if let Ok(mem_str) = String::from_utf8(output.stdout) {
            if let Ok(mem_bytes) = mem_str.trim().parse::<usize>() {
                mem_bytes / (1024 * 1024)
            } else {
                8192
            }
        } else {
            8192
        }
    } else {
        8192
    };
    
    info!("Metal GPU detected: {} ({}MB unified memory)", name, memory);
    Some((name.to_string(), memory))
}

/// Log GPU memory usage if enabled
pub fn log_gpu_memory_usage(config: &GpuConfig) {
    if !config.log_memory_usage {
        return;
    }
    
    #[cfg(feature = "gpu-cuda")]
    {
        use nvml_wrapper::Nvml;
        
        if let Ok(nvml) = Nvml::init() {
            if let Ok(device) = nvml.device_by_index(0) {
                if let Ok(mem_info) = device.memory_info() {
                    let used_mb = (mem_info.total - mem_info.free) / (1024 * 1024);
                    let total_mb = mem_info.total / (1024 * 1024);
                    info!("GPU memory: {}MB / {}MB ({:.1}%)", 
                        used_mb, 
                        total_mb,
                        (used_mb as f64 / total_mb as f64) * 100.0
                    );
                }
            }
        }
    }
}