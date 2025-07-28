use crate::ai::{Result, AIError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::sync::Arc;

#[async_trait]
pub trait APIClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
    async fn validate_api_key(&self) -> Result<()>;
}

pub fn create_client(config: &super::CloudAIConfig) -> Result<Arc<dyn APIClient>> {
    match config.provider {
        super::AIProviderType::OpenRouter => {
            Ok(Arc::new(OpenRouterClient::new(
                config.api_key.clone(),
                config.model.clone(),
                config.base_url.clone(),
            )))
        }
        super::AIProviderType::Requesty => {
            Ok(Arc::new(RequestyClient::new(
                config.api_key.clone(),
                config.model.clone(),
                config.base_url.clone(),
            )))
        }
        super::AIProviderType::OpenAI => {
            Ok(Arc::new(OpenAIClient::new(
                config.api_key.clone(),
                config.model.clone(),
                config.base_url.clone(),
            )))
        }
    }
}

pub struct OpenRouterClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenRouterClient {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
            client,
        }
    }
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: usize,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[async_trait]
impl APIClient for OpenRouterClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful AI assistant focused on text enhancement, grammar correction, and style improvement.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 1000,
        };

        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/bestme-app")
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIError::NetworkError(format!(
                "API request failed: {}",
                error_text
            )));
        }

        let api_response: OpenRouterResponse = response
            .json()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AIError::InferenceError("No response from API".to_string()))
    }

    async fn validate_api_key(&self) -> Result<()> {
        // Make a simple request to validate the API key
        let response = self.client
            .get(&format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AIError::ConfigError("Invalid API key".to_string()))
        }
    }
}

pub struct RequestyClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl RequestyClient {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| "https://api.requesty.ai/v1".to_string()),
            client,
        }
    }
}

#[derive(Serialize)]
struct RequestyRequest {
    model: String,
    prompt: String,
    max_tokens: usize,
    temperature: f32,
    system_prompt: Option<String>,
}

#[derive(Deserialize)]
struct RequestyResponse {
    output: String,
    usage: Option<RequestyUsage>,
}

#[derive(Deserialize)]
struct RequestyUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl APIClient for RequestyClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let request = RequestyRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            max_tokens: 1000,
            temperature: 0.3,
            system_prompt: Some("You are a helpful AI assistant focused on text enhancement, grammar correction, and style improvement.".to_string()),
        };

        let response = self.client
            .post(&format!("{}/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIError::NetworkError(format!(
                "API request failed: {}",
                error_text
            )));
        }

        let api_response: RequestyResponse = response
            .json()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        Ok(api_response.output)
    }

    async fn validate_api_key(&self) -> Result<()> {
        // Validate key with a minimal request
        let request = RequestyRequest {
            model: self.model.clone(),
            prompt: "test".to_string(),
            max_tokens: 1,
            temperature: 0.0,
            system_prompt: None,
        };

        let response = self.client
            .post(&format!("{}/validate", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AIError::ConfigError("Invalid API key".to_string()))
        }
    }
}

pub struct OpenAIClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAIClient {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            client,
        }
    }
}

#[async_trait]
impl APIClient for OpenAIClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        // OpenAI uses the same format as OpenRouter
        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful AI assistant focused on text enhancement, grammar correction, and style improvement.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 1000,
        };

        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIError::NetworkError(format!(
                "API request failed: {}",
                error_text
            )));
        }

        let api_response: OpenRouterResponse = response
            .json()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AIError::InferenceError("No response from API".to_string()))
    }

    async fn validate_api_key(&self) -> Result<()> {
        let response = self.client
            .get(&format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AIError::ConfigError("Invalid API key".to_string()))
        }
    }
}