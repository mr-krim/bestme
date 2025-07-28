//! Text injection plugin for Tauri

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{
    plugin::{Builder, Plugin},
    AppHandle, Manager, Runtime, State,
};
use tokio::sync::Mutex;

// Conditionally import text injection types
use bestme::text_injection::{
    InjectionManager,
};

// Import types that we need regardless of feature
use bestme::text_injection::{
    InjectionConfig, InjectionMode, PermissionStatus, WindowInfo,
};

/// Text injection state for the Tauri plugin
pub struct TextInjectionState {
        manager: Option<Arc<Mutex<InjectionManager>>>,
        config: Arc<Mutex<InjectionConfig>>,
    enabled: Arc<Mutex<bool>>,
}

impl TextInjectionState {
    pub fn new() -> Self {
        Self {
                        manager: None,
                        config: Arc::new(Mutex::new(InjectionConfig::default())),
            enabled: Arc::new(Mutex::new(false)),
        }
    }
    
        pub async fn initialize(&mut self) -> Result<()> {
        let config = self.config.lock().await.clone();
        match InjectionManager::new(config) {
            Ok(manager) => {
                self.manager = Some(Arc::new(Mutex::new(manager)));
                *self.enabled.lock().await = true;
                info!("Text injection system initialized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize text injection: {}", e);
                Err(e)
            }
        }
    }
    
        pub async fn get_manager(&self) -> Option<Arc<Mutex<InjectionManager>>> {
        self.manager.clone()
    }
}

/// Enable or disable text injection
#[tauri::command]
pub async fn enable_text_injection(
    enabled: bool,
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<(), String> {
    let mut state = _state.lock().await;
    
    if enabled && state.manager.is_none() {
        // Initialize if not already initialized
        state.initialize().await.map_err(|e| e.to_string())?;
    }
    
    *state.enabled.lock().await = enabled;
    info!("Text injection {}", if enabled { "enabled" } else { "disabled" });
    Ok(())
}

/// Inject text into the active application
#[tauri::command]
pub async fn inject_text(
    text: String,
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<(), String> {
    let state = _state.lock().await;
    
    if !*state.enabled.lock().await {
        return Err("Text injection is disabled".to_string());
    }
    
    if let Some(manager) = state.get_manager().await {
        let manager = manager.lock().await;
        manager.inject_text(&text).await.map_err(|e| e.to_string())?;
        debug!("Injected text: {} chars", text.len());
        Ok(())
    } else {
        Err("Text injection not initialized".to_string())
    }
}

/// Get the current active window information
#[tauri::command]
pub async fn get_active_window(
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<WindowInfo, String> {
        {
        let state = _state.lock().await;
        
        if let Some(manager) = state.get_manager().await {
            let manager = manager.lock().await;
            
            // For now, return a placeholder since we need access to the injector
            Ok(WindowInfo {
                title: "Active Window".to_string(),
                app_name: "Unknown".to_string(),
                app_path: None,
                pid: None,
                is_elevated: false,
                is_focused: true,
            })
        } else {
            Err("Text injection not initialized".to_string())
        }
    }
    
    #[cfg(not(feature = "text-injection"))]
    {
        Err("Text injection feature not enabled".to_string())
    }
}

/// Check text injection permissions
#[tauri::command]
pub async fn check_injection_permissions(
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<PermissionStatus, String> {
        {
        let state = _state.lock().await;
        
        if let Some(manager) = state.get_manager().await {
            let manager = manager.lock().await;
            Ok(manager.check_permissions())
        } else {
            // Return status for uninitialized state
            Ok(PermissionStatus {
                available: false,
                accessibility_granted: None,
                has_required_privileges: false,
                message: "Text injection not initialized".to_string(),
                required_actions: vec!["Initialize text injection first".to_string()],
            })
        }
    }
    
    #[cfg(not(feature = "text-injection"))]
    {
        Ok(PermissionStatus {
            available: false,
            accessibility_granted: None,
            has_required_privileges: false,
            message: "Text injection feature not enabled in build".to_string(),
            required_actions: vec!["Rebuild with text-injection feature enabled".to_string()],
        })
    }
}

/// Request necessary permissions for text injection
#[tauri::command]
pub async fn request_injection_permissions(
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<(), String> {
        {
        let state = _state.lock().await;
        
        if let Some(manager) = state.get_manager().await {
            let manager = manager.lock().await;
            manager.request_permissions().await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Text injection not initialized".to_string())
        }
    }
    
    #[cfg(not(feature = "text-injection"))]
    {
        Err("Text injection feature not enabled".to_string())
    }
}

/// Update text injection configuration
#[tauri::command]
pub async fn update_injection_config(
    config: InjectionConfig,
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<(), String> {
        {
        let mut state = _state.lock().await;
        *state.config.lock().await = config.clone();
        
        // Reinitialize with new config if already initialized
        if state.manager.is_some() {
            state.initialize().await.map_err(|e| e.to_string())?;
        }
        
        info!("Text injection configuration updated");
        Ok(())
    }
    
    #[cfg(not(feature = "text-injection"))]
    {
        Err("Text injection feature not enabled".to_string())
    }
}

/// Set injection mode
#[tauri::command]
pub async fn set_injection_mode(
    mode: InjectionMode,
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<(), String> {
        {
        let state = _state.lock().await;
        let mut config = state.config.lock().await;
        config.default_mode = mode;
        
        info!("Injection mode updated");
        Ok(())
    }
    
    #[cfg(not(feature = "text-injection"))]
    {
        Err("Text injection feature not enabled".to_string())
    }
}

/// Text injection plugin for Tauri 2.0
#[derive(Default)]
pub struct TextInjectionPlugin<R: Runtime> {
    _phantom: std::marker::PhantomData<fn() -> R>,
}

impl<R: Runtime> TextInjectionPlugin<R> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R: Runtime> Plugin<R> for TextInjectionPlugin<R> {
    fn name(&self) -> &'static str {
        "text-injection"
    }
    
    fn initialize(&mut self, app: &AppHandle<R>, _: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let injection_state = Arc::new(Mutex::new(TextInjectionState::new()));
        
        // Initialize in background
        let state_clone = injection_state.clone();
        tauri::async_runtime::spawn(async move {
                        {
                let mut state = state_clone.lock().await;
                if let Err(e) = state.initialize().await {
                    error!("Failed to initialize text injection plugin: {}", e);
                } else {
                    info!("Text injection plugin initialized successfully");
                }
            }
            
            #[cfg(not(feature = "text-injection"))]
            {
                warn!("Text injection feature not enabled");
            }
        });
        
        app.manage(injection_state);
        
        Ok(())
    }
    
}

// Re-export the types needed by the UI
pub use bestme::text_injection::{InjectionConfig as TextInjectionConfig, InjectionMode as TextInjectionMode};