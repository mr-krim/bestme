// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{info, error};
use tauri::Manager as Manager2;

// Define a simple plugin for voice commands
mod plugin {
    use tauri::{Runtime, Manager, plugin::{Builder, TauriPlugin}};
    
    pub struct VoiceCommandPlugin2;
    
    impl VoiceCommandPlugin2 {
        pub fn new() -> Self {
            Self {}
        }
    }
    
    impl<R: Runtime> tauri::Plugin<R> for VoiceCommandPlugin2 {
        fn name(&self) -> &'static str {
            "voice_commands"
        }
        
        fn initialize(&mut self, app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
            log::info!("Initializing voice command plugin (Tauri 2.0)");
            Ok(())
        }
    }
}

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .format_module_path(true)
        .init();
    
    info!("Starting BestMe 2.0 Tauri application");

    tauri::Builder::default()
        .plugin(plugin::VoiceCommandPlugin2::new())
        .setup(|app| {
            info!("Setting up Tauri 2.0 application");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
