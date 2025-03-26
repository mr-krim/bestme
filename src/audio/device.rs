use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use log::{debug, info};
use std::collections::HashMap;

use super::get_device_description;

/// Audio device manager
pub struct DeviceManager {
    /// Available input devices
    input_devices: HashMap<String, cpal::Device>,
    
    /// Default input device ID
    default_input_device: Option<String>,
}

impl Clone for DeviceManager {
    fn clone(&self) -> Self {
        // Note: cpal::Device doesn't implement Clone, so we need to recreate the device manager
        // This is a simplified version - in production, we'd want to handle errors better
        DeviceManager::new().unwrap_or_else(|_| {
            Self {
                input_devices: HashMap::new(),
                default_input_device: self.default_input_device.clone(),
            }
        })
    }
}

impl DeviceManager {
    /// Create a new device manager
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        
        // Get input devices
        let mut input_devices = HashMap::new();
        let mut default_input_device = None;
        
        let devices = host.input_devices()
            .context("Failed to get input devices")?;
        
        // Get default input device
        if let Some(default) = host.default_input_device() {
            let name = default.name()?;
            default_input_device = Some(name.clone());
            debug!("Default input device: {}", name);
        }
        
        // Collect all input devices
        for device in devices {
            let name = device.name()?;
            input_devices.insert(name, device);
        }
        
        info!("Found {} input devices", input_devices.len());
        
        Ok(Self {
            input_devices,
            default_input_device,
        })
    }
    
    /// Get available input devices
    pub fn get_input_devices(&self) -> Vec<(String, String)> {
        let mut devices = Vec::new();
        
        for (id, device) in &self.input_devices {
            if let Ok(description) = get_device_description(device) {
                devices.push((id.clone(), description));
            }
        }
        
        // Sort by name, but put default device first
        devices.sort_by(|a, b| {
            if let Some(ref default) = self.default_input_device {
                if a.0 == *default {
                    return std::cmp::Ordering::Less;
                }
                if b.0 == *default {
                    return std::cmp::Ordering::Greater;
                }
            }
            a.1.cmp(&b.1)
        });
        
        devices
    }
    
    /// Get a specific input device by ID
    pub fn get_input_device(&self, id: &str) -> Option<&cpal::Device> {
        self.input_devices.get(id)
    }
    
    /// Get the default input device
    pub fn get_default_input_device(&self) -> Option<&cpal::Device> {
        self.default_input_device
            .as_ref()
            .and_then(|id| self.input_devices.get(id))
    }
    
    /// Get the supported configurations for a device
    pub fn get_supported_configs(&self, device: &cpal::Device) -> Result<Vec<cpal::SupportedStreamConfig>> {
        let configs = device.supported_input_configs()
            .context("Failed to get supported input configurations")?
            .collect::<Vec<_>>();
        
        let configs = configs.into_iter()
            .map(|config| config.with_max_sample_rate())
            .collect();
        
        Ok(configs)
    }
} 
