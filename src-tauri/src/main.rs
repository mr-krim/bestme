#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plugin;

use log::{error, info};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{Manager, Runtime};

// Import from main bestme crate
use bestme::audio::device::DeviceManager;
use bestme::config::ConfigManager;
use bestme::config::WhisperModelSize;
use bestme::audio::voice_commands::VoiceCommandConfig;

// Import our custom plugins
use plugin::{
    AudioPlugin, 
    AudioState, 
    TranscribePlugin, 
    TranscribeState,
    voice_commands::{VoiceCommandPlugin, VoiceCommandState}
};

use plugin::transcribe::SUPPORTED_LANGUAGES;

// Commands that will be exposed to the frontend
#[tauri::command]
fn get_audio_devices(state: tauri::State<'_, AppState>) -> Vec<String> {
    let device_manager = state.device_manager.lock();
    device_manager.list_devices()
        .unwrap_or_else(|_| Vec::new())
        .into_iter()
        .map(|device| device.name().to_string())
        .collect()
}

#[tauri::command]
fn get_whisper_models() -> Vec<String> {
    vec![
        "tiny".to_string(),
        "base".to_string(),
        "small".to_string(),
        "medium".to_string(),
        "large".to_string(),
    ]
}

#[tauri::command]
fn get_model_download_info() -> Vec<serde_json::Value> {
    use serde_json::json;
    
    vec![
        json!({
            "name": "tiny",
            "size_mb": 75,
            "description": "Fastest, lowest accuracy, minimal resources"
        }),
        json!({
            "name": "base", 
            "size_mb": 142,
            "description": "Fast with basic accuracy"
        }),
        json!({
            "name": "small",
            "size_mb": 466,
            "description": "Good balance of speed and accuracy"
        }),
        json!({
            "name": "medium",
            "size_mb": 1500,
            "description": "High accuracy, moderate resource usage"
        }),
        json!({
            "name": "large",
            "size_mb": 2900,
            "description": "Highest accuracy, significant resource usage"
        })
    ]
}

#[tauri::command]
fn get_supported_languages() -> Vec<[String; 2]> {
    // Convert language data to format expected by frontend
    SUPPORTED_LANGUAGES
        .iter()
        .map(|(code, name)| [code.to_string(), name.to_string()])
        .collect()
}

#[tauri::command]
fn save_all_settings(
    device_name: String,
    model_name: String,
    auto_transcribe: bool,
    offline_mode: bool,
    speech_settings: serde_json::Value,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    info!("Saving all settings with speech settings: {:?}", speech_settings);
    
    let mut config_manager = state.config_manager.lock();
    
    // Update basic configuration
    if let Err(e) = config_manager.set_preferred_device_name(device_name) {
        return Err(format!("Failed to set device: {}", e));
    }
    
    if let Err(e) = config_manager.whisper_config_mut().set_model_size_from_str(&model_name) {
        return Err(format!("Failed to set model: {}", e));
    }
    
    config_manager.set_auto_transcribe(auto_transcribe);
    config_manager.set_offline_mode(offline_mode);
    
    // Update speech settings
    if let Ok(speech_obj) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(speech_settings) {
        let speech_config = config_manager.whisper_config_mut();
        
        // Update language
        if let Some(language) = speech_obj.get("language").and_then(|v| v.as_str()) {
            speech_config.language = language.to_string();
        }
        
        // Update auto-punctuation
        if let Some(auto_punct) = speech_obj.get("auto_punctuate").and_then(|v| v.as_bool()) {
            speech_config.auto_punctuate = auto_punct;
        }
        
        // Update translation setting
        if let Some(translate) = speech_obj.get("translate_to_english").and_then(|v| v.as_bool()) {
            speech_config.translate_to_english = translate;
        }
        
        // Update context formatting
        if let Some(formatting) = speech_obj.get("context_formatting").and_then(|v| v.as_bool()) {
            speech_config.context_formatting = formatting;
        }
        
        // Update segment duration
        if let Some(duration) = speech_obj.get("segment_duration").and_then(|v| v.as_f64()) {
            speech_config.segment_duration = duration as f32;
        }
        
        // Update buffer size
        if let Some(buffer) = speech_obj.get("buffer_size").and_then(|v| v.as_f64()) {
            speech_config.buffer_size = buffer as f32;
        }
    }
    
    // Save to disk
    if let Err(e) = config_manager.save() {
        return Err(format!("Failed to save config: {}", e));
    }
    
    Ok(())
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let config_manager = state.config_manager.lock();
    let config = config_manager.get_config();
    
    // Build settings object with enhanced speech settings
    let settings = serde_json::json!({
        "device_name": config_manager.preferred_device_name(),
        "model_name": config_manager.whisper_config().model_size.to_string(),
        "auto_transcribe": config_manager.auto_transcribe(),
        "offline_mode": config_manager.offline_mode(),
        "speech": {
            "language": config.audio.speech.language,
            "auto_punctuate": config.audio.speech.auto_punctuate,
            "translate_to_english": config.audio.speech.translate_to_english,
            "context_formatting": config.audio.speech.context_formatting,
            "segment_duration": config.audio.speech.segment_duration,
            "buffer_size": config.audio.speech.buffer_size
        }
    });
    
    Ok(settings)
}

