use anyhow::Result;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use log::{debug, error, info, warn};
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc;

use super::AudioConfig;

/// Size of the ring buffer for audio samples
#[allow(dead_code)]
const RING_BUFFER_SIZE: usize = 16 * 1024;

/// Audio event types that can be emitted by the capture system
#[derive(Debug, Clone)]
pub enum AudioEvent {
    /// Audio level update (peak level between 0.0 and 1.0)
    Level(f32),
    /// Audio data received
    Data(AudioData),
    /// Error occurred
    Error(String),
    /// Audio capture stopped
    Stopped,
    /// Audio capture started
    Started,
    /// Legacy name for level changes (for compatibility)
    LevelChanged(f32),
}

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
    
    /// Get an iterator over the samples
    pub fn iter(&self) -> impl Iterator<Item = &f32> {
        self.samples.iter()
    }
}

unsafe impl Send for AudioData {}
unsafe impl Sync for AudioData {}

/// Audio capture manager
pub struct CaptureManager {
    /// Audio configuration
    config: AudioConfig,
    
    /// Current audio stream
    audio_stream: Option<cpal::Stream>,
    
    /// Peak audio level (for visualization)
    peak_level: Arc<Mutex<f32>>,
    
    /// Callback for peak level updates (use Arc to make it clonable)
    peak_level_callback: Option<Arc<dyn Fn(f32) + Send + Sync + 'static>>,
    
    /// Callback for audio data (use Arc to make it clonable)
    audio_data_callback: Option<Arc<dyn Fn(AudioData) + Send + Sync + 'static>>,
    
    /// Flag indicating if recording is active
    is_recording: bool,
    
    /// Sender for audio events
    event_sender: mpsc::Sender<AudioEvent>,
}

impl CaptureManager {
    /// Create a new capture manager
    pub fn new() -> Result<(Self, mpsc::Receiver<AudioEvent>)> {
        // Create a channel for audio events
        let (event_sender, event_receiver) = mpsc::channel(100);
        
        let manager = Self {
            config: AudioConfig::default(),
            audio_stream: None,
            peak_level: Arc::new(Mutex::new(0.0)),
            peak_level_callback: None,
            audio_data_callback: None,
            is_recording: false,
            event_sender,
        };
        
        Ok((manager, event_receiver))
    }
    
    /// Set a callback for peak level updates
    pub fn on_peak_level<F: Fn(f32) + Send + Sync + 'static>(&mut self, callback: F) {
        self.peak_level_callback = Some(Arc::new(callback));
    }
    
    /// Set a callback for audio data
    pub fn on_audio_data<F: Fn(AudioData) + Send + Sync + 'static>(&mut self, callback: F) {
        self.audio_data_callback = Some(Arc::new(callback));
    }
    
    /// Set the audio device
    pub fn set_device(&mut self, device: cpal::Device) {
        // Update device name in config
        if let Ok(name) = device.name() {
            self.config.input_device = Some(name);
        }
    }
    
    /// Start audio capture and send events
    pub fn start(&mut self) -> Result<()> {
        if self.is_recording {
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
        
        // Set up references to be moved into closures
        let peak_level = self.peak_level.clone();
        
        // Create weak references to callbacks that will be captured by the closure
        let peak_callback = self.peak_level_callback.clone();
        let audio_callback = self.audio_data_callback.clone();
        let input_event_sender = self.event_sender.clone();
        
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
            
            // Send peak level event
            let peak_sender = input_event_sender.clone();
            tokio::spawn(async move {
                if let Err(e) = peak_sender.send(AudioEvent::Level(peak)).await {
                    error!("Failed to send audio level event: {}", e);
                }
            });
            
            // Call peak level callback if provided
            if let Some(callback) = &peak_callback {
                callback(peak);
            }
            
            // Create audio data and call audio data callback if provided
            let audio_data = AudioData::new(buffer, sample_rate, channels);
            
            if let Some(callback) = &audio_callback {
                callback(audio_data.clone());
            }
            
            // Send audio data event
            let data_sender = input_event_sender.clone();
            let data_clone = audio_data;
            tokio::spawn(async move {
                if let Err(e) = data_sender.send(AudioEvent::Data(data_clone)).await {
                    error!("Failed to send audio data event: {}", e);
                }
            });
        };
        
        // Create an error callback
        let err_event_sender = self.event_sender.clone();
        let err_fn = move |err| {
            let err_str = format!("Audio capture error: {}", err);
            error!("{}", err_str);
            
            let sender = err_event_sender.clone();
            tokio::spawn(async move {
                if let Err(e) = sender.send(AudioEvent::Error(err_str.clone())).await {
                    error!("Failed to send audio error event: {}", e);
                }
            });
        };
        
        // Build and store the input stream
        let stream = device.build_input_stream(
            &stream_config,
            input_data_fn,
            err_fn,
            None
        )?;
        
        // Store the stream in the struct
        self.audio_stream = Some(stream);
        
        // Start playing the stream
        self.audio_stream.as_ref().unwrap().play()?;
        
        info!("Started audio recording");
        self.is_recording = true;
        
        // Send started event
        let event_sender = self.event_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = event_sender.send(AudioEvent::Started).await {
                error!("Failed to send audio start event: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// Stop audio capture
    pub fn stop(&mut self) -> Result<()> {
        if !self.is_recording {
            return Ok(());
        }
        
        // Drop the stream to stop recording
        self.audio_stream = None;
        
        info!("Stopped audio recording");
        self.is_recording = false;
        
        // Send stopped event
        let event_sender = self.event_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = event_sender.send(AudioEvent::Stopped).await {
                error!("Failed to send audio stop event: {}", e);
            }
        });
        
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
        self.is_recording
    }
}

