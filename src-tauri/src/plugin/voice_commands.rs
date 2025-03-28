use anyhow::{Result, anyhow};
use log::{info, error, debug, warn};
use parking_lot::Mutex;
use std::{path::PathBuf, sync::Arc, collections::HashMap};
use tauri::{AppHandle, Manager, State};
use tokio::sync::mpsc;
use serde::Serialize;
use std::collections::VecDeque;
use regex;
use chrono;
use std::marker::PhantomData;

use bestme::audio::voice_commands::{
    Command,
    CommandContext,
    CommandEvent,
    CommandHistory,
    CommandResult,
    CommandTrigger,
    VoiceCommandConfig,
    VoiceCommandEvent,
    VoiceCommandManager as TauriVoiceCommandManager,
    TextEditOperation,
    DeleteScope,
    FormatOperation,
    TextStyle,
    TextOperationHistory,
};

use crate::plugin::TranscribeState;

/// Maximum number of commands to keep in history
const MAX_COMMAND_HISTORY: usize = 20;

/// Configuration for voice commands
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TauriVoiceCommandConfig {
    /// Whether voice commands are enabled
    pub enabled: bool,
    /// The prefix that must be spoken before commands
    pub command_prefix: Option<String>,
    /// Whether the prefix is required
    pub require_prefix: bool,
    /// Confidence threshold for command detection (0.0-1.0)
    pub sensitivity: f32,
    /// Custom command mappings
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_commands: Vec<(String, String)>,
    /// Whether this is the default configuration
    pub default_commands: bool,
}

impl Default for TauriVoiceCommandConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            command_prefix: Some("computer".to_string()),
            require_prefix: true,
            sensitivity: 0.7,
            custom_commands: Vec::new(),
            default_commands: true,
        }
    }
}

/// Data structure for a detected command
#[derive(Debug, Clone, Serialize)]
pub struct CommandData {
    /// Type of command detected
    pub command_type: String,
    /// The text that triggered the command
    pub trigger_text: String,
    /// When the command was detected
    pub timestamp: String,
}

impl From<Command> for CommandData {
    fn from(cmd: Command) -> Self {
        Self {
            command_type: format!("{:?}", cmd.command_type),
            trigger_text: cmd.trigger_text,
            timestamp: chrono::Local::now().to_rfc3339(),
        }
    }
}

/// Text operation data for the frontend
#[derive(Debug, Clone, Serialize)]
pub struct TextEditData {
    /// Type of operation
    pub operation_type: String,
    /// Text before the operation
    pub previous_text: String,
    /// Text after the operation
    pub current_text: String,
    /// When the operation occurred
    pub timestamp: String,
}

impl From<TextOperationHistory> for TextEditData {
    fn from(hist: TextOperationHistory) -> Self {
        Self {
            operation_type: format!("{:?}", hist.operation),
            previous_text: hist.previous_text,
            current_text: hist.current_text,
            timestamp: hist.timestamp.to_rfc3339(),
        }
    }
}

/// Structure to hold voice command state
pub struct VoiceCommandState {
    /// Voice command manager
    manager: Arc<Mutex<Option<TauriVoiceCommandManager>>>,
    
    /// Whether the system is enabled
    is_enabled: Arc<Mutex<bool>>,
    
    /// Last detected command
    last_command: Arc<Mutex<Option<Command>>>,
    
    /// Command history (most recent first)
    command_history: Arc<Mutex<VecDeque<CommandData>>>,
    
    /// Current text being edited
    current_text: Arc<Mutex<String>>,
    
    /// App handle for Tauri 2.0
    app_handle: Option<AppHandle>,
}

