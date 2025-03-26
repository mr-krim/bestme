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

// Import our custom plugins
use plugin::{AudioPlugin, AudioState, TranscribePlugin, TranscribeState};

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
fn save_settings(
    device_name: String,
    model_name: String,
    auto_transcribe: bool,
    offline_mode: bool,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    info!("Saving settings - Device: {}, Model: {}, Auto: {}, Offline: {}", 
          device_name, model_name, auto_transcribe, offline_mode);
    
    let mut config_manager = state.config_manager.lock();
    
    // Update configuration
    if let Err(e) = config_manager.set_preferred_device_name(device_name) {
        return Err(format!("Failed to set device: {}", e));
    }
    
    if let Err(e) = config_manager.whisper_config_mut().set_model_size_from_str(&model_name) {
        return Err(format!("Failed to set model: {}", e));
    }
    
    config_manager.set_auto_transcribe(auto_transcribe);
    config_manager.set_offline_mode(offline_mode);
    
    // Save to disk
    if let Err(e) = config_manager.save() {
        return Err(format!("Failed to save config: {}", e));
    }
    
    Ok(())
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let config_manager = state.config_manager.lock();
    
    // Build settings object
    let settings = serde_json::json!({
        "device_name": config_manager.preferred_device_name(),
        "model_name": config_manager.whisper_config().model_size.to_string(),
        "auto_transcribe": config_manager.auto_transcribe(),
        "offline_mode": config_manager.offline_mode(),
    });
    
    Ok(settings)
}

// Shared application state
struct AppState {
    audio_state: AudioState,
    transcribe_state: Arc<Mutex<TranscribeState>>,
    config_manager: Arc<Mutex<ConfigManager>>,
    device_manager: Arc<Mutex<DeviceManager>>,
}

fn main() {
    env_logger::init();
    info!("Starting BestMe Tauri application");

    let config_manager = Arc::new(Mutex::new(ConfigManager::new()));
    let device_manager = Arc::new(Mutex::new(DeviceManager::new()));
    let audio_state = AudioState::new(config_manager.clone(), device_manager.clone());
    let transcribe_state = TranscribeState::new(config_manager.clone(), None); // Will set app_handle in setup

    let app_state = AppState {
        audio_state,
        transcribe_state,
        config_manager,
        device_manager,
    };

    tauri::Builder::default()
        .manage(app_state)
        .manage(audio_state)
        .manage(transcribe_state)
        .plugin(AudioPlugin::new())
        .plugin(TranscribePlugin::new())
        .invoke_handler(tauri::generate_handler![
            get_audio_devices,
            plugin::audio::start_listening,
            plugin::audio::stop_listening,
            plugin::audio::get_current_audio_level,
            plugin::transcribe::start_transcription,
            plugin::transcribe::stop_transcription,
            plugin::transcribe::get_transcription_text,
            plugin::transcribe::is_transcribing,
            plugin::transcribe::clear_transcription,
            plugin::transcribe::download_model_command,
            plugin::transcribe::get_download_progress,
            save_settings,
            get_settings,
            get_whisper_models,
            get_model_download_info,
        ])
        .setup(|app| {
            info!("Setting up Tauri application");
            
            // Update app_handle in transcribe_state
            {
                let mut transcribe_state = app_state.transcribe_state.lock().unwrap();
                transcribe_state.app_handle = Some(app.app_handle());
            }
            
            // Setup system tray
            #[cfg(desktop)]
            setup_system_tray(app, config_manager.clone(), device_manager.clone());
            
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
