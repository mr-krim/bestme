use crate::ai::{Result, AIError};
use crate::ai::context::{ContextConfig, ContextManager, ContextWindow, Turn, Role};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// In-memory conversation manager
pub struct InMemoryConversationManager {
    config: ContextConfig,
    conversations: Arc<RwLock<HashMap<String, ContextWindow>>>,
}

impl InMemoryConversationManager {
    /// Create a new in-memory conversation manager
    pub fn new(config: ContextConfig) -> Self {
        Self {
            config,
            conversations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add a user message to a conversation
    pub async fn add_user_message(
        &self,
        conversation_id: &str,
        content: String,
        tokens: usize,
    ) -> Result<()> {
        let turn = Turn {
            id: Uuid::new_v4().to_string(),
            role: Role::User,
            content,
            timestamp: SystemTime::now(),
            tokens,
            metadata: None,
        };
        
        self.add_turn(conversation_id, turn).await
    }
    
    /// Add an assistant message to a conversation
    pub async fn add_assistant_message(
        &self,
        conversation_id: &str,
        content: String,
        tokens: usize,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let turn = Turn {
            id: Uuid::new_v4().to_string(),
            role: Role::Assistant,
            content,
            timestamp: SystemTime::now(),
            tokens,
            metadata,
        };
        
        self.add_turn(conversation_id, turn).await
    }
    
    /// Add a system message to a conversation
    pub async fn add_system_message(
        &self,
        conversation_id: &str,
        content: String,
    ) -> Result<()> {
        let turn = Turn {
            id: Uuid::new_v4().to_string(),
            role: Role::System,
            content: content.clone(),
            timestamp: SystemTime::now(),
            tokens: content.split_whitespace().count(), // Simple token estimate
            metadata: None,
        };
        
        self.add_turn(conversation_id, turn).await
    }
    
    /// Add a turn to a conversation
    async fn add_turn(&self, conversation_id: &str, turn: Turn) -> Result<()> {
        let mut conversations = self.conversations.write().await;
        
        let context = conversations
            .entry(conversation_id.to_string())
            .or_insert_with(|| ContextWindow::new(conversation_id.to_string()));
        
        context.add_turn(turn);
        context.compress(&self.config);
        
        Ok(())
    }
    
    /// Get conversation context for model input
    pub async fn get_model_context(
        &self,
        conversation_id: &str,
        system_prompt: Option<&str>,
    ) -> Result<Option<String>> {
        let conversations = self.conversations.read().await;
        
        if let Some(context) = conversations.get(conversation_id) {
            Ok(Some(context.format_for_model(system_prompt)))
        } else {
            Ok(None)
        }
    }
    
    /// Get recent conversation history
    pub async fn get_recent_history(
        &self,
        conversation_id: &str,
        max_turns: usize,
    ) -> Result<Vec<Turn>> {
        let conversations = self.conversations.read().await;
        
        if let Some(context) = conversations.get(conversation_id) {
            Ok(context.recent_turns(max_turns).to_vec())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Fork a conversation (create a new branch)
    pub async fn fork_conversation(
        &self,
        source_id: &str,
        turns_to_copy: Option<usize>,
    ) -> Result<String> {
        let conversations = self.conversations.read().await;
        
        if let Some(source) = conversations.get(source_id) {
            let new_id = Uuid::new_v4().to_string();
            let mut new_context = ContextWindow::new(new_id.clone());
            
            // Copy metadata
            new_context.metadata = source.metadata.clone();
            
            // Copy turns
            let turns_to_copy = turns_to_copy.unwrap_or(source.turns.len());
            for turn in source.recent_turns(turns_to_copy) {
                new_context.add_turn(turn.clone());
            }
            
            drop(conversations);
            
            let mut conversations = self.conversations.write().await;
            conversations.insert(new_id.clone(), new_context);
            
            Ok(new_id)
        } else {
            Err(AIError::ConfigError("Source conversation not found".to_string()))
        }
    }
    
    /// Merge two conversations
    pub async fn merge_conversations(
        &self,
        target_id: &str,
        source_id: &str,
    ) -> Result<()> {
        let mut conversations = self.conversations.write().await;
        
        let source_turns = if let Some(source) = conversations.get(source_id) {
            source.turns.clone()
        } else {
            return Err(AIError::ConfigError("Source conversation not found".to_string()));
        };
        
        if let Some(target) = conversations.get_mut(target_id) {
            for turn in source_turns {
                target.add_turn(turn);
            }
            target.compress(&self.config);
            Ok(())
        } else {
            Err(AIError::ConfigError("Target conversation not found".to_string()))
        }
    }
}

#[async_trait::async_trait]
impl ContextManager for InMemoryConversationManager {
    async fn create_conversation(&self) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let context = ContextWindow::new(id.clone());
        
        let mut conversations = self.conversations.write().await;
        conversations.insert(id.clone(), context);
        
        Ok(id)
    }
    
    async fn get_context(&self, conversation_id: &str) -> Result<Option<ContextWindow>> {
        let conversations = self.conversations.read().await;
        Ok(conversations.get(conversation_id).cloned())
    }
    
    async fn update_context(&self, context: ContextWindow) -> Result<()> {
        let mut conversations = self.conversations.write().await;
        conversations.insert(context.conversation_id.clone(), context);
        Ok(())
    }
    
    async fn delete_conversation(&self, conversation_id: &str) -> Result<()> {
        let mut conversations = self.conversations.write().await;
        conversations.remove(conversation_id);
        Ok(())
    }
    
    async fn list_conversations(&self) -> Result<Vec<String>> {
        let conversations = self.conversations.read().await;
        Ok(conversations.keys().cloned().collect())
    }
    
    async fn cleanup_expired(&self, expiration: Duration) -> Result<usize> {
        let mut conversations = self.conversations.write().await;
        let initial_count = conversations.len();
        
        conversations.retain(|_, context| !context.is_expired(expiration));
        
        let removed = initial_count - conversations.len();
        Ok(removed)
    }
}

/// Conversation manager with persistence
pub struct PersistentConversationManager {
    memory_manager: InMemoryConversationManager,
    storage_path: String,
}

impl PersistentConversationManager {
    /// Create a new persistent conversation manager
    pub fn new(config: ContextConfig, storage_path: String) -> Self {
        Self {
            memory_manager: InMemoryConversationManager::new(config),
            storage_path,
        }
    }
    
    /// Save a conversation to disk
    pub async fn save_conversation(&self, conversation_id: &str) -> Result<()> {
        if let Some(context) = self.memory_manager.get_context(conversation_id).await? {
            let file_path = format!("{}/{}.json", self.storage_path, conversation_id);
            let json = serde_json::to_string_pretty(&context)
                .map_err(|e| AIError::ConfigError(format!("Failed to serialize: {}", e)))?;
            
            tokio::fs::write(&file_path, json).await
                .map_err(|e| AIError::ConfigError(format!("Failed to write file: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Load a conversation from disk
    pub async fn load_conversation(&self, conversation_id: &str) -> Result<()> {
        let file_path = format!("{}/{}.json", self.storage_path, conversation_id);
        
        if let Ok(json) = tokio::fs::read_to_string(&file_path).await {
            let context: ContextWindow = serde_json::from_str(&json)
                .map_err(|e| AIError::ConfigError(format!("Failed to deserialize: {}", e)))?;
            
            self.memory_manager.update_context(context).await?;
        }
        
        Ok(())
    }
    
    /// Save all conversations
    pub async fn save_all(&self) -> Result<()> {
        let conversations = self.memory_manager.list_conversations().await?;
        
        for id in conversations {
            self.save_conversation(&id).await?;
        }
        
        Ok(())
    }
    
    /// Load all conversations
    pub async fn load_all(&self) -> Result<()> {
        let entries = tokio::fs::read_dir(&self.storage_path).await
            .map_err(|e| AIError::ConfigError(format!("Failed to read directory: {}", e)))?;
        
        let mut entries = entries;
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    let id = name.trim_end_matches(".json");
                    let _ = self.load_conversation(id).await;
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_conversation_manager() {
        let manager = InMemoryConversationManager::new(ContextConfig::default());
        
        // Create conversation
        let id = manager.create_conversation().await.unwrap();
        
        // Add messages
        manager.add_user_message(&id, "Hello".to_string(), 2).await.unwrap();
        manager.add_assistant_message(&id, "Hi there!".to_string(), 3, None).await.unwrap();
        
        // Get context
        let context = manager.get_context(&id).await.unwrap().unwrap();
        assert_eq!(context.turns.len(), 2);
        assert_eq!(context.total_tokens, 5);
    }
    
    #[tokio::test]
    async fn test_conversation_fork() {
        let manager = InMemoryConversationManager::new(ContextConfig::default());
        
        // Create and populate conversation
        let id1 = manager.create_conversation().await.unwrap();
        manager.add_user_message(&id1, "Message 1".to_string(), 3).await.unwrap();
        manager.add_user_message(&id1, "Message 2".to_string(), 3).await.unwrap();
        
        // Fork conversation
        let id2 = manager.fork_conversation(&id1, Some(1)).await.unwrap();
        
        // Verify fork
        let context2 = manager.get_context(&id2).await.unwrap().unwrap();
        assert_eq!(context2.turns.len(), 1);
        assert_eq!(context2.turns[0].content, "Message 2");
    }
}