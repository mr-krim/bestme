use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralSettings,
    pub audio: AudioSettings,
    pub speech: SpeechSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub theme: String,
    pub auto_start: bool,
    pub show_in_system_tray: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub device_name: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSettings {
    pub language: String,
    pub model_size: WhisperModelSize,
    pub translate_to_english: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhisperModelSize {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                theme: "dark".to_string(),
                auto_start: false,
                show_in_system_tray: true,
            },
            audio: AudioSettings {
                device_name: None,
                sample_rate: 16000,
                channels: 1,
            },
            speech: SpeechSettings {
                language: "auto".to_string(),
                model_size: WhisperModelSize::Base,
                translate_to_english: false,
            },
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn load_config(&self) -> anyhow::Result<Config> {
        Ok(Config::default())
    }
    
    pub fn save_config(&self, _config: &Config) -> anyhow::Result<()> {
        Ok(())
    }
}