pub mod api_client;
pub mod privacy;

use crate::ai::{AIProvider, EnhancementOptions, EnhancedText, Result, AIError, PrivacyLevel};
use crate::ai::security::{ApiKeyManager, get_api_key_from_env};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAIConfig {
    pub provider: AIProviderType,
    pub api_key: String,
    pub model: String,
    pub privacy_level: PrivacyLevel,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProviderType {
    OpenRouter,
    Requesty,
    OpenAI,
}

pub struct CloudAI {
    config: Arc<RwLock<CloudAIConfig>>,
    client: Arc<dyn api_client::APIClient>,
    privacy_manager: Arc<privacy::PrivacyManager>,
    key_manager: Arc<RwLock<ApiKeyManager>>,
}

impl CloudAI {
    pub fn new(mut config: CloudAIConfig) -> Result<Self> {
        // Try to get API key from keychain or environment if not provided
        if config.api_key.is_empty() {
            let provider_name = match &config.provider {
                AIProviderType::OpenRouter => "openrouter",
                AIProviderType::Requesty => "requesty",
                AIProviderType::OpenAI => "openai",
            };
            
            // Try environment variable first
            if let Some(env_key) = get_api_key_from_env(provider_name) {
                config.api_key = env_key;
            }
        }
        
        let client = api_client::create_client(&config)?;
        let privacy_manager = Arc::new(privacy::PrivacyManager::new(config.privacy_level.clone()));
        let mut key_manager = ApiKeyManager::new();
        let _ = key_manager.load_metadata(); // Load existing metadata
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            client,
            privacy_manager,
            key_manager: Arc::new(RwLock::new(key_manager)),
        })
    }

    pub async fn update_config(&self, config: CloudAIConfig) -> Result<()> {
        let mut current = self.config.write().await;
        *current = config;
        Ok(())
    }

    async fn should_use_cloud(&self, text: &str) -> bool {
        let config = self.config.read().await;
        match config.privacy_level {
            PrivacyLevel::Strict => false,
            PrivacyLevel::Balanced => text.len() > 50, // Only for longer texts
            PrivacyLevel::Permissive => true,
        }
    }
}

#[async_trait::async_trait]
impl AIProvider for CloudAI {
    fn name(&self) -> &str {
        "Cloud AI"
    }

    fn is_available(&self) -> bool {
        // Check if API key is configured
        tokio::task::block_in_place(|| {
            let config = tokio::runtime::Handle::current()
                .block_on(self.config.read());
            !config.api_key.is_empty()
        })
    }

    async fn initialize(&mut self) -> Result<()> {
        // Validate API key with a test request
        self.client.validate_api_key().await?;
        Ok(())
    }

    async fn enhance_text(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Result<EnhancedText> {
        // Check privacy settings
        if !self.should_use_cloud(text).await {
            return Err(AIError::PrivacyViolation(
                "Text processing blocked by privacy settings".to_string()
            ));
        }

        // Anonymize text if needed
        let (processed_text, deanonymizer) = self.privacy_manager.anonymize(text).await?;
        
        // Build prompt based on options
        let prompt = build_enhancement_prompt(&processed_text, options);
        
        // Call API
        let response = self.client.complete(&prompt).await?;
        
        // Parse response and extract enhanced text
        let mut enhanced = parse_api_response(&response, &processed_text)?;
        
        // Deanonymize if needed
        if let Some(deanonymizer) = deanonymizer {
            enhanced = deanonymizer.restore(&enhanced)?;
        }
        
        Ok(enhanced)
    }
}

fn build_enhancement_prompt(text: &str, options: &EnhancementOptions) -> String {
    let mut tasks = Vec::new();
    
    if options.correct_grammar {
        tasks.push("correct grammar");
    }
    if options.improve_punctuation {
        tasks.push("improve punctuation");
    }
    if options.detect_intent {
        tasks.push("detect if this is a command or dictation");
    }
    
    format!(
        "Please {} for the following text. Preserve the original style and meaning. \
         Return the enhanced text and list any corrections made.\n\nText: {}",
        tasks.join(", "),
        text
    )
}

fn parse_api_response(response: &str, original: &str) -> Result<EnhancedText> {
    // This is a simple parser - in production, you'd want more robust parsing
    // potentially using structured output from the API
    
    Ok(EnhancedText {
        original: original.to_string(),
        enhanced: response.trim().to_string(),
        intent: None, // TODO: Parse intent from response
        confidence: 0.9, // TODO: Calculate based on response
        corrections: Vec::new(), // TODO: Extract corrections
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAIUsage {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub last_reset: chrono::DateTime<chrono::Utc>,
}

impl CloudAI {
    pub async fn save_api_key(&self, api_key: &str) -> Result<()> {
        let config = self.config.read().await;
        let provider_name = match &config.provider {
            AIProviderType::OpenRouter => "openrouter",
            AIProviderType::Requesty => "requesty",
            AIProviderType::OpenAI => "openai",
        };
        
        let mut key_manager = self.key_manager.write().await;
        key_manager.store_api_key(provider_name, api_key).await
    }
    
    pub async fn load_api_key_from_keychain(&self) -> Result<()> {
        let mut config = self.config.write().await;
        let provider_name = match &config.provider {
            AIProviderType::OpenRouter => "openrouter",
            AIProviderType::Requesty => "requesty",
            AIProviderType::OpenAI => "openai",
        };
        
        let mut key_manager = self.key_manager.write().await;
        match key_manager.get_api_key(provider_name).await {
            Ok(api_key) => {
                config.api_key = api_key;
                Ok(())
            }
            Err(_) => {
                // Try environment variable as fallback
                if let Some(env_key) = get_api_key_from_env(provider_name) {
                    config.api_key = env_key;
                    Ok(())
                } else {
                    Err(AIError::ConfigError("No API key found".to_string()))
                }
            }
        }
    }
    
    pub async fn get_usage(&self) -> Result<CloudAIUsage> {
        // TODO: Implement usage tracking
        Ok(CloudAIUsage {
            total_requests: 0,
            total_tokens: 0,
            total_cost: 0.0,
            last_reset: chrono::Utc::now(),
        })
    }
    
    pub async fn summarize_text(&self, text: &str, max_length: usize) -> Result<String> {
        let prompt = format!(
            "Summarize the following text in no more than {} words:\n\n{}",
            max_length / 5, // Rough word count
            text
        );
        
        let response = self.client.complete(&prompt).await?;
        Ok(response.trim().to_string())
    }
    
    pub async fn transform_style(
        &self,
        text: &str,
        target_style: &str,
    ) -> Result<String> {
        let prompt = format!(
            "Rewrite the following text in a {} style while preserving the meaning:\n\n{}",
            target_style,
            text
        );
        
        let response = self.client.complete(&prompt).await?;
        Ok(response.trim().to_string())
    }
    
    pub async fn translate_text(
        &self,
        text: &str,
        target_language: &str,
        source_language: Option<&str>,
    ) -> Result<String> {
        let prompt = if let Some(source_lang) = source_language {
            format!(
                "Translate the following text from {} to {}:\n\n{}",
                source_lang,
                target_language,
                text
            )
        } else {
            format!(
                "Translate the following text to {}:\n\n{}",
                target_language,
                text
            )
        };
        
        let response = self.client.complete(&prompt).await?;
        Ok(response.trim().to_string())
    }
}