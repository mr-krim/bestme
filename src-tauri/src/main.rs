#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plugin;
mod system_monitor;

use log::{error, info, debug, warn};
use parking_lot::Mutex;
use std::sync::Arc;
use crate::plugin::transcribe::SUPPORTED_LANGUAGES;

// Tauri 2.0 imports
use tauri::{Manager, Listener};
use tauri::AppHandle;

// Import from main bestme crate
use bestme::audio::device::DeviceManager;
use bestme::config::ConfigManager;
use bestme::config::WhisperModelSize;

// Import our custom plugins
use plugin::{
    AudioPlugin, 
    AudioState, 
    TranscribePlugin, 
    TranscribeState,
    AIPlugin,
    voice_commands::{VoiceCommandPlugin, VoiceCommandState,
        get_voice_commands_status, get_voice_commands_text, get_voice_commands_history,
        update_text, apply_delete_operation, undo_operation, redo_operation,
        get_ai_voice_settings, save_ai_voice_settings, process_ai_voice_command
    },
    storage::{StoragePlugin, 
        list_saved_transcripts_db, get_saved_transcript_db, save_transcript_db, 
        delete_saved_transcript_db, search_transcripts_db, export_transcripts_db
    },
    text_injection::{TextInjectionPlugin, init_text_injection, inject_text, get_active_window,
        check_injection_permissions, request_injection_permissions,
        update_injection_config, set_injection_mode
    },
    audio::{start_recording, stop_recording, get_level, is_recording},
    transcribe::{
        start_transcription, stop_transcription, get_transcription, is_transcribing,
        clear_transcription, get_download_progress, download_model_command, is_model_downloaded,
        is_voice_commands_enabled, set_voice_commands_enabled, update_whisper_params,
        get_whisper_params, add_vocabulary_entry, remove_vocabulary_entry, search_vocabulary,
        get_vocabulary_entries, import_vocabulary_csv, export_vocabulary_csv
    },
    ai::{
        init_local_ai, init_cloud_ai, enhance_text, list_available_models, download_model,
        list_downloaded_models, summarize_text, transform_style, translate_text, get_cloud_usage,
        save_api_key, list_saved_api_keys, delete_api_key, get_gpu_info, get_model_download_progress,
        get_model_performance, get_current_ai_metrics, get_ai_performance_summary, get_active_downloads,
        cancel_model_download, get_model_config, update_model_config, load_ai_model, unload_ai_model,
        list_loaded_models, auto_select_model, analyze_text_characteristics, get_model_requirements,
        update_model_selector_config, get_model_selection_history, clear_model_selection_cache,
        check_model_updates, check_single_model_update, download_model_update, configure_update_checker,
        get_update_history, clear_update_cache, validate_onnx_model, import_custom_model,
        list_custom_models, get_custom_model, update_custom_model, delete_custom_model,
        export_custom_model
    }
};


// Import system monitor commands
use system_monitor::{get_cpu_usage, get_memory_usage, get_online_status};

// Import serde
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use chrono::{Utc, TimeZone};
use uuid::Uuid;
use reqwest::Client;
use serde_json::json; // For creating JSON values manually if needed

// Extension trait for DeviceManager to implement list_devices
trait DeviceManagerExt {
    fn list_devices(&self) -> Result<Vec<(String, String)>, String>;
}

impl DeviceManagerExt for DeviceManager {
    fn list_devices(&self) -> Result<Vec<(String, String)>, String> {
        Ok(self.get_input_devices())
    }
}

// Commands that will be exposed to the frontend
#[tauri::command]
async fn get_audio_devices(
    device_manager: tauri::State<'_, Arc<Mutex<DeviceManager>>>
) -> Result<Vec<(String, String)>, String> {
    let device_manager = device_manager.inner();
    let devices = device_manager.lock().list_devices()
        .map_err(|e| e.to_string())?;
    
    Ok(devices)
}

#[tauri::command]
async fn get_recent_transcription_list() -> Result<Vec<serde_json::Value>, String> {
    // Return empty list for now - this is just to fix the UI error
    Ok(vec![])
}

#[tauri::command]
async fn get_whisper_models() -> Vec<String> {
    // Add all the available Whisper models
    vec![
        "tiny".to_string(),
        "base".to_string(),
        "small".to_string(),
        "medium".to_string(),
        "large".to_string(),
    ]
}

#[tauri::command]
async fn get_model_download_info() -> Vec<serde_json::Value> {
    use serde_json::json;
    
    vec![
        json!({
            "name": "tiny",
            "size": "75 MB",
            "description": "Fastest model, lower accuracy"
        }),
        json!({
            "name": "base",
            "size": "142 MB",
            "description": "Fast with decent accuracy"
        }),
        json!({
            "name": "small",
            "size": "466 MB",
            "description": "Good balance of speed and accuracy"
        }),
        json!({
            "name": "medium",
            "size": "1.5 GB",
            "description": "High accuracy, slower processing"
        }),
        json!({
            "name": "large",
            "size": "3 GB",
            "description": "Highest accuracy, slowest processing"
        })
    ]
}

#[tauri::command]
async fn get_supported_languages() -> Vec<[String; 2]> {
        
    SUPPORTED_LANGUAGES.iter()
        .map(|&(code, name)| [code.to_string(), name.to_string()])
        .collect()
}

