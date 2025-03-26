use anyhow::Result;
use log::{error, info};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{plugin::Plugin, Invoke, Runtime, State};
use serde::Serialize;

use bestme::audio::voice_commands::{
    VoiceCommandManager, 
    VoiceCommandEvent, 
    VoiceCommand, 
    VoiceCommandType, 
    VoiceCommandConfig
};

use crate::plugin::TranscribeState;

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
}

impl VoiceCommandState {
    /// Create a new voice command state
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(None)),
            transcribe_state: None,
            is_enabled: Arc::new(Mutex::new(false)),
            last_command: Arc::new(Mutex::new(None)),
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
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    VoiceCommandEvent::CommandDetected(command) => {
                        info!("Voice command detected: {:?}", command);
                        
                        // Update last command
                        let mut last = last_command.lock();
                        *last = Some(command);
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
            manager.process_transcription(text)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get the last detected command
    pub fn get_last_command(&self) -> Option<VoiceCommand> {
        let last_command = self.last_command.lock();
        last_command.clone()
    }
    
    /// Clear the last command
    pub fn clear_last_command(&self) {
        let mut last_command = self.last_command.lock();
        *last_command = None;
    }
    
    /// Check if voice command detection is enabled
    pub fn is_enabled(&self) -> bool {
        *self.is_enabled.lock()
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
                is_voice_commands_enabled
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

/// Structure for returning command data to the frontend
#[derive(Serialize)]
struct CommandData {
    command_type: String,
    trigger_text: String,
    parameters: Option<String>,
}

impl From<VoiceCommand> for CommandData {
    fn from(cmd: VoiceCommand) -> Self {
        let command_type = match cmd.command_type {
            VoiceCommandType::Delete => "delete",
            VoiceCommandType::Undo => "undo",
            VoiceCommandType::Redo => "redo",
            VoiceCommandType::Capitalize => "capitalize",
            VoiceCommandType::Lowercase => "lowercase",
            VoiceCommandType::NewLine => "newline",
            VoiceCommandType::NewParagraph => "newparagraph",
            VoiceCommandType::Period => "period",
            VoiceCommandType::Comma => "comma",
            VoiceCommandType::QuestionMark => "questionmark",
            VoiceCommandType::ExclamationMark => "exclamationmark",
            VoiceCommandType::Pause => "pause",
            VoiceCommandType::Resume => "resume",
            VoiceCommandType::Stop => "stop",
            VoiceCommandType::Custom(ref s) => return Self {
                command_type: "custom".to_string(),
                trigger_text: cmd.trigger_text,
                parameters: Some(s.clone()),
            },
        }.to_string();
        
        Self {
            command_type,
            trigger_text: cmd.trigger_text,
            parameters: cmd.parameters,
        }
    }
}

// Tauri command handlers

#[tauri::command]
async fn start_voice_commands(
    state: State<'_, VoiceCommandState>
) -> Result<(), String> {
    state.inner().start()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_voice_commands(
    state: State<'_, VoiceCommandState>
) -> Result<(), String> {
    state.inner().stop()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_last_command(
    state: State<'_, VoiceCommandState>
) -> Option<CommandData> {
    state.inner().get_last_command().map(CommandData::from)
}

#[tauri::command]
fn clear_last_command(
    state: State<'_, VoiceCommandState>
) {
    state.inner().clear_last_command();
}

#[tauri::command]
fn is_voice_commands_enabled(
    state: State<'_, VoiceCommandState>
) -> bool {
    state.inner().is_enabled()
} 
