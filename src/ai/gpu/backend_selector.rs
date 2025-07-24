use super::{AIGpuBackend, AIGpuInfo, GpuDetector};
use log::{debug, info, warn};
use std::sync::Arc;

/// Default GPU detector implementation
pub struct DefaultGpuDetector {
    backends: Vec<AIGpuInfo>,
}

impl DefaultGpuDetector {
    pub fn new() -> Self {
        let backends = Self::detect_all_backends();
        Self { backends }
    }
    
    fn detect_all_backends() -> Vec<AIGpuInfo> {
        let mut backends = Vec::new();
        
        // Detect CUDA
        #[cfg(feature = "gpu-cuda")]
        {
            if let Some(cuda_info) = Self::detect_cuda() {
                backends.push(cuda_info);
            }
        }
        
        // Detect Metal (macOS)
        #[cfg(all(feature = "gpu-metal", target_os = "macos"))]
        {
            if let Some(metal_info) = Self::detect_metal() {
                backends.push(metal_info);
            }
        }
        
        // Detect DirectML (Windows)
        #[cfg(target_os = "windows")]
        {
            if let Some(directml_info) = Self::detect_directml() {
                backends.push(directml_info);
            }
        }
        
        // Always add CPU as fallback
        backends.push(Self::get_cpu_info());
        
        info!("Detected {} backends for AI inference", backends.len());
        for backend in &backends {
            info!("  - {} ({}MB VRAM)", backend.device_name, backend.memory_mb);
        }
        
        backends
    }
    
    #[cfg(feature = "gpu-cuda")]
    fn detect_cuda() -> Option<AIGpuInfo> {
        // Try to detect NVIDIA GPU using nvml
        #[cfg(feature = "nvml-wrapper")]
        {
            use nvml_wrapper::Nvml;
            
            match Nvml::init() {
                Ok(nvml) => {
                    match nvml.device_count() {
                        Ok(count) if count > 0 => {
                            // Get first device info
                            if let Ok(device) = nvml.device_by_index(0) {
                                let name = device.name().unwrap_or_else(|_| "Unknown NVIDIA GPU".to_string());
                                let memory = device.memory_info()
                                    .map(|m| m.total / 1024 / 1024)
                                    .unwrap_or(0);
                                let available = device.memory_info()
                                    .map(|m| m.free / 1024 / 1024)
                                    .unwrap_or(0);
                                
                                // Get compute capability
                                let compute_cap = device.cuda_compute_capability()
                                    .ok()
                                    .map(|cc| (cc.major, cc.minor));
                                
                                info!("CUDA GPU detected: {} ({}MB)", name, memory);
                                
                                return Some(AIGpuInfo {
                                    backend: AIGpuBackend::CUDA,
                                    device_name: name,
                                    device_index: 0,
                                    memory_mb: memory,
                                    available_memory_mb: available,
                                    compute_capability: compute_cap,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    debug!("NVML not available: {}", e);
                }
            }
        }
        
        // Fallback: Check if CUDA runtime is available
        if Self::is_cuda_available() {
            info!("CUDA runtime detected (GPU details unavailable)");
            Some(AIGpuInfo {
                backend: AIGpuBackend::CUDA,
                device_name: "NVIDIA GPU (Unknown Model)".to_string(),
                device_index: 0,
                memory_mb: 4096, // Assume 4GB
                available_memory_mb: 2048,
                compute_capability: None,
            })
        } else {
            None
        }
    }
    
    #[cfg(feature = "gpu-cuda")]
    fn is_cuda_available() -> bool {
        // Check if CUDA libraries are present
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/usr/local/cuda/lib64/libcudart.so").exists() ||
            std::path::Path::new("/usr/lib/x86_64-linux-gnu/libcudart.so").exists()
        }
        #[cfg(target_os = "windows")]
        {
            std::path::Path::new("C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA").exists()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }
    
    #[cfg(all(feature = "gpu-metal", target_os = "macos"))]
    fn detect_metal() -> Option<AIGpuInfo> {
        // Metal is always available on modern macOS
        use std::process::Command;
        
        // Try to get GPU info using system_profiler
        let output = Command::new("system_profiler")
            .args(&["SPDisplaysDataType", "-json"])
            .output()
            .ok()?;
        
        if output.status.success() {
            // Parse JSON output to get GPU info
            // For now, return generic Metal info
            info!("Metal GPU detected");
            Some(AIGpuInfo {
                backend: AIGpuBackend::Metal,
                device_name: "Apple GPU".to_string(),
                device_index: 0,
                memory_mb: 8192, // Assume 8GB unified memory
                available_memory_mb: 4096,
                compute_capability: None,
            })
        } else {
            None
        }
    }
    
    #[cfg(target_os = "windows")]
    fn detect_directml() -> Option<AIGpuInfo> {
        // DirectML is available on Windows 10+ with compatible GPU
        // For now, return generic DirectML info if Windows version supports it
        info!("DirectML available for GPU acceleration");
        Some(AIGpuInfo {
            backend: AIGpuBackend::DirectML,
            device_name: "DirectML Compatible GPU".to_string(),
            device_index: 0,
            memory_mb: 4096,
            available_memory_mb: 2048,
            compute_capability: None,
        })
    }
    
    fn get_cpu_info() -> AIGpuInfo {
        let cpu_name = if cfg!(target_arch = "x86_64") {
            "x86_64 CPU"
        } else if cfg!(target_arch = "aarch64") {
            "ARM64 CPU"
        } else {
            "CPU"
        };
        
        AIGpuInfo {
            backend: AIGpuBackend::CPU,
            device_name: cpu_name.to_string(),
            device_index: -1,
            memory_mb: Self::get_system_memory_mb(),
            available_memory_mb: Self::get_available_memory_mb(),
            compute_capability: None,
        }
    }
    
    fn get_system_memory_mb() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb / 1024;
                            }
                        }
                    }
                }
            }
        }
        
        // Default fallback
        8192 // 8GB
    }
    
    fn get_available_memory_mb() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb / 1024;
                            }
                        }
                    }
                }
            }
        }
        
        // Default fallback
        4096 // 4GB
    }
}

