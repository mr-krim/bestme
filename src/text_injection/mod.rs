//! Text injection system for typing text into applications
//! 
//! This module provides platform-specific implementations for simulating
//! keyboard input to type text into any application.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

// Platform-specific modules
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
mod windows_types;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

// Common modules
mod context;
pub mod keycodes;

// Re-exports
pub use context::{WindowInfo, ContextDetector};

#[cfg(target_os = "windows")]
pub use windows::WindowsInjector as PlatformInjector;
#[cfg(target_os = "macos")]
pub use macos::MacOSInjector as PlatformInjector;
#[cfg(target_os = "linux")]
pub use linux::LinuxInjector as PlatformInjector;

/// Text injection modes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum InjectionMode {
    /// Type character by character with configurable delay
    Type {
        /// Delay between keystrokes in milliseconds
        delay_ms: u32,
        /// Use shift key for capitals instead of caps lock
        use_shift_for_caps: bool,
        /// Simulate natural typing variation
        natural_typing: bool,
    },
    /// Use clipboard for faster insertion
    Paste {
        /// Restore original clipboard content after paste
        restore_clipboard: bool,
        /// Clear clipboard after paste for security
        clear_after_paste: bool,
    },
    /// Direct application API (future enhancement)
    Direct,
}

impl Default for InjectionMode {
    fn default() -> Self {
        InjectionMode::Type {
            delay_ms: 10,
            use_shift_for_caps: true,
            natural_typing: false,
        }
    }
}

/// Permission status for text injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    /// Whether basic injection is available
    pub available: bool,
    /// Whether accessibility permissions are granted (macOS)
    pub accessibility_granted: Option<bool>,
    /// Whether the app is running with required privileges
    pub has_required_privileges: bool,
    /// Human-readable status message
    pub message: String,
    /// Actions user can take to grant permissions
    pub required_actions: Vec<String>,
}

/// Application profile for customized injection behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppProfile {
    /// Application name or identifier
    pub app_name: String,
    /// Preferred injection mode for this app
    pub preferred_mode: InjectionMode,
    /// Whether this app requires special handling
    pub requires_special_handling: bool,
    /// Custom settings specific to this application
    pub custom_settings: HashMap<String, serde_json::Value>,
}

/// Text injection service trait
#[async_trait]
pub trait TextInjector: Send + Sync {
    /// Inject text using the specified mode
    async fn inject_text(&self, text: &str, mode: &InjectionMode) -> Result<()>;
    
    /// Get current active window information
    async fn get_active_window(&self) -> Result<WindowInfo>;
    
    /// Check if injection is available
    fn is_available(&self) -> bool;
    
    /// Get required permissions status
    fn check_permissions(&self) -> PermissionStatus;
    
    /// Request necessary permissions (platform-specific)
    async fn request_permissions(&self) -> Result<()>;
    
    /// Stop any ongoing injection
    async fn stop_injection(&self) -> Result<()>;
}

/// Configuration for text injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionConfig {
    /// Default injection mode
    pub default_mode: InjectionMode,
    /// Whether to auto-detect best mode based on application
    pub auto_detect_mode: bool,
    /// Application profiles
    pub app_profiles: Vec<AppProfile>,
    /// Global enable/disable
    pub enabled: bool,
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            default_mode: InjectionMode::default(),
            auto_detect_mode: true,
            app_profiles: Self::default_profiles(),
            enabled: true,
        }
    }
}

impl InjectionConfig {
    /// Get default application profiles
    fn default_profiles() -> Vec<AppProfile> {
        vec![
            AppProfile {
                app_name: "Terminal".to_string(),
                preferred_mode: InjectionMode::Type {
                    delay_ms: 5,
                    use_shift_for_caps: true,
                    natural_typing: false,
                },
                requires_special_handling: true,
                custom_settings: HashMap::new(),
            },
            AppProfile {
                app_name: "Code".to_string(),
                preferred_mode: InjectionMode::Type {
                    delay_ms: 1,
                    use_shift_for_caps: true,
                    natural_typing: false,
                },
                requires_special_handling: false,
                custom_settings: HashMap::new(),
            },
            AppProfile {
                app_name: "Chrome".to_string(),
                preferred_mode: InjectionMode::Paste {
                    restore_clipboard: true,
                    clear_after_paste: false,
                },
                requires_special_handling: false,
                custom_settings: HashMap::new(),
            },
        ]
    }
    
    /// Find profile for a given application
    pub fn find_profile(&self, app_name: &str) -> Option<&AppProfile> {
        self.app_profiles.iter()
            .find(|profile| profile.app_name.eq_ignore_ascii_case(app_name))
    }
}

/// Manager for text injection with profile support
pub struct InjectionManager {
    injector: Box<dyn TextInjector>,
    config: InjectionConfig,
    _context_detector: ContextDetector,
}

impl InjectionManager {
    /// Create a new injection manager
    pub fn new(config: InjectionConfig) -> Result<Self> {
        let injector = Box::new(PlatformInjector::new()?);
        let context_detector = ContextDetector::new();
        
        Ok(Self {
            injector,
            config,
            _context_detector: context_detector,
        })
    }
    
    /// Inject text with automatic mode selection
    pub async fn inject_text(&self, text: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Get current window context
        let window_info = self.injector.get_active_window().await?;
        
        // Determine injection mode
        let mode = if self.config.auto_detect_mode {
            self.determine_mode(&window_info)
        } else {
            &self.config.default_mode
        };
        
        // Perform injection
        self.injector.inject_text(text, mode).await
    }
    
    /// Determine best injection mode based on active window
    fn determine_mode(&self, window_info: &WindowInfo) -> &InjectionMode {
        // Check for app-specific profile
        if let Some(profile) = self.config.find_profile(&window_info.app_name) {
            return &profile.preferred_mode;
        }
        
        // Use default mode
        &self.config.default_mode
    }
    
    /// Check permissions
    pub fn check_permissions(&self) -> PermissionStatus {
        self.injector.check_permissions()
    }
    
    /// Request permissions
    pub async fn request_permissions(&self) -> Result<()> {
        self.injector.request_permissions().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_injection_mode_serialization() {
        let mode = InjectionMode::Type {
            delay_ms: 10,
            use_shift_for_caps: true,
            natural_typing: false,
        };
        
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: InjectionMode = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            InjectionMode::Type { delay_ms, .. } => assert_eq!(delay_ms, 10),
            _ => panic!("Wrong mode type"),
        }
    }
    
    #[test]
    fn test_config_default_profiles() {
        let config = InjectionConfig::default();
        assert!(!config.app_profiles.is_empty());
        
        let terminal_profile = config.find_profile("terminal");
        assert!(terminal_profile.is_some());
    }
}

// #[cfg(test)]
// mod tests;