#[tauri::command]
async fn save_all_settings(
    device_name: Option<String>, // Allow null from frontend via Option<String>
    model_name: String,
    auto_transcribe: bool,
    offline_mode: bool,
    speech_settings: serde_json::Value, // Keep as JSON value for flexibility
    config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>
) -> Result<(), String> {
    info!("Saving settings...");
    let config_manager_guard = config_manager.inner();
    let mut config_manager = config_manager_guard.lock();
    
    // Get a mutable reference to the config
    let config = config_manager.get_config_mut();
    
    // Update General Settings
    config.general.auto_transcribe = auto_transcribe;
    config.general.offline_mode = offline_mode;
    
    // Update Audio Device
    // Handle empty string from frontend as None
    config.audio.input_device = device_name.filter(|s| !s.is_empty()); 
    
    // Update Speech Settings
    let speech = &mut config.audio.speech;

    // Update model size using the existing method in config.rs
    if let Err(e) = speech.set_model_size_from_str(&model_name) {
        warn!("Invalid model size provided '{}', defaulting to Small. Error: {}", model_name, e);
        speech.model_size = WhisperModelSize::Small; // Default on error
    } 
    // The match statement below is now redundant due to set_model_size_from_str
    /*
    speech.model_size = match model_name.as_str() {
        "tiny" => bestme::config::WhisperModelSize::Tiny,
        "base" => bestme::config::WhisperModelSize::Base,
        "small" => bestme::config::WhisperModelSize::Small,
        "medium" => bestme::config::WhisperModelSize::Medium,
        "large" => bestme::config::WhisperModelSize::Large,
        _ => speech.model_size, // Keep existing if invalid
    };
    */
    
    // Update detailed speech settings from the JSON object
    if let Some(speech_obj) = speech_settings.as_object() {
        if let Some(language) = speech_obj.get("language").and_then(|v| v.as_str()) {
            // Allow empty string or "auto"
            speech.language = if language.is_empty() { "auto".to_string() } else { language.to_string() };
        }
        
        if let Some(auto_punctuate) = speech_obj.get("auto_punctuate").and_then(|v| v.as_bool()) {
            speech.auto_punctuate = auto_punctuate;
        }
        
        if let Some(translate_to_english) = speech_obj.get("translate_to_english").and_then(|v| v.as_bool()) {
            speech.translate_to_english = translate_to_english;
        }
        
        // Keep other fields if needed
        /*
        if let Some(context_formatting) = speech_obj.get("context_formatting").and_then(|v| v.as_bool()) {
            speech.context_formatting = context_formatting;
        }
        
        if let Some(segment_duration) = speech_obj.get("segment_duration").and_then(|v| v.as_f64()) {
            speech.segment_duration = segment_duration as f32;
        }
        
        if let Some(buffer_size) = speech_obj.get("buffer_size").and_then(|v| v.as_u64()) {
            speech.buffer_size = buffer_size as f32;
        }
        */
    }

    // Save the modified config
    match config_manager.save() {
        Ok(_) => {
            info!("Settings saved successfully.");
            // TODO: Potentially emit an event to notify other parts of the app about config changes
            // app_handle.emit_all("config_updated", config.clone()).unwrap_or_default();
            Ok(())
        },
        Err(e) => {
            error!("Failed to save settings: {}", e);
            Err(format!("Failed to save settings: {}", e))
        },
    }
}

#[tauri::command]
async fn get_settings(config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>) -> Result<serde_json::Value, String> {
    info!("Fetching settings...");
    let config_manager_guard = config_manager.inner();
    let config_manager = config_manager_guard.lock();
    let config = config_manager.get_config();
    
    // Convert the Config struct to a serde_json::Value
    // Serde should use the Display trait for WhisperModelSize when serializing enums to strings 
    // within complex types if configured correctly (usually default for simple enums).
    // If it serializes as { "model_size": "Small" } (enum variant name) instead of 
    // { "model_size": "small" } (display trait), we might need a custom serializer or wrapper type.
    match serde_json::to_value(config) {
        Ok(mut value) => {
            // Ensure model_size is lowercase string using Display trait implementation
            if let Some(audio) = value.get_mut("audio") {
                if let Some(speech) = audio.get_mut("speech") {
                    if let Some(_model_size_enum) = speech.get("model_size") {
                        // Re-serialize just the model size using its Display trait
                        // Note: This assumes the original serialization produced the enum variant name.
                        // If `to_value(config)` already uses Display, this is redundant but safe.
                        let model_size_str = config.audio.speech.model_size.to_string();
                        speech["model_size"] = json!(model_size_str); 
                    }
                }
            }
            info!("Settings fetched successfully.");
            Ok(value)
        },
        Err(e) => {
            error!("Failed to serialize settings: {}", e);
            Err(format!("Failed to serialize settings: {}", e))
        },
    }
}