#[tauri::command]
fn toggle_voice_commands(
    enabled: bool,
    state: tauri::State<'_, VoiceCommandState>
) -> Result<(), String> {
    info!("Voice commands toggled: {}", enabled);
    
    if enabled {
        state.inner().start()
    } else {
        state.inner().stop()
    }.map_err(|e| e.to_string())
}

#[tauri::command]
fn get_voice_command_settings(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let config_manager = state.config_manager.lock();
    let voice_command_config = &config_manager.get_config().audio.voice_commands;
    
    // Build settings object
    let settings = serde_json::json!({
        "enabled": voice_command_config.enabled,
        "command_prefix": voice_command_config.command_prefix,
        "require_prefix": voice_command_config.require_prefix,
        "sensitivity": voice_command_config.sensitivity,
    });
    
    Ok(settings)
}

#[tauri::command]
fn save_voice_command_settings(
    enabled: bool,
    command_prefix: Option<String>,
    require_prefix: bool,
    sensitivity: f32,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    info!("Saving voice command settings - Enabled: {}, Prefix: {:?}, Require: {}, Sensitivity: {}", 
          enabled, command_prefix, require_prefix, sensitivity);
    
    let mut config_manager = state.config_manager.lock();
    let voice_command_config = &mut config_manager.get_config_mut().audio.voice_commands;
    
    voice_command_config.enabled = enabled;
    voice_command_config.command_prefix = command_prefix;
    voice_command_config.require_prefix = require_prefix;
    voice_command_config.sensitivity = sensitivity.clamp(0.0, 1.0); // Ensure value is between 0 and 1
    
    // Save to disk
    if let Err(e) = config_manager.save() {
        return Err(format!("Failed to save config: {}", e));
    }
    
    // Update voice command state if settings changed
    if let Err(e) = state.voice_command_state.lock().initialize(voice_command_config.clone()) {
        return Err(format!("Failed to update voice command system: {}", e));
    }
    
    Ok(())
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
    env_logger::init();
    info!("Starting BestMe Tauri application");

    // Initialize shared components
    let config_manager = Arc::new(Mutex::new(ConfigManager::new().expect("Failed to initialize config manager")));
    let device_manager = Arc::new(Mutex::new(DeviceManager::new().expect("Failed to initialize device manager")));
    
    // Create state objects
    let audio_state = Arc::new(Mutex::new(AudioState::new(device_manager.clone())));
    let transcribe_state = Arc::new(TranscribeState::new(config_manager.clone(), None)); // Will set app_handle in setup
    let voice_command_state = Arc::new(Mutex::new(VoiceCommandState::new()));
    
    // Connect the states
    {
        let mut audio = audio_state.lock();
        audio.set_transcribe_state(Arc::clone(&transcribe_state));
    }
    
    {
        let mut voice_commands = voice_command_state.lock();
        voice_commands.set_transcribe_state(Arc::clone(&transcribe_state));
        
        // Initialize voice command manager with config
        let voice_command_config = config_manager.lock().get_config().audio.voice_commands.clone();
        if let Err(e) = voice_commands.initialize(voice_command_config) {
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

    tauri::Builder::default()
        .manage(app_state)
        .manage(audio_state)
        .manage(transcribe_state)
        .manage(voice_command_state)
        .plugin(AudioPlugin::new())
        .plugin(TranscribePlugin::new())
        .plugin(VoiceCommandPlugin::new())
        .invoke_handler(tauri::generate_handler![
            get_audio_devices,
            plugin::audio::start_recording,
            plugin::audio::stop_recording,
            plugin::audio::get_peak_level,
            plugin::audio::is_recording,
            plugin::transcribe::start_transcription,
            plugin::transcribe::stop_transcription,
            plugin::transcribe::get_transcription,
            plugin::transcribe::is_transcribing,
            plugin::transcribe::clear_transcription,
            plugin::transcribe::get_download_progress,
            plugin::voice_commands::start_voice_commands,
            plugin::voice_commands::stop_voice_commands,
            plugin::voice_commands::get_last_command,
            plugin::voice_commands::clear_last_command,
            plugin::voice_commands::is_voice_commands_enabled,
            toggle_voice_commands,
            get_voice_command_settings,
            save_voice_command_settings,
            save_all_settings,
            get_settings,
            get_whisper_models,
            get_model_download_info,
            plugin::transcribe::download_model_command,
            get_supported_languages,
        ])
        .setup(|app| {
            info!("Setting up Tauri application");
            
            // Update app_handle in transcribe_state
            {
                let mut transcribe_state = app.state::<TranscribeState>();
                transcribe_state.set_app_handle(app.app_handle());
            }
            
            // Setup integration between transcription and voice commands
            {
                let transcribe_state = app.state::<TranscribeState>();
                let mut voice_command_state = app.state::<Arc<Mutex<VoiceCommandState>>>();
                let voice_command_state = voice_command_state.lock();
                
                // Register integration events
                app.listen_global("transcription:update", move |event| {
                    if let Some(payload) = event.payload() {
                        if let Ok(text) = serde_json::from_str::<String>(payload) {
                            // Process transcription for voice commands
                            if let Err(e) = voice_command_state.process_transcription(&text) {
                                error!("Failed to process transcription for voice commands: {}", e);
                            }
                        }
                    }
                });
            }
            
            // Setup system tray
            #[cfg(desktop)]
            setup_system_tray(app, Arc::clone(&app_state.config_manager), Arc::clone(&app_state.device_manager));
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

#[cfg(desktop)]
fn setup_system_tray<R: Runtime>(
    app: &tauri::App<R>,
    config_manager: Arc<Mutex<ConfigManager>>,
    device_manager: Arc<Mutex<DeviceManager>>,
) {
    use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

    // Create system tray menu
    let start = CustomMenuItem::new("start".to_string(), "Start Transcription");
    let stop = CustomMenuItem::new("stop".to_string(), "Stop Transcription").disabled();
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");

    let tray_menu = SystemTrayMenu::new()
        .add_item(start)
        .add_item(stop)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    // Tray event handler
    let app_handle = app.app_handle();
    app.system_tray_handle().set_menu(tray_menu).unwrap();

    app.on_system_tray_event(move |app, event| match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "start" => {
                info!("Start transcription from tray");
                // Update menu items
                app.tray_handle().get_item("start").set_disabled(true).unwrap();
                app.tray_handle().get_item("stop").set_disabled(false).unwrap();
                
                // Get the selected device name from config
                let device_name = {
                    let config = config_manager.lock();
                    config.preferred_device_name().to_string()
                };
                
                // Get audio and transcribe states
                let audio_state = app.state::<AudioState>();
                let transcribe_state = app.state::<TranscribeState>();
                
                // Start audio recording and transcription
                if let Some(mut audio_state) = audio_state.try_inner() {
                    if let Err(e) = audio_state.start_recording(&device_name) {
                        error!("Failed to start audio recording: {}", e);
                    } else if let Some(transcribe_state) = transcribe_state.try_inner() {
                        if let Err(e) = transcribe_state.start_transcription() {
                            error!("Failed to start transcription: {}", e);
                        }
                    }
                }
            }
            "stop" => {
                info!("Stop transcription from tray");
                // Update menu items
                app.tray_handle().get_item("start").set_disabled(false).unwrap();
                app.tray_handle().get_item("stop").set_disabled(true).unwrap();
                
                // Get audio and transcribe states
                let audio_state = app.state::<AudioState>();
                let transcribe_state = app.state::<TranscribeState>();
                
                // Stop transcription and audio recording
                if let Some(transcribe_state) = transcribe_state.try_inner() {
                    if let Err(e) = transcribe_state.stop_transcription() {
                        error!("Failed to stop transcription: {}", e);
                    }
                }
                
                if let Some(mut audio_state) = audio_state.try_inner() {
                    if let Err(e) = audio_state.stop_recording() {
                        error!("Failed to stop audio recording: {}", e);
                    }
                }
            }
            "settings" => {
                info!("Show settings");
                // Show settings window
                app.get_window("settings").unwrap_or_else(|| {
                    // Create settings window if it doesn't exist
                    tauri::WindowBuilder::new(
                        app,
                        "settings",
                        tauri::WindowUrl::App("settings.html".into()),
                    )
                    .title("BestMe Settings")
                    .resizable(true)
                    .build()
                    .unwrap()
                }).show().unwrap();
            }
            "quit" => {
                info!("Quit application");
                app.exit(0);
            }
            _ => {}
        },
        _ => {}
    });

    app.manage(system_tray);
} 
