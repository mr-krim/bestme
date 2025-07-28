//! OpenCL backend for cross-platform GPU acceleration

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::path::Path;

use super::{DeviceInfo, GpuAccelerator, GpuBackend};
use crate::audio::transcribe::Transcript;

/// OpenCL backend for cross-platform GPU support
pub struct OpenCLBackend {
    device_index: usize,
    device_info: Option<DeviceInfo>,
    initialized: bool,
}

impl OpenCLBackend {
    /// Create a new OpenCL backend
    pub fn new(device_index: usize) -> Result<Self> {
        debug!("Initializing OpenCL backend for device {}", device_index);
        
        let mut backend = Self {
            device_index,
            device_info: None,
            initialized: false,
        };
        
        // Try to detect OpenCL devices
        backend.detect_device()?;
        
        Ok(backend)
    }
    
    /// Detect OpenCL device information
    fn detect_device(&mut self) -> Result<()> {
        #[cfg(feature = "gpu-opencl")]
        {
            use opencl3::platform::get_platforms;
            use opencl3::device::{Device, CL_DEVICE_TYPE_GPU};
            
            let platforms = get_platforms()
                .map_err(|e| anyhow::anyhow!("Failed to get OpenCL platforms: {}", e))?;
            
            if platforms.is_empty() {
                return Err(anyhow::anyhow!("No OpenCL platforms found"));
            }
            
            // Find all GPU devices across all platforms
            let mut gpu_devices = Vec::new();
            for platform in platforms {
                if let Ok(devices) = platform.get_devices(CL_DEVICE_TYPE_GPU) {
                    gpu_devices.extend(devices);
                }
            }
            
            if gpu_devices.is_empty() {
                return Err(anyhow::anyhow!("No OpenCL GPU devices found"));
            }
            
            if self.device_index >= gpu_devices.len() {
                return Err(anyhow::anyhow!(
                    "OpenCL device index {} out of range (found {} devices)",
                    self.device_index,
                    gpu_devices.len()
                ));
            }
            
            let device = &gpu_devices[self.device_index];
            
            // Get device information
            let name = device.name()
                .unwrap_or_else(|_| "Unknown OpenCL Device".to_string());
            
            let global_mem = device.global_mem_size()
                .unwrap_or(0) / (1024 * 1024);
            
            let vendor = device.vendor()
                .unwrap_or_else(|_| "Unknown".to_string());
            
            // Determine if it's a discrete GPU based on vendor and memory
            let is_discrete = (vendor.contains("NVIDIA") || 
                              vendor.contains("AMD") || 
                              vendor.contains("Intel")) && 
                              global_mem > 2048; // More than 2GB suggests discrete
            
            self.device_info = Some(DeviceInfo {
                name: format!("{} ({})", name, vendor),
                memory_mb: global_mem as usize,
                available_memory_mb: (global_mem * 3 / 4) as usize, // Estimate 75%
                backend: "OpenCL".to_string(),
                is_discrete,
                compute_capability: None,
                device_index: self.device_index,
            });
            
            info!("OpenCL device detected: {} ({}MB memory)",
                self.device_info.as_ref().unwrap().name,
                self.device_info.as_ref().unwrap().memory_mb
            );
            
            Ok(())
        }
        
        #[cfg(not(feature = "gpu-opencl"))]
        {
            Err(anyhow::anyhow!("OpenCL support not compiled in"))
        }
    }
}

impl GpuAccelerator for OpenCLBackend {
    fn is_available(&self) -> bool {
        self.device_info.is_some()
    }
    
    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone().unwrap_or(DeviceInfo {
            name: "Unknown OpenCL Device".to_string(),
            memory_mb: 0,
            available_memory_mb: 0,
            backend: "OpenCL".to_string(),
            is_discrete: false,
            compute_capability: None,
            device_index: self.device_index,
        })
    }
    
    fn get_backend_type(&self) -> GpuBackend {
        GpuBackend::OpenCL
    }
    
    fn initialize(&mut self) -> Result<()> {
        if !self.is_available() {
            return Err(anyhow::anyhow!("OpenCL device not available"));
        }
        
        info!("OpenCL backend initialized");
        self.initialized = true;
        Ok(())
    }
    
    fn load_model(&mut self, model_path: &Path) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("OpenCL backend not initialized"));
        }
        
        info!("Loading model {} on OpenCL device", model_path.display());
        
        if !model_path.exists() {
            return Err(anyhow::anyhow!("Model file not found: {:?}", model_path));
        }
        
        Ok(())
    }
    
    fn transcribe(&self, _audio: &[f32], _sample_rate: u32) -> Result<Transcript> {
        if !self.initialized {
            return Err(anyhow::anyhow!("OpenCL backend not initialized"));
        }
        
        Err(anyhow::anyhow!("OpenCL transcription not yet implemented"))
    }
    
    fn get_memory_usage(&self) -> Result<(usize, usize)> {
        // OpenCL doesn't provide easy memory usage queries
        // Return estimates based on device info
        if let Some(info) = &self.device_info {
            Ok((0, info.memory_mb))
        } else {
            Ok((0, 0))
        }
    }
    
    fn unload_model(&mut self) -> Result<()> {
        info!("Unloading model from OpenCL device");
        Ok(())
    }
}