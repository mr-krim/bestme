//! Context detection for active window and application information

use anyhow::Result;
use serde::{Deserialize, Serialize};

mod context_simple;

/// Information about the currently active window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    /// Window title
    pub title: String,
    /// Application name (e.g., "Chrome", "Terminal")
    pub app_name: String,
    /// Full path to application executable
    pub app_path: Option<String>,
    /// Process ID
    pub pid: Option<u32>,
    /// Whether the process is running with elevated privileges
    pub is_elevated: bool,
    /// Platform-specific window identifier
    pub window_id: usize,
    /// Window class name (if available)
    pub class_name: Option<String>,
}

/// Context detector for gathering information about active windows
pub struct ContextDetector;

impl ContextDetector {
    /// Create a new context detector
    pub fn new() -> Self {
        Self
    }
    
    /// Get information about the currently active window
    pub async fn get_active_window(&self) -> Result<WindowInfo> {
        context_simple::get_active_window().await
    }
    
    /// Check if a specific application is active
    pub async fn is_app_active(&self, app_name: &str) -> Result<bool> {
        let window = self.get_active_window().await?;
        Ok(window.app_name.eq_ignore_ascii_case(app_name))
    }
}