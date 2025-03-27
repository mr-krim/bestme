#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plugin;

use log::{error, info, debug, warn};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{Manager, Runtime, State};
use serde_json::Value as JsonValue;

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
    let app_state = state.inner();
    let device_manager = app_state.device_manager.lock();
    
    match device_manager.list_devices() {
        Ok(devices) => devices.into_iter().map(|d| d.name().to_string()).collect(),
        Err(e) => {
            error!("Failed to get audio devices: {}", e);
            Vec::new()
        }
    }
}

#[tauri::command]
fn get_whisper_models() -> Vec<String> {
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
fn get_model_download_info() -> Vec<serde_json::Value> {
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
fn get_supported_languages() -> Vec<[String; 2]> {
    use plugin::transcribe::SUPPORTED_LANGUAGES;
    
    SUPPORTED_LANGUAGES.iter()
        .map(|&(code, name)| [code.to_string(), name.to_string()])
        .collect()
}

#[tauri::command]
fn save_all_settings(
    device_name: String,
    model_name: String,
    auto_transcribe: bool,
    offline_mode: bool,
    speech_settings: serde_json::Value,
    config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>
) -> Result<(), String> {
    let mut config_manager = config_manager.inner().lock();
    
    // Get the current config
    let mut config = config_manager.get_config_mut();
    
    // Update audio device
    config.audio.device_name = device_name;
    
    // Update model settings
    config.transcription.model_name = model_name;
    config.transcription.auto_transcribe = auto_transcribe;
    config.transcription.offline_mode = offline_mode;
    
    // Update speech settings if provided
    if let Some(speech_obj) = speech_settings.as_object() {
        if let Some(language) = speech_obj.get("language").and_then(|v| v.as_str()) {
            config.transcription.language = language.to_string();
        }
        
        if let Some(auto_punctuate) = speech_obj.get("auto_punctuate").and_then(|v| v.as_bool()) {
            config.transcription.auto_punctuate = auto_punctuate;
        }
        
        if let Some(translate_to_english) = speech_obj.get("translate_to_english").and_then(|v| v.as_bool()) {
            config.transcription.translate_to_english = translate_to_english;
        }
        
        if let Some(context_formatting) = speech_obj.get("context_formatting").and_then(|v| v.as_bool()) {
            config.transcription.context_formatting = context_formatting;
        }
        
        if let Some(segment_duration) = speech_obj.get("segment_duration").and_then(|v| v.as_f64()) {
            config.transcription.segment_duration = segment_duration as f32;
        }
        
        if let Some(buffer_size) = speech_obj.get("buffer_size").and_then(|v| v.as_u64()) {
            config.transcription.buffer_size = buffer_size as usize;
        }
    }
    
    // Save the config
    match config_manager.save_config() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to save settings: {}", e)),
    }
}

#[tauri::command]
fn get_settings(config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>) -> Result<serde_json::Value, String> {
    let config_manager = config_manager.inner().lock();
    let config = config_manager.get_config();
    
    match serde_json::to_value(config) {
        Ok(value) => Ok(value),
        Err(e) => Err(format!("Failed to serialize settings: {}", e)),
    }
}

#[tauri::command]
fn toggle_voice_commands(
    enabled: bool,
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Result<(), String> {
    let mut state = state.inner().lock();
    
    if enabled {
        state.enable().map_err(|e| e.to_string())
    } else {
        state.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_voice_command_settings(config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>) -> Result<serde_json::Value, String> {
    let config_manager = config_manager.inner().lock();
    let voice_commands = &config_manager.get_config().audio.voice_commands;
    
    match serde_json::to_value(voice_commands) {
        Ok(value) => Ok(value),
        Err(e) => Err(format!("Failed to serialize voice command settings: {}", e)),
    }
}

#[tauri::command]
fn save_voice_command_settings(
    enabled: bool,
    command_prefix: Option<String>,
    require_prefix: bool,
    sensitivity: f32,
    config_manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>,
    voice_command_state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Result<(), String> {
    let mut config_manager = config_manager.inner().lock();
    
    // Create the voice command config using the local type
    let mut voice_command_config = config_manager.get_config_mut().audio.voice_commands.clone();
    voice_command_config.enabled = enabled;
    
    if let Some(prefix) = command_prefix {
        voice_command_config.command_prefix = prefix;
    }
    
    voice_command_config.require_prefix = require_prefix;
    voice_command_config.sensitivity = sensitivity;
    
    // Save the config to ConfigManager
    config_manager.get_config_mut().audio.voice_commands = voice_command_config.clone();
    
    // Save the updated config
    if let Err(e) = config_manager.save_config() {
        return Err(format!("Failed to save voice command settings: {}", e));
    }
    
    // Update the voice command state
    let mut voice_command_state = voice_command_state.inner().lock();
    if let Err(e) = voice_command_state.initialize(voice_command_config) {
        return Err(format!("Failed to update voice command system: {}", e));
    }
    
    // Start voice commands if enabled
    if enabled {
        if let Err(e) = voice_command_state.enable() {
            return Err(format!("Failed to enable voice commands: {}", e));
        }
    } else {
        if let Err(e) = voice_command_state.disable() {
            return Err(format!("Failed to disable voice commands: {}", e));
        }
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
    // Initialize logging with environment variables
    // Set RUST_LOG=debug to enable debug logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .format_module_path(true)
        .init();
    
    info!("Starting BestMe Tauri application");

    // Initialize shared components
    let config_manager = Arc::new(Mutex::new(ConfigManager::new().expect("Failed to initialize config manager")));
    let device_manager = Arc::new(Mutex::new(DeviceManager::new().expect("Failed to initialize device manager")));
    
    // Create state objects
    let audio_state = Arc::new(Mutex::new(AudioState::new(device_manager.clone())));
    
    // Fix TranscribeState initialization
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
        // Manage individual state components directly, not the wrapper
        .manage(app_state.config_manager.clone())
        .manage(app_state.device_manager.clone())
        .manage(app_state.audio_state.clone())
        .manage(app_state.transcribe_state.clone())
        .manage(app_state.voice_command_state.clone())
        .plugin(AudioPlugin::new())
        .plugin(TranscribePlugin::new())
        .plugin(VoiceCommandPlugin::new())
        .invoke_handler(tauri::generate_handler![
            get_audio_devices,
            get_whisper_models,
            get_model_download_info,
            get_supported_languages,
            save_all_settings,
            get_settings,
            toggle_voice_commands,
            get_voice_command_settings,
            save_voice_command_settings,
        ])
        .setup(|app| {
            info!("Setting up Tauri application");
            
            // Update app_handle in transcribe_state
            let app_handle = app.app_handle();
            {
                let transcribe_state = app.state::<Arc<TranscribeState>>();
                transcribe_state.inner().set_app_handle(app_handle.clone());
            }
            
            // Setup integration between transcription and voice commands
            {
                let transcribe_state = app.state::<Arc<TranscribeState>>();
                let voice_command_state = app.state::<Arc<Mutex<VoiceCommandState>>>();
                
                // Register integration events
                app.listen_global("transcription:update", move |event| {
                    if let Some(payload) = event.payload() {
                        if let Ok(text) = serde_json::from_str::<String>(payload) {
                            debug!("Processing transcription for voice commands: '{}'", text);
                            
                            // Process transcription for voice commands
                            let voice_state = voice_command_state.inner().lock();
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
                    } else {
                        warn!("Received transcription update event with no payload");
                    }
                });
            }
            
            // Setup system tray
            #[cfg(target_os = "macos")]
            {
                let config_manager = app.state::<Arc<Mutex<ConfigManager>>>();
                let device_manager = app.state::<Arc<Mutex<DeviceManager>>>();
                setup_system_tray(app, config_manager.inner().clone(), device_manager.inner().clone());
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                let config_manager = app.state::<Arc<Mutex<ConfigManager>>>();
                let device_manager = app.state::<Arc<Mutex<DeviceManager>>>();
                setup_system_tray(app, config_manager.inner().clone(), device_manager.inner().clone());
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

#[cfg(target_os = "macos")]
fn setup_system_tray<R: Runtime>(
    app: &tauri::App<R>,
    config_manager: Arc<Mutex<ConfigManager>>,
    device_manager: Arc<Mutex<DeviceManager>>,
) {
    use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

    // Create system tray menu
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show".to_string(), "Show"))
        .add_item(CustomMenuItem::new("hide".to_string(), "Hide"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

    // Set the menu for the system tray
    let _ = app.tray_handle().set_menu(tray_menu);

    // Register event handler for tray events
    app.tray_handle().on_event(move |event| match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "show" => {
                info!("Show system tray");
                // TODO: Implement show window logic
            }
            "hide" => {
                info!("Hide system tray");
                // TODO: Implement hide window logic
            }
            "quit" => {
                info!("Quit application");
                std::process::exit(0);
            }
            _ => {}
        },
        _ => {}
    });
}

#[cfg(not(target_os = "macos"))]
fn setup_system_tray<R: Runtime>(
    app: &tauri::App<R>,
    config_manager: Arc<Mutex<ConfigManager>>,
    device_manager: Arc<Mutex<DeviceManager>>,
) {
    use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

    // Create system tray menu
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show".to_string(), "Show"))
        .add_item(CustomMenuItem::new("hide".to_string(), "Hide"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

    // Set the menu for the system tray
    let _ = app.tray_handle().set_menu(tray_menu);

    // Register event handler for tray events
    app.tray_handle().on_event(move |event| match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "show" => {
                info!("Show system tray");
                // TODO: Implement show window logic
            }
            "hide" => {
                info!("Hide system tray");
                // TODO: Implement hide window logic
            }
            "quit" => {
                info!("Quit application");
                std::process::exit(0);
            }
            _ => {}
        },
        _ => {}
    });
} 
