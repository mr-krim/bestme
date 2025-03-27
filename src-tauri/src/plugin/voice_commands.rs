use anyhow::Result;
use log::{error, info, debug};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{plugin::Plugin, Invoke, Runtime, State};
use serde::Serialize;
use std::collections::VecDeque;
use std::collections::HashMap;
use regex;
use chrono;

use bestme::audio::voice_commands::{
    VoiceCommandManager, 
    VoiceCommandEvent, 
    VoiceCommand, 
    VoiceCommandType, 
    VoiceCommandConfig
};

use crate::plugin::TranscribeState;

/// Maximum number of commands to keep in history
const MAX_COMMAND_HISTORY: usize = 20;

/// Configuration for voice commands
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceCommandConfig {
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
}

impl Default for VoiceCommandConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            command_prefix: Some("computer".to_string()),
            require_prefix: true,
            sensitivity: 0.7,
            custom_commands: Vec::new(),
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

impl From<VoiceCommand> for CommandData {
    fn from(cmd: VoiceCommand) -> Self {
        Self {
            command_type: format!("{:?}", cmd.command_type),
            trigger_text: cmd.trigger_text,
            timestamp: chrono::Local::now().to_rfc3339(),
        }
    }
}

/// Structure to hold voice command state
pub struct VoiceCommandState {
    /// Voice command manager
    manager: Arc<Mutex<Option<VoiceCommandManager>>>,
    
    /// Transcribe state for integration
    transcribe_state: Option<Arc<TranscribeState>>,
    
    /// Whether the system is enabled
    is_enabled: Arc<Mutex<bool>>,
    
    /// Last detected command
    last_command: Arc<Mutex<Option<VoiceCommand>>>,
    
    /// Command history (most recent first)
    command_history: Arc<Mutex<VecDeque<CommandData>>>,
}

