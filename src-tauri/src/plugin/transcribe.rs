use anyhow::Result;
use log::{error, info, warn, debug};
use parking_lot::Mutex;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use tauri::{plugin::Plugin, Invoke, Runtime, AppHandle, Manager};
use tokio::sync::mpsc;
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};
use futures::StreamExt;
use serde_json::json;

use bestme::audio::capture::AudioData;
use bestme::config::{ConfigManager, WhisperModelSize};

// Constants for audio processing
const WHISPER_SAMPLE_RATE: usize = 16000;
const AUDIO_BUFFER_SIZE: usize = WHISPER_SAMPLE_RATE * 5; // 5 seconds of audio
const MAX_TEXT_LENGTH: usize = 8192;

/// The model URLs for each Whisper model size
const MODEL_URLS: &[(&str, &str)] = &[
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
                config_manager.get_config().transcription.model_path.clone()
            };
            
            if let Some(path) = custom_path {
                if !path.trim().is_empty() {
                    return PathBuf::from(path)
                        .join(format!("ggml-{}.bin", model_size));
                }
            }
            
            // Otherwise use app directory for models
            let app_dir = app_handle.as_ref().and_then(|handle| {
                handle.path_resolver().app_dir()
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
    
    pub fn set_app_handle(&self, app_handle: AppHandle) {
        let mut app_handle_value = None;
        std::mem::swap(&mut app_handle_value, &mut self.app_handle);
        app_handle_value = Some(app_handle);
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
        let mut downloaded: u64 = 0;
        let progress = self.download_progress.clone();
        
        use tokio_util::codec::{BytesCodec, FramedRead};
        use tokio_stream::StreamExt;
        
        while let Some(item) = stream.next().await {
            let chunk = item?;
            downloaded += chunk.len() as u64;
            file.write_all(&chunk).await?;
            
            // Update progress
            let progress_value = downloaded as f32 / total_size as f32;
            {
                let mut p = progress.lock();
                *p = Some((model_name.to_string(), progress_value));
            }
            
            // Emit an event to inform UI of download progress
            if let Some(app_handle) = &self.app_handle {
                let _ = app_handle.app_handle().emit_all("model-download-progress", 
                    serde_json::json!({
                        "model": model_name,
                        "progress": progress_value
                    })
                );
            }
        }
        
        // Close file
        drop(file);
        
        // Rename temp file to final path
        tokio::fs::rename(&temp_path, model_path).await?;
        
        // Reset progress
        {
            let mut p = self.download_progress.lock();
            *p = None;
        }
        
        info!("Model downloaded successfully to: {:?}", model_path);
        
        // Emit an event to inform UI that download is complete
        if let Some(app_handle) = &self.app_handle {
            let _ = app_handle.app_handle().emit_all("model-download-complete", 
                serde_json::json!({
                    "model": model_name,
                    "path": model_path.to_string_lossy().to_string()
                })
            );
        }
        
        Ok(())
    }
    
    // Process accumulated audio data with Whisper model
    async fn process_audio_buffer(&self, audio_buffer: Vec<f32>) -> Result<String> {
        // Get the context from mutex
        let context_option = {
            let context = self.whisper_context.lock();
            context.clone()
        };
        
        // If no context, return error
        let context = match context_option {
            Some(ctx) => ctx,
            None => return Err(anyhow::anyhow!("Whisper model not loaded")),
        };
        
        // Get language and other settings from config
        let (language, auto_punctuate, translate_to_english, segment_duration) = {
            let config = self.config_manager.lock();
            let speech_config = &config.get_config().audio.speech;
            (
                speech_config.language.clone(),
                speech_config.auto_punctuate,
                speech_config.translate_to_english,
                speech_config.segment_duration as f32,
            )
        };
        
        // Create params with enhanced configuration
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // Configure language settings
        if language.is_empty() || language == "auto" {
            // Auto-detect language
            params.set_language(None);
        } else {
            // Use specific language
            params.set_language(Some(&language));
        }
        
        // Configure translation if needed
        if translate_to_english {
            params.set_translate(true);
        }
        
        // Enable timestamps for better segmentation
        params.set_print_timestamps(true);
        params.set_print_special(true);
        
        // Configure auto punctuation
        if auto_punctuate {
            // For now, this is built into Whisper by default when using language models
            // We can add more controls if needed in the future
        }
        
        // Process audio in a blocking task as it's CPU-intensive
        let audio_data = audio_buffer.clone();
        match tokio::task::spawn_blocking(move || {
            // Create state safely with error handling
            let state = match context.create_state() {
                Ok(state) => state,
                Err(e) => {
                    error!("Failed to create whisper state: {}", e);
                    return Err(format!("Failed to create whisper state: {}", e));
                }
            };
            
            // Run inference with error handling
            if let Err(e) = state.full(params, &audio_data) {
                error!("Failed to process audio with whisper: {}", e);
                return Err(format!("Failed to process audio: {}", e));
            }
            
            // Extract text from segments
            let num_segments = state.full_n_segments().expect("Failed to get number of segments");
            let mut text = String::new();
            
            for i in 0..num_segments {
                if let Ok(segment_text) = state.full_get_segment_text(i) {
                    text.push_str(&segment_text);
                    text.push(' ');
                }
            }
            
            let result = text.trim().to_string();
            if result.is_empty() {
                Err("No transcription detected".to_string())
            } else {
                Ok(result)
            }
        }).await {
            Ok(Ok(text)) => Ok(text),
            Ok(Err(e)) => Err(anyhow::anyhow!("{}", e)),
            Err(e) => Err(anyhow::anyhow!("Task failed: {}", e)),
        }
    }
    
    // Helper to get model size string
    fn get_model_size_string(&self, model_size: &WhisperModelSize) -> &'static str {
        match model_size {
            WhisperModelSize::Tiny => "tiny",
            WhisperModelSize::Base => "base",
            WhisperModelSize::Small => "small",
            WhisperModelSize::Medium => "medium",
            WhisperModelSize::Large => "large",
        }
    }

    pub fn start_transcription(&self) -> Result<()> {
        let mut active = self.transcription_active.lock();
        if *active {
            return Ok(());
        }
        
        *active = true;
        
        // Clear existing text
        {
            let mut text = self.transcription_text.lock();
            text.clear();
        }
        
        // Clear audio buffer
        {
            let mut buffer = self.audio_buffer.lock();
            buffer.clear();
        }
        
        // Get model size from config
        let model_size = {
            let config = self.config_manager.lock();
            config.get_config().audio.speech.model_size.clone()
        };
        
        // Clone references for the async task
        let transcription_text = Arc::clone(&self.transcription_text);
        let transcription_active = Arc::clone(&self.transcription_active);
        let audio_receiver = Arc::clone(&self.audio_receiver);
        let whisper_context = Arc::clone(&self.whisper_context);
        let audio_buffer = Arc::clone(&self.audio_buffer);
        let self_clone = self.clone();
        
        // Launch transcription in a background task
        tokio::spawn(async move {
            // Load Whisper model if not already loaded
            if whisper_context.lock().is_none() {
                if let Err(e) = self_clone.load_whisper_model(&model_size).await {
                    error!("Failed to load Whisper model: {}", e);
                    // If we have an app handle, emit an error event
                    if let Some(app_handle) = &self_clone.app_handle {
                        let _ = app_handle.app_handle().emit_all("transcription-error", 
                            format!("Failed to load Whisper model: {}", e));
                    }
                    return;
                }
            }
            
            let mut receiver = {
                let mut recv = audio_receiver.lock();
                recv.take()
            };
            
            if let Some(mut receiver) = receiver {
                info!("Starting transcription with model size: {:?}", model_size);
                
                // Process audio data and update transcription
                while {
                    let active = transcription_active.lock();
                    *active
                } {
                    match receiver.recv().await {
                        Some(audio_data) => {
                            // Accumulate audio in buffer
                            {
                                let mut buffer = audio_buffer.lock();
                                
                                // Convert audio data to expected format
                                // Resample to whisper sample rate (16kHz)
                                let samples = audio_data.to_whisper_input(WHISPER_SAMPLE_RATE as u32);
                                buffer.extend_from_slice(&samples);
                                
                                // Process buffer when it gets big enough
                                if buffer.len() >= AUDIO_BUFFER_SIZE {
                                    let buffer_copy = buffer.clone();
                                    buffer.clear();
                                    
                                    // Drop the lock before async processing
                                    drop(buffer);
                                    
                                    // Process the copied buffer
                                    match self_clone.process_audio_buffer(buffer_copy).await {
                                        Ok(result) => {
                                            if !result.is_empty() {
                                                let mut text = transcription_text.lock();
                                                // Append new text or replace depending on application needs
                                                if text.len() > MAX_TEXT_LENGTH {
                                                    *text = format!("{}... {}", &text[..MAX_TEXT_LENGTH/2], result);
                                                } else {
                                                    *text = format!("{} {}", text, result);
                                                }
                                                
                                                // Emit transcription update event
                                                if let Some(app_handle) = &self_clone.app_handle {
                                                    let _ = app_handle.app_handle().emit_all("transcription-update", text.clone());
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            error!("Failed to process audio: {}", e);
                                            // Emit error event
                                            if let Some(app_handle) = &self_clone.app_handle {
                                                let _ = app_handle.app_handle().emit_all("transcription-error", 
                                                    format!("Processing error: {}", e));
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        None => {
                            error!("Audio channel closed");
                            break;
                        }
                    }
                }
                
                // Process any remaining audio in buffer
                {
                    let buffer = audio_buffer.lock();
                    if !buffer.is_empty() {
                        let buffer_copy = buffer.clone();
                        drop(buffer);
                        
                        if let Ok(result) = self_clone.process_audio_buffer(buffer_copy).await {
                            if !result.is_empty() {
                                let mut text = transcription_text.lock();
                                *text = format!("{} {}", text, result);
                                
                                // Emit transcription update event
                                if let Some(app_handle) = &self_clone.app_handle {
                                    let _ = app_handle.app_handle().emit_all("transcription-update", text.clone());
                                }
                            }
                        }
                    }
                }
                
                info!("Transcription task completed");
                
                // Emit completion event
                if let Some(app_handle) = &self_clone.app_handle {
                    let _ = app_handle.app_handle().emit_all("transcription-complete", "");
                }
            } else {
                error!("No audio receiver available for transcription");
                
                // Emit error event
                if let Some(app_handle) = &self_clone.app_handle {
                    let _ = app_handle.app_handle().emit_all("transcription-error", 
                        "No audio receiver available for transcription");
                }
            }
        });
        
        Ok(())
    }

    pub fn stop_transcription(&self) -> Result<()> {
        let mut active = self.transcription_active.lock();
        *active = false;
        
        info!("Stopped transcription");
        Ok(())
    }
    
    pub fn is_transcribing(&self) -> bool {
        let active = self.transcription_active.lock();
        *active
    }
    
    pub fn clear_transcription(&self) -> Result<()> {
        let mut text = self.transcription_text.lock();
        *text = String::new();
        
        info!("Cleared transcription");
        Ok(())
    }

    /// Check if the model file exists, and if not, download it.
    /// Returns the path to the model file.
    pub fn ensure_model_exists(&self, model_size: &str) -> Result<PathBuf, String> {
        let model_path = (self.get_model_path)(model_size);
        
        // If the model file already exists, return its path
        if model_path.exists() {
            return Ok(model_path);
        }
        
        // Model doesn't exist, return error
        Err(format!("Model {} not found at path {:?}. Please download it from the settings page.", 
                   model_size, model_path))
    }
}

// Implement Clone for TranscribeState
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

// Define the Transcribe plugin
pub struct TranscribePlugin<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> TranscribePlugin<R> {
    pub fn new() -> Self {
        Self {
            invoke_handler: Box::new(tauri::generate_handler![
                start_transcription,
                stop_transcription,
                get_transcription,
                is_transcribing,
                clear_transcription,
                get_download_progress,
                download_model_command,
                is_model_downloaded,
            ]),
        }
    }
}

impl<R: Runtime> Plugin<R> for TranscribePlugin<R> {
    fn name(&self) -> &'static str {
        "transcribe"
    }

    fn extend_api(&mut self, message: Invoke<R>) {
        (self.invoke_handler)(message)
    }
    
    fn initialize(&mut self, app: &AppHandle<R>) -> tauri::plugin::Result<()> {
        info!("Initializing transcribe plugin");
        
        // Try to access the transcribe state
        if let Some(state) = app.state::<TranscribeState>() {
            // State already exists, nothing to do
            debug!("Transcribe state already exists");
            Ok(())
        } else {
            // Create and manage a new transcribe state
            let state = TranscribeState::new(Arc::new(Mutex::new(ConfigManager::new())), app.clone());
            app.manage(state);
            debug!("Created and managed new transcribe state");
            Ok(())
        }
    }
}

// Tauri command handlers
#[tauri::command]
async fn start_transcription(
    options: Option<serde_json::Value>,
    state: State<'_, TranscribeState>
) -> Result<(), String> {
    // Extract options if provided
    let mut language = String::new();
    let mut translate_to_english = false;
    
    if let Some(opts) = options {
        if let Some(lang) = opts.get("language").and_then(|l| l.as_str()) {
            language = lang.to_string();
        }
        
        if let Some(translate) = opts.get("translate_to_english").and_then(|t| t.as_bool()) {
            translate_to_english = translate;
        }
    }
    
    // Get existing settings from config
    let config_manager = state.inner().config_manager.clone();
    
    // Update settings with any provided options
    {
        let mut config = config_manager.lock();
        let speech_config = &mut config.get_config_mut().audio.speech;
        
        // Only update if options were provided
        if !language.is_empty() {
            speech_config.language = language;
        }
        
        speech_config.translate_to_english = translate_to_english;
        
        // Save changes
        if let Err(e) = config.save() {
            warn!("Failed to save updated speech settings: {}", e);
        }
    }
    
    // Start transcription
    state.inner().start_transcription()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_transcription(
    state: tauri::State<'_, TranscribeState>
) -> Result<(), String> {
    state.inner().stop_transcription()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_transcription(state: tauri::State<'_, TranscribeState>) -> String {
    state.inner().get_transcription()
}

#[tauri::command]
fn is_transcribing(state: tauri::State<'_, TranscribeState>) -> bool {
    state.inner().is_transcribing()
}

#[tauri::command]
async fn clear_transcription(
    state: tauri::State<'_, TranscribeState>
) -> Result<(), String> {
    state.inner().clear_transcription()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_download_progress(state: tauri::State<'_, TranscribeState>) -> Option<(String, f32)> {
    state.inner().get_download_progress()
}

#[tauri::command]
pub fn download_model_command(
    state: tauri::State<'_, crate::AppState>,
    model_size: String,
) -> Result<(), String> {
    let mut ts = state.transcribe_state.lock().unwrap();
    
    // Start the download in a background task
    let model_size_clone = model_size.clone();
    let get_model_path = ts.get_model_path.clone();
    let download_progress = Arc::clone(&ts.download_progress);
    let app_handle = ts.app_handle.clone();
    
    std::thread::spawn(move || {
        let result = download_model(
            &model_size_clone,
            get_model_path(&model_size_clone),
            download_progress,
            app_handle,
        );
        
        if let Err(e) = result {
            eprintln!("Error downloading model {}: {}", model_size_clone, e);
        }
    });
    
    Ok(())
}

#[tauri::command]
pub fn is_model_downloaded(
    state: tauri::State<'_, crate::AppState>,
    model_size: String,
) -> Result<bool, String> {
    let ts = state.transcribe_state.lock().unwrap();
    let model_path = (ts.get_model_path)(&model_size);
    
    // Check if the model file exists
    Ok(model_path.exists())
}

/// Download a Whisper model to the specified path
pub fn download_model(
    model_size: &str,
    target_path: PathBuf,
    progress: Arc<Mutex<Option<(String, f32)>>>,
    app_handle: Option<AppHandle>,
) -> Result<(), String> {
    use std::io::Write;
    use reqwest::Client;
    
    // Find the URL for the requested model size
    let model_url = MODEL_URLS
        .iter()
        .find(|(size, _)| *size == model_size)
        .map(|(_, url)| *url)
        .ok_or_else(|| format!("Unknown model size: {}", model_size))?;
    
    // Create client for downloading
    let client = Client::new();
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }
    
    // Create temporary file for downloading
    let temp_path = target_path.with_extension("download");
    let mut file = fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    // Start download
    println!("Downloading model {} from {}", model_size, model_url);
    
    // Set initial progress
    {
        let mut progress_guard = progress.lock().unwrap();
        *progress_guard = Some((model_size.to_string(), 0.0));
        
        // Notify UI about download start
        if let Some(handle) = &app_handle {
            let _ = handle.app_handle().emit_all("model-download-progress", 
                serde_json::json!({
                    "model": model_size,
                    "progress": 0.0
                }));
        }
    }
    
    // Run download in a blocking task
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;
    
    rt.block_on(async {
        // Make request for the file
        let response = client.get(model_url)
            .send()
            .await
            .map_err(|e| format!("Failed to download model: {}", e))?;
        
        // Get total size for progress reporting
        let total_size = response.content_length()
            .ok_or_else(|| "Failed to get content length".to_string())?;
        
        // Create stream to download file
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_percentage: i32 = -1;
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Error while downloading: {}", e))?;
            file.write_all(&chunk)
                .map_err(|e| format!("Error while writing to file: {}", e))?;
            
            // Update progress
            downloaded += chunk.len() as u64;
            let percentage = (downloaded as f32 / total_size as f32) * 100.0;
            let rounded_percentage = percentage.round() as i32;
            
            // Only update progress if it changed by at least 1%
            if rounded_percentage != last_percentage {
                last_percentage = rounded_percentage;
                let progress_fraction = downloaded as f32 / total_size as f32;
                
                // Update progress atomic
                {
                    let mut progress_guard = progress.lock().unwrap();
                    *progress_guard = Some((model_size.to_string(), progress_fraction));
                }
                
                // Emit progress event to UI
                if let Some(handle) = &app_handle {
                    let _ = handle.app_handle().emit_all("model-download-progress", 
                        serde_json::json!({
                            "model": model_size,
                            "progress": progress_fraction
                        }));
                }
            }
        }
        
        // Flush and sync the file to ensure all data is written
        file.flush().map_err(|e| format!("Failed to flush file: {}", e))?;
        
        // Rename temp file to final name
        fs::rename(&temp_path, &target_path)
            .map_err(|e| format!("Failed to rename file: {}", e))?;
        
        // Clear progress once complete
        {
            let mut progress_guard = progress.lock().unwrap();
            *progress_guard = None;
        }
        
        // Emit completion event to UI
        if let Some(handle) = &app_handle {
            let _ = handle.app_handle().emit_all("model-download-complete", 
                serde_json::json!({
                    "model": model_size,
                    "path": target_path.to_string_lossy()
                }));
        }
        
        println!("Download complete: {}", target_path.display());
        Ok(())
    })
} 
