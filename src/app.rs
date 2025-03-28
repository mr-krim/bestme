use anyhow::{Context, Result};
use log::{error, info, warn};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::io::{self, Write};
use std::sync::Arc;
use parking_lot;

use crate::audio::{
    device::DeviceManager,
    capture::{CaptureManager, AudioEvent},
    transcribe::{TranscriptionManager, TranscriptionEvent},
    AudioConfig,
};
use crate::config::{Config, ConfigManager};
use crate::gui::Gui;

/// Main application struct
pub struct App {
    /// Configuration manager
    config_manager: ConfigManager,
    
    /// Audio device manager
    device_manager: DeviceManager,
    
    /// GUI manager
    gui_manager: Option<Gui>,
    
    /// Whether to use GUI mode
    use_gui: bool,
    
    /// Audio capture manager
    capture_manager: Option<CaptureManager>,
    
    /// Audio event receiver
    audio_receiver: Option<mpsc::Receiver<AudioEvent>>,
    
    /// Audio processing task
    audio_task: Option<JoinHandle<()>>,
    
    /// Transcription manager
    transcription_manager: Option<TranscriptionManager>,
    
    /// Transcription event receiver
    transcription_receiver: Option<mpsc::Receiver<TranscriptionEvent>>,
    
    /// Transcription processing task
    transcription_task: Option<JoinHandle<()>>,
    
    /// Whether to continue running the application
    running: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(config_manager: ConfigManager) -> Result<Self> {
        // Initialize audio device manager
        let device_manager = DeviceManager::new()
            .context("Failed to initialize audio device manager")?;
        
        // Determine if we should use GUI mode
        // For now, we'll always use GUI mode on Windows
        #[cfg(target_os = "windows")]
        let use_gui = true;
        
        #[cfg(not(target_os = "windows"))]
        let use_gui = false;
        
        Ok(Self {
            config_manager,
            device_manager,
            gui_manager: None,
            use_gui,
            capture_manager: None,
            audio_receiver: None,
            audio_task: None,
            transcription_manager: None,
            transcription_receiver: None,
            transcription_task: None,
            running: true,
        })
    }
    
    /// Run the application
    pub fn run(&mut self) -> Result<()> {
        let config = self.config_manager.get_config();
        
        // Display application info
        self.display_info(config)?;
        
        // Display available audio devices
        self.list_audio_devices()?;
        
        // Initialize and run GUI if in GUI mode
        if self.use_gui {
            info!("Starting in GUI mode");
            
            // Initialize GUI
            let mut gui_manager = Gui::new(
                Arc::new(parking_lot::Mutex::new(self.config_manager.clone())),
                Arc::new(parking_lot::Mutex::new(self.device_manager.clone())),
            );
            
            gui_manager.initialize()?;
            
            // Store and run GUI
            self.gui_manager = Some(gui_manager);
            
            if let Some(gui) = &mut self.gui_manager {
                gui.run()?;
            }
        } else {
            info!("Starting in console mode");
            
            // Create a more robust runtime for async tasks
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()
                .context("Failed to create tokio runtime")?;
            
            // Run the main menu loop
            rt.block_on(async {
                self.main_menu().await
            })?;
        }
        
        Ok(())
    }
    
    /// Display application info
    fn display_info(&self, config: &Config) -> Result<()> {
        println!("BestMe Application");
        println!("-----------------");
        println!("Version: {}", config.version);
        println!("Theme: {}", config.general.theme);
        println!("Auto-start: {}", config.general.auto_start);
        println!("Input volume: {}", config.audio.input_volume);
        println!("Input device: {} ({})", 
            config.audio.input_device.as_deref().unwrap_or("None"), 
            if config.audio.input_device.is_none() { "default" } else { "custom" }
        );
        
        Ok(())
    }
    
