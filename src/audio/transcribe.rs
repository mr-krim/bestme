use anyhow::{Context, Result};
use log::{info, warn, debug};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use thiserror::Error;

use crate::config::{SpeechSettings, WhisperModelSize};
use crate::audio::voice_commands::{VoiceCommandProcessor, VoiceCommandEvent};

#[cfg(feature = "storage")]
use crate::storage::{StorageManager, Transcript};

#[cfg(feature = "whisper")]
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy, WhisperContextParameters};

/// Buffer size for audio accumulation before processing
const AUDIO_BUFFER_SECONDS: usize = 3;
const SAMPLE_RATE: usize = 16000;

/// Custom error types for transcription
#[derive(Error, Debug)]
pub enum TranscriptionError {
    #[error("Failed to initialize Whisper model: {0}")]
    ModelInitialization(String),
    
    #[error("Failed to create Whisper state: {0}")]
    StateCreation(String),
    
    #[error("Failed to run inference: {0}")]
    InferenceFailure(String),
    
    #[error("Failed to process transcription segments: {0}")]
    SegmentProcessing(String),
    
    #[error("Task cancelled: {0}")]
    TaskCancelled(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Transcription manager for handling speech recognition using Whisper
#[derive(Clone)]
pub struct TranscriptionManager {
    /// Transcription settings
    settings: SpeechSettings,
    
    /// Path to model files
    model_path: PathBuf,
    
    /// Transcription state
    state: TranscriptionState,
    
    /// Sender for transcription events
    event_sender: mpsc::Sender<TranscriptionEvent>,
    
    /// Current transcription text
    current_text: Arc<Mutex<String>>,
    
    /// Voice command processor
    voice_command_processor: Option<Arc<Mutex<VoiceCommandProcessor>>>,
    
    /// Audio buffer for accumulating audio before processing
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    
    /// Whisper context (only with whisper feature)
    #[cfg(feature = "whisper")]
    whisper_context: Option<Arc<WhisperContext>>,
    
    /// Storage manager for saving transcripts
    #[cfg(feature = "storage")]
    storage_manager: Option<Arc<StorageManager>>,
}

/// Transcription state
#[derive(Debug, Clone, PartialEq)]
enum TranscriptionState {
    /// Not initialized
    Uninitialized,
    
    /// Ready to transcribe
    Ready,
    
    /// Currently transcribing
    Transcribing,
    
    /// Paused
    #[allow(dead_code)]
    Paused,
    
    /// Error state
    #[allow(dead_code)]
    Error(String),
}

/// Transcription events
#[derive(Debug, Clone)]
pub enum TranscriptionEvent {
    /// New transcription available
    Transcription(String),
    
    /// Partial transcription available
    PartialTranscription(String),
    
    /// Voice command detected and executed
    CommandExecuted {
        command: String,
        result: Result<String, String>,
    },
    
    /// Transcription started
    Started,
    
    /// Transcription stopped
    Stopped,
    
    /// Transcription error
    Error(String),
}

impl TranscriptionManager {
    /// Create a new transcription manager
    pub fn new(settings: SpeechSettings) -> Result<(Self, mpsc::Receiver<TranscriptionEvent>)> {
        let (event_sender, event_receiver) = mpsc::channel(100);
        
        // Determine model path
        let model_path = if let Some(path) = &settings.model_path {
            PathBuf::from(path)
        } else {
            Self::get_default_model_path()?
        };
        
        let manager = Self {
            settings,
            model_path,
            state: TranscriptionState::Uninitialized,
            event_sender,
            current_text: Arc::new(Mutex::new(String::new())),
            voice_command_processor: None, // Will be initialized when enabled
            audio_buffer: Arc::new(Mutex::new(Vec::with_capacity(AUDIO_BUFFER_SECONDS * SAMPLE_RATE))),
            #[cfg(feature = "whisper")]
            whisper_context: None,
            #[cfg(feature = "storage")]
            storage_manager: None,
        };
        
        Ok((manager, event_receiver))
    }
    
    /// Get the default model path
    fn get_default_model_path() -> Result<PathBuf> {
        // Look for models in config directory
        let project_dirs = directories::ProjectDirs::from("com", "bestme", "BestMe")
            .context("Failed to determine project directories")?;
        
        let models_dir = project_dirs.config_dir().join("models");
        
        // Create directory if it doesn't exist
        if !models_dir.exists() {
            std::fs::create_dir_all(&models_dir)
                .context("Failed to create models directory")?;
        }
        
        Ok(models_dir)
    }
    
    /// Initialize the transcription manager
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Whisper transcription with model size: {:?}", self.settings.model_size);
        
        // Initialize Whisper if the feature is enabled
        #[cfg(feature = "whisper")]
        {
            // Get model file path
            let model_file = self.model_path.join(format!("whisper-{}.bin", self.get_model_size_string()));
            
            if !model_file.exists() {
                warn!("Whisper model file not found: {:?}", model_file);
                warn!("Running in simulation mode without actual transcription");
            } else {
                info!("Loading Whisper model from {:?}", model_file);
                
                // Create context parameters with GPU support if available
                let params = WhisperContextParameters::new();
                
                // Enable GPU if any GPU feature is enabled
                #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal", feature = "gpu-vulkan", feature = "gpu-hipblas"))]
                {
                    params.use_gpu(true);
                    info!("GPU acceleration enabled for Whisper");
                }
                
                let whisper = WhisperContext::new_with_params(&model_file.to_string_lossy(), params)
                    .map_err(|e| anyhow::anyhow!("Failed to load whisper model: {}", e))?;
                self.whisper_context = Some(Arc::new(whisper));
                
                #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal", feature = "gpu-vulkan", feature = "gpu-hipblas"))]
                info!("Whisper model loaded successfully with GPU acceleration");
                
                #[cfg(not(any(feature = "gpu-cuda", feature = "gpu-metal", feature = "gpu-vulkan", feature = "gpu-hipblas")))]
                info!("Whisper model loaded successfully (CPU mode)");
            }
        }
        
        #[cfg(not(feature = "whisper"))]
        {
            info!("Whisper feature is not enabled, running in simulation mode");
        }
        
        self.state = TranscriptionState::Ready;
        info!("Whisper transcription initialized (simulation mode)");
        
        Ok(())
    }
    
    /// Start transcription
    pub async fn start(&mut self) -> Result<()> {
        if self.state == TranscriptionState::Uninitialized {
            self.initialize().await?;
        }
        
        if self.state == TranscriptionState::Transcribing {
            warn!("Transcription already running");
            return Ok(());
        }
        
        self.state = TranscriptionState::Transcribing;
        
        // Clear audio buffer
        {
            let mut buffer = self.audio_buffer.lock();
            buffer.clear();
        }
        
        // Send started event
        let _ = self.event_sender.send(TranscriptionEvent::Started).await;
        
        info!("Transcription started");
        
        Ok(())
    }
    
    /// Stop transcription
    pub async fn stop(&mut self) -> Result<()> {
        if self.state != TranscriptionState::Transcribing {
            warn!("Transcription not running");
            return Ok(());
        }
        
        self.state = TranscriptionState::Ready;
        
        // Process any remaining audio in the buffer
        self.process_buffer().await?;
        
        // Send stopped event
        let _ = self.event_sender.send(TranscriptionEvent::Stopped).await;
        
        info!("Transcription stopped");
        
        Ok(())
    }
    
    /// Process audio data for transcription
    pub async fn process_audio(&self, audio_data: &[f32]) -> Result<Option<String>> {
        if self.state != TranscriptionState::Transcribing {
            return Ok(None);
        }
        
        // Create a scope to ensure the lock is released before the await
        let buffer_clone = {
            let mut buffer = self.audio_buffer.lock();
            buffer.extend_from_slice(audio_data);
            
            // If buffer is large enough, process it
            if buffer.len() >= AUDIO_BUFFER_SECONDS * SAMPLE_RATE {
                let buffer_clone = buffer.clone();
                buffer.clear();
                Some(buffer_clone)
            } else {
                None
            }
            // Lock is released here when buffer goes out of scope
        };
        
        // Process the audio buffer if we got a clone
        if let Some(buffer) = buffer_clone {
            self.transcribe_audio(&buffer).await
        } else {
            Ok(None)
        }
    }
    
    /// Process the current audio buffer
    async fn process_buffer(&self) -> Result<()> {
        // Create a scope to ensure the lock is released before the await
        let buffer_to_process = {
            let buffer = self.audio_buffer.lock();
            if buffer.is_empty() {
                None
            } else {
                Some(buffer.clone())
            }
            // Lock is released here when buffer goes out of scope
        };
        
        // Process the buffer if we have one
        if let Some(buffer) = buffer_to_process {
            self.transcribe_audio(&buffer).await?;
        }
        
        Ok(())
    }
    
    /// Transcribe audio data
    #[cfg(feature = "whisper")]
    async fn transcribe_audio(&self, audio_data: &[f32]) -> Result<Option<String>> {
        // Ensure we have a whisper context
        if let Some(context) = &self.whisper_context {
            // Gather settings needed for the closure first
            let language = self.settings.language.clone();
            let translate_to_english = self.settings.translate_to_english;
            
            // Set up parameters and clone context and data for the blocking task
            let context = Arc::clone(context);
            let audio_data = audio_data.to_vec(); // Create owned copy for the blocking task
            
            // Spawn a blocking task for CPU-intensive processing
            let transcription = tokio::task::spawn_blocking(move || {
                // Set up parameters inside the closure
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                
                // Configure language settings 
                if language.is_empty() || language == "auto" {
                    params.set_language(None);
                } else {
                    params.set_language(Some(&language));
                }
                
                // Configure translation if needed
                if translate_to_english {
                    params.set_translate(true);
                }
                
                // Create the state
                let mut state = match context.create_state() {
                    Ok(state) => state,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to create whisper state: {}", e));
                    }
                };
                
                // Run inference
                if let Err(e) = state.full(params, &audio_data) {
                    return Err(anyhow::anyhow!("Failed to run inference: {}", e));
                }
                
                // Extract text from segments
                let num_segments = match state.full_n_segments() {
                    Ok(n) => n,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to get segments: {}", e));
                    }
                };
                
                let mut text = String::new();
                
                for i in 0..num_segments {
                    if let Ok(segment) = state.full_get_segment_text(i) {
                        text.push_str(&segment);
                        text.push(' ');
                    }
                }
                
                let result = text.trim().to_string();
                if result.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(result))
                }
            }).await.context("Failed to run transcription task")?;
            
            // Handle the transcription result
            match transcription {
                Ok(Some(text)) => {
                    // Update current text
                    {
                        let mut current = self.current_text.lock();
                        *current = text.clone();
                    }
                    
                    // Check for voice commands first
                    if let Some((command, result)) = self.process_for_commands(&text).await {
                        // Send command event
                        if let Err(e) = self.event_sender.send(TranscriptionEvent::CommandExecuted {
                            command,
                            result,
                        }).await {
                            warn!("Failed to send command event: {}", e);
                        }
                    } else {
                        // No command detected, send as regular transcription
                        // Handle post-processing
                        if self.settings.save_transcription {
                            if let Err(e) = self.save_transcription(&text).await {
                                warn!("Failed to save transcription: {}", e);
                            }
                        }
                        
                        // Send transcription event
                        if let Err(e) = self.event_sender.send(TranscriptionEvent::Transcription(text.clone())).await {
                            warn!("Failed to send transcription event: {}", e);
                        }
                    }
                    
                    Ok(Some(text))
                },
                Ok(None) => Ok(None),
                Err(e) => {
                    // Forward the error
                    Err(e)
                }
            }
        } else {
            self.simulate_transcription().await
        }
    }
    
    /// Transcribe audio data (simulation when whisper is not enabled)
    #[cfg(not(feature = "whisper"))]
    async fn transcribe_audio(&self, _audio_data: &[f32]) -> Result<Option<String>> {
        // Simulation mode - generate some fake transcription
        self.simulate_transcription().await
    }
    
    /// Generate a simulated transcription for testing
    async fn simulate_transcription(&self) -> Result<Option<String>> {
        // Add a small delay to simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let fake_text = format!("This is a simulated transcription with the {} model", 
                              self.get_model_size_string());
        
        // Update current text
        {
            let mut current = self.current_text.lock();
            *current = fake_text.clone();
        }
        
        // Check for voice commands in simulated text
        if let Some((command, result)) = self.process_for_commands(&fake_text).await {
            // Send command event
            if let Err(e) = self.event_sender.send(TranscriptionEvent::CommandExecuted {
                command,
                result,
            }).await {
                warn!("Failed to send command event: {}", e);
            }
        } else {
            // Send the simulated text
            if let Err(e) = self.event_sender.send(TranscriptionEvent::Transcription(fake_text.clone())).await {
                warn!("Failed to send simulated transcription: {}", e);
            }
        }
        
        Ok(Some(fake_text))
    }
    
    /// Save transcription to file and/or storage
    async fn save_transcription(&self, text: &str) -> Result<()> {
        if !self.settings.save_transcription {
            return Ok(());
        }
        
        // Save to storage if available
        #[cfg(feature = "storage")]
        if let Some(storage) = &self.storage_manager {
            // Create transcript
            let mut transcript = Transcript::new(
                storage.current_session_id().unwrap_or_default(),
                text.to_string(),
            );
            
            // Set metadata
            transcript.language = Some(
                if self.settings.language.is_empty() || self.settings.language == "auto" {
                    "auto".to_string()
                } else {
                    self.settings.language.clone()
                }
            );
            transcript.model_size = Some(self.get_model_size_string().to_string());
            transcript.metadata.translated = self.settings.translate_to_english;
            transcript.metadata.auto_punctuated = self.settings.auto_punctuate;
            
            // Save to storage
            match storage.save_transcript(transcript).await {
                Ok(id) => debug!("Saved transcript to storage: {}", id),
                Err(e) => warn!("Failed to save transcript to storage: {}", e),
            }
        }
        
        // Create transcription directory if it doesn't exist
        let project_dirs = directories::ProjectDirs::from("com", "bestme", "BestMe")
            .context("Failed to determine project directories")?;
        
        let transcription_dir = project_dirs.data_dir().join("transcriptions");
        
        if !transcription_dir.exists() {
            std::fs::create_dir_all(&transcription_dir)
                .context("Failed to create transcriptions directory")?;
        }
        
        // Create a filename with timestamp
        let now = chrono::Local::now();
        let filename = match self.settings.output_format.as_str() {
            "json" => format!("transcription_{}.json", now.format("%Y%m%d_%H%M%S")),
            _ => format!("transcription_{}.txt", now.format("%Y%m%d_%H%M%S")),
        };
        
        let file_path = transcription_dir.join(filename);
        
        // Write the file
        match self.settings.output_format.as_str() {
            "json" => {
                let json = serde_json::json!({
                    "timestamp": now.to_rfc3339(),
                    "text": text,
                    "model": self.get_model_size_string(),
                    "language": if self.settings.language.is_empty() { "auto" } else { &self.settings.language },
                });
                
                tokio::fs::write(file_path, serde_json::to_string_pretty(&json)?)
                    .await
                    .context("Failed to write JSON transcription file")?;
            },
            _ => {
                tokio::fs::write(file_path, text)
                    .await
                    .context("Failed to write TXT transcription file")?;
            }
        }
        
        Ok(())
    }
    
    /// Get the current transcription text
    pub fn get_current_text(&self) -> String {
        self.current_text.lock().clone()
    }
    
    /// Get model size string
    pub fn get_model_size_string(&self) -> &'static str {
        match self.settings.model_size {
            WhisperModelSize::Tiny => "tiny",
            WhisperModelSize::Base => "base",
            WhisperModelSize::Small => "small",
            WhisperModelSize::Medium => "medium",
            WhisperModelSize::Large => "large",
        }
    }
    
    /// Get the transcription settings
    pub fn get_settings(&self) -> &SpeechSettings {
        &self.settings
    }
    
    /// Update the transcription settings
    pub fn update_settings(&mut self, settings: SpeechSettings) {
        self.settings = settings;
    }
    
    // Add an alias method for compatibility
    #[allow(dead_code)]
    fn get_model_size_name(&self) -> &'static str {
        self.get_model_size_string()
    }
    
    /// Enable voice command processing
    pub fn enable_voice_commands(&mut self, processor: Arc<Mutex<VoiceCommandProcessor>>) {
        self.voice_command_processor = Some(processor);
        info!("Voice command processing enabled");
    }
    
    /// Disable voice command processing
    pub fn disable_voice_commands(&mut self) {
        self.voice_command_processor = None;
        info!("Voice command processing disabled");
    }
    
    /// Set storage manager for saving transcripts
    #[cfg(feature = "storage")]
    pub fn set_storage_manager(&mut self, storage_manager: Arc<StorageManager>) {
        self.storage_manager = Some(storage_manager);
        info!("Storage manager set for transcription saving");
    }
    
    /// Process transcribed text for voice commands
    async fn process_for_commands(&self, text: &str) -> Option<(String, Result<String, String>)> {
        if let Some(processor) = &self.voice_command_processor {
            let mut processor = processor.lock();
            
            // Check if the text contains a voice command
            if let Some(event) = processor.process_text(text) {
                match event {
                    VoiceCommandEvent::CommandDetected(command) => {
                        info!("Voice command detected: {:?}", command);
                        
                        // Execute the command and get the result
                        let result = processor.execute_command(&command);
                        
                        // Return command info for event
                        return Some((
                            format!("{:?}", command.command_type),
                            result
                        ));
                    }
                    VoiceCommandEvent::Error(err) => {
                        warn!("Voice command error: {}", err);
                        return Some((
                            "Error".to_string(),
                            Err(err)
                        ));
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::voice_commands::{VoiceCommandConfig, VoiceCommandManager};
    
    #[tokio::test]
    async fn test_voice_command_integration() {
        // Create a transcription manager
        let speech_settings = SpeechSettings::default();
        let (mut manager, mut receiver) = TranscriptionManager::new(speech_settings).unwrap();
        
        // Create and enable voice command processor
        let voice_config = VoiceCommandConfig::default();
        let (processor, _cmd_receiver) = VoiceCommandManager::new(voice_config).unwrap();
        let processor = Arc::new(Mutex::new(processor));
        manager.enable_voice_commands(processor);
        
        // Simulate transcription with a command
        manager.process_for_commands("delete last word").await;
        
        // The actual processing happens in process_transcription_internal
        // For now, just verify the setup works
        assert!(manager.voice_command_processor.is_some());
    }
    
    #[tokio::test]
    async fn test_command_detection_in_transcription() {
        let speech_settings = SpeechSettings::default();
        let (mut manager, mut receiver) = TranscriptionManager::new(speech_settings).unwrap();
        
        // Enable voice commands
        let voice_config = VoiceCommandConfig {
            enabled: true,
            sensitivity: 0.7,
            ..Default::default()
        };
        let (processor, _cmd_receiver) = VoiceCommandManager::new(voice_config).unwrap();
        let processor = Arc::new(Mutex::new(processor));
        manager.enable_voice_commands(processor.clone());
        
        // Test command detection
        let result = manager.process_for_commands("please delete the last word").await;
        assert!(result.is_some());
        
        if let Some((command, result)) = result {
            assert_eq!(command, "Delete");
            assert!(result.is_ok());
        }
    }
    
    #[tokio::test]
    async fn test_regular_transcription_without_commands() {
        let speech_settings = SpeechSettings::default();
        let (mut manager, mut receiver) = TranscriptionManager::new(speech_settings).unwrap();
        
        // Enable voice commands
        let voice_config = VoiceCommandConfig::default();
        let (processor, _cmd_receiver) = VoiceCommandManager::new(voice_config).unwrap();
        let processor = Arc::new(Mutex::new(processor));
        manager.enable_voice_commands(processor);
        
        // Test text without commands
        let result = manager.process_for_commands("This is just regular text").await;
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_disable_voice_commands() {
        let speech_settings = SpeechSettings::default();
        let (mut manager, _receiver) = TranscriptionManager::new(speech_settings).unwrap();
        
        // Enable then disable
        let voice_config = VoiceCommandConfig::default();
        let (processor, _cmd_receiver) = VoiceCommandManager::new(voice_config).unwrap();
        let processor = Arc::new(Mutex::new(processor));
        manager.enable_voice_commands(processor);
        assert!(manager.voice_command_processor.is_some());
        
        manager.disable_voice_commands();
        assert!(manager.voice_command_processor.is_none());
        
        // Commands should not be detected when disabled
        let result = manager.process_for_commands("delete last word").await;
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_command_execution_result() {
        let speech_settings = SpeechSettings::default();
        let (mut manager, _receiver) = TranscriptionManager::new(speech_settings).unwrap();
        
        // Enable voice commands
        let voice_config = VoiceCommandConfig::default();
        let (processor, _cmd_receiver) = VoiceCommandManager::new(voice_config).unwrap();
        let processor = Arc::new(Mutex::new(processor));
        manager.enable_voice_commands(processor);
        
        // Test successful command execution
        let result = manager.process_for_commands("delete").await;
        assert!(result.is_some());
        
        if let Some((command, exec_result)) = result {
            assert_eq!(command, "Delete");
            assert!(exec_result.is_ok());
        }
        
        // Test command with longer text
        let result = manager.process_for_commands("can you please delete the last word").await;
        assert!(result.is_some());
    }
} 