#[tauri::command]
async fn toggle_voice_commands(
    enabled: bool,
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Result<(), String> {
    // Don't hold the lock across await
    if enabled {
        let mut state_lock = state.inner().lock();
        // Call the synchronous start method instead
        state_lock.start().map_err(|e| e.to_string())
    } else {
        let mut state_lock = state.inner().lock();
        // Call the synchronous stop method instead
        state_lock.stop().map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn get_voice_command_settings(config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>) -> Result<serde_json::Value, String> {
    let config_manager = config_manager.inner().lock();
    let voice_commands = &config_manager.get_config().audio.voice_commands;
    
    match serde_json::to_value(voice_commands) {
        Ok(value) => Ok(value),
        Err(e) => Err(format!("Failed to serialize voice command settings: {}", e)),
    }
}

#[tauri::command]
async fn save_voice_command_settings(
    enabled: bool,
    command_prefix: Option<String>,
    require_prefix: bool,
    sensitivity: f32,
    config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>,
    voice_command_state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Result<(), String> {
    // Update and save configuration
    let voice_command_config = {
        let mut config_manager = config_manager.inner().lock();
        
        // Create the voice command config using the local type
        let mut voice_command_config = config_manager.get_config_mut().audio.voice_commands.clone();
        voice_command_config.enabled = enabled;
        
        if let Some(prefix) = command_prefix {
            voice_command_config.command_prefix = Some(prefix);
        }
        
        voice_command_config.require_prefix = require_prefix;
        voice_command_config.sensitivity = sensitivity;
        
        // Save the config to ConfigManager
        config_manager.get_config_mut().audio.voice_commands = voice_command_config.clone();
        
        // Save the updated config
        if let Err(e) = config_manager.save() {
            return Err(format!("Failed to save voice command settings: {}", e));
        }
        
        voice_command_config
    };
    
    // Update the voice command state
    {
        let mut voice_command_state = voice_command_state.inner().lock();
        // Convert from library VoiceCommandConfig to TauriVoiceCommandConfig
        let tauri_config = plugin::voice_commands::TauriVoiceCommandConfig {
            enabled: voice_command_config.enabled,
            command_prefix: voice_command_config.command_prefix.clone(),
            require_prefix: voice_command_config.require_prefix,
            sensitivity: voice_command_config.sensitivity,
            custom_commands: Vec::new(), // TODO: Convert custom commands
            default_commands: true,
        };
        if let Err(e) = voice_command_state.initialize(tauri_config) {
            return Err(format!("Failed to update voice command system: {}", e));
        }
    }
    
    // Start voice commands if enabled (must be outside the lock)
    if enabled {
        let mut voice_command_state = voice_command_state.inner().lock();
        if let Err(e) = voice_command_state.start() {
            return Err(format!("Failed to enable voice commands: {}", e));
        }
    } else {
        let mut voice_command_state = voice_command_state.inner().lock();
        if let Err(e) = voice_command_state.stop() {
            return Err(format!("Failed to disable voice commands: {}", e));
        }
    }
    
    Ok(())
}

// Struct for deserializing the content of a saved transcript file
#[derive(Serialize, Deserialize, Debug)]
struct SavedTranscriptFileContent {
    title: String,
    timestamp_ms: i64,
    content: String, // Keep content field for structure, though not read in list command
}

// Struct for the items sent to the frontend list
#[derive(Serialize, Deserialize, Debug, Clone)] // Added Clone
pub struct SavedTranscriptListItem {
    id: String,
    title: String,
    date: String, // ISO 8601 format string
}

// --- Chat Structs ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    sender: String, // "user" or "ai"
    text: String,
    timestamp_ms: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatSessionFileContent {
    id: String, // Session UUID
    title: String, // Maybe first user message or user-defined?
    created_timestamp_ms: i64,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatSessionListItem {
    id: String,
    title: String,
    date: String, // ISO 8601 format string of created_timestamp_ms
}

// --- Structs for OpenAI Compatible API ---
#[derive(Serialize, Debug, Clone)] // Only need Serialize for request
pub struct OpenAiChatMessage {
    role: String, // "user", "assistant", or "system"
    content: String,
}

#[derive(Serialize, Debug)] // Only need Serialize for request
struct OpenAiChatCompletionRequest {
    model: String,
    messages: Vec<OpenAiChatMessage>,
    // Add other optional fields like temperature, max_tokens if needed
}

// Need Deserialize for response
#[derive(Deserialize, Debug)] 
struct OpenAiChatCompletionResponse {
    choices: Vec<OpenAiChatCompletionChoice>,
    // Add other fields if needed (e.g., usage)
}

#[derive(Deserialize, Debug)]
struct OpenAiChatCompletionChoice {
    message: OpenAiChatMessageResponse,
    // Add other fields if needed (e.g., finish_reason)
}

#[derive(Deserialize, Debug, Clone)] // Clone needed for ai_chat_message
#[allow(dead_code)]
struct OpenAiChatMessageResponse {
    role: String,
    content: String,
}

#[tauri::command]
async fn list_saved_transcripts(
    app_handle: AppHandle
) -> Result<Vec<SavedTranscriptListItem>, String> {
    info!("Listing saved transcripts");

    let data_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Could not determine app data directory: {}", e))?;
    let transcripts_dir = data_dir.join("saved_transcripts");

    // Ensure the directory exists
    fs::create_dir_all(&transcripts_dir)
        .map_err(|e| format!("Failed to create transcripts directory: {}", e))?;

    let mut items = Vec::new();

    match fs::read_dir(&transcripts_dir) {
        Ok(entries) => {
            for entry_result in entries {
                match entry_result {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                let id = stem.to_string();
                                match fs::read_to_string(&path) {
                                    Ok(content_str) => {
                                        match serde_json::from_str::<SavedTranscriptFileContent>(&content_str) {
                                            Ok(file_content) => {
                                                // Convert timestamp_ms to DateTime<Utc>
                                                let timestamp = Utc.timestamp_millis_opt(file_content.timestamp_ms).single()
                                                    .unwrap_or_else(|| Utc::now()); // Fallback to now if timestamp is invalid
                                                
                                                items.push(SavedTranscriptListItem {
                                                    id,
                                                    title: file_content.title,
                                                    date: timestamp.to_rfc3339(), // Format as ISO string
                                                });
                                            }
                                            Err(e) => {
                                                warn!("Failed to parse JSON in file {}: {}", path.display(), e);
                                                // Optionally: Try to get metadata date as fallback?
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to read file {}: {}", path.display(), e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read directory entry: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to read transcripts directory '{}': {}", transcripts_dir.display(), e);
            // Return empty list or error?
            return Err(format!("Failed to read transcripts directory: {}", e));
        }
    }

    // Sort items by date, newest first
    items.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(items)
}

#[tauri::command]
async fn save_transcript(
    title: String,
    content: String,
    app_handle: AppHandle
) -> Result<(), String> {
    info!("Saving transcript: {}", title);

    // Generate unique ID
    let id = Uuid::new_v4().to_string();
    let timestamp_ms = Utc::now().timestamp_millis();

    let data_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Could not determine app data directory: {}", e))?;
    let transcripts_dir = data_dir.join("saved_transcripts");

    // Ensure the directory exists
    fs::create_dir_all(&transcripts_dir)
        .map_err(|e| format!("Failed to create transcripts directory: {}", e))?;

    // Create file content struct
    let file_content = SavedTranscriptFileContent {
        title: title.clone(),
        timestamp_ms,
        content,
    };

    // Serialize to JSON
    let json_content = serde_json::to_string_pretty(&file_content)
        .map_err(|e| format!("Failed to serialize transcript content: {}", e))?;

    // Determine file path
    let file_path = transcripts_dir.join(format!("{}.json", id));

    // Write to file
    fs::write(&file_path, json_content)
        .map_err(|e| format!("Failed to write transcript file '{}': {}", file_path.display(), e))?;

    info!("Transcript saved successfully: {}", file_path.display());
    Ok(())
}

#[tauri::command]
async fn get_saved_transcript(id: String, app_handle: AppHandle) -> Result<SavedTranscriptFileContent, String> {
    info!("Getting saved transcript: {}", id);

    let data_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Could not determine app data directory: {}", e))?;
    let transcripts_dir = data_dir.join("saved_transcripts");
    let file_path = transcripts_dir.join(format!("{}.json", id));

    if !file_path.exists() {
        return Err(format!("Transcript file not found: {}", id));
    }

    // Read the file content
    let content_str = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read transcript file '{}': {}", file_path.display(), e))?;

    // Deserialize JSON
    let file_content: SavedTranscriptFileContent = serde_json::from_str(&content_str)
        .map_err(|e| format!("Failed to parse transcript content for ID '{}': {}", id, e))?;

    Ok(file_content)
}

#[tauri::command]
async fn delete_saved_transcript(id: String, app_handle: AppHandle) -> Result<(), String> {
    info!("Deleting saved transcript: {}", id);

    let data_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Could not determine app data directory: {}", e))?;
    let transcripts_dir = data_dir.join("saved_transcripts");
    let file_path = transcripts_dir.join(format!("{}.json", id));

    if !file_path.exists() {
        // Arguably, not finding it means it's already deleted, so maybe Ok(())?
        // But returning an error might be clearer if the UI expected it to exist.
        return Err(format!("Transcript file not found for deletion: {}", id));
    }

    // Attempt to delete the file
    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete transcript file '{}': {}", file_path.display(), e))?;

    info!("Transcript deleted successfully: {}", id);
    Ok(())
}

#[tauri::command]
async fn list_chat_sessions(app_handle: AppHandle) -> Result<Vec<ChatSessionListItem>, String> {
    info!("Listing chat sessions");

    let data_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Could not determine app data directory: {}", e))?;
    let sessions_dir = data_dir.join("chat_sessions");

    // Ensure the directory exists
    fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("Failed to create chat sessions directory: {}", e))?;

    let mut items = Vec::new();

    match fs::read_dir(&sessions_dir) {
        Ok(entries) => {
            for entry_result in entries {
                match entry_result {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                            // Optimization: Could try to read only metadata, but reading full file is simpler for now
                            match fs::read_to_string(&path) {
                                Ok(content_str) => {
                                    match serde_json::from_str::<ChatSessionFileContent>(&content_str) {
                                        Ok(session_content) => {
                                            let timestamp = Utc.timestamp_millis_opt(session_content.created_timestamp_ms).single()
                                                .unwrap_or_else(Utc::now);
                                            
                                            items.push(ChatSessionListItem {
                                                id: session_content.id,
                                                title: session_content.title, // Use title stored in file
                                                date: timestamp.to_rfc3339(),
                                            });
                                        }
                                        Err(e) => {
                                            warn!("Failed to parse JSON in chat session file {}: {}", path.display(), e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to read chat session file {}: {}", path.display(), e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read directory entry in chat_sessions: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to read chat sessions directory '{}': {}", sessions_dir.display(), e);
            return Err(format!("Failed to read chat sessions directory: {}", e));
        }
    }

    // Sort items by date, newest first
    items.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(items)
}

#[tauri::command]
async fn get_chat_session(id: String, app_handle: AppHandle) -> Result<ChatSessionFileContent, String> {
    info!("Getting chat session: {}", id);

    let data_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Could not determine app data directory: {}", e))?;
    let sessions_dir = data_dir.join("chat_sessions");
    let file_path = sessions_dir.join(format!("{}.json", id));

    if !file_path.exists() {
        return Err(format!("Chat session file not found: {}", id));
    }

    // Read the file content
    let content_str = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read chat session file '{}': {}", file_path.display(), e))?;

    // Deserialize JSON
    let session_content: ChatSessionFileContent = serde_json::from_str(&content_str)
        .map_err(|e| format!("Failed to parse chat session content for ID '{}': {}", id, e))?;

    Ok(session_content)
}

#[tauri::command]
async fn send_chat_message(
    session_id: Option<String>,
    user_message: String,
    app_handle: AppHandle,
    config_manager_state: tauri::State<'_, Arc<Mutex<ConfigManager>>>
) -> Result<(String, ChatMessage), String> { // Returns our internal ChatMessage type
    info!("Sending chat message. Session: {:?}, Message: {}", session_id, user_message);

    let data_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Could not determine app data directory: {}", e))?;
    let sessions_dir = data_dir.join("chat_sessions");
    fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("Failed to create chat sessions directory: {}", e))?;

    let timestamp_now_ms = Utc::now().timestamp_millis();

    let user_chat_message_internal = ChatMessage {
        sender: "user".to_string(),
        text: user_message.clone(),
        timestamp_ms: timestamp_now_ms,
    };

    // --- Prepare data for API call --- 
    let ai_config = config_manager_state.lock().get_config().ai.clone();
    let api_key = ai_config.requesty_api_key;
    let chat_model_name = ai_config.chat_model;

    // Load existing messages if session_id is provided
    let mut history: Vec<ChatMessage> = Vec::new();
    let mut current_session_content: Option<ChatSessionFileContent> = None;

    if let Some(ref id) = session_id {
        let file_path = sessions_dir.join(format!("{}.json", id));
        if file_path.exists() {
            let content_str = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read chat session file '{}': {}", file_path.display(), e))?;
            let loaded_content: ChatSessionFileContent = serde_json::from_str(&content_str)
                .map_err(|e| format!("Failed to parse chat session content for ID '{}': {}", id, e))?;
            history = loaded_content.messages.clone(); // Clone history
            current_session_content = Some(loaded_content);
        } else {
             warn!("Existing session ID provided, but file not found: {}. Starting new chat.", id);
             // Fall through to create new session logic below
             // Ensure history is empty if file wasn't found
             history.clear(); 
        }
    }

    // --- Limit History Size --- 
    // Format messages for OpenAI API
    let mut api_messages: Vec<OpenAiChatMessage> = history.iter().map(|msg| OpenAiChatMessage {
        role: msg.sender.clone(), // Assuming sender is "user" or "ai" ("assistant")
        content: msg.text.clone(),
    }).collect();
    
    // Add the new user message
    api_messages.push(OpenAiChatMessage {
        role: "user".to_string(),
        content: user_message.clone(),
    });

    // --- Get AI Response --- 
    let ai_response_text: String;

    if let Some(key) = api_key {
        info!("API Key found, calling Requesty API...");
        let client = Client::new();
        let request_url = "https://router.requesty.ai/v1/chat/completions"; // Use deduced endpoint
        
        let payload = OpenAiChatCompletionRequest {
            model: chat_model_name.clone(),
            messages: api_messages, // Send history + new message
        };

        match client.post(request_url)
            .bearer_auth(key)
            .json(&payload)
            .send()
            .await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<OpenAiChatCompletionResponse>().await {
                        Ok(parsed) => {
                            if let Some(choice) = parsed.choices.first() {
                                ai_response_text = choice.message.content.clone();
                                info!("Received AI response successfully.");
                            } else {
                                error!("AI response contained no choices.");
                                ai_response_text = "AI Error: No response choices received.".to_string();
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse AI response: {}", e);
                            ai_response_text = format!("AI Error: Failed to parse response - {}", e);
                        }
                    }
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    error!("AI API request failed with status {}: {}", status, error_text);
                    ai_response_text = format!("AI Error: API request failed ({}) - {}", status, error_text);
                }
            }
            Err(e) => {
                error!("Failed to send request to AI API: {}", e);
                ai_response_text = format!("AI Error: Network error - {}", e);
            }
        }
    } else {
        warn!("Requesty API Key not found in config. Returning placeholder response.");
        ai_response_text = format!("AI Placeholder: API key missing. Cannot connect to AI.");
    }

    let ai_chat_message_internal = ChatMessage {
        sender: "ai".to_string(), // Use "ai" for our internal representation
        text: ai_response_text,
        timestamp_ms: Utc::now().timestamp_millis(),
    };
    // --- End Get AI Response ---

    let final_session_id: String;
    let final_session_content: ChatSessionFileContent;

    if let Some(mut existing_content) = current_session_content {
        // Append to existing session
        final_session_id = existing_content.id.clone();
        existing_content.messages.push(user_chat_message_internal);
        existing_content.messages.push(ai_chat_message_internal.clone());
        final_session_content = existing_content;
    } else {
        // Create new session
        final_session_id = Uuid::new_v4().to_string();
        final_session_content = ChatSessionFileContent {
            id: final_session_id.clone(),
            title: user_message.chars().take(30).collect::<String>() + (if user_message.len() > 30 { "..." } else { "" }),
            created_timestamp_ms: timestamp_now_ms,
            messages: vec![user_chat_message_internal, ai_chat_message_internal.clone()],
        };
    }

    // Save the updated/new session content
    let file_path = sessions_dir.join(format!("{}.json", final_session_id));
    let json_content = serde_json::to_string_pretty(&final_session_content)
        .map_err(|e| format!("Failed to serialize chat session content: {}", e))?;
    fs::write(&file_path, json_content)
        .map_err(|e| format!("Failed to write chat session file '{}': {}", file_path.display(), e))?;

    info!("Chat message processed for session: {}", final_session_id);
    Ok((final_session_id, ai_chat_message_internal))
}

#[tauri::command]
async fn delete_chat_session(id: String, app_handle: AppHandle) -> Result<(), String> {
    info!("Deleting chat session: {}", id);

    let data_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Could not determine app data directory: {}", e))?;
    let sessions_dir = data_dir.join("chat_sessions");
    let file_path = sessions_dir.join(format!("{}.json", id));

    if !file_path.exists() {
        return Err(format!("Chat session file not found for deletion: {}", id));
    }

    // Attempt to delete the file
    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete chat session file '{}': {}", file_path.display(), e))?;

    info!("Chat session deleted successfully: {}", id);
    Ok(())
}

// New command to save only Whisper-related settings
#[tauri::command]
async fn save_whisper_settings(
    model_name: String,
    language: String,
    auto_punctuate: bool, // Example: Add other relevant Whisper settings as params
    translate_to_english: bool,
    context_formatting: bool,
    segment_duration: f32,
    buffer_size: f32,
    config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>
) -> Result<(), String> {
    info!("Saving Whisper settings: Model={}, Lang={}, Punct={}, Translate={}, Context={}, SegDur={}, BufSize={}", 
        model_name, language, auto_punctuate, translate_to_english, context_formatting, segment_duration, buffer_size);
    
    let mut config_manager = config_manager.inner().lock();
    let config = config_manager.get_config_mut();
    let speech = &mut config.audio.speech;

    // Update Whisper model size
    speech.model_size = match model_name.as_str() {
        "tiny" => bestme::config::WhisperModelSize::Tiny,
        "base" => bestme::config::WhisperModelSize::Base,
        "small" => bestme::config::WhisperModelSize::Small,
        "medium" => bestme::config::WhisperModelSize::Medium,
        "large" => bestme::config::WhisperModelSize::Large,
        _ => {
            warn!("Unknown Whisper model size '{}', defaulting to Small", model_name);
            bestme::config::WhisperModelSize::Small // Default or return error?
        },
    };

    // Update other Whisper settings
    speech.language = language;
    speech.auto_punctuate = auto_punctuate;
    speech.translate_to_english = translate_to_english;
    speech.context_formatting = context_formatting;
    speech.segment_duration = segment_duration;
    speech.buffer_size = buffer_size;

    // Save the config
    match config_manager.save() {
        Ok(_) => {
            info!("Whisper settings saved successfully.");
            Ok(())
        },
        Err(e) => {
            error!("Failed to save Whisper settings: {}", e);
            Err(format!("Failed to save Whisper settings: {}", e))
        },
    }
}

// New command to save only AI Chat settings
#[tauri::command]
async fn save_ai_settings(
    requesty_api_key: Option<String>,
    chat_model: String,
    config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>
) -> Result<(), String> {
    info!("Saving AI Chat settings: Model={}", chat_model);
    // Avoid logging the API key directly for security
    if requesty_api_key.is_some() {
        debug!("Saving non-empty Requesty API Key.");
    } else {
        debug!("Saving empty/null Requesty API Key.");
    }
    
    let mut config_manager = config_manager.inner().lock();
    let config = config_manager.get_config_mut();
    
    // Update AI settings
    // If the key is an empty string from the frontend, treat it as None
    config.ai.requesty_api_key = requesty_api_key.filter(|k| !k.is_empty());
    config.ai.chat_model = chat_model;

    // Save the config
    match config_manager.save() {
        Ok(_) => {
            info!("AI Chat settings saved successfully.");
            Ok(())
        },
        Err(e) => {
            error!("Failed to save AI Chat settings: {}", e);
            Err(format!("Failed to save AI Chat settings: {}", e))
        },
    }
}

// Shared application state
struct AppState {
    audio_state: Arc<Mutex<AudioState>>,
    transcribe_state: Arc<TranscribeState>,
    voice_command_state: Arc<Mutex<VoiceCommandState>>,
    config_manager: Arc<Mutex<ConfigManager>>,
    device_manager: Arc<Mutex<DeviceManager>>,
}

fn main() {
    // Set up panic hook to log panics
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s
        } else {
            "Unknown panic"
        };
        
        let location = if let Some(location) = panic_info.location() {
            format!(" at {}:{}:{}", location.file(), location.line(), location.column())
        } else {
            String::new()
        };
        
        eprintln!("PANIC{}: {}", location, msg);
        error!("PANIC{}: {}", location, msg);
    }));
    
    // Initialize logging with environment variables and file output
    // Set RUST_LOG=debug to enable debug logging
    use std::fs::OpenOptions;
    use std::io::Write;
    
    // Get the app data directory for logs
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bestme-tauri")
        .join("logs");
    
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir).ok();
    
    let log_file_path = log_dir.join(format!("bestme_{}.log", 
        chrono::Local::now().format("%Y%m%d_%H%M%S")));
    
    let log_file_path_clone = log_file_path.clone();
    
    // Initialize logger with both console and file output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .format_module_path(true)
        .format(move |buf, record| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let formatted = format!("{} [{}] {} - {}\n", 
                timestamp, record.level(), record.target(), record.args());
            
            // Also write to file
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path_clone)
            {
                file.write_all(formatted.as_bytes()).ok();
            }
            
            writeln!(buf, "{}", formatted.trim())
        })
        .init();
    
    info!("Starting BestMe Tauri 2.0 application");
    info!("Log file: {:?}", log_file_path);

    // Initialize shared components
    let config_manager = match ConfigManager::new() {
        Ok(manager) => Arc::new(Mutex::new(manager)),
        Err(e) => {
            // Handle error: maybe create default config or panic
            eprintln!("Failed to initialize config manager: {}. Using default config.", e);
            // Consider creating a default config file here if it doesn't exist
            // Or proceed with a default in-memory config
            Arc::new(Mutex::new(ConfigManager::new().expect("Failed to create ConfigManager")))
        }
    };
    let device_manager = match DeviceManager::new() {
        Ok(manager) => Arc::new(Mutex::new(manager)),
        Err(e) => {
            eprintln!("Failed to initialize device manager: {}", e);
            // Handle error gracefully, maybe disable audio features
            Arc::new(Mutex::new(DeviceManager::new().expect("Failed to create DeviceManager")))
        }
    };
    
    // Create state objects
    let audio_state = Arc::new(Mutex::new(AudioState::new(config_manager.clone(), device_manager.clone())));
    
    // Initialize TranscribeState
    let transcribe_state = match TranscribeState::new(config_manager.clone(), None) {
        Ok(state) => Arc::new(state),
        Err(e) => {
            error!("Failed to initialize transcribe state: {}", e);
            panic!("Failed to initialize transcribe state: {}", e);
        }
    };
    
    let voice_command_state = Arc::new(Mutex::new(VoiceCommandState::new()));
    
    // Connect the states
    {
        let mut audio = audio_state.lock();
        audio.set_transcribe_state(Arc::clone(&transcribe_state));
    }
    
    {
        let mut voice_commands = voice_command_state.lock();
        
        // Initialize voice command manager with config
        let config = config_manager.lock().get_config().clone();
        // Convert from library VoiceCommandConfig to TauriVoiceCommandConfig
        let lib_config = &config.audio.voice_commands;
        let tauri_config = plugin::voice_commands::TauriVoiceCommandConfig {
            enabled: lib_config.enabled,
            command_prefix: lib_config.command_prefix.clone(),
            require_prefix: lib_config.require_prefix,
            sensitivity: lib_config.sensitivity,
            custom_commands: Vec::new(), // TODO: Convert VoiceCommandType to String
            default_commands: true,
        };
        if let Err(e) = voice_commands.initialize(tauri_config) {
            error!("Failed to initialize voice command system: {}", e);
        }
       
    }

    // Create app state
    let app_state = AppState {
        audio_state: Arc::clone(&audio_state),
        transcribe_state: Arc::clone(&transcribe_state),
        voice_command_state: Arc::clone(&voice_command_state),
        config_manager,
        device_manager,
    };

    let result = tauri::Builder::default()
        // Manage individual state components directly
        .manage(app_state.config_manager.clone())
        .manage(app_state.device_manager.clone())
        .manage(app_state.audio_state.clone())
        .manage(app_state.transcribe_state.clone())
        .manage(app_state.voice_command_state.clone())
        // Also register the complete AppState for convenience
        .manage(app_state)
        .plugin({
            info!("Initializing AudioPlugin...");
            AudioPlugin::new()
        })
        .plugin({
            info!("Initializing TranscribePlugin...");
            TranscribePlugin::new()
        })
        .plugin({
            info!("Initializing VoiceCommandPlugin...");
            VoiceCommandPlugin::new()
        })
        .plugin({
            info!("Initializing StoragePlugin...");
            StoragePlugin::new()
        })
        .plugin({
            info!("Initializing TextInjectionPlugin...");
            TextInjectionPlugin::new()
        })
        .plugin({
            info!("Initializing AIPlugin...");
            AIPlugin::default()
        })
        .invoke_handler(tauri::generate_handler![
            get_audio_devices,
            get_whisper_models,
            get_model_download_info,
            get_supported_languages,
            get_recent_transcription_list,
            save_all_settings,
            get_settings,
            toggle_voice_commands,
            get_voice_command_settings,
            save_voice_command_settings,
            // System monitor commands
            get_cpu_usage,
            get_memory_usage,
            get_online_status,
            // New history command
            list_saved_transcripts,
            save_transcript,
            get_saved_transcript,
            delete_saved_transcript,
            list_chat_sessions,
            get_chat_session,
            send_chat_message,
            delete_chat_session,
            save_whisper_settings,
            save_ai_settings,
            // Audio plugin commands
            start_recording,
            stop_recording,
            get_level,
            is_recording,
            // Transcribe plugin commands
            start_transcription,
            stop_transcription,
            get_transcription,
            is_transcribing,
            clear_transcription,
            get_download_progress,
            download_model_command,
            is_model_downloaded,
            is_voice_commands_enabled,
            set_voice_commands_enabled,
            update_whisper_params,
            get_whisper_params,
            add_vocabulary_entry,
            remove_vocabulary_entry,
            search_vocabulary,
            get_vocabulary_entries,
            import_vocabulary_csv,
            export_vocabulary_csv,
            // Voice commands plugin commands
            get_voice_commands_status,
            get_voice_commands_text,
            get_voice_commands_history,
            update_text,
            apply_delete_operation,
            undo_operation,
            redo_operation,
            get_ai_voice_settings,
            save_ai_voice_settings,
            process_ai_voice_command,
            // Storage plugin commands
            list_saved_transcripts_db,
            get_saved_transcript_db,
            save_transcript_db,
            delete_saved_transcript_db,
            search_transcripts_db,
            export_transcripts_db,
            // Text injection plugin commands
            init_text_injection,
            inject_text,
            get_active_window,
            check_injection_permissions,
            request_injection_permissions,
            update_injection_config,
            set_injection_mode,
            // AI plugin commands
            init_local_ai,
            init_cloud_ai,
            enhance_text,
            list_available_models,
            download_model,
            list_downloaded_models,
            summarize_text,
            transform_style,
            translate_text,
            get_cloud_usage,
            save_api_key,
            list_saved_api_keys,
            delete_api_key,
            get_gpu_info,
            get_model_download_progress,
            get_model_performance,
            get_current_ai_metrics,
            get_ai_performance_summary,
            get_active_downloads,
            cancel_model_download,
            get_model_config,
            update_model_config,
            load_ai_model,
            unload_ai_model,
            list_loaded_models,
            auto_select_model,
            analyze_text_characteristics,
            get_model_requirements,
            update_model_selector_config,
            get_model_selection_history,
            clear_model_selection_cache,
            check_model_updates,
            check_single_model_update,
            download_model_update,
            configure_update_checker,
            get_update_history,
            clear_update_cache,
            validate_onnx_model,
            import_custom_model,
            list_custom_models,
            get_custom_model,
            update_custom_model,
            delete_custom_model,
            export_custom_model
        ])
        .setup(|app| {
            info!("Setting up Tauri 2.0 application");
            
            // First, let's ensure the window exists and is visible
            info!("Checking for main window early in setup...");
            info!("Available windows: {:?}", app.webview_windows().keys().collect::<Vec<_>>());
            
            if let Some(window) = app.get_webview_window("main") {
                info!("Found main window early, checking URL...");
                info!("Window URL: {:?}", window.url());
                
                // Don't show the window if it's still on about:blank
                if window.url().map(|u| u.as_str() != "about:blank").unwrap_or(false) {
                    info!("Window has proper URL, making it visible...");
                    if let Err(e) = window.show() {
                        error!("Failed to show window early: {}", e);
                    }
                    // Try to bring to front
                    if let Err(e) = window.set_focus() {
                        error!("Failed to focus window: {}", e);
                    }
                } else {
                    info!("Window still on about:blank, will show later");
                }
            } else {
                error!("Main window not found early in setup!");
            }
            
            // Set app handles for components that need it
            let app_handle = app.app_handle();
            {
                let mut voice_state = voice_command_state.lock();
                voice_state.set_app_handle(app_handle.clone());
            }
            
            // Setup integration between transcription and voice commands
            {
                let app_handle_clone = app.app_handle();
                app_handle_clone.listen("transcription:update", move |event| {
                    let payload = event.payload();
                    if let Ok(text) = serde_json::from_str::<String>(payload) {
                            debug!("Processing transcription for voice commands: '{}'", text);
                            
                            // Process transcription for voice commands
                            let voice_state = voice_command_state.lock();
                            match voice_state.process_transcription(&text) {
                                Ok(commands) => {
                                    if !commands.is_empty() {
                                        info!("Detected {} voice commands in transcription", commands.len());
                                        for cmd in &commands {
                                            info!("Command: {:?}, Trigger: {}", cmd.command_type, cmd.trigger_text);
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("Failed to process transcription for voice commands: {}", e);
                                }
                            }
                    } else {
                        warn!("Failed to parse transcription payload: {}", payload);
                    }
                });
            }
            
            // Get the main window to set event listener
            info!("Looking for main window...");
            if let Some(window) = app.get_webview_window("main") {
                info!("Main window found, setting up navigation handler...");
                
                // Since we added the URL to the config, the window should load the correct content
                // Let's just show it after a small delay to ensure content is loaded
                let window_clone_nav = window.clone();
                
                // Remove the navigation attempt - let Tauri handle it based on config
                info!("Window will load content based on tauri.conf.json settings");
                
                // Use a timer to show the window after content loads
                std::thread::spawn(move || {
                    info!("Window show timer started, waiting 1 second...");
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    info!("Timer expired, showing window now");
                    
                    match window_clone_nav.show() {
                        Ok(_) => info!("Window successfully shown after delay"),
                        Err(e) => error!("Failed to show window after delay: {}", e),
                    }
                    
                    match window_clone_nav.set_focus() {
                        Ok(_) => info!("Window successfully focused"),
                        Err(e) => error!("Failed to focus window after delay: {}", e),
                    }
                    
                    info!("Window show thread completed");
                });
                
                // Clone window for use in closure
                let window_clone = window.clone();
                // Setup window events
                window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            info!("Window close requested");
                            // Hide the window instead of closing it
                            window_clone.hide().unwrap();
                            api.prevent_close();
                        }
                        tauri::WindowEvent::Destroyed => {
                            error!("Window was destroyed!");
                        }
                        tauri::WindowEvent::Resized(_) => {
                            // Normal resize, ignore
                        }
                        tauri::WindowEvent::Moved(_) => {
                            // Normal move, ignore
                        }
                        _ => {
                            debug!("Window event: {:?}", event);
                        }
                    }
                });
            } else {
                error!("Main window not found!");
            }
            
            // Set up system tray
            info!("Setting up system tray...");
            use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder};
            
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            
            let menu = Menu::with_items(app, &[&show, &hide, &quit])?;
            info!("Tray menu created");
            
            info!("Creating tray icon...");
            let tray_result = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("BestMe - Speech to Text")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| {
                    info!("Tray menu event: {}", event.id.as_ref());
                    match event.id.as_ref() {
                        "quit" => {
                            info!("Quit requested from tray");
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        "hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.hide().unwrap();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::TrayIconEvent;
                    match event {
                        TrayIconEvent::Click { button, button_state, .. } => {
                            if button == tauri::tray::MouseButton::Left && button_state == tauri::tray::MouseButtonState::Up {
                                // Show window on left click
                                if let Some(window) = tray.app_handle().get_webview_window("main") {
                                    window.show().unwrap();
                                    window.set_focus().unwrap();
                                }
                            }
                        }
                        TrayIconEvent::DoubleClick { .. } => {
                            // Also show on double click
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app);
            
            match tray_result {
                Ok(_tray) => info!("System tray initialized successfully"),
                Err(e) => error!("Failed to create system tray: {}", e),
            }
            
            // Initialize AudioState properly after it's managed
            info!("Initializing AudioState...");
            let audio_state_managed = app.state::<Arc<Mutex<AudioState>>>();
            match audio_state_managed.lock().initialize() {
                Ok(_) => info!("AudioState initialized successfully"),
                Err(e) => {
                    error!("Failed to initialize AudioState during setup: {}", e);
                    // Continue anyway - audio might not be critical
                }
            }

            // Set AppHandle for voice commands state AFTER getting it from managed state
            let voice_state_managed = app.state::<Arc<Mutex<VoiceCommandState>>>();
            voice_state_managed.lock().set_app_handle(app.handle().clone());

            // Start voice commands if enabled (moved from plugin initialize to ensure state is ready)
             let config_state = app.state::<Arc<Mutex<ConfigManager>>>();
            let initial_config = config_state.lock().get_config().clone();
            
            {
                 if initial_config.audio.voice_commands.enabled {
                     info!("Voice commands auto-start temporarily disabled to fix runtime issue");
                     // TODO: Fix the voice command initialization in async context
                     // // Use the already managed voice_state_managed
                     // let voice_state_clone = Arc::clone(&voice_state_managed);
                     // tokio::spawn(async move {
                     //      // We need to lock the state inside the async block
                     //      let mut voice_state_lock = voice_state_clone.lock();
                     //      if let Err(e) = voice_state_lock.start() {
                     //            error!("Failed to auto-start voice commands: {}", e);
                     //      } else {
                     //            info!("Voice commands started successfully via config.");
                     //      }
                     // });
                 } else {
                     info!("Voice commands disabled in initial config.");
                 }
            }
            
            // Initialize system monitor
            let handle_clone = app_handle.clone();
            system_monitor::start_monitoring(handle_clone);

            // Start recording automatically if configured
            let should_auto_start = {
                let config = config_state.lock().get_config().clone();
                 config.general.auto_transcribe
            };

            if should_auto_start {
                info!("Auto-starting recording as per config...");
                // Use the managed audio state
                let audio_state_clone = Arc::clone(&audio_state_managed);
                // Start recording in a separate thread to avoid Tokio runtime issues
                std::thread::spawn(move || {
                     let audio_state_lock = audio_state_clone.lock();
                     if let Err(e) = audio_state_lock.start_recording() {
                         error!("Failed to auto-start recording: {}", e);
                     } else {
                         info!("Auto-start recording initiated successfully.");
                     }
                });
            } else {
                 info!("Auto-start recording disabled in config.");
            }

            info!("App setup finished successfully.");
            Ok(())
        })
        .run(tauri::generate_context!());
    
    match result {
        Ok(_) => info!("Tauri application exited successfully"),
        Err(e) => {
            error!("Error while running Tauri application: {}", e);
            eprintln!("Fatal error: {}", e);
            std::process::exit(1);
        }
    }
} 
