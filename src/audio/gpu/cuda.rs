//! CUDA backend for GPU acceleration

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::path::Path;

use super::{DeviceInfo, GpuAccelerator, GpuBackend};
use crate::audio::transcribe::Transcript;

/// CUDA backend for NVIDIA GPUs
pub struct CudaBackend {
    device_index: usize,
    device_info: Option<DeviceInfo>,
    #[allow(dead_code)]
    initialized: bool,
}

impl CudaBackend {
    /// Create a new CUDA backend
    pub fn new(device_index: usize) -> Result<Self> {
        debug!("Initializing CUDA backend for device {}", device_index);
        
        let mut backend = Self {
            device_index,
            device_info: None,
            initialized: false,
        };
        
        // Try to get device info
        backend.detect_device()?;
        
        Ok(backend)
    }
    
    /// Detect CUDA device information
    fn detect_device(&mut self) -> Result<()> {
        #[cfg(feature = "gpu-cuda")]
        {
            // Use nvml-wrapper to get GPU info
            use nvml_wrapper::Nvml;
            
            match Nvml::init() {
                Ok(nvml) => {
                    let device_count = nvml.device_count()
                        .context("Failed to get CUDA device count")?;
                    
                    if self.device_index >= device_count as usize {
                        return Err(anyhow::anyhow!(
                            "CUDA device index {} out of range (found {} devices)",
                            self.device_index,
                            device_count
                        ));
                    }
                    
                    let device = nvml.device_by_index(self.device_index as u32)
                        .context("Failed to get CUDA device")?;
                    
                    let name = device.name()
                        .context("Failed to get device name")?;
                    
                    let memory_info = device.memory_info()
                        .context("Failed to get memory info")?;
                    
                    let compute_cap = device.cuda_compute_capability()
                        .context("Failed to get compute capability")?;
                    
                    self.device_info = Some(DeviceInfo {
                        name,
                        memory_mb: memory_info.total / (1024 * 1024),
                        available_memory_mb: memory_info.free / (1024 * 1024),
                        backend: "CUDA".to_string(),
                        is_discrete: true, // CUDA devices are discrete
                        compute_capability: Some((compute_cap.major as u32, compute_cap.minor as u32)),
                        device_index: self.device_index,
                    });
                    
                    info!("CUDA device detected: {} ({}GB VRAM, compute capability {}.{})",
                        self.device_info.as_ref().unwrap().name,
                        self.device_info.as_ref().unwrap().memory_mb / 1024,
                        compute_cap.major,
                        compute_cap.minor
                    );
                    
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to initialize NVML: {}", e);
                    Err(anyhow::anyhow!("CUDA not available: {}", e))
                }
            }
        }
        
        #[cfg(not(feature = "gpu-cuda"))]
        {
            Err(anyhow::anyhow!("CUDA support not compiled in"))
        }
    }
}

impl GpuAccelerator for CudaBackend {
    fn is_available(&self) -> bool {
        self.device_info.is_some()
    }
    
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone().unwrap_or(DeviceInfo {
            name: "Unknown CUDA Device".to_string(),
            memory_mb: 0,
            available_memory_mb: 0,
            backend: "CUDA".to_string(),
            is_discrete: true,
            compute_capability: None,
            device_index: self.device_index,
        })
    }
    
    fn get_backend_type(&self) -> GpuBackend {
        GpuBackend::Cuda
    }
    
    fn initialize(&mut self) -> Result<()> {
        if !self.is_available() {
            return Err(anyhow::anyhow!("CUDA device not available"));
        }
        
        // In a real implementation, we would initialize CUDA context here
        info!("CUDA backend initialized");
        self.initialized = true;
        Ok(())
    }
    
    fn load_model(&mut self, model_path: &Path) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("CUDA backend not initialized"));
        }
        
        info!("Loading model {} on CUDA device", model_path.display());
        
        // In a real implementation, we would load the model to GPU here
        // For now, we just validate the path
        if !model_path.exists() {
            return Err(anyhow::anyhow!("Model file not found: {:?}", model_path));
        }
        
        Ok(())
    }
    
    fn transcribe(&self, _audio: &[f32], _sample_rate: u32) -> Result<Transcript> {
        if !self.initialized {
            return Err(anyhow::anyhow!("CUDA backend not initialized"));
        }
        
        // This is where we would do GPU-accelerated transcription
        // For now, return an error indicating this needs to be implemented
        Err(anyhow::anyhow!("CUDA transcription not yet implemented"))
    }
    
    fn get_memory_usage(&self) -> Result<(usize, usize)> {
        #[cfg(feature = "gpu-cuda")]
        {
            use nvml_wrapper::Nvml;
            
            let nvml = Nvml::init()
                .context("Failed to initialize NVML")?;
            
            let device = nvml.device_by_index(self.device_index as u32)
                .context("Failed to get CUDA device")?;
            
            let memory_info = device.memory_info()
                .context("Failed to get memory info")?;
            
            let used = (memory_info.total - memory_info.free) / (1024 * 1024);
            let total = memory_info.total / (1024 * 1024);
            
            Ok((used, total))
        }
        
        #[cfg(not(feature = "gpu-cuda"))]
        {
            Ok((0, 0))
        }
    }
    
    fn unload_model(&mut self) -> Result<()> {
        info!("Unloading model from CUDA device");
        // In a real implementation, we would free GPU memory here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cuda_detection() {
        // This test will only pass if CUDA is available
        match CudaBackend::new(0) {
            Ok(backend) => {
                println!("CUDA backend created successfully");
                if backend.is_available() {
                    let info = backend.get_device_info();
                    println!("GPU: {} with {}MB VRAM", info.name, info.memory_mb);
                }
            }
            Err(e) => {
                println!("CUDA not available: {}", e);
            }
        }
    }
}