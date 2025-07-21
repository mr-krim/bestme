use anyhow::{Result, anyhow};
use log::{info, debug, error, warn};
use parking_lot::Mutex;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs;
use tauri::{Manager, AppHandle, State, plugin};
use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};
use futures::StreamExt;
use serde_json::json;
use std::marker::PhantomData;

use bestme::audio::capture::AudioData;
use bestme::config::{ConfigManager, WhisperModelSize, SpeechSettings};

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
    whisper_context: Arc<Mutex<Option<WhisperContext>>>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    app_handle: Option<AppHandle>,
    download_progress: Arc<Mutex<Option<(String, f32)>>>, // (model_size, progress 0.0-1.0)
    get_model_path: Box<dyn Fn(&str) -> PathBuf + Send + Sync>,
}

impl TranscribeState {
    pub fn new(config_manager: Arc<Mutex<ConfigManager>>, app_handle: Option<AppHandle>) -> Result<Self, anyhow::Error> {
        let (audio_sender, audio_receiver) = tokio::sync::mpsc::channel(100);
        
        // Default function to get model path - uses app directory
        let get_model_path: Box<dyn Fn(&str) -> PathBuf + Send + Sync> = Box::new(move |model_size| {
            // First check if there's a custom model path in config
            let custom_path = {
                let config_manager = config_manager.lock();
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
            let app_dir = app_handle.as_ref().and_then(|handle| {
                handle.path_resolver().app_data_dir()
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
        
        Ok(Self {
            config_manager,
            transcription_text: Arc::new(Mutex::new(String::new())),
            transcription_active: Arc::new(Mutex::new(false)),
            audio_receiver: Arc::new(Mutex::new(Some(audio_receiver))),
            audio_sender: Arc::new(Mutex::new(Some(audio_sender))),
            whisper_context: Arc::new(Mutex::new(None)),
            audio_buffer: Arc::new(Mutex::new(Vec::with_capacity(AUDIO_BUFFER_SIZE))),
            app_handle,
            download_progress: Arc::new(Mutex::new(None)),
            get_model_path,
        })
    }
    
    pub fn set_app_handle(&mut self, app_handle: AppHandle) -> Result<()> {
        self.app_handle = Some(app_handle);
        Ok(())
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
        match tokio::task::spawn_blocking(move || {
            // Use the new_with_params method instead of the deprecated new method
            WhisperContext::new_with_params(&model_path_str, Default::default())
        }).await? {
            Ok(context) => {
                let mut whisper_context = self.whisper_context.lock();
                *whisper_context = Some(context);
                info!("Whisper model loaded successfully");
                Ok(())
            },
            Err(e) => {
                error!("Failed to load Whisper model: {}", e);
                Err(anyhow::anyhow!("Failed to load Whisper model: {}", e))
            }
        }
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
                    let _ = handle.emit_all(
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
    
    // Process audio buffer using Whisper
    async fn process_audio_buffer(&self, audio_buffer: Vec<f32>) -> Result<String> {
        let context_lock = self.whisper_context.lock();
        let context = context_lock.as_ref().ok_or_else(|| anyhow!("Whisper context not loaded"))?;
        
        // Get relevant settings from config
        let speech_config = self.config_manager.lock().get_config().audio.speech.clone();
        
        debug!("Processing audio buffer. Language: {}, Translate: {}", 
               speech_config.language.clone().unwrap_or_else(|| "auto".to_string()), 
               speech_config.translate_to_english);

        // Create parameters for transcription using the helper function
        let mut params = self.create_whisper_params(&speech_config);
        
        // Configure other parameters as needed (potentially from config later)
        // NOTE: The best_of parameter is now set within create_whisper_params
        params.set_print_special_tokens(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        // params.set_n_threads(4); // Example: adjust thread count
        
        // Run transcription in a blocking task
        let params_clone = params.clone(); // Clone params for the blocking task
        let audio_buffer_clone = audio_buffer.clone(); // Clone buffer
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
        }).await??; // Double ?? to handle JoinError and inner Result

        debug!("Transcription segment result: {}", result);
        Ok(result)
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
            let whisper_context = Arc::clone(&self.whisper_context);
            let self_clone = self.clone();
            let app_handle = self.app_handle.clone();
            
            // Spawn a task to process audio data
            tokio::spawn(async move {
                let mut buffer_timer = tokio::time::interval(std::time::Duration::from_secs(1));
                
                // Load model eagerly
                {
                    let config = config_manager.lock().get_config().audio.speech.clone();
                    if let Err(e) = self_clone.load_whisper_model(&config.model_size).await {
                        error!("Failed to load Whisper model: {}", e);
                        
                        // Update active flag
                        let mut active = transcription_active.lock();
                        *active = false;
                        
                        // Emit error event to frontend
                        if let Some(handle) = &app_handle {
                            let _ = handle.emit_all(
                                "transcribe:error",
                                json!({
                                    "error": format!("Failed to load Whisper model: {}", e)
                                })
                            );
                        }
                        
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
                        buffer.extend(audio_data.data.iter());
                        
                        // Resize if buffer is too large
                        if buffer.len() > AUDIO_BUFFER_SIZE {
                            buffer.drain(0..(buffer.len() - AUDIO_BUFFER_SIZE));
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
                                    
                                    // Emit transcription event to frontend
                                    if let Some(handle) = &app_handle {
                                        let _ = handle.emit_all(
                                            "transcription:update",
                                            json!(&text)
                                        );
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Transcription error: {}", e);
                                
                                // Emit error event to frontend
                                if let Some(handle) = &app_handle {
                                    let _ = handle.emit_all(
                                        "transcribe:error",
                                        json!({
                                            "error": format!("Transcription error: {}", e)
                                        })
                                    );
                                }
                            }
                        }
                        
                        last_processed = now;
                    }
                }
                
                // Update active flag when done
                let mut active = transcription_active.lock();
                *active = false;
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
            let _ = handle.emit_all("transcription:clear", ());
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

    fn create_whisper_params(&self, speech_settings: &SpeechSettings) -> FullParams<'static, 'static> {
        // Access whisper_params correctly from config_manager
        let whisper_params_config = self.config_manager.lock().get_config().whisper_params.clone();
        
        let best_of = whisper_params_config.best_of;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of });

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

        params
    }
}

impl Clone for TranscribeState {
    fn clone(&self) -> Self {
        Self {
            config_manager: Arc::clone(&self.config_manager),
            transcription_text: Arc::clone(&self.transcription_text),
            transcription_active: Arc::clone(&self.transcription_active),
            audio_receiver: Arc::clone(&self.audio_receiver),
            audio_sender: Arc::clone(&self.audio_sender),
            whisper_context: Arc::clone(&self.whisper_context),
            audio_buffer: Arc::clone(&self.audio_buffer),
            app_handle: self.app_handle.clone(),
            download_progress: Arc::clone(&self.download_progress),
            get_model_path: self.get_model_path.clone(),
        }
    }
}

#[derive(Default)]
pub struct TranscribePlugin {
    _phantom: PhantomData<()>,
}

impl TranscribePlugin {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl tauri::Plugin for TranscribePlugin {
    fn name(&self) -> &'static str {
        "transcribe"
    }
    
    fn initialize(&mut self, app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing transcription plugin");
        
        // Register the plugin
        app.plugin(
            tauri::plugin::Builder::new("transcribe")
                .js_init_script(include_str!("./transcribe_init.js"))
                .setup(|_app, _| {
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
pub async fn get_transcription(state: State<'_, Arc<TranscribeState>>) -> String {
    state.get_transcription()
}

#[tauri::command]
pub async fn is_transcribing(state: State<'_, Arc<TranscribeState>>) -> bool {
    state.is_transcribing()
}

#[tauri::command]
pub async fn clear_transcription(
    state: State<'_, Arc<TranscribeState>>
) -> Result<(), String> {
    state.clear_transcription().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_download_progress(state: State<'_, Arc<TranscribeState>>) -> Option<(String, f32)> {
    state.get_download_progress()
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
    
    // Start download
    tokio::spawn(async move {
        if let Err(e) = state.download_model(&model_size_enum, &model_path).await {
            error!("Failed to download model: {}", e);
            
            // Emit error event to frontend
            if let Some(handle) = &state.app_handle {
                let _ = handle.emit_all(
                    "transcribe:error",
                    json!({
                        "error": format!("Failed to download model: {}", e)
                    })
                );
            }
        } else {
            info!("Model download completed successfully");
            
            // Emit success event to frontend
            if let Some(handle) = &state.app_handle {
                let _ = handle.emit_all(
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
        let get_model_path: Box<dyn Fn(&str) -> PathBuf + Send + Sync> = Box::new(move |model_size| {
            model_dir.join(format!("ggml-{}.bin", model_size))
        });

        TranscribeState {
            config_manager,
            transcription_text: Arc::new(Mutex::new(String::new())),
            transcription_active: Arc::new(Mutex::new(false)),
            audio_receiver: Arc::new(Mutex::new(None)), // Not needed for these tests
            audio_sender: Arc::new(Mutex::new(None)), // Not needed for these tests
            whisper_context: Arc::new(Mutex::new(None)), // Mock or load later if needed
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            app_handle: None, // Not needed for these tests
            download_progress: Arc::new(Mutex::new(None)),
            get_model_path,
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
    // TODO: Add test for process_audio_buffer parameters (might need mock context)
} 
