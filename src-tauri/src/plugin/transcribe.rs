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
    
    // Download the model (using method that mirrors Tauri 2.0's model)
    async fn download_model(&self, model_size: &WhisperModelSize, model_path: &Path) -> Result<()> {
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
        
        let mut downloaded: u64 = 0;
        let mut last_progress: f32 = 0.0;
        
        while let Some(item) = stream.next().await {
            let chunk = match item {
                Ok(chunk) => chunk,
                Err(e) => return Err(anyhow::anyhow!("Error during download: {}", e)),
            };
            
            // Write the chunk to file
            file.write_all(&chunk).await?;
            
            // Update download progress
            downloaded += chunk.len() as u64;
            let current_progress = downloaded as f32 / total_size as f32;
            
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
        // Get the Whisper context
        let context = {
            let whisper_context = self.whisper_context.lock();
            
            if whisper_context.is_none() {
                // Ensure model is loaded first
                drop(whisper_context);
                
                let config = self.config_manager.lock().get_config().audio.speech.clone();
                self.load_whisper_model(&config.model_size).await?;
                
                // Now get the context again
                let whisper_context = self.whisper_context.lock();
                whisper_context.as_ref().ok_or_else(|| anyhow::anyhow!("Failed to load Whisper model"))?
            } else {
                whisper_context.as_ref().ok_or_else(|| anyhow::anyhow!("Whisper context not available"))?
            }
        };
        
        // Get config
        let speech_config = self.config_manager.lock().get_config().audio.speech.clone();
        
        // Set up parameters for Whisper
        let mut params = whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 0 });
        
        // Set language if specified, otherwise auto-detect
        if speech_config.language != "auto" {
            params.set_language(Some(&speech_config.language));
        }
        
        // Set translation if enabled
        if speech_config.translate_to_english {
            params.set_translate(true);
        }
        
        // Other parameters
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        
        // Process audio in a blocking task (Whisper is CPU-intensive)
        let result = tokio::task::spawn_blocking(move || {
            let audio_buffer = audio_buffer;
            let context = context;
            
            // Run Whisper inference
            match context.full(params, &audio_buffer) {
                Ok(_) => {
                    // Extract number of segments
                    let num_segments = context.full_n_segments();
                    
                    // Get text from each segment
                    let mut text = String::new();
                    for i in 0..num_segments {
                        if let Ok(segment) = context.full_get_segment_text(i) {
                            text.push_str(&segment);
                            text.push(' ');
                        }
                    }
                    
                    Ok(text)
                },
                Err(e) => Err(anyhow::anyhow!("Whisper inference failed: {}", e)),
            }
        }).await??;
        
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