    /// Display available audio devices
    fn list_audio_devices(&self) -> Result<()> {
        println!("\nAvailable Audio Devices:");
        println!("------------------------");
        
        let devices = self.device_manager.get_input_devices();
        
        if devices.is_empty() {
            println!("No input devices found");
        } else {
            for (i, (_, name)) in devices.iter().enumerate() {
                println!("{}. {}", i + 1, name);
            }
        }
        
        Ok(())
    }
    
    /// Start audio capture
    async fn start_audio_capture(&mut self, device_id: Option<&str>) -> Result<()> {
        // Stop any existing capture
        self.stop_audio_capture().await;
        
        // Get device to use
        let _device = if let Some(id) = device_id {
            self.device_manager.get_input_device(id)
                .ok_or_else(|| anyhow::anyhow!("Device with ID {} not found", id))?
        } else {
            self.device_manager.get_default_input_device()
                .ok_or_else(|| anyhow::anyhow!("No default input device found"))?
        };
        
        // Create audio config from the application config
        let _audio_config = AudioConfig {
            input_device: device_id.map(String::from),
            input_volume: self.config_manager.get_config().audio.input_volume,
            ..AudioConfig::default()
        };
        
        // Create capture manager
        let _audio_config = self.config_manager.get_config().audio.clone();
        let (capture_manager, receiver) = match CaptureManager::new() {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to create capture manager: {}", e);
                return Err(e.into());
            }
        };
        
        // Store capture manager and receiver
        self.capture_manager = Some(capture_manager);
        self.audio_receiver = Some(receiver);
        
        // Initialize transcription if not initialized
        if self.transcription_manager.is_none() {
            let speech_settings = self.config_manager.get_config().audio.speech.clone();
            let (transcription_manager, transcription_receiver) = TranscriptionManager::new(speech_settings)
                .context("Failed to create transcription manager")?;
            
            self.transcription_manager = Some(transcription_manager);
            self.transcription_receiver = Some(transcription_receiver);
        }
        