impl GpuDetector for DefaultGpuDetector {
    fn detect_backends(&self) -> Vec<AIGpuInfo> {
        self.backends.clone()
    }
    
    fn select_best_backend(&self, prefer_gpu: bool) -> AIGpuBackend {
        if !prefer_gpu {
            return AIGpuBackend::CPU;
        }
        
        // Score each backend
        let mut best_backend = AIGpuBackend::CPU;
        let mut best_score = 0;
        
        for info in &self.backends {
            if !info.backend.is_gpu() {
                continue;
            }
            
            let mut score = 0;
            
            // Base score by backend type
            score += match info.backend {
                AIGpuBackend::CUDA => 1000,  // Preferred for ML
                AIGpuBackend::Metal => 900,  // Native on macOS
                AIGpuBackend::DirectML => 800, // Good compatibility
                AIGpuBackend::CPU => 0,
            };
            
            // Score by available memory (100 points per GB)
            score += (info.available_memory_mb / 1024) as i32 * 100;
            
            // Bonus for newer compute capability
            if let Some((major, _)) = info.compute_capability {
                score += major as i32 * 50;
            }
            
            if score > best_score {
                best_score = score;
                best_backend = info.backend;
            }
        }
        
        info!("Selected backend: {} (score: {})", best_backend.as_str(), best_score);
        best_backend
    }
    
    fn get_backend_info(&self, backend: AIGpuBackend) -> Option<AIGpuInfo> {
        self.backends.iter()
            .find(|info| info.backend == backend)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_detection() {
        let detector = DefaultGpuDetector::new();
        let backends = detector.detect_backends();
        
        // Should always have at least CPU
        assert!(!backends.is_empty());
        assert!(backends.iter().any(|b| b.backend == AIGpuBackend::CPU));
    }
    
    #[test]
    fn test_backend_selection() {
        let detector = DefaultGpuDetector::new();
        
        // CPU should be selected when prefer_gpu is false
        let cpu_backend = detector.select_best_backend(false);
        assert_eq!(cpu_backend, AIGpuBackend::CPU);
    }
}