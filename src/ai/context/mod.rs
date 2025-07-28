pub mod conversation;
pub mod memory;
pub mod templates;

use crate::ai::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Context configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum context window size in tokens
    pub max_tokens: usize,
    /// Maximum number of turns to keep
    pub max_turns: usize,
    /// Context expiration time
    pub expiration: Duration,
    /// Enable context compression
    pub enable_compression: bool,
    /// Context storage backend
    pub storage: ContextStorage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextStorage {
    Memory,
    Disk(String),
    Database(String),
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            max_turns: 20,
            expiration: Duration::from_secs(3600), // 1 hour
            enable_compression: true,
            storage: ContextStorage::Memory,
        }
    }
}

/// Conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// Unique ID for this turn
    pub id: String,
    /// Role (user, assistant, system)
    pub role: Role,
    /// Content of the turn
    pub content: String,
    /// Timestamp
    pub timestamp: SystemTime,
    /// Token count
    pub tokens: usize,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
        }
    }
}

/// Context window for managing conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    /// Conversation ID
    pub conversation_id: String,
    /// List of turns
    pub turns: Vec<Turn>,
    /// Total token count
    pub total_tokens: usize,
    /// Creation time
    pub created_at: SystemTime,
    /// Last update time
    pub updated_at: SystemTime,
    /// Context metadata
    pub metadata: ContextMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// User preferences
    pub user_preferences: Option<serde_json::Value>,
    /// Conversation topic/domain
    pub topic: Option<String>,
    /// Language
    pub language: String,
    /// Custom attributes
    pub attributes: std::collections::HashMap<String, String>,
}

impl Default for ContextMetadata {
    fn default() -> Self {
        Self {
            user_preferences: None,
            topic: None,
            language: "en".to_string(),
            attributes: std::collections::HashMap::new(),
        }
    }
}

impl ContextWindow {
    /// Create a new context window
    pub fn new(conversation_id: String) -> Self {
        let now = SystemTime::now();
        Self {
            conversation_id,
            turns: Vec::new(),
            total_tokens: 0,
            created_at: now,
            updated_at: now,
            metadata: ContextMetadata::default(),
        }
    }
    
    /// Add a turn to the context
    pub fn add_turn(&mut self, turn: Turn) {
        self.total_tokens += turn.tokens;
        self.turns.push(turn);
        self.updated_at = SystemTime::now();
    }
    
    /// Get the last N turns
    pub fn recent_turns(&self, n: usize) -> &[Turn] {
        let start = self.turns.len().saturating_sub(n);
        &self.turns[start..]
    }
    
    /// Compress context by removing old turns
    pub fn compress(&mut self, config: &ContextConfig) {
        // Remove turns that exceed max_turns
        while self.turns.len() > config.max_turns {
            let removed = self.turns.remove(0);
            self.total_tokens = self.total_tokens.saturating_sub(removed.tokens);
        }
        
        // Remove turns that exceed max_tokens
        while self.total_tokens > config.max_tokens && !self.turns.is_empty() {
            let removed = self.turns.remove(0);
            self.total_tokens = self.total_tokens.saturating_sub(removed.tokens);
        }
    }
    
    /// Check if context has expired
    pub fn is_expired(&self, expiration: Duration) -> bool {
        if let Ok(elapsed) = self.updated_at.elapsed() {
            elapsed > expiration
        } else {
            true
        }
    }
    
    /// Format context for model input
    pub fn format_for_model(&self, system_prompt: Option<&str>) -> String {
        let mut formatted = String::new();
        
        // Add system prompt if provided
        if let Some(prompt) = system_prompt {
            formatted.push_str(&format!("System: {}\n\n", prompt));
        }
        
        // Add conversation turns
        for turn in &self.turns {
            formatted.push_str(&format!("{}: {}\n", 
                turn.role.as_str().to_uppercase(), 
                turn.content
            ));
        }
        
        formatted
    }
    
    /// Extract key information from context
    pub fn extract_summary(&self) -> ContextSummary {
        let topics = self.extract_topics();
        let entities = self.extract_entities();
        let sentiment = self.analyze_sentiment();
        
        ContextSummary {
            conversation_id: self.conversation_id.clone(),
            turn_count: self.turns.len(),
            total_tokens: self.total_tokens,
            topics,
            entities,
            sentiment,
            duration: self.created_at.elapsed().ok(),
        }
    }
    
    /// Extract topics from conversation (placeholder)
    fn extract_topics(&self) -> Vec<String> {
        // In a real implementation, use NLP to extract topics
        vec![]
    }
    
    /// Extract entities from conversation (placeholder)
    fn extract_entities(&self) -> Vec<String> {
        // In a real implementation, use NER to extract entities
        vec![]
    }
    
    /// Analyze sentiment (placeholder)
    fn analyze_sentiment(&self) -> f32 {
        // In a real implementation, use sentiment analysis
        0.0
    }
}

/// Summary of a conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub conversation_id: String,
    pub turn_count: usize,
    pub total_tokens: usize,
    pub topics: Vec<String>,
    pub entities: Vec<String>,
    pub sentiment: f32,
    pub duration: Option<Duration>,
}

/// Context manager trait
#[async_trait::async_trait]
pub trait ContextManager: Send + Sync {
    /// Create a new conversation
    async fn create_conversation(&self) -> Result<String>;
    
    /// Get a conversation context
    async fn get_context(&self, conversation_id: &str) -> Result<Option<ContextWindow>>;
    
    /// Update a conversation context
    async fn update_context(&self, context: ContextWindow) -> Result<()>;
    
    /// Delete a conversation
    async fn delete_conversation(&self, conversation_id: &str) -> Result<()>;
    
    /// List all conversations
    async fn list_conversations(&self) -> Result<Vec<String>>;
    
    /// Clean up expired conversations
    async fn cleanup_expired(&self, expiration: Duration) -> Result<usize>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_window() {
        let mut context = ContextWindow::new("test-123".to_string());
        
        let turn = Turn {
            id: "turn-1".to_string(),
            role: Role::User,
            content: "Hello".to_string(),
            timestamp: SystemTime::now(),
            tokens: 2,
            metadata: None,
        };
        
        context.add_turn(turn);
        assert_eq!(context.turns.len(), 1);
        assert_eq!(context.total_tokens, 2);
    }
    
    #[test]
    fn test_context_compression() {
        let mut context = ContextWindow::new("test-123".to_string());
        let config = ContextConfig {
            max_turns: 2,
            ..Default::default()
        };
        
        // Add 3 turns
        for i in 0..3 {
            context.add_turn(Turn {
                id: format!("turn-{}", i),
                role: Role::User,
                content: format!("Message {}", i),
                timestamp: SystemTime::now(),
                tokens: 5,
                metadata: None,
            });
        }
        
        assert_eq!(context.turns.len(), 3);
        
        // Compress should remove oldest turn
        context.compress(&config);
        assert_eq!(context.turns.len(), 2);
        assert_eq!(context.turns[0].id, "turn-1");
    }
}