use anyhow::{Context, Result};
use directories::ProjectDirs;
use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::audio::voice_commands::VoiceCommandConfig;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Version of the configuration
    pub version: String,
    
    /// General application settings
    pub general: GeneralSettings,
    
    /// Audio device settings
    pub audio: AudioSettings,
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    /// Theme (light or dark)
    pub theme: String,
    
    /// Auto-start with Windows
    pub auto_start: bool,
    
    /// Minimize to tray on startup
    pub minimize_to_tray: bool,
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Input device ID
    pub input_device: Option<String>,
    
    /// Input volume level (0.0 - 1.0)
    pub input_volume: f32,
    
    /// Speech recognition settings
    pub speech: SpeechSettings,
    
    /// Voice command settings
    pub voice_commands: VoiceCommandConfig,
}

/// Speech recognition settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSettings {
    /// Whisper model size
    pub model_size: WhisperModelSize,
    
    /// Path to whisper model directory
    pub model_path: Option<String>,
    
    /// Language for transcription (blank or "auto" for auto-detect)
    pub language: String,
    
    /// Whether to automatically add punctuation
    pub auto_punctuate: bool,
    
    /// Whether to translate non-English speech to English
    pub translate_to_english: bool,
    
    /// Whether to use enhanced context-aware formatting
    pub context_formatting: bool,
    
    /// Segment duration in seconds
    pub segment_duration: f32,
    
    /// Whether to save transcription to file
    pub save_transcription: bool,
    
    /// Transcription output format: "txt" or "json"
    pub output_format: String,
    
    /// Buffer size in seconds for optimized streaming
    pub buffer_size: f32,
}

impl SpeechSettings {
    /// Set model size from string
    pub fn set_model_size_from_str(&mut self, model_str: &str) -> Result<()> {
        self.model_size = match model_str.to_lowercase().as_str() {
            "tiny" => WhisperModelSize::Tiny,
            "base" => WhisperModelSize::Base,
            "small" => WhisperModelSize::Small,
            "medium" => WhisperModelSize::Medium,
            "large" => WhisperModelSize::Large,
            _ => return Err(anyhow::anyhow!("Invalid model size: {}", model_str)),
        };
        Ok(())
    }
}

/// Available Whisper model sizes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WhisperModelSize {
    /// Tiny model - fastest, least accurate
    Tiny,
    
    /// Base model - fast, less accurate
    Base,
    
    /// Small model - balanced speed and accuracy
    Small,
    
    /// Medium model - slower, more accurate
    Medium,
    
    /// Large model - slowest, most accurate
    Large,
}

impl Default for WhisperModelSize {
    fn default() -> Self {
        Self::Small
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            general: GeneralSettings {
                theme: "system".to_string(),
                auto_start: false,
                minimize_to_tray: true,
            },
            audio: AudioSettings {
                input_device: None,
                input_volume: 1.0,
                speech: SpeechSettings {
                    model_size: WhisperModelSize::default(),
                    model_path: None,
                    language: "auto".to_string(),
                    auto_punctuate: true,
                    translate_to_english: false,
                    context_formatting: true,
                    segment_duration: 5.0,
                    save_transcription: false,
                    output_format: "txt".to_string(),
                    buffer_size: 3.0,
                },
                voice_commands: VoiceCommandConfig::default(),
            },
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    /// Configuration directory
    config_dir: PathBuf,
    
    /// Configuration file path
    config_file: PathBuf,
    
    /// Config
    config: Config,
}

// Implement Clone for ConfigManager
impl Clone for ConfigManager {
    fn clone(&self) -> Self {
        Self {
            config_dir: self.config_dir.clone(),
            config_file: self.config_file.clone(),
            config: self.config.clone(),
        }
    }
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let project_dirs = match ProjectDirs::from("com", "bestme", "BestMe") {
            Some(dirs) => {
                info!("Project directories found");
                dirs
            },
            None => {
                error!("Failed to determine project directories");
                return Err(anyhow::anyhow!("Failed to determine project directories"));
            }
        };
        
        let config_dir = project_dirs.config_dir().to_path_buf();
        let config_file = config_dir.join("config.json");
        
        info!("Config directory: {:?}", config_dir);
        info!("Config file: {:?}", config_file);
        
        // Check for config.json in the application's config directory
        let current_dir = match std::env::current_dir() {
            Ok(dir) => {
                info!("Current directory: {:?}", dir);
                Some(dir)
            },
            Err(e) => {
                warn!("Failed to get current directory: {}", e);
                None
            }
        };
        
        // Try to load config from the app's config directory first
        let mut app_config_file = None;
        if let Some(dir) = &current_dir {
            let app_config_path = dir.join("config").join("config.json");
            info!("Looking for config.json at: {:?}", app_config_path);
            if app_config_path.exists() {
                info!("Found config.json in app directory: {:?}", app_config_path);
                app_config_file = Some(app_config_path);
            } else {
                info!("Config file not found at {:?}", app_config_path);
            }
        }
        
