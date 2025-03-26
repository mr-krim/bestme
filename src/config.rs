use anyhow::{Context, Result};
use directories::ProjectDirs;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
}

/// Speech recognition settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSettings {
    /// Whisper model size
    pub model_size: WhisperModelSize,
    
    /// Path to whisper model directory
    pub model_path: Option<String>,
    
    /// Language for transcription (blank for auto-detect)
    pub language: String,
    
    /// Segment duration in seconds
    pub segment_duration: f32,
    
    /// Whether to save transcription to file
    pub save_transcription: bool,
    
    /// Transcription output format: "txt" or "json"
    pub output_format: String,
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
                    language: "".to_string(),
                    segment_duration: 5.0,
                    save_transcription: false,
                    output_format: "txt".to_string(),
                },
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
        let project_dirs = ProjectDirs::from("com", "bestme", "BestMe")
            .context("Failed to determine project directories")?;
        
        let config_dir = project_dirs.config_dir().to_path_buf();
        let config_file = config_dir.join("config.json");
        
        debug!("Config directory: {:?}", config_dir);
        debug!("Config file: {:?}", config_file);
        
        // Check for settings.cfg in the current directory
        let mut settings_file = None;
        let current_dir = std::env::current_dir().ok();
        
        if let Some(dir) = current_dir {
            let settings_path = dir.join("settings.cfg");
            if settings_path.exists() {
                debug!("Found settings.cfg: {:?}", settings_path);
                settings_file = Some(settings_path);
            }
        }
        
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("Failed to create configuration directory")?;
        }
        
        // Load or create configuration
        let mut config = if config_file.exists() {
            // Load existing configuration
            let config_str = fs::read_to_string(&config_file)
                .context("Failed to read configuration file")?;
            
            serde_json::from_str(&config_str)
                .context("Failed to parse configuration file")?
        } else {
            // Create default configuration
            let default_config = Config::default();
            let config_str = serde_json::to_string_pretty(&default_config)
                .context("Failed to serialize default configuration")?;
            
            fs::write(&config_file, config_str)
                .context("Failed to write default configuration file")?;
            
            default_config
        };
        
        // Override with settings from settings.cfg if available
        if let Some(settings_path) = &settings_file {
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
        }
        
        // Process speech settings
        if let Some(speech) = table.get("speech").and_then(|v| v.as_table()) {
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
            
            if let Some(segment_duration) = speech.get("segment_duration").and_then(|v| v.as_float()) {
                config.audio.speech.segment_duration = segment_duration as f32;
            }
            
            if let Some(save_transcription) = speech.get("save_transcription").and_then(|v| v.as_bool()) {
                config.audio.speech.save_transcription = save_transcription;
            }
            
            if let Some(output_format) = speech.get("output_format").and_then(|v| v.as_str()) {
                config.audio.speech.output_format = output_format.to_string();
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
