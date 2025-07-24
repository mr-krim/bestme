use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_bytes: u64,
    pub parameters: String,
    pub capabilities: Vec<String>,
    pub requirements: ModelRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub min_ram_gb: f32,
    pub min_vram_gb: Option<f32>,
    pub supports_gpu: bool,
    pub supports_quantization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: PathBuf,
    pub use_gpu: bool,
    pub quantization: Option<QuantizationType>,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QuantizationType {
    None,
    Int8,
    Int4,
    FP16,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            use_gpu: true,
            quantization: None,
            max_tokens: 512,
            temperature: 0.3,
            top_p: 0.9,
        }
    }
}

pub fn get_models_directory() -> PathBuf {
    let app_data_dir = crate::config::get_app_data_dir();
    app_data_dir.join("models").join("ai")
}

pub fn ensure_models_directory() -> std::io::Result<PathBuf> {
    let dir = get_models_directory();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub trait TextProcessor: Send + Sync {
    fn preprocess(&self, text: &str) -> String;
    fn postprocess(&self, text: &str) -> String;
}

#[derive(Default)]
pub struct StandardTextProcessor;

impl TextProcessor for StandardTextProcessor {
    fn preprocess(&self, text: &str) -> String {
        text.trim()
            .replace("  ", " ")
            .to_string()
    }

    fn postprocess(&self, text: &str) -> String {
        let mut result = text.trim().to_string();
        
        // Ensure sentence ends with punctuation
        if !result.is_empty() && !result.chars().last().unwrap().is_ascii_punctuation() {
            result.push('.');
        }
        
        // Capitalize first letter
        if let Some(first_char) = result.chars().next() {
            if first_char.is_lowercase() {
                let mut chars = result.chars();
                chars.next();
                result = first_char.to_uppercase().collect::<String>() + chars.as_str();
            }
        }
        
        result
    }
}

pub const AVAILABLE_MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "phi-3-mini".to_string(),
        size_bytes: 3_800_000_000,
        parameters: "3.8B".to_string(),
        capabilities: vec![
            "grammar_correction".to_string(),
            "punctuation".to_string(),
            "intent_detection".to_string(),
        ],
        requirements: ModelRequirements {
            min_ram_gb: 8.0,
            min_vram_gb: Some(4.0),
            supports_gpu: true,
            supports_quantization: true,
        },
    },
    ModelInfo {
        name: "llama-3.2-1b".to_string(),
        size_bytes: 1_000_000_000,
        parameters: "1B".to_string(),
        capabilities: vec![
            "grammar_correction".to_string(),
            "punctuation".to_string(),
        ],
        requirements: ModelRequirements {
            min_ram_gb: 4.0,
            min_vram_gb: Some(2.0),
            supports_gpu: true,
            supports_quantization: true,
        },
    },
];

pub fn find_model_info(model_name: &str) -> Option<&'static ModelInfo> {
    AVAILABLE_MODELS.iter().find(|m| m.name == model_name)
}