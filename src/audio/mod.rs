pub mod device;
pub mod capture;
pub mod transcribe;

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};

/// Audio configuration for the application
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Selected input device
    pub input_device: Option<String>,
    
    /// Input volume (0.0 - 1.0)
    pub input_volume: f32,
    
    /// Sample rate for audio capture
    pub sample_rate: u32,
    
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    
    /// Whether audio processing is currently active
    pub is_active: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            input_device: None,
            input_volume: 1.0,
            sample_rate: 44100,
            channels: 1, // mono default
            is_active: false,
        }
    }
}

/// Get a formatted description of an audio device
pub fn get_device_description(device: &cpal::Device) -> Result<String> {
    let name = device.name()?;
    
    let default_input = cpal::default_host()
        .default_input_device()
        .map(|d| d.name().map(|n| n == name).unwrap_or(false))
        .unwrap_or(false);
    
    let description = if default_input {
        format!("{} (Default)", name)
    } else {
        name
    };
    
    Ok(description)
} 