// Command enum for communicating with the isolated CaptureManager thread
pub enum CaptureCommand {
    Start,
    Stop,
    SetDevice(cpal::Device),
    SetPeakCallback(Box<dyn Fn(f32) + Send + Sync + 'static>),
    SetAudioCallback(Box<dyn Fn(AudioData) + Send + Sync + 'static>),
    Exit,
}

// Thread-safe wrapper for CaptureManager
#[derive(Clone)]
pub struct ThreadedCaptureManager {
    command_sender: mpsc::Sender<CaptureCommand>,
}

impl ThreadedCaptureManager {
    pub fn create_threaded() -> Result<(Self, mpsc::Receiver<AudioEvent>)> {
        CaptureManager::create_threaded()
    }
    
    pub fn create_from_capture_manager() -> Result<(Self, mpsc::Receiver<AudioEvent>)> {
        Self::create_threaded()
    }
    
    pub fn start(&self) -> Result<()> {
        self.command_sender.blocking_send(CaptureCommand::Start)
            .map_err(|e| anyhow::anyhow!("Failed to send start command: {}", e))
    }
    
    pub fn stop(&self) -> Result<()> {
        self.command_sender.blocking_send(CaptureCommand::Stop)
            .map_err(|e| anyhow::anyhow!("Failed to send stop command: {}", e))
    }
    
    pub fn set_device(&self, device: cpal::Device) -> Result<()> {
        self.command_sender.blocking_send(CaptureCommand::SetDevice(device))
            .map_err(|e| anyhow::anyhow!("Failed to send set device command: {}", e))
    }
    
    pub fn on_peak_level<F: Fn(f32) + Send + Sync + 'static>(&self, callback: F) -> Result<()> {
        self.command_sender.blocking_send(CaptureCommand::SetPeakCallback(Box::new(callback)))
            .map_err(|e| anyhow::anyhow!("Failed to send peak callback command: {}", e))
    }
    
    pub fn on_audio_data<F: Fn(AudioData) + Send + Sync + 'static>(&self, callback: F) -> Result<()> {
        self.command_sender.blocking_send(CaptureCommand::SetAudioCallback(Box::new(callback)))
            .map_err(|e| anyhow::anyhow!("Failed to send audio callback command: {}", e))
    }
}

impl Drop for ThreadedCaptureManager {
    fn drop(&mut self) {
        // Try to send exit command, but don't panic if it fails
        let _ = self.command_sender.blocking_send(CaptureCommand::Exit);
    }
}

impl CaptureManager {
    // Create a thread-safe wrapper around CaptureManager
    pub fn create_threaded() -> Result<(ThreadedCaptureManager, mpsc::Receiver<AudioEvent>)> {
        let (_event_sender, event_receiver) = mpsc::channel(100);
        let (cmd_sender, mut cmd_receiver) = mpsc::channel(10);
        
        // Create the manager and spawn a thread to manage it
        std::thread::spawn(move || {
            // Create manager in this thread
            match Self::new() {
                Ok((mut manager, _)) => {
                    // Main loop for processing commands
                    while let Some(cmd) = cmd_receiver.blocking_recv() {
                        match cmd {
                            CaptureCommand::Start => {
                                if let Err(e) = manager.start() {
                                    error!("Failed to start capture: {}", e);
                                }
                            },
                            CaptureCommand::Stop => {
                                if let Err(e) = manager.stop() {
                                    error!("Failed to stop capture: {}", e);
                                }
                            },
                            CaptureCommand::SetDevice(device) => {
                                manager.set_device(device);
                            },
                            CaptureCommand::SetPeakCallback(callback) => {
                                manager.on_peak_level(callback);
                            },
                            CaptureCommand::SetAudioCallback(callback) => {
                                manager.on_audio_data(callback);
                            },
                            CaptureCommand::Exit => break,
                        }
                    }
                    
                    // Clean up when finished
                    let _ = manager.stop();
                },
                Err(e) => {
                    error!("Failed to create CaptureManager: {}", e);
                }
            }
        });
        
        Ok((ThreadedCaptureManager { command_sender: cmd_sender }, event_receiver))
    }
} 
