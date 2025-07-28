//! Metal backend for GPU acceleration on macOS

use anyhow::{Context, Result};
use log::{debug, info};
use std::path::Path;

use super::{DeviceInfo, GpuAccelerator, GpuBackend};
use crate::audio::transcribe::Transcript;

/// Metal backend for Apple Silicon GPUs
pub struct MetalBackend {
    device_info: Option<DeviceInfo>,
    initialized: bool,
}

impl MetalBackend {
    /// Create a new Metal backend
    pub fn new() -> Result<Self> {
        debug!("Initializing Metal backend");
        
        let mut backend = Self {
            device_info: None,
            initialized: false,
        };
        
        // Detect Metal device
        backend.detect_device()?;
        
        Ok(backend)
    }
    
    /// Detect Metal device information
    fn detect_device(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            // Metal is always available on modern macOS
            // Get system info to determine GPU type
            let device_name = Self::get_metal_device_name();
            let memory_mb = Self::get_unified_memory_mb();
            
            self.device_info = Some(DeviceInfo {
                name: device_name,
                memory_mb,
                available_memory_mb: memory_mb * 3 / 4, // Estimate 75% available
                backend: "Metal".to_string(),
                is_discrete: false, // Apple Silicon uses unified memory
                compute_capability: None,
                device_index: 0,
            });
            
            info!("Metal device detected: {} ({}GB unified memory)",
                self.device_info.as_ref().unwrap().name,
                self.device_info.as_ref().unwrap().memory_mb / 1024
            );
            
            Ok(())
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            Err(anyhow::anyhow!("Metal is only available on macOS"))
        }
    }
    
    #[cfg(target_os = "macos")]
    fn get_metal_device_name() -> String {
        // In a real implementation, we would query the actual device
        // For now, we'll use a placeholder that detects the chip type
        use std::process::Command;
        
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
        {
            let cpu_info = String::from_utf8_lossy(&output.stdout);
            if cpu_info.contains("M1") {
                "Apple M1 GPU".to_string()
            } else if cpu_info.contains("M2") {
                "Apple M2 GPU".to_string()
            } else if cpu_info.contains("M3") {
                "Apple M3 GPU".to_string()
            } else if cpu_info.contains("M4") {
                "Apple M4 GPU".to_string()
            } else {
                "Apple Silicon GPU".to_string()
            }
        } else {
            "Apple GPU".to_string()
        }
    }
    
    #[cfg(target_os = "macos")]
    fn get_unified_memory_mb() -> usize {
        // Get total system memory as unified memory
        use std::process::Command;
        
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output()
        {
            if let Ok(mem_str) = String::from_utf8(output.stdout) {
                if let Ok(mem_bytes) = mem_str.trim().parse::<usize>() {
                    return mem_bytes / (1024 * 1024);
                }
            }
        }
        
        // Default to 8GB if detection fails
        8192
    }
}

impl GpuAccelerator for MetalBackend {
    fn is_available(&self) -> bool {
        self.device_info.is_some()
    }
    
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone().unwrap_or(DeviceInfo {
            name: "Unknown Metal Device".to_string(),
            memory_mb: 0,
            available_memory_mb: 0,
            backend: "Metal".to_string(),
            is_discrete: false,
            compute_capability: None,
            device_index: 0,
        })
    }
    
    fn get_backend_type(&self) -> GpuBackend {
        GpuBackend::Metal
    }
    
    fn initialize(&mut self) -> Result<()> {
        if !self.is_available() {
            return Err(anyhow::anyhow!("Metal device not available"));
        }
        
        // In a real implementation, we would create Metal device and queue here
        info!("Metal backend initialized");
        self.initialized = true;
        Ok(())
    }
    
    fn load_model(&mut self, model_path: &Path) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Metal backend not initialized"));
        }
        
        info!("Loading model {} on Metal device", model_path.display());
        
        // Validate model path
        if !model_path.exists() {
            return Err(anyhow::anyhow!("Model file not found: {:?}", model_path));
        }
        
        // In a real implementation, we would:
        // 1. Convert model to CoreML format if needed
        // 2. Load onto Metal device
        // 3. Compile Metal shaders
        
        Ok(())
    }
    
    fn transcribe(&self, _audio: &[f32], _sample_rate: u32) -> Result<Transcript> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Metal backend not initialized"));
        }
        
        // This is where we would do GPU-accelerated transcription
        // For now, return an error indicating this needs to be implemented
        Err(anyhow::anyhow!("Metal transcription not yet implemented"))
    }
    
    fn get_memory_usage(&self) -> Result<(usize, usize)> {
        #[cfg(target_os = "macos")]
        {
            // For Apple Silicon, we can't directly query GPU memory
            // as it's unified. We'll estimate based on process memory
            use std::process::Command;
            
            if let Ok(output) = Command::new("ps")
                .arg("-o")
                .arg("rss=")
                .arg("-p")
                .arg(format!("{}", std::process::id()))
                .output()
            {
                if let Ok(rss_str) = String::from_utf8(output.stdout) {
                    if let Ok(rss_kb) = rss_str.trim().parse::<usize>() {
                        let used_mb = rss_kb / 1024;
                        let total_mb = self.device_info.as_ref()
                            .map(|d| d.memory_mb)
                            .unwrap_or(8192);
                        return Ok((used_mb, total_mb));
                    }
                }
            }
            
            Ok((0, 0))
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            Ok((0, 0))
        }
    }
    
    fn unload_model(&mut self) -> Result<()> {
        info!("Unloading model from Metal device");
        // In a real implementation, we would release Metal resources here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(target_os = "macos")]
    fn test_metal_detection() {
        match MetalBackend::new() {
            Ok(backend) => {
                assert!(backend.is_available());
                let info = backend.get_device_info();
                println!("Metal GPU: {} with {}GB unified memory", 
                    info.name, 
                    info.memory_mb / 1024
                );
            }
            Err(e) => {
                panic!("Metal should be available on macOS: {}", e);
            }
        }
    }
    
    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_metal_not_available() {
        assert!(MetalBackend::new().is_err());
    }
}