        // Check for settings.cfg in the current directory
        let mut settings_file = None;
        if let Some(dir) = &current_dir {
            let settings_path = dir.join("settings.cfg");
            info!("Looking for settings.cfg at: {:?}", settings_path);
            if settings_path.exists() {
                info!("Found settings.cfg: {:?}", settings_path);
                settings_file = Some(settings_path);
            } else {
                info!("Settings file not found at {:?}", settings_path);
            }
        }
        
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            info!("Creating config directory: {:?}", config_dir);
            fs::create_dir_all(&config_dir)
                .context("Failed to create configuration directory")?;
        }
        
        // Load or create configuration
        let mut config = if let Some(app_config) = &app_config_file {
            // Load from application config directory first
            info!("Loading configuration from app directory: {:?}", app_config);
            let config_str = match fs::read_to_string(app_config) {
                Ok(str) => str,
                Err(e) => {
                    error!("Failed to read app configuration file: {}", e);
                    return Err(anyhow::anyhow!("Failed to read app configuration file: {}", e));
                }
            };
            
            match serde_json::from_str(&config_str) {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("Failed to parse app configuration file: {}", e);
                    return Err(anyhow::anyhow!("Failed to parse app configuration file: {}", e));
                }
            }
        } else if config_file.exists() {
            // Try loading from user config directory next
            info!("Loading existing configuration from: {:?}", config_file);
            let config_str = match fs::read_to_string(&config_file) {
                Ok(str) => str,
                Err(e) => {
                    error!("Failed to read configuration file: {}", e);
                    return Err(anyhow::anyhow!("Failed to read configuration file: {}", e));
                }
            };
            
            match serde_json::from_str(&config_str) {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("Failed to parse configuration file: {}", e);
                    return Err(anyhow::anyhow!("Failed to parse configuration file: {}", e));
                }
            }
        } else {
            // Create default configuration
            info!("Config file not found, creating default configuration");
            let default_config = Config::default();
            let config_str = match serde_json::to_string_pretty(&default_config) {
                Ok(str) => str,
                Err(e) => {
                    error!("Failed to serialize default configuration: {}", e);
                    return Err(anyhow::anyhow!("Failed to serialize default configuration: {}", e));
                }
            };
            
            match fs::write(&config_file, config_str) {
                Ok(_) => {},
                Err(e) => {
                    error!("Failed to write default configuration file: {}", e);
                    return Err(anyhow::anyhow!("Failed to write default configuration file: {}", e));
                }
            }
            
            default_config
        };
        
        // Override with settings from settings.cfg if available
        if let Some(settings_path) = &settings_file {
            info!("Applying settings from: {:?}", settings_path);
            if let Err(e) = Self::apply_settings_from_file(&mut config, settings_path) {
                warn!("Failed to apply settings from settings.cfg: {}", e);
            } else {
                info!("Applied settings from settings.cfg");
            }
        }
        
        info!("Configuration loaded successfully");
        
        Ok(Self {
            config,
            config_dir,
            config_file,
        })
    }
    
    /// Apply settings from a TOML configuration file
    fn apply_settings_from_file(config: &mut Config, path: &PathBuf) -> Result<()> {
        let content = fs::read_to_string(path)
            .context("Failed to read settings file")?;
        
        let table = content.parse::<toml::Table>()
            .context("Failed to parse settings file as TOML")?;
        
        // Process general settings
        if let Some(general) = table.get("general").and_then(|v| v.as_table()) {
            if let Some(theme) = general.get("theme").and_then(|v| v.as_str()) {
                config.general.theme = theme.to_string();
            }
            
            if let Some(auto_start) = general.get("auto_start").and_then(|v| v.as_bool()) {
                config.general.auto_start = auto_start;
            }
            
            if let Some(minimize_to_tray) = general.get("minimize_to_tray").and_then(|v| v.as_bool()) {
                config.general.minimize_to_tray = minimize_to_tray;
            }
        }
        
        // Process audio settings
        if let Some(audio) = table.get("audio").and_then(|v| v.as_table()) {
            if let Some(input_device) = audio.get("input_device").and_then(|v| v.as_str()) {
                if !input_device.is_empty() {
                    config.audio.input_device = Some(input_device.to_string());
                }
            }
            
            if let Some(input_volume) = audio.get("input_volume").and_then(|v| v.as_float()) {
                config.audio.input_volume = input_volume as f32;
            }
            
            // Process speech settings under audio.speech
            if let Some(speech) = audio.get("speech").and_then(|v| v.as_table()) {
                if let Some(model_size) = speech.get("model_size").and_then(|v| v.as_str()) {
                    config.audio.speech.model_size = match model_size.to_lowercase().as_str() {
                        "tiny" => WhisperModelSize::Tiny,
                        "base" => WhisperModelSize::Base,
                        "small" => WhisperModelSize::Small,
                        "medium" => WhisperModelSize::Medium,
                        "large" => WhisperModelSize::Large,
                        _ => WhisperModelSize::Small,
                    };
                }
                
                if let Some(model_path) = speech.get("model_path").and_then(|v| v.as_str()) {
                    if !model_path.is_empty() {
                        config.audio.speech.model_path = Some(model_path.to_string());
                    }
                }
                
                if let Some(language) = speech.get("language").and_then(|v| v.as_str()) {
                    config.audio.speech.language = language.to_string();
                }
                
                if let Some(auto_punctuate) = speech.get("auto_punctuate").and_then(|v| v.as_bool()) {
                    config.audio.speech.auto_punctuate = auto_punctuate;
                }
                
                if let Some(translate_to_english) = speech.get("translate_to_english").and_then(|v| v.as_bool()) {
                    config.audio.speech.translate_to_english = translate_to_english;
                }
                
                if let Some(context_formatting) = speech.get("context_formatting").and_then(|v| v.as_bool()) {
                    config.audio.speech.context_formatting = context_formatting;
                }
                
                if let Some(segment_duration) = speech.get("segment_duration").and_then(|v| v.as_float()) {
                    config.audio.speech.segment_duration = segment_duration as f32;
                }
                
                if let Some(save_transcription) = speech.get("save_transcription").and_then(|v| v.as_bool()) {
                    config.audio.speech.save_transcription = save_transcription;
                }
                
                if let Some(output_format) = speech.get("output_format").and_then(|v| v.as_str()) {
                    config.audio.speech.output_format = output_format.to_string();
                }
                
                if let Some(buffer_size) = speech.get("buffer_size").and_then(|v| v.as_float()) {
                    config.audio.speech.buffer_size = buffer_size as f32;
                }
            }
            
            // Process voice commands settings
            if let Some(voice_commands) = audio.get("voice_commands").and_then(|v| v.as_table()) {
                if let Some(enabled) = voice_commands.get("enabled").and_then(|v| v.as_bool()) {
                    config.audio.voice_commands.enabled = enabled;
                }
                
                if let Some(command_prefix) = voice_commands.get("command_prefix").and_then(|v| v.as_str()) {
                    config.audio.voice_commands.command_prefix = Some(command_prefix.to_string());
                }
                
                if let Some(require_prefix) = voice_commands.get("require_prefix").and_then(|v| v.as_bool()) {
                    config.audio.voice_commands.require_prefix = require_prefix;
                }
                
                if let Some(sensitivity) = voice_commands.get("sensitivity").and_then(|v| v.as_float()) {
                    config.audio.voice_commands.sensitivity = sensitivity as f32;
                }
                
                // Note: custom_commands are not handled here as they have a more complex format
                // that would require special parsing from the TOML structure
            }
        }
        
        Ok(())
    }
    
    /// Get a reference to the configuration
    pub fn get_config(&self) -> &Config {
        &self.config
    }
    
    /// Get a mutable reference to the configuration
    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
    
    /// Save the configuration to disk
    pub fn save(&self) -> Result<()> {
        let config_str = serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize configuration")?;
        
        fs::write(&self.config_file, config_str)
            .context("Failed to write configuration file")?;
        
        info!("Configuration saved successfully");
        
        Ok(())
    }
    
    /// Get the preferred audio device name
    pub fn preferred_device_name(&self) -> &str {
        self.config.audio.input_device.as_deref().unwrap_or("")
    }
    
    /// Set the preferred audio device name
    pub fn set_preferred_device_name(&mut self, name: String) -> Result<()> {
        self.config.audio.input_device = if name.is_empty() { None } else { Some(name) };
        Ok(())
    }
    
    /// Get a reference to the whisper config
    pub fn whisper_config(&self) -> &SpeechSettings {
        &self.config.audio.speech
    }
    
    /// Get a mutable reference to the whisper config
    pub fn whisper_config_mut(&mut self) -> &mut SpeechSettings {
        &mut self.config.audio.speech
    }
    
    /// Set auto transcribe flag
    pub fn set_auto_transcribe(&mut self, auto_transcribe: bool) {
        // This is a new feature, so we'll just print for now
        println!("Setting auto transcribe: {}", auto_transcribe);
        // In a real implementation, this would modify a field in the config
    }
    
    /// Get auto transcribe flag
    pub fn auto_transcribe(&self) -> bool {
        // This is a new feature, so we'll just return a default for now
        true
    }
    
    /// Set offline mode flag
    pub fn set_offline_mode(&mut self, offline_mode: bool) {
        // This is a new feature, so we'll just print for now
        println!("Setting offline mode: {}", offline_mode);
        // In a real implementation, this would modify a field in the config
    }
    
    /// Get offline mode flag
    pub fn offline_mode(&self) -> bool {
        // This is a new feature, so we'll just return a default for now
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.theme, "system");
        assert_eq!(config.general.auto_start, false);
        assert_eq!(config.general.minimize_to_tray, true);
        assert_eq!(config.audio.input_volume, 1.0);
        assert!(config.audio.input_device.is_none());
    }
} 