impl VoiceCommandState {
    /// Create a new voice command state
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(None)),
            is_enabled: Arc::new(Mutex::new(false)),
            last_command: Arc::new(Mutex::new(None)),
            command_history: Arc::new(Mutex::with_capacity(MAX_COMMAND_HISTORY)),
            current_text: Arc::new(Mutex::new(String::new())),
            app_handle: None,
        }
    }
    
    /// Set the app handle for Tauri 2.0
    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }
    
    /// Initialize voice command manager
    pub fn initialize(&mut self, config: TauriVoiceCommandConfig) -> Result<()> {
        let (manager, receiver) = TauriVoiceCommandManager::new(config)?;
        
        // Set up event handling for voice commands
        let commands_history = Arc::clone(&self.command_history);
        let last_command = Arc::clone(&self.last_command);
        let is_enabled = Arc::clone(&self.is_enabled);
        let app_handle = self.app_handle.clone();
        
        // Start processing voice command events
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    VoiceCommandEvent::CommandDetected(cmd) => {
                        // Store the last command
                        {
                            let mut last = last_command.lock();
                            *last = Some(cmd.clone());
                        }
                        
                        // Add to history
                        {
                            let mut history = commands_history.lock();
                            history.push_front(CommandData::from(cmd.clone()));
                            
                            // Limit history size
                            while history.len() > 50 {
                                history.pop_back();
                            }
                        }
                        
                        // Emit event to frontend
                        if let Some(handle) = &app_handle {
                            let command_data = CommandData::from(cmd);
                            if let Err(e) = handle.emit_all("voice-command:detected", command_data) {
                                error!("Failed to emit voice command event: {}", e);
                            }
                        }
                    },
                    VoiceCommandEvent::Error(err) => {
                        error!("Voice command error: {}", err);
                        
                        // Emit error event
                        if let Some(handle) = &app_handle {
                            if let Err(e) = handle.emit_all("voice-command:error", err.to_string()) {
                                error!("Failed to emit voice command error event: {}", e);
                            }
                        }
                    },
                    VoiceCommandEvent::Stopped => {
                        let mut enabled = is_enabled.lock();
                        *enabled = false;
                        
                        // Emit stopped event
                        if let Some(handle) = &app_handle {
                            if let Err(e) = handle.emit_all("voice-command:stopped", ()) {
                                error!("Failed to emit voice command stopped event: {}", e);
                            }
                        }
                    },
                }
            }
        });
        
        // Store the manager
        {
            let mut mgr = self.manager.lock();
            *mgr = Some(manager);
        }
        
        Ok(())
    }
    
    /// Start voice command processing
    pub fn start(&mut self) -> Result<()> {
        let config = self.create_default_config();
        self.initialize(config)?;
        
        {
            let mut enabled = self.is_enabled.lock();
            *enabled = true;
        }
        
        {
            let mut manager = self.manager.lock();
            if let Some(manager) = manager.as_mut() {
                manager.start()?;
            }
        }
        
        Ok(())
    }
    
    /// Stop voice command processing
    pub fn stop(&mut self) -> Result<()> {
        let mut manager_lock = self.manager.lock();
        
        if let Some(manager) = manager_lock.as_mut() {
            manager.stop()?;
            
            let mut enabled = self.is_enabled.lock();
            *enabled = false;
            
            Ok(())
        } else {
            // Not an error if already stopped
            Ok(())
        }
    }
    
    /// Process transcription text for voice commands
    pub fn process_transcription(&self, text: &str) -> Result<Vec<Command>> {
        if !*self.is_enabled.lock() {
            return Ok(Vec::new());
        }
        
        let manager = self.manager.lock();
        if let Some(manager) = manager.as_ref() {
            match manager.process_transcription(text) {
                Ok(commands) => Ok(commands),
                Err(_) => Ok(Vec::new())
            }
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Enable voice commands with async interface for Tauri 2.0
    pub async fn enable(&mut self) -> Result<(), String> {
        self.start().map_err(|e| e.to_string())
    }
    
    /// Disable voice commands with async interface for Tauri 2.0
    pub async fn disable(&mut self) -> Result<(), String> {
        self.stop().map_err(|e| e.to_string())
    }
    
    // Common methods that are the same in both versions
    pub fn get_last_command_data(&self) -> Option<CommandData> {
        let last_command = self.last_command.lock();
        last_command.clone().map(CommandData::from)
    }
    
    pub fn clear_last_command(&self) {
        let mut last_command = self.last_command.lock();
        *last_command = None;
    }
    
    pub fn get_command_history(&self) -> Vec<CommandData> {
        let history = self.command_history.lock();
        history.iter().cloned().collect()
    }
    
    pub fn clear_command_history(&self) {
        let mut history = self.command_history.lock();
        history.clear();
    }
    
    pub fn is_enabled(&self) -> bool {
        *self.is_enabled.lock()
    }
    
    pub fn create_default_config(&self) -> TauriVoiceCommandConfig {
        TauriVoiceCommandConfig {
            enabled: true,
            command_prefix: Some("computer".to_string()),
            require_prefix: true,
            sensitivity: 0.5,
            custom_commands: Vec::new(),
            default_commands: true,
        }
    }
    
    pub fn update_text(&self, text: &str) -> Result<(), String> {
        // Update our internal text buffer
        {
            let mut current_text = self.current_text.lock();
            *current_text = text.to_string();
        }
        
        // If we have an active manager, update its text as well
        let manager = self.manager.lock();
        if let Some(manager) = manager.as_ref() {
            manager.set_current_text(text);
        }
        
        Ok(())
    }
    
    pub fn get_text(&self) -> String {
        let manager = self.manager.lock();
        if let Some(manager) = manager.as_ref() {
            manager.get_current_text()
        } else {
            self.current_text.lock().clone()
        }
    }
    
    /// Helper to convert from TauriVoiceCommandConfig to VoiceCommandConfig
    pub fn convert_config(&self, tauri_config: TauriVoiceCommandConfig) -> VoiceCommandConfig {
        let mut config = VoiceCommandConfig::default();
        
        config.enabled = tauri_config.enabled;
        config.command_prefix = tauri_config.command_prefix;
        config.require_prefix = tauri_config.require_prefix;
        config.sensitivity = tauri_config.sensitivity;
        
        // Map custom commands
        let mut custom_commands = HashMap::new();
        for (phrase, action) in tauri_config.custom_commands {
            custom_commands.insert(phrase, action);
        }
        config.custom_commands = custom_commands;
        
        config
    }
    
    /// Apply a delete operation to the current text
    pub fn apply_delete(&self, scope_name: &str) -> Result<String, String> {
        let manager = self.manager.lock();
        
        if let Some(manager) = manager.as_ref() {
            // Match the scope name to a DeleteScope
            let scope = match scope_name.to_lowercase().as_str() {
                "word" => DeleteScope::LastWord,
                "sentence" => DeleteScope::LastSentence,
                "paragraph" => DeleteScope::LastParagraph,
                "all" => DeleteScope::All,
                _ => return Err(format!("Unknown delete scope: {}", scope_name)),
            };
            
            // Apply the delete operation
            let operation = TextEditOperation::Delete(scope);
            match manager.apply_text_operation(operation) {
                Ok(text) => Ok(text),
                Err(e) => Err(format!("Failed to apply delete operation: {}", e)),
            }
        } else {
            Err("Voice command manager not initialized".to_string())
        }
    }
    
    /// Undo the last text operation
    pub fn undo(&self) -> Result<String, String> {
        let manager = self.manager.lock();
        
        if let Some(manager) = manager.as_ref() {
            match manager.undo_last_operation() {
                Ok(text) => Ok(text),
                Err(e) => Err(format!("Failed to undo operation: {}", e)),
            }
        } else {
            Err("Voice command manager not initialized".to_string())
        }
    }
    
    /// Redo the last undone text operation
    pub fn redo(&self) -> Result<String, String> {
        let manager = self.manager.lock();
        
        if let Some(manager) = manager.as_ref() {
            match manager.redo_last_operation() {
                Ok(text) => Ok(text),
                Err(e) => Err(format!("Failed to redo operation: {}", e)),
            }
        } else {
            Err("Voice command manager not initialized".to_string())
        }
    }
}

/// Voice command plugin for Tauri 2.0
#[derive(Default)]
pub struct VoiceCommandPlugin {
    _phantom: PhantomData<()>,
}

impl VoiceCommandPlugin {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl tauri::Plugin for VoiceCommandPlugin {
    fn name(&self) -> &'static str {
        "voice_commands"
    }
    
    fn initialize(&mut self, app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing voice command plugin");
        
        // Register commands
        app.plugin(
            tauri::plugin::Builder::new("voice_commands")
                .js_init_script(include_str!("./voice_commands_init.js"))
                .setup(|app, _| {
                    app.listen("voice-command:start", |event| {
                        info!("Voice command start requested from frontend");
                        
                        let app_handle = event.window().app_handle();
                        let state = app_handle.state::<Arc<Mutex<VoiceCommandState>>>();
                        let mut voice_state = state.lock();
                        
                        tokio::spawn(async move {
                            if let Err(e) = voice_state.enable().await {
                                error!("Failed to start voice commands: {}", e);
                                if let Err(emit_err) = app_handle.emit_all("voice-command:error", e) {
                                    error!("Failed to emit error event: {}", emit_err);
                                }
                            } else {
                                info!("Voice commands started successfully");
                                if let Err(emit_err) = app_handle.emit_all("voice-command:started", ()) {
                                    error!("Failed to emit started event: {}", emit_err);
                                }
                            }
                        });
                    });
                    
                    app.listen("voice-command:stop", |event| {
                        info!("Voice command stop requested from frontend");
                        
                        let app_handle = event.window().app_handle();
                        let state = app_handle.state::<Arc<Mutex<VoiceCommandState>>>();
                        let mut voice_state = state.lock();
                        
                        tokio::spawn(async move {
                            if let Err(e) = voice_state.disable().await {
                                error!("Failed to stop voice commands: {}", e);
                                if let Err(emit_err) = app_handle.emit_all("voice-command:error", e) {
                                    error!("Failed to emit error event: {}", emit_err);
                                }
                            } else {
                                info!("Voice commands stopped successfully");
                                if let Err(emit_err) = app_handle.emit_all("voice-command:stopped", ()) {
                                    error!("Failed to emit stopped event: {}", emit_err);
                                }
                            }
                        });
                    });
                    
                    app.listen("voice-command:update-text", move |event| {
                        if let Some(payload) = event.payload() {
                            let app_handle = event.window().app_handle();
                            let state = app_handle.state::<Arc<Mutex<VoiceCommandState>>>();
                            let voice_state = state.lock();
                            
                            if let Ok(text) = serde_json::from_str::<String>(payload) {
                                if let Err(e) = voice_state.update_text(&text) {
                                    error!("Failed to update text: {}", e);
                                }
                            }
                        }
                    });
                    
                    Ok(())
                })
                .build()
        )?;
        
        Ok(())
    }
}

// Add Tauri 2.0 command handlers
#[tauri::command]
pub async fn get_voice_commands_status(state: State<'_, Arc<Mutex<VoiceCommandState>>>) -> Result<bool, String> {
    let voice_state = state.lock();
    Ok(voice_state.is_enabled())
}

#[tauri::command]
pub async fn get_voice_commands_text(state: State<'_, Arc<Mutex<VoiceCommandState>>>) -> Result<String, String> {
    let voice_state = state.lock();
    Ok(voice_state.get_text())
}

#[tauri::command]
pub async fn get_voice_commands_history(state: State<'_, Arc<Mutex<VoiceCommandState>>>) -> Result<Vec<CommandData>, String> {
    let voice_state = state.lock();
    Ok(voice_state.get_command_history())
}

#[tauri::command]
pub async fn update_text(text: String, state: State<'_, Arc<Mutex<VoiceCommandState>>>) -> Result<(), String> {
    let voice_state = state.lock();
    voice_state.update_text(&text)
}

#[tauri::command]
pub async fn apply_delete_operation(scope: String, state: State<'_, Arc<Mutex<VoiceCommandState>>>) -> Result<String, String> {
    let voice_state = state.lock();
    voice_state.apply_delete(&scope)
}

#[tauri::command]
pub async fn undo_operation(state: State<'_, Arc<Mutex<VoiceCommandState>>>) -> Result<String, String> {
    let voice_state = state.lock();
    voice_state.undo()
}

#[tauri::command]
pub async fn redo_operation(state: State<'_, Arc<Mutex<VoiceCommandState>>>) -> Result<String, String> {
    let voice_state = state.lock();
    voice_state.redo()
} 
