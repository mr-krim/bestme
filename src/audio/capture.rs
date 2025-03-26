use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, StreamTrait};
use log::{debug, error, info, warn};
use ringbuf::HeapRb;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc;

use super::AudioConfig;

/// Size of the ring buffer for audio samples
const RING_BUFFER_SIZE: usize = 16 * 1024;

/// Audio data structure
#[derive(Debug, Clone)]
pub struct AudioData {
    /// Audio samples
    samples: Vec<f32>,
    
    /// Sample rate
    sample_rate: u32,
    
    /// Number of channels
    channels: u16,
}

impl AudioData {
    /// Create a new AudioData instance
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
        }
    }
    
    /// Get audio samples
    pub fn get_samples(&self) -> &[f32] {
        &self.samples
    }
    
    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    /// Get the number of channels
    pub fn channels(&self) -> u16 {
        self.channels
    }
    
    /// Convert to mono and resample to target sample rate if needed
    pub fn to_whisper_input(&self, target_sample_rate: u32) -> Vec<f32> {
        let mut result = Vec::with_capacity(self.samples.len());
        
        // If stereo, convert to mono by averaging channels
        if self.channels == 2 {
            for i in 0..(self.samples.len() / 2) {
                let mono_sample = (self.samples[i * 2] + self.samples[i * 2 + 1]) / 2.0;
                result.push(mono_sample);
            }
        } else {
            // Already mono
            result = self.samples.clone();
        }
        
        // Simple resampling if needed (this is a basic implementation)
        // For production, use a proper resampling library
        if self.sample_rate != target_sample_rate {
            // Basic linear interpolation for resampling
            let ratio = self.sample_rate as f32 / target_sample_rate as f32;
            let target_len = (result.len() as f32 / ratio) as usize;
            let mut resampled = Vec::with_capacity(target_len);
            
            for i in 0..target_len {
                let src_idx = i as f32 * ratio;
                let src_idx_floor = src_idx.floor() as usize;
                let src_idx_ceil = (src_idx_floor + 1).min(result.len() - 1);
                let t = src_idx - src_idx_floor as f32;
                
                let sample = result[src_idx_floor] * (1.0 - t) + result[src_idx_ceil] * t;
                resampled.push(sample);
            }
            
            return resampled;
        }
        
        result
    }
}

/// Audio capture manager
pub struct CaptureManager {
    /// Audio configuration
    config: AudioConfig,
    
    /// Audio stream
    stream: Option<cpal::Stream>,
    
    /// Peak audio level (for visualization)
    peak_level: Arc<Mutex<f32>>,
    
    /// Callback for peak level updates
    peak_level_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
    
    /// Callback for audio data
    audio_data_callback: Option<Box<dyn Fn(AudioData) + Send + Sync>>,
}

impl CaptureManager {
    /// Create a new capture manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: AudioConfig::default(),
            stream: None,
            peak_level: Arc::new(Mutex::new(0.0)),
            peak_level_callback: None,
            audio_data_callback: None,
        })
    }
    
    /// Set a callback for peak level updates
    pub fn on_peak_level<F: Fn(f32) + Send + Sync + 'static>(&mut self, callback: F) {
        self.peak_level_callback = Some(Box::new(callback));
    }
    
    /// Set a callback for audio data
    pub fn on_audio_data<F: Fn(AudioData) + Send + Sync + 'static>(&mut self, callback: F) {
        self.audio_data_callback = Some(Box::new(callback));
    }
    
    /// Set the audio device
    pub fn set_device(&mut self, device: cpal::Device) {
        // Update device name in config
        if let Ok(name) = device.name() {
            self.config.input_device = Some(name);
        }
    }
    
    /// Start audio capture
    pub fn start(&mut self) -> Result<()> {
        if self.stream.is_some() {
            warn!("Audio capture already running");
            return Ok(());
        }
        
        // Find the device
        let host = cpal::default_host();
        let device = if let Some(device_name) = &self.config.input_device {
            // Try to find device by name
            let devices = host.input_devices()?;
            let mut found_device = None;
            
            for device in devices {
                if let Ok(name) = device.name() {
                    if name == *device_name {
                        found_device = Some(device);
                        break;
                    }
                }
            }
            
            found_device.unwrap_or_else(|| host.default_input_device()
                .expect("No input device available"))
        } else {
            // Use default device
            host.default_input_device()
                .ok_or_else(|| anyhow::anyhow!("No default input device"))?
        };
        
        info!("Using audio device: {}", device.name()?);
        
        // Get a config we can use
        let config = match device.default_input_config() {
            Ok(config) => config,
            Err(_) => {
                // If default config fails, try to find one manually
                let supported_configs = device.supported_input_configs()?
                    .collect::<Vec<_>>();
                
                let config_range = supported_configs.iter()
                    .find(|c| c.channels() == self.config.channels && c.sample_format() == cpal::SampleFormat::F32)
                    .cloned()
                    .or_else(|| supported_configs.into_iter().next())
                    .ok_or_else(|| anyhow::anyhow!("No supported audio configuration found"))?;
                
                // Convert the config range to a specific config by selecting the max sample rate
                config_range.with_max_sample_rate()
            }
        };
        
        info!("Using audio config: {:?}", config);
        debug!("Sample format: {:?}", config.sample_format());
        
        // Create a config to use for the stream
        let stream_config = cpal::StreamConfig {
            channels: config.channels(),
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: match config.buffer_size() {
                cpal::SupportedBufferSize::Range { min: _, max: _ } => cpal::BufferSize::Default,
                cpal::SupportedBufferSize::Unknown => cpal::BufferSize::Default,
            },
        };
        
        info!("Using stream config: {:?}", stream_config);
        
        // Store actual config values for audio data
        let sample_rate = stream_config.sample_rate.0;
        let channels = stream_config.channels;
        
        // Set up callback references
        let peak_level = self.peak_level.clone();
        let peak_callback = self.peak_level_callback.clone();
        let audio_callback = self.audio_data_callback.clone();
        
        // Input data callback - receives audio samples
        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut peak = 0.0f32;
            let mut buffer = Vec::with_capacity(data.len());
            
            // Calculate peak level for visualization
            for &sample in data.iter() {
                let abs_sample = sample.abs();
                if abs_sample > peak {
                    peak = abs_sample;
                }
                buffer.push(sample);
            }
            
            // Update peak level
            {
                let mut level = peak_level.lock();
                *level = peak;
            }
            
            // Call peak level callback
            if let Some(callback) = &peak_callback {
                callback(peak);
            }
            
            // Call audio data callback
            if let Some(callback) = &audio_callback {
                let audio_data = AudioData::new(buffer, sample_rate, channels);
                callback(audio_data);
            }
        };
        
        // Error callback
        let err_fn = move |err| {
            error!("Audio capture error: {}", err);
        };
        
        // Build and start stream
        let stream = device.build_input_stream(
            &stream_config,
            input_data_fn,
            err_fn,
            None
        )?;
        
        stream.play()?;
        
        // Store stream
        self.stream = Some(stream);
        
        info!("Audio capture started");
        self.config.is_active = true;
        
        Ok(())
    }
    
    /// Stop audio capture
    pub fn stop(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
            self.config.is_active = false;
            info!("Audio capture stopped");
        }
        
        Ok(())
    }
    
    /// Get the current peak audio level
    pub fn get_peak_level(&self) -> f32 {
        *self.peak_level.lock()
    }
    
    /// Get the audio configuration
    pub fn get_config(&self) -> &AudioConfig {
        &self.config
    }
    
    /// Check if audio capture is active
    pub fn is_active(&self) -> bool {
        self.config.is_active
    }
} 
