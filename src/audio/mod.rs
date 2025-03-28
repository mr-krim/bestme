pub mod capture;
pub mod device;
pub mod transcribe;
pub mod voice_commands;

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};

/// Audio configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Input device ID
    pub input_device: Option<String>,
    
    /// Input volume level (0.0 - 1.0)
    pub input_volume: f32,
    
    /// Sample rate
    pub sample_rate: u32,
    
    /// Number of channels
    pub channels: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            input_device: None,
            input_volume: 1.0,
            sample_rate: 16000,
            channels: 1,
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
