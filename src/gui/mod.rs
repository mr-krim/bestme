pub mod icons;
pub mod settings;
pub mod tray;
pub mod window;

use anyhow::Result;
use log::info;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::audio::device::DeviceManager;
use crate::config::ConfigManager;

/// Main GUI manager that handles both the system tray and main window
pub struct Gui {
    #[allow(dead_code)]
    config_manager: Arc<Mutex<ConfigManager>>,
    #[allow(dead_code)]
    device_manager: Arc<Mutex<DeviceManager>>,
}

impl Gui {
    /// Create a new GUI manager
    pub fn new(
        config_manager: Arc<Mutex<ConfigManager>>,
        device_manager: Arc<Mutex<DeviceManager>>,
    ) -> Self {
        Self {
            config_manager,
            device_manager,
        }
    }

    /// Initialize the GUI
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing GUI");
        // Will be implemented with Tauri integration
        Ok(())
    }

    /// Show the main window
    pub fn show_window(&mut self) -> Result<()> {
        info!("Showing main window");
        // Will be implemented with Tauri integration
        Ok(())
    }

    /// Hide the main window
    pub fn hide_window(&mut self) -> Result<()> {
        info!("Hiding main window");
        // Will be implemented with Tauri integration
        Ok(())
    }

    /// Run the GUI message loop
    pub fn run(&mut self) -> Result<()> {
        info!("Running GUI message loop");
        // Will be implemented with Tauri integration
        Ok(())
    }

    /// Clean up GUI resources
    pub fn cleanup(&mut self) -> Result<()> {
        info!("Cleaning up GUI resources");
        // Will be implemented with Tauri integration
        Ok(())
    }
}

// Will be expanded with Tauri-specific utilities
// e.g., state management, IPC helpers, etc. 
