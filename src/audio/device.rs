use anyhow::{Context, Result};
use log::info;
use std::collections::HashMap;
use cpal::traits::{DeviceTrait, HostTrait};
use super::get_device_description;

/// Audio device manager
#[derive(Clone)]
pub struct DeviceManager {
    /// Input devices
    input_devices: HashMap<String, String>,
    /// Default input device ID
    default_input_device: Option<String>,
}

impl DeviceManager {
    /// Create a new device manager
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            input_devices: HashMap::new(),
            default_input_device: None,
        };
        
        // Try to find input devices
        manager.refresh_devices()?;
        
        info!("Found {} input devices", manager.input_devices.len());
        
        Ok(manager)
    }
    
    /// Refresh device list
    pub fn refresh_devices(&mut self) -> Result<()> {
        self.input_devices.clear();
        
        // Platform-specific implementations
        #[cfg(target_os = "windows")]
        {
            self.refresh_devices_windows()?;
            return Ok(());
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Default implementation for non-Windows platforms
            let host = cpal::default_host();
            
            // Try to get the default input device
            if let Some(default_device) = host.default_input_device() {
                let device_name = default_device.name().context("Could not get default device name")?;
                self.default_input_device = Some(device_name.clone());
                self.input_devices.insert(device_name.clone(), device_name);
            }
            
            // Try to get all input devices
            match host.input_devices() {
                Ok(devices) => {
                    for device in devices {
                        if let Ok(name) = device.name() {
                            self.input_devices.insert(name.clone(), name);
                        }
                    }
                },
                Err(e) => {
                    info!("Could not get input devices: {}", e);
                }
            };
            
            // If no devices found and we're in a headless environment like WSL, add a mock device
            if self.input_devices.is_empty() && cfg!(target_os = "linux") {
                self.add_mock_device_for_testing();
            }
        }
        
        Ok(())
    }
    
    /// Add a mock audio device for testing in headless/WSL environments
    fn add_mock_device_for_testing(&mut self) {
        info!("Adding mock audio device for testing purposes");
        let mock_id = "mock-device-id".to_string();
        let mock_name = "Mock Audio Input (Testing Only)".to_string();
        self.input_devices.insert(mock_id.clone(), mock_name);
        
        // If no default device is set, use the mock device as default
        if self.default_input_device.is_none() {
            self.default_input_device = Some(mock_id);
        }
    }
    
    /// Get all input devices
    pub fn get_input_devices(&self) -> Vec<(String, String)> {
        self.input_devices.iter()
            .map(|(id, name)| (id.clone(), name.clone()))
            .collect()
    }
    
    /// Get input device by ID
    pub fn get_input_device(&self, id: &str) -> Option<(String, String)> {
        self.input_devices.get(id)
            .map(|name| (id.to_string(), name.clone()))
    }
    
    /// Get default input device
    pub fn get_default_input_device(&self) -> Option<(String, String)> {
        if let Some(default_id) = &self.default_input_device {
            self.get_input_device(default_id)
        } else {
            self.input_devices.iter()
                .next()
                .map(|(id, name)| (id.clone(), name.clone()))
        }
    }
    
    /// Get device name
    pub fn get_device_name(&self, id: &str) -> Option<String> {
        self.input_devices.get(id).cloned()
    }
    
    /// Get the supported configurations for a device 
    /// Note: This is a stub method since we're no longer storing actual devices
    pub fn get_supported_configs(&self, _device_id: &str) -> Result<Vec<cpal::SupportedStreamConfig>> {
        // In a real implementation, we would get the device by ID and return its configurations
        // For now, return a minimal default configuration for testing
        Ok(Vec::new())
    }
    
    /// Refresh device list with Windows-specific optimizations
    #[cfg(target_os = "windows")]
    pub fn refresh_devices_windows(&mut self) -> Result<()> {
        info!("Using Windows-specific audio device detection");
        self.input_devices.clear();
        
        // Use Windows-specific APIs to get devices more reliably
        // This is a simple implementation for now - in a real app, we might use
        // the windows crate with more detailed device enumeration
        let host = cpal::default_host();
        
        if let Some(default_device) = host.default_input_device() {
            if let Ok(name) = default_device.name() {
                info!("Found Windows default input device: {}", name);
                self.default_input_device = Some(name.clone());
                self.input_devices.insert(name.clone(), name);
            }
        }
        
        // Get all Windows input devices
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    info!("Found Windows input device: {}", name);
                    self.input_devices.insert(name.clone(), name);
                }
            }
        }
        
        info!("Windows audio device detection found {} devices", self.input_devices.len());
        Ok(())
    }
} 