        // Start audio capture
        if let Some(capture_manager) = &mut self.capture_manager {
            match capture_manager.start() {
                Ok(()) => {
                    info!("Started audio capture");
                    // Start audio processing task
                    let mut receiver = self.audio_receiver.take().unwrap();
                    let transcription_manager = self.transcription_manager.as_ref().unwrap().clone();
                    
                    // Start transcription
                    if let Some(manager) = &mut self.transcription_manager {
                        manager.start().await?;
                    }
                    
                    // Process transcription events
                    let mut transcription_receiver = self.transcription_receiver.take().unwrap();
                    let transcription_task = tokio::spawn(async move {
                        while let Some(event) = transcription_receiver.recv().await {
                            match event {
                                TranscriptionEvent::Transcription(text) => {
                                    println!("\nTranscription: {}", text);
                                },
                                TranscriptionEvent::PartialTranscription(text) => {
                                    print!("\rPartial: {}", text);
                                    let _ = io::stdout().flush();
                                },
                                TranscriptionEvent::Started => {
                                    println!("Transcription started");
                                },
                                TranscriptionEvent::Stopped => {
                                    println!("Transcription stopped");
                                },
                                TranscriptionEvent::Error(err) => {
                                    eprintln!("Transcription error: {}", err);
                                },
                            }
                        }
                    });
                    self.transcription_task = Some(transcription_task);
                    
                    // Process audio with improved error handling
                    let transcription_manager_clone = transcription_manager.clone();
                    let task = tokio::spawn(async move {
                        while let Some(event) = receiver.recv().await {
                            match event {
                                AudioEvent::Data(audio_data) => {
                                    // Extract raw samples for transcription processing
                                    let samples = audio_data.get_samples();
                                    
                                    // Pass the samples to the transcription manager
                                    if let Err(e) = transcription_manager_clone.process_audio(samples).await {
                                        error!("Error processing audio for transcription: {}", e);
                                    }
                                },
                                AudioEvent::Level(_level) => {
                                    // Handle audio level event
                                },
                                AudioEvent::Started => {
                                    println!("Audio processing started");
                                },
                                AudioEvent::Stopped => {
                                    println!("Audio processing stopped");
                                    break;
                                },
                                AudioEvent::Error(error) => {
                                    // Handle error event
                                    error!("Audio capture error: {}", error);
                                },
                                AudioEvent::LevelChanged(_level) => {
                                    // Handle level changed event
                                },
                            }
                        }
                    });
                    
                    self.audio_task = Some(task);
                },
                Err(e) => {
                    error!("Failed to start audio capture: {}", e);
                    return Err(e.into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Stop audio capture
    async fn stop_audio_capture(&mut self) {
        // Shutdown async tasks directly without creating a new runtime
        if let Err(e) = self.shutdown_async_tasks().await {
            error!("Error during async task shutdown: {}", e);
        }
    }
    
    /// Run the main menu
    async fn main_menu(&mut self) -> Result<()> {
        let mut input = String::new();
        let devices = self.device_manager.get_input_devices();
        
        while self.running {
            println!("\nMain Menu:");
            println!("---------");
            println!("1. Start audio capture (default device)");
            println!("2. Select device and start capture");
            println!("3. Stop audio capture");
            println!("4. List audio devices");
            println!("5. Configure Whisper settings");
            println!("6. Exit");
            
            print!("> ");
            io::stdout().flush()?;
            
            input.clear();
            io::stdin().read_line(&mut input)?;
            
            match input.trim() {
                "1" => {
                    println!("Starting audio capture with default device...");
                    if let Err(e) = self.start_audio_capture(None).await {
                        error!("Failed to start audio capture: {}", e);
                    }
                },
                "2" => {
                    if devices.is_empty() {
                        println!("No devices available");
                        continue;
                    }
                    
                    println!("Select a device:");
                    for (i, (_, name)) in devices.iter().enumerate() {
                        println!("{}. {}", i + 1, name);
                    }
                    
                    print!("> ");
                    io::stdout().flush()?;
                    
                    input.clear();
                    io::stdin().read_line(&mut input)?;
                    
                    if let Ok(index) = input.trim().parse::<usize>() {
                        if index > 0 && index <= devices.len() {
                            let (id, _) = &devices[index - 1];
                            println!("Starting audio capture with device: {}", id);
                            
                            if let Err(e) = self.start_audio_capture(Some(id)).await {
                                error!("Failed to start audio capture: {}", e);
                            }
                        } else {
                            println!("Invalid device index");
                        }
                    } else {
                        println!("Invalid input");
                    }
                },
                "3" => {
                    println!("Stopping audio capture...");
                    self.stop_audio_capture().await;
                },
                "4" => {
                    self.list_audio_devices()?;
                },
                "5" => {
                    self.configure_whisper().await?;
                },
                "6" => {
                    println!("Exiting...");
                    self.running = false;
                },
                _ => {
                    println!("Invalid option");
                }
            }
        }
        
        Ok(())
    }
    
    /// Configure Whisper settings
    async fn configure_whisper(&mut self) -> Result<()> {
        let mut input = String::new();
        
        println!("\nWhisper Configuration:");
        println!("---------------------");
        
        {
            let config = self.config_manager.get_config();
            
            println!("Current model size: {:?}", config.audio.speech.model_size);
            println!("Select model size:");
            println!("1. Tiny (fastest, least accurate)");
            println!("2. Base (fast, less accurate)");
            println!("3. Small (balanced)");
            println!("4. Medium (more accurate)");
            println!("5. Large (most accurate)");
            
            if let Some(path) = &config.audio.speech.model_path {
                println!("Current model path: {}", path);
            } else {
                println!("Current model path: <default>");
            }
            
            println!("Current language: {}", if config.audio.speech.language.is_empty() { "<auto>" } else { &config.audio.speech.language });
            println!("Save transcription: {}", config.audio.speech.save_transcription);
            println!("Output format: {}", config.audio.speech.output_format);
        }
        
        // Get model size selection
        print!("\nSelect model size (1-5) > ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        
        let model_size = match input.trim() {
            "1" => crate::config::WhisperModelSize::Tiny,
            "2" => crate::config::WhisperModelSize::Base,
            "3" => crate::config::WhisperModelSize::Small,
            "4" => crate::config::WhisperModelSize::Medium,
            "5" => crate::config::WhisperModelSize::Large,
            _ => {
                println!("Invalid option, keeping current setting");
                self.config_manager.get_config().audio.speech.model_size.clone()
            },
        };
        
        // Get custom model path
        println!("\nCustom model path (leave empty for default):");
        print!("> ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        
        let model_path = {
            let input_path = input.trim();
            if input_path.is_empty() {
                None
            } else {
                Some(input_path.to_string())
            }
        };
        
        // Get language
        println!("\nLanguage (leave empty for auto-detect):");
        print!("> ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        
        let language = input.trim().to_string();
        
        // Get save transcription option
        println!("\nSave transcription to file (y/n):");
        print!("> ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        
        let save_transcription = input.trim().to_lowercase().starts_with('y');
        
        // Get output format if saving is enabled
        let output_format = if save_transcription {
            println!("\nOutput format (txt/json):");
            print!("> ");
            io::stdout().flush()?;
            
            input.clear();
            io::stdin().read_line(&mut input)?;
            
            let format = input.trim().to_lowercase();
            if format == "json" {
                "json".to_string()
            } else {
                "txt".to_string()
            }
        } else {
            "txt".to_string()
        };
        
        // Update configuration
        {
            let config = self.config_manager.get_config_mut();
            config.audio.speech.model_size = model_size;
            config.audio.speech.model_path = model_path;
            config.audio.speech.language = language;
            config.audio.speech.save_transcription = save_transcription;
            config.audio.speech.output_format = output_format;
        }
        
        // Save configuration
        self.config_manager.save()?;
        
        // Recreate transcription manager if it exists
        if self.transcription_manager.is_some() {
            let speech_settings = self.config_manager.get_config().audio.speech.clone();
            let (transcription_manager, transcription_receiver) = TranscriptionManager::new(speech_settings)
                .context("Failed to create transcription manager")?;
            
            self.transcription_manager = Some(transcription_manager);
            self.transcription_receiver = Some(transcription_receiver);
        }
        
        println!("Whisper configuration saved");
        
        Ok(())
    }

    // Add a method to cleanly shut down async tasks
    async fn shutdown_async_tasks(&mut self) -> Result<()> {
        info!("Shutting down async tasks");
        
        // First, stop the audio capture to prevent new events
        if let Some(capture_manager) = &mut self.capture_manager {
            let _ = capture_manager.stop();
        }
        
        // Stop transcription if running
        if let Some(transcription_manager) = &mut self.transcription_manager {
            if let Err(e) = transcription_manager.stop().await {
                warn!("Error stopping transcription: {}", e);
            }
        }
        
        // Take ownership of tasks
        let audio_task = self.audio_task.take();
        let transcription_task = self.transcription_task.take();
        
        // Wait for tasks to complete or force abort after timeout
        if let Some(task) = audio_task {
            if !task.is_finished() {
                match tokio::time::timeout(std::time::Duration::from_secs(2), task).await {
                    Ok(_) => info!("Audio task completed gracefully"),
                    Err(_) => {
                        warn!("Audio task did not complete within timeout, will be aborted");
                    }
                }
            }
        }
        
        if let Some(task) = transcription_task {
            if !task.is_finished() {
                match tokio::time::timeout(std::time::Duration::from_secs(2), task).await {
                    Ok(_) => info!("Transcription task completed gracefully"),
                    Err(_) => {
                        warn!("Transcription task did not complete within timeout, will be aborted");
                    }
                }
            }
        }
        
        // Clean up remaining resources
        self.capture_manager = None;
        
        info!("Async tasks shutdown complete");
        Ok(())
    }
} 
