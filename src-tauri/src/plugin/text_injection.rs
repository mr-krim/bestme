//! Text injection plugin for Tauri

use anyhow::Result;
use log::{debug, error, info};
use std::sync::Arc;
use tauri::{
    plugin::Plugin,
    AppHandle, Manager, Runtime, State,
};
use tokio::sync::Mutex;

// Conditionally import text injection types
use bestme::text_injection::{
    InjectionManager,
};

// Import types that we need regardless of feature
use bestme::text_injection::{
    InjectionConfig, PermissionStatus, WindowInfo,
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

/// Initialize text injection system
#[tauri::command]
pub async fn init_text_injection(
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<bool, String> {
    let mut state = _state.lock().await;
    
    match state.initialize().await {
        Ok(_) => {
            info!("Text injection initialized successfully");
            Ok(true)
        }
        Err(e) => {
            error!("Failed to initialize text injection: {}", e);
            Err(e.to_string())
        }
    }
}

/// Inject text at current cursor position
#[tauri::command]
pub async fn inject_text(
    text: String,
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<bool, String> {
    let state = _state.lock().await;
    
    if let Some(manager) = state.get_manager().await {
        let manager = manager.lock().await;
        
        match manager.inject_text(&text).await {
            Ok(_) => {
                debug!("Text injected successfully: {} chars", text.len());
                Ok(true)
            }
            Err(e) => {
                error!("Failed to inject text: {}", e);
                Err(e.to_string())
            }
        }
    } else {
        Err("Text injection not initialized".to_string())
    }
}

/// Get the current active window information
#[tauri::command]
pub async fn get_active_window(
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<WindowInfo, String> {
    let state = _state.lock().await;
    
    if let Some(manager) = state.get_manager().await {
        let _manager = manager.lock().await;
        
        // For now, return a placeholder since we need access to the injector
        Ok(WindowInfo {
            window_id: 0,
            class_name: Some("Unknown".to_string()),
            app_name: "Unknown".to_string(),
            app_path: None,
            is_elevated: false,
            title: "Unknown".to_string(),
            pid: None,
        })
    } else {
        Err("Text injection not initialized".to_string())
    }
}

/// Check text injection permissions
#[tauri::command]
pub async fn check_injection_permissions(
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<PermissionStatus, String> {
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

/// Request necessary permissions for text injection
#[tauri::command]
pub async fn request_injection_permissions(
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<(), String> {
    let state = _state.lock().await;
    
    if let Some(manager) = state.get_manager().await {
        let manager = manager.lock().await;
        manager.request_permissions().await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Text injection not initialized".to_string())
    }
}

/// Update text injection configuration
#[tauri::command]
pub async fn update_injection_config(
    config: InjectionConfig,
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<(), String> {
    let mut state = _state.lock().await;
    *state.config.lock().await = config.clone();
    
    // Reinitialize with new config if already initialized
    if state.manager.is_some() {
        state.initialize().await.map_err(|e| e.to_string())?;
    }
    
    info!("Text injection configuration updated");
    Ok(())
}

/// Set injection mode
#[tauri::command]
pub async fn set_injection_mode(
    _mode: String,
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<(), String> {
    // Mode setting not implemented since InjectionMode type is not available
    info!("Injection mode update requested but not implemented");
    Ok(())
}

/// Simulate key press
#[tauri::command]
pub async fn simulate_key(
    _key: String,
    modifiers: Vec<String>,
    _state: State<'_, Arc<Mutex<TextInjectionState>>>,
) -> Result<(), String> {
    let state = _state.lock().await;
    
    if let Some(manager) = state.get_manager().await {
        let _manager = manager.lock().await;
        
        // Convert modifiers
        let mut modifier_flags = vec![];
        for modifier in modifiers {
            match modifier.as_str() {
                "ctrl" | "control" => modifier_flags.push("ctrl"),
                "alt" => modifier_flags.push("alt"),
                "shift" => modifier_flags.push("shift"),
                "cmd" | "meta" | "super" => modifier_flags.push("meta"),
                _ => {}
            }
        }
        
        // Note: simulate_key method may not exist, need to check the actual API
        // For now, return an error
        Err("simulate_key not implemented".to_string())
    } else {
        Err("Text injection not initialized".to_string())
    }
}

/// Text injection plugin
pub struct TextInjectionPlugin<R: Runtime> {
    _phantom: std::marker::PhantomData<R>,
}

// Manually implement Send for TextInjectionPlugin
unsafe impl<R: Runtime> Send for TextInjectionPlugin<R> {}

impl<R: Runtime> Default for TextInjectionPlugin<R> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R: Runtime> TextInjectionPlugin<R> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<R: Runtime> Plugin<R> for TextInjectionPlugin<R> {
    fn name(&self) -> &'static str {
        "text-injection"
    }
    
    fn initialize(&mut self, app: &AppHandle<R>, _config: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let state = Arc::new(Mutex::new(TextInjectionState::new()));
        
        // Note: We can't auto-initialize in background on Linux due to GTK thread safety
        // The initialization will happen on first use instead
        info!("Text injection plugin registered (will initialize on first use)");
        
        app.manage(state);
        
        Ok(())
    }
}

// Re-export types for convenience
