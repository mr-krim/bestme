pub mod icons;
pub mod settings;
pub mod tray;
pub mod window;

use anyhow::{Context, Result};
use log::{error, info};
use parking_lot::Mutex;
use std::sync::Arc;

use crate::audio::device::DeviceManager;
use crate::config::ConfigManager;

/// Handles the GUI components of the application
pub struct GuiManager {
    config_manager: Arc<Mutex<ConfigManager>>,
    device_manager: Arc<Mutex<DeviceManager>>,
    // Will be replaced with Tauri app instance
    // For now, keep Windows-specific window and tray
    window: Option<window::MainWindow>,
    tray: Option<tray::SystemTray>,
}

impl GuiManager {
    /// Creates a new GUI manager
    pub fn new(
        config_manager: Arc<Mutex<ConfigManager>>,
        device_manager: Arc<Mutex<DeviceManager>>,
    ) -> Self {
        Self {
            config_manager,
            device_manager,
            window: None,
            tray: None,
        }
    }

    /// Initializes the GUI components
    ///
    /// # Note
    /// This will be significantly refactored when integrating Tauri.
    /// Current implementation uses Windows-specific code.
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing GUI components");

        // Create and initialize the system tray
        let tray = tray::SystemTray::new(
            self.config_manager.clone(),
            self.device_manager.clone(),
        )?;
        self.tray = Some(tray);

        // Create the main window (will be replaced with Tauri window)
        let window = window::MainWindow::new(
            self.config_manager.clone(),
            self.device_manager.clone(),
        )?;
        self.window = Some(window);

        Ok(())
    }

    /// Runs the GUI message loop
    ///
    /// # Note
    /// This will be replaced with Tauri's event loop when integrated
    pub fn run(&mut self) -> Result<()> {
        info!("Starting GUI message loop");

        // TODO: Replace with Tauri event loop
        // For now, we'll use the Windows message loop
        if let Some(window) = &mut self.window {
            window.show()?;
            window.run_message_loop()?;
        } else {
            error!("Window not initialized");
            return Err(anyhow::anyhow!("Window not initialized"));
        }

        info!("GUI message loop ended");
        Ok(())
    }

    /// Shuts down the GUI components
    pub fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down GUI components");

        // Hide and destroy the window
        if let Some(window) = &mut self.window {
            window.hide()?;
        }
        self.window = None;

        // Remove the system tray icon
        if let Some(tray) = &mut self.tray {
            tray.remove()?;
        }
        self.tray = None;

        Ok(())
    }

    // Additional methods will be added to interact with Tauri components
    // when the integration is implemented
}

// Will be expanded with Tauri-specific utilities
// e.g., state management, IPC helpers, etc. 