impl VoiceCommandState {
    /// Create a new voice command state
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(None)),
            transcribe_state: None,
            is_enabled: Arc::new(Mutex::new(false)),
            last_command: Arc::new(Mutex::new(None)),
            command_history: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_COMMAND_HISTORY))),
        }
    }
    
    /// Set the transcribe state for integration
    pub fn set_transcribe_state(&mut self, transcribe_state: Arc<TranscribeState>) {
        self.transcribe_state = Some(transcribe_state);
    }
    
    /// Initialize voice command manager
    pub fn initialize(&mut self, config: VoiceCommandConfig) -> Result<()> {
        let (manager, receiver) = VoiceCommandManager::new(config)?;
        
        // Set up event handling for voice commands
        let last_command = Arc::clone(&self.last_command);
        let command_history = Arc::clone(&self.command_history);
        
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    VoiceCommandEvent::CommandDetected(command) => {
                        info!("Voice command detected: {:?}", command);
                        
                        // Update last command
                        let mut last = last_command.lock();
                        *last = Some(command.clone());
                        
                        // Add to command history
                        let mut history = command_history.lock();
                        history.push_front(CommandData::from(command));
                        while history.len() > MAX_COMMAND_HISTORY {
                            history.pop_back();
                        }
                    },
                    VoiceCommandEvent::Error(err) => {
                        error!("Voice command error: {}", err);
                    }
                }
            }
        });
        
        // Store manager
        let mut manager_lock = self.manager.lock();
        *manager_lock = Some(manager);
        
        Ok(())
    }
    
    /// Start voice command detection
    pub fn start(&self) -> Result<()> {
        let mut manager_lock = self.manager.lock();
        if let Some(manager) = manager_lock.as_mut() {
            manager.start()?;
            
            let mut enabled = self.is_enabled.lock();
            *enabled = true;
            
            info!("Voice command detection started");
            Ok(())
        } else {
            error!("Voice command manager not initialized");
            Err(anyhow::anyhow!("Voice command manager not initialized"))
        }
    }
    
    /// Stop voice command detection
    pub fn stop(&self) -> Result<()> {
        let mut manager_lock = self.manager.lock();
        if let Some(manager) = manager_lock.as_mut() {
            manager.stop()?;
            
            let mut enabled = self.is_enabled.lock();
            *enabled = false;
            
            info!("Voice command detection stopped");
            Ok(())
        } else {
            error!("Voice command manager not initialized");
            Err(anyhow::anyhow!("Voice command manager not initialized"))
        }
    }
    
    /// Process transcription text for voice commands
    pub fn process_transcription(&self, text: &str) -> Result<Vec<VoiceCommand>> {
        if !*self.is_enabled.lock() {
            return Ok(Vec::new());
        }
        
        let manager_lock = self.manager.lock();
        if let Some(manager) = manager_lock.as_ref() {
            // Log the transcription we're processing
            debug!("Processing transcription for commands: {}", text);
            
            // Process the text for commands
            let commands = manager.process_transcription(text)?;
            
            // If commands were detected, log them
            if !commands.is_empty() {
                info!("Detected {} commands in transcription", commands.len());
                for cmd in &commands {
                    debug!("Command: {:?}, Trigger: {}", cmd.command_type, cmd.trigger_text);
                }
            }
            
            Ok(commands)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get the last detected command as CommandData for the frontend
    pub fn get_last_command_data(&self) -> Option<CommandData> {
        let last_command = self.last_command.lock();
        last_command.clone().map(CommandData::from)
    }
    
    /// Clear the last command
    pub fn clear_last_command(&self) {
        let mut last_command = self.last_command.lock();
        *last_command = None;
    }

    /// Get the command history
    pub fn get_command_history(&self) -> Vec<CommandData> {
        let history = self.command_history.lock();
        history.iter().cloned().collect()
    }
    
    /// Clear the command history
    pub fn clear_command_history(&self) {
        let mut history = self.command_history.lock();
        history.clear();
    }
    
    /// Check if voice command detection is enabled
    pub fn is_enabled(&self) -> bool {
        *self.is_enabled.lock()
    }

    /// Enable voice commands
    pub fn enable(&mut self) -> Result<()> {
        self.start()
    }
    
    /// Disable voice commands
    pub fn disable(&mut self) -> Result<()> {
        self.stop()
    }
}

/// Voice command plugin
pub struct VoiceCommandPlugin<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> VoiceCommandPlugin<R> {
    pub fn new() -> Self {
        Self {
            invoke_handler: Box::new(tauri::generate_handler![
                start_voice_commands,
                stop_voice_commands,
                get_last_command,
                clear_last_command,
                get_command_history,
                clear_command_history,
                get_voice_command_config,
                set_voice_command_config,
            ]),
        }
    }
}

impl<R: Runtime> Plugin<R> for VoiceCommandPlugin<R> {
    fn name(&self) -> &'static str {
        "voice_commands"
    }

    fn extend_api(&mut self, message: Invoke<R>) {
        (self.invoke_handler)(message)
    }
}

// Tauri command handlers

#[tauri::command]
fn start_voice_commands(
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Result<(), String> {
    state.inner().lock().start().map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_voice_commands(
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Result<(), String> {
    state.inner().lock().stop().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_last_command(
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Option<CommandData> {
    state.inner().lock().get_last_command_data()
}

#[tauri::command]
fn clear_last_command(
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) {
    state.inner().lock().clear_last_command();
}

#[tauri::command]
fn get_command_history(
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Vec<CommandData> {
    state.inner().lock().get_command_history()
}

#[tauri::command]
fn clear_command_history(
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) {
    state.inner().lock().clear_command_history();
}

#[tauri::command]
fn get_voice_command_config(
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Result<VoiceCommandConfig, String> {
    let state = state.inner().lock();
    let manager = state.manager.lock();
    
    if let Some(manager) = manager.as_ref() {
        Ok(manager.get_config().clone())
    } else {
        Err("Voice command manager not initialized".to_string())
    }
}

#[tauri::command]
fn set_voice_command_config(
    config: VoiceCommandConfig,
    state: tauri::State<'_, Arc<Mutex<VoiceCommandState>>>
) -> Result<(), String> {
    let mut state = state.inner().lock();
    
    // Save current enabled state
    let was_enabled = state.is_enabled();
    
    // Stop if running
    if was_enabled {
        state.stop()?;
    }
    
    // Update configuration
    state.initialize(config)
        .map_err(|e| format!("Failed to update voice command config: {}", e))?;
    
    // Restart if it was enabled
    if was_enabled {
        state.start()?;
    }
    
    Ok(())
} 
