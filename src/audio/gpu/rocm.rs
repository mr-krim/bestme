//! ROCm backend for AMD GPU acceleration

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::path::Path;

use super::{DeviceInfo, GpuAccelerator, GpuBackend};
use crate::audio::transcribe::Transcript;

/// ROCm backend for AMD GPUs
pub struct RocmBackend {
    device_index: usize,
    device_info: Option<DeviceInfo>,
    initialized: bool,
}

impl RocmBackend {
    /// Create a new ROCm backend
    pub fn new(device_index: usize) -> Result<Self> {
        debug!("Initializing ROCm backend for device {}", device_index);
        
        let mut backend = Self {
            device_index,
            device_info: None,
            initialized: false,
        };
        
        // Try to detect ROCm devices
        backend.detect_device()?;
        
        Ok(backend)
    }
    
    /// Detect ROCm device information
    fn detect_device(&mut self) -> Result<()> {
        #[cfg(all(feature = "gpu-rocm", target_os = "linux"))]
        {
            // Check if ROCm is installed by looking for rocm-smi
            use std::process::Command;
            
            let output = Command::new("rocm-smi")
                .arg("--showid")
                .arg("--showname")
                .arg("--showmeminfo")
                .arg("vram")
                .output();
            
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    
                    // Parse rocm-smi output to get device info
                    // This is a simplified parser
                    let lines: Vec<&str> = stdout.lines().collect();
                    
                    // Find device at index
                    let mut device_name = "Unknown AMD GPU".to_string();
                    let mut memory_mb = 0;
                    
                    // Look for GPU info in output
                    for line in lines {
                        if line.contains("GPU") && line.contains(":") {
                            // Extract device name
                            if let Some(name_part) = line.split(':').nth(1) {
                                device_name = name_part.trim().to_string();
                            }
                        }
                        if line.contains("VRAM Total") {
                            // Extract VRAM size
                            if let Some(mem_str) = line.split_whitespace().nth(2) {
                                if let Ok(mem) = mem_str.parse::<u64>() {
                                    memory_mb = mem as usize;
                                }
                            }
                        }
                    }
                    
                    // Detect specific AMD GPU models
                    let is_consumer_gpu = device_name.contains("RX") || 
                                         device_name.contains("Radeon");
                    
                    self.device_info = Some(DeviceInfo {
                        name: device_name,
                        memory_mb,
                        available_memory_mb: memory_mb * 3 / 4, // Estimate
                        backend: "ROCm".to_string(),
                        is_discrete: true,
                        compute_capability: None,
                        device_index: self.device_index,
                    });
                    
                    info!("ROCm device detected: {} ({}MB VRAM)",
                        self.device_info.as_ref().unwrap().name,
                        self.device_info.as_ref().unwrap().memory_mb
                    );
                    
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to run rocm-smi: {}", e);
                    Err(anyhow::anyhow!("ROCm not available: rocm-smi not found"))
                }
            }
        }
        
        #[cfg(not(all(feature = "gpu-rocm", target_os = "linux")))]
        {
            Err(anyhow::anyhow!("ROCm is only available on Linux"))
        }
    }
}

impl GpuAccelerator for RocmBackend {
    fn is_available(&self) -> bool {
        self.device_info.is_some()
    }
    
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone().unwrap_or(DeviceInfo {
            name: "Unknown ROCm Device".to_string(),
            memory_mb: 0,
            available_memory_mb: 0,
            backend: "ROCm".to_string(),
            is_discrete: true,
            compute_capability: None,
            device_index: self.device_index,
        })
    }
    
    fn get_backend_type(&self) -> GpuBackend {
        GpuBackend::Rocm
    }
    
    fn initialize(&mut self) -> Result<()> {
        if !self.is_available() {
            return Err(anyhow::anyhow!("ROCm device not available"));
        }
        
        // In a real implementation, we would:
        // 1. Initialize HIP runtime
        // 2. Create HIP context
        // 3. Allocate memory pools
        
        info!("ROCm backend initialized");
        self.initialized = true;
        Ok(())
    }
    
    fn load_model(&mut self, model_path: &Path) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("ROCm backend not initialized"));
        }
        
        info!("Loading model {} on ROCm device", model_path.display());
        
        if !model_path.exists() {
            return Err(anyhow::anyhow!("Model file not found: {:?}", model_path));
        }
        
        // In a real implementation, we would:
        // 1. Load model weights
        // 2. Convert to HIP-compatible format
        // 3. Upload to GPU memory
        
        Ok(())
    }
    
    fn transcribe(&self, _audio: &[f32], _sample_rate: u32) -> Result<Transcript> {
        if !self.initialized {
            return Err(anyhow::anyhow!("ROCm backend not initialized"));
        }
        
        // This would use HIP kernels for GPU computation
        Err(anyhow::anyhow!("ROCm transcription not yet implemented"))
    }
    
    fn get_memory_usage(&self) -> Result<(usize, usize)> {
        #[cfg(all(feature = "gpu-rocm", target_os = "linux"))]
        {
            use std::process::Command;
            
            let output = Command::new("rocm-smi")
                .arg("--showmeminfo")
                .arg("vram")
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                // Parse memory usage from rocm-smi output
                let mut used_mb = 0;
                let total_mb = self.device_info.as_ref()
                    .map(|d| d.memory_mb)
                    .unwrap_or(0);
                
                for line in stdout.lines() {
                    if line.contains("VRAM Total Used") {
                        if let Some(used_str) = line.split_whitespace().nth(3) {
                            if let Ok(used) = used_str.parse::<usize>() {
                                used_mb = used;
                            }
                        }
                    }
                }
                
                return Ok((used_mb, total_mb));
            }
        }
        
        Ok((0, 0))
    }
    
    fn unload_model(&mut self) -> Result<()> {
        info!("Unloading model from ROCm device");
        // Free HIP memory allocations
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(all(feature = "gpu-rocm", target_os = "linux"))]
    fn test_rocm_detection() {
        match RocmBackend::new(0) {
            Ok(backend) => {
                if backend.is_available() {
                    let info = backend.get_device_info();
                    println!("ROCm GPU: {} with {}MB VRAM", 
                        info.name, 
                        info.memory_mb
                    );
                } else {
                    println!("ROCm backend created but no device available");
                }
            }
            Err(e) => {
                println!("ROCm not available: {}", e);
            }
        }
    }
}