use anyhow::Result;
use log::{error, info, LevelFilter};
use std::env;
use bestme::config::ConfigManager;
use bestme::app::App;

/// Main entry point
fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Log startup information
    info!("Initializing BestMe application");
    
    // Check for GUI mode command-line argument
    let args: Vec<String> = std::env::args().collect();
    let force_gui = args.len() > 1 && (args[1] == "--gui" || args[1] == "-g");
    
    // Create configuration manager
    let config_manager = ConfigManager::new()?;
    
    // Create application instance
    let mut app = App::new(config_manager)?;
    
    // Force GUI mode if requested
    if force_gui {
        info!("Forcing GUI mode from command-line argument");
        app.set_gui_mode(true);
    }
    
    // Run the application
    app.run()?;
    
    Ok(())
} 
