pub mod common;
pub mod local;
pub mod cloud;
pub mod security;
pub mod models;
pub mod services;
pub mod gpu;
pub mod benchmarks;
pub mod telemetry;
pub mod context;
pub mod resilience;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

// pub mod integration;

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use async_trait::async_trait;

#[derive(Debug)]
pub enum AIError {
    ModelNotFound(String),
    InferenceError(String),
    ConfigError(String),
    NetworkError(String),
    PrivacyViolation(String),
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AIError::ModelNotFound(msg) => write!(f, "Model not found: {}", msg),
            AIError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            AIError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            AIError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AIError::PrivacyViolation(msg) => write!(f, "Privacy violation: {}", msg),
        }
    }
}

impl Error for AIError {}

pub type Result<T> = std::result::Result<T, AIError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Strict,     // All processing local only
    Balanced,   // Anonymized cloud requests
    Permissive, // Full cloud features
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    Dictation,
    Command(String),
    Question,
    Conversation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementOptions {
    pub correct_grammar: bool,
    pub improve_clarity: bool,
    pub preserve_style: bool,
    pub detect_intent: bool,
    pub format_markdown: bool,
    pub improve_punctuation: bool,
    pub confidence_threshold: f32,
}

impl Default for EnhancementOptions {
    fn default() -> Self {
        Self {
            correct_grammar: true,
            improve_clarity: true,
            preserve_style: true,
            detect_intent: false,
            format_markdown: false,
            improve_punctuation: true,
            confidence_threshold: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedText {
    pub original: String,
    pub enhanced: String,
    pub intent: Option<Intent>,
    pub confidence: f32,
    pub corrections: Vec<Correction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub start: usize,
    pub end: usize,
    pub original: String,
    pub corrected: String,
    pub reason: String,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    async fn initialize(&mut self) -> Result<()>;
    async fn enhance_text(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Result<EnhancedText>;
}