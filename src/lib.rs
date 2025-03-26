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
    
    // Run the application
    info!("Running BestMe application");
    app.run()?;
    
    info!("BestMe application completed successfully");
    Ok(())
} 
