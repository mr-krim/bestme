use anyhow::{Result, anyhow};
use log::{info, debug, error, warn};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs;
use tauri::{AppHandle, State, plugin::Plugin, Runtime, Emitter, Manager};
use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;
// use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};
use futures::StreamExt;
use serde_json::json;
use std::marker::PhantomData;

use bestme::audio::capture::AudioData;
use bestme::audio::voice_commands::{VoiceCommandManager as VoiceCommandProcessor, VoiceCommandEvent};
use bestme::audio::vad::{VoiceActivityDetector, VADResult};
use bestme::audio::enhanced_transcribe::{EnhancedWhisperProcessor, HallucinationDetector};
use bestme::audio::vocabulary::{VocabularyManager, VocabularyEntry};
use bestme::config::{ConfigManager, WhisperModelSize};

// Constants for audio processing
const WHISPER_SAMPLE_RATE: usize = 16000;
const AUDIO_BUFFER_SIZE: usize = WHISPER_SAMPLE_RATE * 5; // 5 seconds of audio
const MAX_TEXT_LENGTH: usize = 8192;

/// The model URLs for each Whisper model size
const MODEL_URLS: [(&str, &str); 5] = [
    ("tiny", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"),
    ("base", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"),
    ("small", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"),
    ("medium", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"),
    ("large", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large.bin"),
];

/// Supported language codes for Whisper
pub const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("auto", "Auto-detect"),
    ("en", "English"),
    ("zh", "Chinese"),
    ("de", "German"),
    ("es", "Spanish"),
    ("ru", "Russian"),
    ("ko", "Korean"),
    ("fr", "French"),
    ("ja", "Japanese"),
    ("pt", "Portuguese"),
    ("tr", "Turkish"),
    ("pl", "Polish"),
    ("it", "Italian"),
    ("nl", "Dutch"),
    ("ar", "Arabic"),
    ("hi", "Hindi"),
    ("id", "Indonesian"),
    ("fi", "Finnish"),
    ("vi", "Vietnamese"),
    ("he", "Hebrew"),
    ("uk", "Ukrainian"),
    ("sv", "Swedish"),
    ("cs", "Czech"),
    ("el", "Greek"),
    ("ro", "Romanian"),
    ("da", "Danish"),
    ("hu", "Hungarian"),
    ("th", "Thai"),
    ("fa", "Persian"),
    ("bg", "Bulgarian"),
    ("sk", "Slovak"),
    ("ca", "Catalan"),
    ("hr", "Croatian"),
    ("lt", "Lithuanian"),
    ("et", "Estonian"),
    ("sl", "Slovenian"),
    ("lv", "Latvian"),
    ("mk", "Macedonian"),
    ("sr", "Serbian"),
    ("az", "Azerbaijani"),
];

// Structure to hold transcription state
pub struct TranscribeState {
    config_manager: Arc<Mutex<ConfigManager>>,
    transcription_text: Arc<Mutex<String>>,
    transcription_active: Arc<Mutex<bool>>,
    audio_receiver: Arc<Mutex<Option<mpsc::Receiver<AudioData>>>>,
    audio_sender: Arc<Mutex<Option<mpsc::Sender<AudioData>>>>,
    // whisper_context: Arc<Mutex<Option<WhisperContext>>>,
    processor: Arc<Mutex<Option<Arc<EnhancedWhisperProcessor>>>>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    app_handle: Option<AppHandle>,
    download_progress: Arc<Mutex<Option<(String, f32)>>>, // (model_size, progress 0.0-1.0)
    get_model_path: Arc<dyn Fn(&str) -> PathBuf + Send + Sync>,
    voice_command_processor: Arc<Mutex<Option<VoiceCommandProcessor>>>,
    enhanced_processor: Arc<Mutex<Option<EnhancedWhisperProcessor>>>,
    vad: Arc<Mutex<Option<VoiceActivityDetector>>>,
    hallucination_detector: Arc<HallucinationDetector>,
    vocabulary_manager: Arc<Mutex<Option<VocabularyManager>>>,
}

impl TranscribeState {
    pub fn new(config_manager: Arc<Mutex<ConfigManager>>, app_handle: Option<AppHandle>) -> Result<Self, anyhow::Error> {
        let (audio_sender, audio_receiver) = tokio::sync::mpsc::channel(100);
        
        // Default function to get model path - uses app directory
        let config_manager_clone = config_manager.clone();
        let app_handle_clone = app_handle.clone();
        let get_model_path: Arc<dyn Fn(&str) -> PathBuf + Send + Sync> = Arc::new(move |model_size| {
            // First check if there's a custom model path in config
            let custom_path = {
                let config_manager = config_manager_clone.lock();
                if let Some(speech) = config_manager.get_config().audio.speech.model_path.as_ref() {
                    Some(speech.clone())
                } else {
                    None
                }
            };
            
            if let Some(path) = custom_path {
                if !path.trim().is_empty() {
                    return PathBuf::from(path)
                        .join(format!("ggml-{}.bin", model_size));
                }
            }
            
            // Otherwise use app directory for models
            let app_dir = app_handle_clone.as_ref().and_then(|handle| {
                handle.path().app_data_dir().ok()
            }).unwrap_or_else(|| {
                PathBuf::from(".")
            });
            
            // Ensure models directory exists
            let models_dir = app_dir.join("models");
            if !models_dir.exists() {
                let _ = fs::create_dir_all(&models_dir);
            }
            
            models_dir.join(format!("ggml-{}.bin", model_size))
        });
        
        // Initialize voice command processor if enabled
        let voice_command_processor = {
            let config = config_manager.lock();
            if config.get_config().audio.voice_commands.enabled {
                match VoiceCommandProcessor::new(config.get_config().audio.voice_commands.clone()) {
                    Ok((processor, _)) => {
                        info!("Voice commands enabled in Tauri plugin");
                        Some(processor)
                    },
                    Err(e) => {
                        warn!("Failed to create voice command processor: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        };
        
        Ok(Self {
            config_manager: config_manager.clone(),
            transcription_text: Arc::new(Mutex::new(String::new())),
            transcription_active: Arc::new(Mutex::new(false)),
            audio_receiver: Arc::new(Mutex::new(Some(audio_receiver))),
            audio_sender: Arc::new(Mutex::new(Some(audio_sender))),
            processor: Arc::new(Mutex::new(None)),
            audio_buffer: Arc::new(Mutex::new(Vec::with_capacity(AUDIO_BUFFER_SIZE))),
            app_handle: app_handle.clone(),
            download_progress: Arc::new(Mutex::new(None)),
            get_model_path,
            voice_command_processor: Arc::new(Mutex::new(voice_command_processor)),
            enhanced_processor: Arc::new(Mutex::new(None)),
            vad: Arc::new(Mutex::new(None)),
            hallucination_detector: Arc::new(HallucinationDetector::new()),
            vocabulary_manager: Arc::new(Mutex::new(None)),
        })
    }
    
    pub fn set_app_handle(&mut self, app_handle: AppHandle) -> Result<()> {
        self.app_handle = Some(app_handle);
        Ok(())
    }
    
    /// Initialize vocabulary manager
    pub fn initialize_vocabulary_manager(&self) -> Result<()> {
        if let Some(app_handle) = &self.app_handle {
            let app_data_dir = app_handle.path()
                .app_data_dir()
                .map_err(|_| anyhow!("Could not determine app data directory"))?;
            
            let vocab_path = app_data_dir.join("vocabulary.json");
            
            match VocabularyManager::new(vocab_path) {
                Ok(manager) => {
                    *self.vocabulary_manager.lock() = Some(manager);
                    info!("Vocabulary manager initialized");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to initialize vocabulary manager: {}", e);
                    Err(e)
                }
            }
        } else {
            Err(anyhow!("App handle not set"))
        }
    }

    pub fn create_audio_channel(&self) -> mpsc::Sender<AudioData> {
        let (sender, receiver) = mpsc::channel::<AudioData>(100);
        
        {
            let mut audio_sender = self.audio_sender.lock();
            *audio_sender = Some(sender.clone());
        }
        
        {
            let mut audio_receiver = self.audio_receiver.lock();
            *audio_receiver = Some(receiver);
        }
        
        sender
    }

    pub fn get_transcription(&self) -> String {
        let text = self.transcription_text.lock();
        text.clone()
    }
    
    pub fn get_download_progress(&self) -> Option<(String, f32)> {
        let progress = self.download_progress.lock();
        progress.clone()
    }

    // Load Whisper model based on model size
    async fn load_whisper_model(&self, model_size: &WhisperModelSize) -> Result<()> {
        // Get model path from config or use default path
        let model_path = (self.get_model_path)(self.get_model_size_string(model_size));
        
        info!("Loading Whisper model: {:?} from {:?}", model_size, model_path);
        
        // Check if model exists, if not, try to download it
        if !model_path.exists() {
            info!("Model file not found, attempting to download it");
            self.download_model(model_size, &model_path).await?;
        }
        
        // Load model in a blocking task since it's CPU-intensive
        let model_path_str = model_path.to_string_lossy().to_string();
        // TODO: Implement whisper model loading without whisper_rs
        return Err(anyhow!("Whisper model loading not implemented without whisper_rs"));
        
        /*
        match tokio::task::spawn_blocking(move || {
            // Use the new_with_params method instead of the deprecated new method
            WhisperContext::new_with_params(&model_path_str, Default::default())
        }).await? {
            Ok(context) => {
                // let mut whisper_context = self.whisper_context.lock();
                // *whisper_context = Some(context);
                info!("Whisper model loaded successfully");
                Ok(())
            },
            Err(e) => {
                error!("Failed to load Whisper model: {}", e);
                Err(anyhow::anyhow!("Failed to load Whisper model: {}", e))
            }
        }*/
    }
    
    // Get model path based on model size
    fn get_model_path(&self, model_size: &WhisperModelSize) -> PathBuf {
        (self.get_model_path)(self.get_model_size_string(model_size))
    }
    
    // Download the model
    async fn download_model(&self, model_size: &WhisperModelSize, model_path: &Path) -> Result<()> {
        // Check offline mode setting first
        let offline_enabled = self.config_manager.lock().get_config().general.offline_mode;
        if offline_enabled {
            error!("Offline mode is enabled. Cannot download model: {:?}", model_size);
            return Err(anyhow!("Download prevented: Offline mode is enabled."));
        }

        let model_name = self.get_model_size_string(model_size);
        
        // Find the URL for the specified model
        let model_url = MODEL_URLS
            .iter()
            .find(|(size, _)| *size == model_name)
            .map(|(_, url)| *url)
            .ok_or_else(|| anyhow::anyhow!("Model URL not found for size: {}", model_name))?;
        
        info!("Downloading Whisper model from: {}", model_url);
        
        // Update download progress state to indicate we're starting
        {
            let mut progress = self.download_progress.lock();
            *progress = Some((model_name.to_string(), 0.0));
        }
        
        // Create a client for downloading
        let client = reqwest::Client::new();
        
        // Start a streaming download
        let response = client.get(model_url).send().await?;
        let total_size = response.content_length().unwrap_or(0);
        
        if total_size == 0 {
            return Err(anyhow::anyhow!("Could not determine file size"));
        }
        
        // Create a temp file and download to it
        let temp_path = model_path.with_extension("tmp");
        let mut file = tokio::fs::File::create(&temp_path).await?;
        let mut stream = response.bytes_stream();
        
        let progress = Arc::clone(&self.download_progress);
        let app_handle = self.app_handle.clone();
        
        let mut downloaded_bytes: u64 = 0;
        let mut last_progress: f32 = 0.0;
        
        while let Some(item) = stream.next().await {
            let chunk = match item {
                Ok(chunk) => chunk,
                Err(e) => return Err(anyhow::anyhow!("Error during download: {}", e)),
            };
            
            // Write the chunk to file
            file.write_all(&chunk).await?;
            
            // Update download progress
            downloaded_bytes += chunk.len() as u64;
            let current_progress = downloaded_bytes as f32 / total_size as f32;
            
            // Only update progress if it's changed significantly (avoid UI spam)
            if current_progress - last_progress > 0.01 {
                last_progress = current_progress;
                
                // Update progress in state
                {
                    let mut p = progress.lock();
                    *p = Some((model_name.to_string(), current_progress));
                }
                
                // Emit download progress event to frontend
                if let Some(handle) = &app_handle {
                    let _ = handle.emit(
                        "transcribe:download-progress", 
                        json!({
                            "model": model_name,
                            "progress": current_progress
                        })
                    );
                }
            }
        }
        
        // Ensure the file is fully written to disk
        file.flush().await?;
        
        // Close the file
        drop(file);
        
        // Rename the temporary file to the final file
        tokio::fs::rename(&temp_path, model_path).await?;
        
        // Reset progress
        {
            let mut p = progress.lock();
            *p = None;
        }
        
        info!("Model download completed: {}", model_path.display());
        Ok(())
    }
    
    // Process audio buffer using Whisper with enhanced features
    async fn process_audio_buffer(&self, audio_buffer: Vec<f32>) -> Result<String> {
        let config = self.config_manager.lock().get_config().clone();
        let speech_config = config.audio.speech.clone();
        let whisper_params = config.whisper_params.clone();
        
        // Apply VAD if enabled
        if whisper_params.vad_enabled {
            let mut vad_lock = self.vad.lock();
            if vad_lock.is_none() {
                *vad_lock = Some(VoiceActivityDetector::new(
                    whisper_params.vad_threshold,
                    whisper_params.min_speech_duration_ms,
                    whisper_params.max_silence_duration_ms,
                    WHISPER_SAMPLE_RATE,
                ));
            }
            
            if let Some(vad) = vad_lock.as_mut() {
                match vad.process(&audio_buffer) {
                    VADResult::Silence => {
                        debug!("VAD: Silence detected, skipping processing");
                        return Ok(String::new());
                    }
                    VADResult::SpeechEnd => {
                        debug!("VAD: Speech ended, processing buffer");
                    }
                    _ => {}
                }
            }
        }
        
        // For now, always use basic processing to avoid the lock issue
        // TODO: Refactor enhanced processor to be Send-safe
        self.process_audio_buffer_basic(audio_buffer).await
    }
    
    // Basic audio buffer processing (fallback)
    async fn process_audio_buffer_basic(&self, audio_buffer: Vec<f32>) -> Result<String> {
        // Placeholder implementation until whisper_rs is re-enabled
        Ok(String::new())
        
        /* Commented out whisper_context usage
        let context_lock = self.whisper_context.lock();
        let context = context_lock.as_ref().ok_or_else(|| anyhow!("Whisper context not loaded"))?;
        
        // Get relevant settings from config
        let speech_config = self.config_manager.lock().get_config().audio.speech.clone();
        
        debug!("Processing audio buffer (basic mode). Language: {}, Translate: {}", 
               speech_config.language.clone().unwrap_or_else(|| "auto".to_string()), 
               speech_config.translate_to_english);

        // Create parameters for transcription using the helper function
        // let mut params = self.create_whisper_params(&speech_config);
        
        // Configure other parameters as needed
        // params.set_print_special_tokens(false);
        // params.set_print_progress(false);
        // params.set_print_realtime(false);
        // params.set_print_timestamps(false);
        
        // Run transcription in a blocking task
        // let params_clone = params.clone();
        let audio_buffer_clone = audio_buffer.clone();
        /* Commented out whisper_rs code
        let result = tokio::task::spawn_blocking(move || {
            let mut state = context.create_state()?;
            state.full(params_clone, &audio_buffer_clone)?;

            let num_segments = state.full_n_segments()?;
            let mut result_text = String::new();
            for i in 0..num_segments {
                if let Ok(segment) = state.full_get_segment_text(i) {
                    result_text.push_str(&segment);
                } else {
                    warn!("Failed to get segment {}", i);
                }
            }
            Ok::<String, anyhow::Error>(result_text)
        }).await??;
        */
        
        // Temporary placeholder implementation
        let result = String::new();

        debug!("Transcription segment result: {}", result);
        Ok(result)
        */
    }
    
    // Get model size string from enum
    fn get_model_size_string(&self, model_size: &WhisperModelSize) -> &'static str {
        match model_size {
            WhisperModelSize::Tiny => "tiny",
            WhisperModelSize::Base => "base",
            WhisperModelSize::Small => "small",
            WhisperModelSize::Medium => "medium",
            WhisperModelSize::Large => "large",
        }
    }
    
    // Start transcription
    pub fn start_transcription(&self) -> Result<()> {
        info!("Starting transcription");
        
        // Already active?
        if *self.transcription_active.lock() {
            return Ok(());
        }
        
        // Set active flag
        {
            let mut active = self.transcription_active.lock();
            *active = true;
        }
        
        // Start processing audio
        let audio_receiver = {
            let mut receiver = self.audio_receiver.lock();
            receiver.take()
        };
        
        if let Some(mut receiver) = audio_receiver {
            let audio_buffer = Arc::clone(&self.audio_buffer);
            let transcription_text = Arc::clone(&self.transcription_text);
            let transcription_active = Arc::clone(&self.transcription_active);
            let config_manager = Arc::clone(&self.config_manager);
            // let whisper_context = Arc::clone(&self.whisper_context);
            let self_clone = self.clone();
            // Create an event channel for sending events from the async task
            let (event_tx, mut event_rx) = mpsc::channel::<(String, serde_json::Value)>(100);
            
            // Spawn a task to handle events with app_handle
            if let Some(handle) = self.app_handle.clone() {
                tokio::spawn(async move {
                    while let Some((event, payload)) = event_rx.recv().await {
                        let _ = handle.emit(&event, payload);
                    }
                });
            }
            
            // Spawn a task to process audio data
            tokio::spawn(async move {
                let mut _buffer_timer = tokio::time::interval(std::time::Duration::from_secs(1));
                
                // Load model eagerly
                {
                    let config = config_manager.lock().get_config().audio.speech.clone();
                    if let Err(e) = self_clone.load_whisper_model(&config.model_size).await {
                        error!("Failed to load Whisper model: {}", e);
                        
                        // Update active flag
                        {
                            let mut active = transcription_active.lock();
                            *active = false;
                        }
                        
                        // Emit error event to frontend
                        let _ = event_tx.send((
                            "transcribe:error".to_string(),
                            json!({
                                "error": format!("Failed to load Whisper model: {}", e)
                            })
                        )).await;
                        
                        return;
                    }
                }
                
                // Custom buffer handling
                let mut last_processed = std::time::Instant::now();
                let segment_duration = {
                    let config = config_manager.lock().get_config().audio.speech.clone();
                    std::time::Duration::from_secs_f32(config.segment_duration)
                };
                
                while let Some(audio_data) = receiver.recv().await {
                    if !*transcription_active.lock() {
                        break;
                    }
                    
                    // Add to buffer
                    {
                        let mut buffer = audio_buffer.lock();
                        buffer.extend(audio_data.get_samples().iter());
                        
                        // Resize if buffer is too large
                        if buffer.len() > AUDIO_BUFFER_SIZE {
                            let drain_amount = buffer.len() - AUDIO_BUFFER_SIZE;
                            buffer.drain(0..drain_amount);
                        }
                    }
                    
                    // Check if it's time to process the buffer
                    let now = std::time::Instant::now();
                    if now.duration_since(last_processed) >= segment_duration {
                        // Process the buffer
                        let buffer_copy = {
                            let buffer = audio_buffer.lock();
                            buffer.clone()
                        };
                        
                        // Skip if buffer is empty
                        if buffer_copy.is_empty() {
                            continue;
                        }
                        
                        // Process the buffer
                        match self_clone.process_audio_buffer(buffer_copy).await {
                            Ok(text) => {
                                if !text.trim().is_empty() {
                                    // Update transcription text
                                    {
                                        let mut t = transcription_text.lock();
                                        // Add the new text with a space
                                        if !t.is_empty() && !t.ends_with(' ') {
                                            *t += " ";
                                        }
                                        *t += &text;
                                        
                                        // Trim to MAX_TEXT_LENGTH
                                        if t.len() > MAX_TEXT_LENGTH {
                                            *t = t.chars().skip(t.len() - MAX_TEXT_LENGTH).collect();
                                        }
                                    }
                                    
                                    // Check for voice commands
                                    let command_event_data = {
                                        let mut processor_guard = self_clone.voice_command_processor.lock();
                                        if let Some(processor) = processor_guard.as_mut() {
                                            if let Some(event) = processor.process_text(&text) {
                                                match event {
                                                    VoiceCommandEvent::CommandDetected(command) => {
                                                        info!("Voice command detected in Tauri: {:?}", command);
                                                        
                                                        // Execute the command
                                                        let result = processor.execute_command(&command);
                                                        
                                                        // Prepare event data
                                                        Some(json!({
                                                            "command": format!("{:?}", command.command_type),
                                                            "success": result.is_ok(),
                                                            "result": result.as_ref().ok().cloned().unwrap_or_default(),
                                                            "error": result.as_ref().err().cloned().unwrap_or_default()
                                                        }))
                                                    },
                                                    VoiceCommandEvent::Error(err) => {
                                                        warn!("Voice command error: {}", err);
                                                        None
                                                    }
                                                }
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    };
                                    
                                    // Send event outside of lock
                                    let voice_command_detected = if let Some(event_data) = command_event_data {
                                        let _ = event_tx.send((
                                            "voice-command:executed".to_string(),
                                            event_data
                                        )).await;
                                        true
                                    } else {
                                        false
                                    };
                                    
                                    // Only emit transcription event if no command was detected
                                    if !voice_command_detected {
                                        let _ = event_tx.send((
                                            "transcription:update".to_string(),
                                            json!(&text)
                                        )).await;
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Transcription error: {}", e);
                                
                                // Emit error event to frontend
                                let _ = event_tx.send((
                                    "transcribe:error".to_string(),
                                    json!({
                                        "error": format!("Transcription error: {}", e)
                                    })
                                )).await;
                            }
                        }
                        
                        last_processed = now;
                    }
                }
                
                // Update active flag when done
                {
                    let mut active = transcription_active.lock();
                    *active = false;
                }
            });
        }
        
        Ok(())
    }
    
    // Stop transcription
    pub fn stop_transcription(&self) -> Result<()> {
        let mut active = self.transcription_active.lock();
        *active = false;
        
        Ok(())
    }
    
    pub fn is_transcribing(&self) -> bool {
        *self.transcription_active.lock()
    }
    
    pub fn clear_transcription(&self) -> Result<()> {
        let mut text = self.transcription_text.lock();
        *text = String::new();
        
        // Emit clear event to frontend
        if let Some(handle) = &self.app_handle {
            let _ = handle.emit("transcription:clear", ());
        }
        
        Ok(())
    }
    
    pub fn ensure_model_exists(&self, model_size: &str) -> Result<PathBuf, String> {
        let path = (self.get_model_path)(model_size);
        
        if path.exists() {
            Ok(path)
        } else {
            Err(format!("Model file not found: {}", path.display()))
        }
    }

    /* Commented out until whisper_rs is re-enabled
    fn create_whisper_params(&self, speech_settings: &SpeechSettings) -> FullParams<'static, 'static> {
        // Access whisper_params correctly from config_manager
        let whisper_params_config = self.config_manager.lock().get_config().whisper_params.clone();
        
        // Create strategy based on beam size
        let mut params = if whisper_params_config.beam_size > 0 {
            FullParams::new(SamplingStrategy::BeamSearch {
                beam_size: whisper_params_config.beam_size as i32,
                patience: whisper_params_config.patience,
            })
        } else {
            FullParams::new(SamplingStrategy::Greedy { 
                best_of: whisper_params_config.best_of 
            })
        };

        // Set language if specified and not "auto"
        if let Some(lang) = &speech_settings.language {
            if !lang.is_empty() && lang != "auto" {
                params.set_language(Some(lang));
                debug!("Setting language parameter to: {}", lang);
            } else {
                debug!("Using auto-detect for language (language set to '{}').", lang);
                params.set_language(None); // Explicitly set to None for auto-detect/empty
            }
        } else {
            debug!("Using auto-detect for language (language is None).");
            params.set_language(None); // Explicitly set to None for auto-detect
        }

        // Set translate flag
        params.set_translate(speech_settings.translate_to_english);
        if speech_settings.translate_to_english {
            debug!("Setting translate parameter to true.");
        }
        
        // Set advanced parameters
        params.set_temperature(whisper_params_config.temperature);
        params.set_no_speech_thold(whisper_params_config.no_speech_threshold);
        
        // Set initial prompt if available
        if let Some(prompt) = &whisper_params_config.initial_prompt {
            if !prompt.is_empty() {
                params.set_initial_prompt(prompt);
                debug!("Setting initial prompt: {}", prompt);
            }
        }
        
        // Enable token timestamps for detailed analysis
        params.set_token_timestamps(true);
        
        debug!("Whisper params: temperature={}, no_speech_threshold={}, beam_size={}",
               whisper_params_config.temperature,
               whisper_params_config.no_speech_threshold,
               whisper_params_config.beam_size);

        params
    }
    */
}

impl Clone for TranscribeState {
    fn clone(&self) -> Self {
        Self {
            config_manager: Arc::clone(&self.config_manager),
            transcription_text: Arc::clone(&self.transcription_text),
            transcription_active: Arc::clone(&self.transcription_active),
            audio_receiver: Arc::clone(&self.audio_receiver),
            audio_sender: Arc::clone(&self.audio_sender),
            // whisper_context: Arc::clone(&self.whisper_context),
            processor: Arc::clone(&self.processor),
            audio_buffer: Arc::clone(&self.audio_buffer),
            app_handle: self.app_handle.clone(),
            download_progress: Arc::clone(&self.download_progress),
            get_model_path: Arc::clone(&self.get_model_path),
            voice_command_processor: Arc::clone(&self.voice_command_processor),
            enhanced_processor: Arc::clone(&self.enhanced_processor),
            vad: Arc::clone(&self.vad),
            hallucination_detector: Arc::clone(&self.hallucination_detector),
            vocabulary_manager: Arc::clone(&self.vocabulary_manager),
        }
    }
}

#[derive(Default)]
pub struct TranscribePlugin<R: Runtime> {
    _phantom: PhantomData<fn() -> R>,
}

impl<R: Runtime> TranscribePlugin<R> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<R: Runtime> Plugin<R> for TranscribePlugin<R> {
    fn name(&self) -> &'static str {
        "transcribe"
    }
    
    fn initialize(&mut self, app: &AppHandle<R>, _: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing transcription plugin");
        
        // Register the plugin
        app.plugin(
            tauri::plugin::Builder::<R>::new("transcribe")
                .js_init_script(include_str!("./transcribe_init.js").to_string())
                .setup(|_app, _api: tauri::plugin::PluginApi<R, ()>| {
                    Ok(())
                })
                .build(),
        )?;
        
        Ok(())
    }
}

// Tauri 2.0 command handlers
#[tauri::command]
pub async fn start_transcription(
    options: Option<serde_json::Value>,
    state: State<'_, Arc<TranscribeState>>
) -> Result<(), String> {
    // Apply any options if provided
    if let Some(options) = options {
        let mut config_manager = state.config_manager.lock();
        let mut config = config_manager.get_config_mut();
        
        if let Some(model_size) = options.get("model_size").and_then(|v| v.as_str()) {
            config.audio.speech.model_size = match model_size {
                "tiny" => WhisperModelSize::Tiny,
                "base" => WhisperModelSize::Base,
                "small" => WhisperModelSize::Small,
                "medium" => WhisperModelSize::Medium,
                "large" => WhisperModelSize::Large,
                _ => WhisperModelSize::Small, // Default
            };
        }
        
        if let Some(language) = options.get("language").and_then(|v| v.as_str()) {
            config.audio.speech.language = language.to_string();
        }
        
        if let Some(translate) = options.get("translate_to_english").and_then(|v| v.as_bool()) {
            config.audio.speech.translate_to_english = translate;
        }
        
        if let Some(auto_punctuate) = options.get("auto_punctuate").and_then(|v| v.as_bool()) {
            config.audio.speech.auto_punctuate = auto_punctuate;
        }
        
        if let Some(context_formatting) = options.get("context_formatting").and_then(|v| v.as_bool()) {
            config.audio.speech.context_formatting = context_formatting;
        }
        
        // Save config changes
        if let Err(e) = config_manager.save() {
            return Err(format!("Failed to save config changes: {}", e));
        }
    }
    
    // Start transcription
    state.start_transcription().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_transcription(
    state: State<'_, Arc<TranscribeState>>
) -> Result<(), String> {
    state.stop_transcription().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_transcription(state: State<'_, Arc<TranscribeState>>) -> Result<String, String> {
    Ok(state.get_transcription())
}

#[tauri::command]
pub async fn is_transcribing(state: State<'_, Arc<TranscribeState>>) -> Result<bool, String> {
    Ok(state.is_transcribing())
}

#[tauri::command]
pub async fn clear_transcription(
    state: State<'_, Arc<TranscribeState>>
) -> Result<(), String> {
    state.clear_transcription().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_download_progress(state: State<'_, Arc<TranscribeState>>) -> Result<Option<(String, f32)>, String> {
    Ok(state.get_download_progress())
}

#[tauri::command]
pub async fn download_model_command(
    model_size: String,
    state: State<'_, Arc<TranscribeState>>
) -> Result<(), String> {
    // Convert string to WhisperModelSize
    let model_size_enum = match model_size.as_str() {
        "tiny" => WhisperModelSize::Tiny,
        "base" => WhisperModelSize::Base,
        "small" => WhisperModelSize::Small,
        "medium" => WhisperModelSize::Medium,
        "large" => WhisperModelSize::Large,
        _ => return Err(format!("Invalid model size: {}", model_size)),
    };
    
    // Get model path
    let model_path = state.get_model_path(&model_size_enum);
    
    // Clone the state Arc for the async block
    let state_clone = state.inner().clone();
    
    // Start download
    tokio::spawn(async move {
        if let Err(e) = state_clone.download_model(&model_size_enum, &model_path).await {
            error!("Failed to download model: {}", e);
            
            // Emit error event to frontend
            if let Some(handle) = &state_clone.app_handle {
                let _ = handle.emit(
                    "transcribe:error",
                    json!({
                        "error": format!("Failed to download model: {}", e)
                    })
                );
            }
        } else {
            info!("Model download completed successfully");
            
            // Emit success event to frontend
            if let Some(handle) = &state_clone.app_handle {
                let _ = handle.emit(
                    "transcribe:download-complete",
                    json!({
                        "model": model_size
                    })
                );
            }
        }
    });
    
    Ok(())
}

#[tauri::command]
pub async fn is_model_downloaded(
    model_size: String,
    state: State<'_, Arc<TranscribeState>>
) -> Result<bool, String> {
    // Check if model exists
    let path = state.ensure_model_exists(&model_size);
    Ok(path.is_ok())
}

#[tauri::command]
pub async fn is_voice_commands_enabled(
    state: State<'_, Arc<TranscribeState>>
) -> Result<bool, String> {
    let config_manager = state.config_manager.lock();
    Ok(config_manager.get_config().audio.voice_commands.enabled)
}

#[tauri::command]
pub async fn set_voice_commands_enabled(
    enabled: bool,
    state: State<'_, Arc<TranscribeState>>
) -> Result<(), String> {
    let config_manager = state.config_manager.clone();
    let mut config_manager = config_manager.lock();
    
    // Get the voice commands config before saving
    let voice_commands_config = {
        let config = config_manager.get_config_mut();
        config.audio.voice_commands.enabled = enabled;
        config.audio.voice_commands.clone()
    };
    
    // Save the config
    config_manager.save().map_err(|e| e.to_string())?;
    
    // Update the voice command processor
    let mut processor_guard = state.voice_command_processor.lock();
    if enabled {
        // Create new processor
        match VoiceCommandProcessor::new(voice_commands_config) {
            Ok((processor, _)) => {
                *processor_guard = Some(processor);
                info!("Voice commands enabled");
            },
            Err(e) => {
                return Err(format!("Failed to enable voice commands: {}", e));
            }
        }
    } else {
        // Disable processor
        *processor_guard = None;
        info!("Voice commands disabled");
    }
    
    Ok(())
}

#[tauri::command]
pub async fn update_whisper_params(
    temperature: Option<f32>,
    vad_enabled: Option<bool>,
    vad_threshold: Option<f32>,
    beam_size: Option<u32>,
    initial_prompt: Option<String>,
    state: State<'_, Arc<TranscribeState>>
) -> Result<(), String> {
    let config_manager = state.config_manager.clone();
    let mut config_manager = config_manager.lock();
    let config = config_manager.get_config_mut();
    
    // Update whisper parameters
    if let Some(temp) = temperature {
        config.whisper_params.temperature = temp;
    }
    if let Some(vad) = vad_enabled {
        config.whisper_params.vad_enabled = vad;
    }
    if let Some(threshold) = vad_threshold {
        config.whisper_params.vad_threshold = threshold;
    }
    if let Some(beam) = beam_size {
        config.whisper_params.beam_size = beam;
    }
    if let Some(prompt) = initial_prompt {
        config.whisper_params.initial_prompt = Some(prompt);
    }
    
    // Save the config
    config_manager.save().map_err(|e| e.to_string())?;
    
    // Reset enhanced processor to use new settings
    let mut enhanced = state.enhanced_processor.lock();
    *enhanced = None;
    
    // Reset VAD if settings changed
    if vad_enabled.is_some() || vad_threshold.is_some() {
        let mut vad = state.vad.lock();
        *vad = None;
    }
    
    info!("Updated Whisper parameters");
    Ok(())
}

#[tauri::command]
pub async fn get_whisper_params(
    state: State<'_, Arc<TranscribeState>>
) -> Result<serde_json::Value, String> {
    let config_manager = state.config_manager.lock();
    let params = &config_manager.get_config().whisper_params;
    
    Ok(json!({
        "temperature": params.temperature,
        "vad_enabled": params.vad_enabled,
        "vad_threshold": params.vad_threshold,
        "beam_size": params.beam_size,
        "initial_prompt": params.initial_prompt,
        "no_speech_threshold": params.no_speech_threshold,
        "min_speech_duration_ms": params.min_speech_duration_ms,
        "max_silence_duration_ms": params.max_silence_duration_ms,
    }))
}

/// Add vocabulary entry
#[tauri::command]
pub async fn add_vocabulary_entry(
    term: String,
    boost: f32,
    category: Option<String>,
    variants: Vec<String>,
    state: State<'_, Arc<TranscribeState>>
) -> Result<(), String> {
    // Initialize vocabulary manager if not exists
    if state.vocabulary_manager.lock().is_none() {
        match state.initialize_vocabulary_manager() {
            Ok(()) => {},
            Err(e) => return Err(format!("Failed to initialize vocabulary manager: {}", e)),
        }
    }
    
    // Create vocabulary entry
    let mut entry = VocabularyEntry::simple(term, boost);
    entry.category = category;
    entry.variants = variants;
    
    // Add to vocabulary
    if let Some(vocab_manager) = state.vocabulary_manager.lock().as_ref() {
        vocab_manager.add_entry(entry)
            .map_err(|e| format!("Failed to add vocabulary entry: {}", e))?;
    }
    
    Ok(())
}

/// Remove vocabulary entry
#[tauri::command]
pub async fn remove_vocabulary_entry(
    term: String,
    state: State<'_, Arc<TranscribeState>>
) -> Result<(), String> {
    if let Some(vocab_manager) = state.vocabulary_manager.lock().as_ref() {
        vocab_manager.remove_entry(&term)
            .map_err(|e| format!("Failed to remove vocabulary entry: {}", e))?;
    }
    
    Ok(())
}

/// Search vocabulary entries
#[tauri::command]
pub async fn search_vocabulary(
    pattern: String,
    state: State<'_, Arc<TranscribeState>>
) -> Result<Vec<VocabularyEntry>, String> {
    if let Some(vocab_manager) = state.vocabulary_manager.lock().as_ref() {
        Ok(vocab_manager.search(&pattern))
    } else {
        Ok(Vec::new())
    }
}

/// Get all vocabulary entries
#[tauri::command]
pub async fn get_vocabulary_entries(
    state: State<'_, Arc<TranscribeState>>
) -> Result<Vec<VocabularyEntry>, String> {
    if let Some(vocab_manager) = state.vocabulary_manager.lock().as_ref() {
        Ok(vocab_manager.get_enabled_entries())
    } else {
        Ok(Vec::new())
    }
}

/// Import vocabulary from CSV
#[tauri::command]
pub async fn import_vocabulary_csv(
    csv_path: String,
    state: State<'_, Arc<TranscribeState>>
) -> Result<usize, String> {
    // Initialize vocabulary manager if not exists
    if state.vocabulary_manager.lock().is_none() {
        match state.initialize_vocabulary_manager() {
            Ok(()) => {},
            Err(e) => return Err(format!("Failed to initialize vocabulary manager: {}", e)),
        }
    }
    
    if let Some(vocab_manager) = state.vocabulary_manager.lock().as_ref() {
        vocab_manager.import_csv(Path::new(&csv_path))
            .map_err(|e| format!("Failed to import vocabulary: {}", e))
    } else {
        Err("Vocabulary manager not initialized".to_string())
    }
}

/// Export vocabulary to CSV
#[tauri::command]
pub async fn export_vocabulary_csv(
    csv_path: String,
    state: State<'_, Arc<TranscribeState>>
) -> Result<usize, String> {
    if let Some(vocab_manager) = state.vocabulary_manager.lock().as_ref() {
        vocab_manager.export_csv(Path::new(&csv_path))
            .map_err(|e| format!("Failed to export vocabulary: {}", e))
    } else {
        Err("Vocabulary manager not initialized".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bestme::config::{Config, GeneralSettings, AudioSettings, SpeechSettings, AiSettings};
    use std::path::PathBuf;
    use tempfile::tempdir; // For creating temporary directories

    // Helper to create a mock ConfigManager with specific settings
    fn create_mock_config_manager(config: Config) -> Arc<Mutex<ConfigManager>> {
        let manager = ConfigManager::from_config(config);
        Arc::new(Mutex::new(manager))
    }

    // Helper to create a TranscribeState with a mock config and temp model dir
    fn create_test_transcribe_state(config: Config, model_dir: PathBuf) -> TranscribeState {
        let config_manager = create_mock_config_manager(config);
        let get_model_path: Arc<dyn Fn(&str) -> PathBuf + Send + Sync> = Arc::new(move |model_size| {
            model_dir.join(format!("ggml-{}.bin", model_size))
        });

        TranscribeState {
            config_manager,
            transcription_text: Arc::new(Mutex::new(String::new())),
            transcription_active: Arc::new(Mutex::new(false)),
            audio_receiver: Arc::new(Mutex::new(None)), // Not needed for these tests
            audio_sender: Arc::new(Mutex::new(None)), // Not needed for these tests
            processor: Arc::new(Mutex::new(None)), // Mock or load later if needed
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            app_handle: None, // Not needed for these tests
            download_progress: Arc::new(Mutex::new(None)),
            get_model_path,
            voice_command_processor: Arc::new(Mutex::new(None)), // Not needed for these tests
        }
    }

    #[tokio::test]
    async fn test_download_model_offline_mode() {
        let temp_dir = tempdir().unwrap();
        let model_dir = temp_dir.path().to_path_buf();
        let model_path = model_dir.join("ggml-tiny.bin");

        // Create config with offline_mode = true
        let mut config = Config::default();
        config.general.offline_mode = true;

        let state = create_test_transcribe_state(config, model_dir);

        // Attempt to download the model
        let result = state.download_model(&WhisperModelSize::Tiny, &model_path).await;

        // Assert that it returns an error due to offline mode
        assert!(result.is_err());
        let error_message = result.err().unwrap().to_string();
        assert!(error_message.contains("Offline mode is enabled"), "Error message mismatch: {}", error_message);

        // Assert that the model file was NOT created (nor the temp file)
        assert!(!model_path.exists());
        assert!(!model_path.with_extension("tmp").exists());
    }

    /* Commented out until whisper_rs is re-enabled
    #[tokio::test]
    async fn test_create_whisper_params() {
        let temp_dir = tempdir().unwrap();
        let model_dir = temp_dir.path().to_path_buf();
        let config = Config::default(); // Use default config for this test
        let state = create_test_transcribe_state(config, model_dir);

        // Case 1: Default settings (language=None, translate=false)
        let settings_default = SpeechSettings::default();
        let params_default = state.create_whisper_params(&settings_default);
        // Assertions: Relying on debug logs and internal logic for now,
        // as FullParams doesn't expose getters easily.
        // We mainly test that the function runs without panic.
        assert_eq!(settings_default.translate_to_english, false); 
        assert!(settings_default.language.is_none());

        // Case 2: Specific language, translate=false
        let settings_lang_en = SpeechSettings {
            language: Some("en".to_string()),
            translate_to_english: false,
            ..Default::default()
        };
        let params_lang_en = state.create_whisper_params(&settings_lang_en);
        assert_eq!(settings_lang_en.translate_to_english, false);
        assert_eq!(settings_lang_en.language, Some("en".to_string()));

        // Case 3: Auto language ("auto"), translate=true
        let settings_auto_translate = SpeechSettings {
            language: Some("auto".to_string()),
            translate_to_english: true,
            ..Default::default()
        };
        let params_auto_translate = state.create_whisper_params(&settings_auto_translate);
        assert_eq!(settings_auto_translate.translate_to_english, true);
        assert_eq!(settings_auto_translate.language, Some("auto".to_string()));

        // Case 4: Specific language, translate=true
        let settings_lang_es_translate = SpeechSettings {
            language: Some("es".to_string()),
            translate_to_english: true,
            ..Default::default()
        };
        let params_lang_es_translate = state.create_whisper_params(&settings_lang_es_translate);
        assert_eq!(settings_lang_es_translate.translate_to_english, true);
        assert_eq!(settings_lang_es_translate.language, Some("es".to_string()));
        
        // Case 5: Empty language string (should be treated as auto), translate=false
        let settings_empty_lang = SpeechSettings {
            language: Some("".to_string()),
            translate_to_english: false,
            ..Default::default()
        };
        let params_empty_lang = state.create_whisper_params(&settings_empty_lang);
        assert_eq!(settings_empty_lang.translate_to_english, false);
        assert_eq!(settings_empty_lang.language, Some("".to_string()));
    }

    // TODO: Fix create_whisper_params to read best_of from config
    */
    // TODO: Add test for process_audio_buffer parameters (might need mock context)
} 
