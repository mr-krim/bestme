pub mod config;
pub mod app;
pub mod audio;
pub mod gui;

use anyhow::Result;
use log::{error, info};

use crate::app::App;
use crate::config::ConfigManager;

/// Initialize and run the application
pub fn run() -> Result<()> {
    // Default to console mode
    run_with_options(false)
}

/// Initialize and run the application with specific options
pub fn run_with_options(use_gui: bool) -> Result<()> {
    info!("Initializing BestMe application");
    
    // Initialize configuration
    let config_manager = match ConfigManager::new() {
        Ok(cm) => cm,
        Err(e) => {
            error!("Failed to initialize configuration: {}", e);
            return Err(e);
        }
    };
    
    // Initialize application
    let mut app = App::new(config_manager)?;
    
    // Run the application with the specified mode
    info!("Running BestMe application");
    if use_gui {
        info!("Using GUI mode");
        app.run_gui()?;
    } else {
        info!("Using console mode");
        app.run()?;
    }
    
    info!("BestMe application completed successfully");
    Ok(())
} 
