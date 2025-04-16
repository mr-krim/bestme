pub mod config;
pub mod app;
pub mod audio;
pub mod gui;

use anyhow::Result;
use log::info;

use crate::app::App;
use crate::config::ConfigManager;

/// Initialize and run the application
pub fn run() -> Result<()> {
    info!("Initializing BestMe application from library");
    
    // Initialize configuration
    let config_manager = ConfigManager::new()?;
    
    // Initialize application
    let mut app = App::new(config_manager)?;
    
    // Run the application
    app.run()?;
    
    info!("BestMe application completed successfully");
    Ok(())
}

/// Initialize and run the application with GUI mode forced
pub fn run_with_gui() -> Result<()> {
    info!("Initializing BestMe application with GUI mode");
    
    // Initialize configuration
    let config_manager = ConfigManager::new()?;
    
    // Initialize application
    let mut app = App::new(config_manager)?;
    
    // Force GUI mode
    app.set_gui_mode(true);
    
    // Run the application
    app.run()?;
    
    info!("BestMe application completed successfully");
    Ok(())
} 
