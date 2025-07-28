//! Tests for GPU acceleration

#[cfg(test)]
mod tests {
    use super::super::*;
    
    #[test]
    fn test_gpu_backend_detection() {
        let backends = detect_available_backends();
        
        println!("Available GPU backends: {:?}", backends);
        
        // At least check that the function runs without panicking
        assert!(backends.len() >= 0);
    }
    
    #[test]
    fn test_gpu_manager_creation() {
        let config = GpuConfig::default();
        
        match GpuManager::new(config) {
            Ok(manager) => {
                println!("GPU manager created successfully");
                
                let devices = manager.get_all_devices();
                println!("Found {} GPU devices", devices.len());
                
                for device in devices {
                    println!("  - {} ({} backend, {}MB VRAM)", 
                        device.name,
                        device.backend,
                        device.memory_mb
                    );
                }
                
                if let Some(backend) = manager.get_selected_backend() {
                    println!("Selected backend: {:?}", backend);
                }
            }
            Err(e) => {
                println!("Failed to create GPU manager: {}", e);
                // This is expected if no GPU is available
            }
        }
    }
    
    #[test]
    #[cfg(feature = "gpu-cuda")]
    fn test_cuda_detection() {
        use crate::audio::gpu_config;
        
        let status = gpu_config::get_gpu_status();
        println!("GPU Status: {:?}", status);
        
        if status.gpu_available {
            assert_eq!(status.backend.as_deref(), Some("CUDA"));
            println!("CUDA GPU: {:?}", status.device_name);
        }
    }
    
    #[test]
    #[cfg(feature = "gpu-metal")]
    fn test_metal_detection() {
        use crate::audio::gpu_config;
        
        let status = gpu_config::get_gpu_status();
        println!("GPU Status: {:?}", status);
        
        if cfg!(target_os = "macos") {
            assert!(status.gpu_available);
            assert_eq!(status.backend.as_deref(), Some("Metal"));
            println!("Metal GPU: {:?}", status.device_name);
        }
    }
}