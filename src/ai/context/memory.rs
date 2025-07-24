use crate::ai::{Result, AIError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Memory types for context management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    /// Short-term memory (current conversation)
    ShortTerm,
    /// Working memory (recent conversations)
    Working,
    /// Long-term memory (persistent facts)
    LongTerm,
    /// Episodic memory (specific events)
    Episodic,
    /// Semantic memory (general knowledge)
    Semantic,
}

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier
    pub id: String,
    /// Memory type
    pub memory_type: MemoryType,
    /// Content
    pub content: String,
    /// Associated embeddings (for similarity search)
    pub embedding: Option<Vec<f32>>,
    /// Relevance score
    pub relevance: f32,
    /// Access count
    pub access_count: u32,
    /// Creation time
    pub created_at: std::time::SystemTime,
    /// Last accessed time
    pub last_accessed: std::time::SystemTime,
    /// Associated metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Memory bank for managing different types of memory
pub struct MemoryBank {
    /// Short-term memory buffer
    short_term: Arc<RwLock<VecDeque<MemoryEntry>>>,
    /// Working memory
    working: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    /// Long-term memory
    long_term: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    /// Memory configuration
    config: MemoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum short-term memory entries
    pub short_term_capacity: usize,
    /// Maximum working memory entries
    pub working_capacity: usize,
    /// Relevance threshold for promotion
    pub promotion_threshold: f32,
    /// Decay factor for relevance
    pub decay_factor: f32,
    /// Enable semantic search
    pub enable_semantic_search: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            short_term_capacity: 100,
            working_capacity: 1000,
            promotion_threshold: 0.7,
            decay_factor: 0.95,
            enable_semantic_search: false,
        }
    }
}

impl MemoryBank {
    /// Create a new memory bank
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            short_term: Arc::new(RwLock::new(VecDeque::new())),
            working: Arc::new(RwLock::new(HashMap::new())),
            long_term: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Store a memory entry
    pub async fn store(&self, entry: MemoryEntry) -> Result<()> {
        match entry.memory_type {
            MemoryType::ShortTerm => {
                let mut short_term = self.short_term.write().await;
                
                // Maintain capacity
                while short_term.len() >= self.config.short_term_capacity {
                    if let Some(old_entry) = short_term.pop_front() {
                        // Consider promoting high-relevance entries
                        if old_entry.relevance > self.config.promotion_threshold {
                            self.promote_to_working(old_entry).await?;
                        }
                    }
                }
                
                short_term.push_back(entry);
            }
            MemoryType::Working => {
                let mut working = self.working.write().await;
                
                // Check capacity
                if working.len() >= self.config.working_capacity {
                    // Remove least relevant entry
                    if let Some(least_relevant_id) = self.find_least_relevant(&working).await {
                        working.remove(&least_relevant_id);
                    }
                }
                
                working.insert(entry.id.clone(), entry);
            }
            MemoryType::LongTerm | MemoryType::Semantic | MemoryType::Episodic => {
                let mut long_term = self.long_term.write().await;
                long_term.insert(entry.id.clone(), entry);
            }
        }
        
        Ok(())
    }
    
    /// Retrieve a memory by ID
    pub async fn retrieve(&self, id: &str) -> Result<Option<MemoryEntry>> {
        // Check all memory stores
        {
            let short_term = self.short_term.read().await;
            if let Some(entry) = short_term.iter().find(|e| e.id == id) {
                return Ok(Some(self.update_access(entry.clone()).await));
            }
        }
        
        {
            let working = self.working.read().await;
            if let Some(entry) = working.get(id) {
                return Ok(Some(self.update_access(entry.clone()).await));
            }
        }
        
        {
            let long_term = self.long_term.read().await;
            if let Some(entry) = long_term.get(id) {
                return Ok(Some(self.update_access(entry.clone()).await));
            }
        }
        
        Ok(None)
    }
    
    /// Search memories by relevance
    pub async fn search(
        &self,
        query: &str,
        memory_types: Vec<MemoryType>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>> {
        let mut results = Vec::new();
        
        // Simple text-based search (in production, use embeddings)
        for memory_type in memory_types {
            match memory_type {
                MemoryType::ShortTerm => {
                    let short_term = self.short_term.read().await;
                    for entry in short_term.iter() {
                        if entry.content.to_lowercase().contains(&query.to_lowercase()) {
                            results.push(entry.clone());
                        }
                    }
                }
                MemoryType::Working => {
                    let working = self.working.read().await;
                    for entry in working.values() {
                        if entry.content.to_lowercase().contains(&query.to_lowercase()) {
                            results.push(entry.clone());
                        }
                    }
                }
                _ => {
                    let long_term = self.long_term.read().await;
                    for entry in long_term.values() {
                        if entry.memory_type == memory_type &&
                           entry.content.to_lowercase().contains(&query.to_lowercase()) {
                            results.push(entry.clone());
                        }
                    }
                }
            }
        }
        
        // Sort by relevance and limit
        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        results.truncate(limit);
        
        Ok(results)
    }
    
    /// Consolidate memories (merge similar ones)
    pub async fn consolidate(&self) -> Result<usize> {
        let mut consolidated = 0;
        
        // Consolidate short-term memories
        let mut short_term = self.short_term.write().await;
        let mut unique_entries = Vec::new();
        let mut seen_content = std::collections::HashSet::new();
        
        for entry in short_term.drain(..) {
            let content_hash = self.hash_content(&entry.content);
            if !seen_content.contains(&content_hash) {
                seen_content.insert(content_hash);
                unique_entries.push(entry);
            } else {
                consolidated += 1;
            }
        }
        
        for entry in unique_entries {
            short_term.push_back(entry);
        }
        
        Ok(consolidated)
    }
    
    /// Apply decay to relevance scores
    pub async fn apply_decay(&self) -> Result<()> {
        // Decay short-term memory
        {
            let mut short_term = self.short_term.write().await;
            for entry in short_term.iter_mut() {
                entry.relevance *= self.config.decay_factor;
            }
        }
        
        // Decay working memory
        {
            let mut working = self.working.write().await;
            for entry in working.values_mut() {
                entry.relevance *= self.config.decay_factor;
            }
        }
        
        Ok(())
    }
    
    /// Get memory statistics
    pub async fn get_stats(&self) -> MemoryStats {
        let short_term_count = self.short_term.read().await.len();
        let working_count = self.working.read().await.len();
        let long_term_count = self.long_term.read().await.len();
        
        MemoryStats {
            short_term_count,
            working_count,
            long_term_count,
            total_count: short_term_count + working_count + long_term_count,
        }
    }
    
    /// Promote entry to working memory
    async fn promote_to_working(&self, mut entry: MemoryEntry) -> Result<()> {
        entry.memory_type = MemoryType::Working;
        entry.relevance = (entry.relevance + 0.1).min(1.0); // Boost relevance
        
        let mut working = self.working.write().await;
        working.insert(entry.id.clone(), entry);
        
        Ok(())
    }
    
    /// Find least relevant entry in a memory store
    async fn find_least_relevant(&self, store: &HashMap<String, MemoryEntry>) -> Option<String> {
        store.iter()
            .min_by(|a, b| a.1.relevance.partial_cmp(&b.1.relevance).unwrap())
            .map(|(id, _)| id.clone())
    }
    
    /// Update access information for an entry
    async fn update_access(&self, mut entry: MemoryEntry) -> MemoryEntry {
        entry.access_count += 1;
        entry.last_accessed = std::time::SystemTime::now();
        entry.relevance = (entry.relevance + 0.05).min(1.0); // Slight boost for access
        entry
    }
    
    /// Hash content for deduplication
    fn hash_content(&self, content: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub short_term_count: usize,
    pub working_count: usize,
    pub long_term_count: usize,
    pub total_count: usize,
}

/// Memory-augmented context builder
pub struct MemoryAugmentedContext {
    memory_bank: Arc<MemoryBank>,
}

impl MemoryAugmentedContext {
    /// Create a new memory-augmented context
    pub fn new(memory_bank: Arc<MemoryBank>) -> Self {
        Self { memory_bank }
    }
    
    /// Build context with relevant memories
    pub async fn build_context(
        &self,
        current_input: &str,
        base_context: &str,
        max_memories: usize,
    ) -> Result<String> {
        // Search for relevant memories
        let memories = self.memory_bank.search(
            current_input,
            vec![MemoryType::Working, MemoryType::LongTerm, MemoryType::Semantic],
            max_memories,
        ).await?;
        
        let mut context = String::new();
        
        // Add relevant memories as context
        if !memories.is_empty() {
            context.push_str("Relevant context from memory:\n");
            for memory in memories {
                context.push_str(&format!("- {}\n", memory.content));
            }
            context.push_str("\n");
        }
        
        // Add base context
        context.push_str(base_context);
        
        Ok(context)
    }
    
    /// Extract and store facts from a response
    pub async fn extract_and_store_facts(&self, response: &str) -> Result<usize> {
        // Simple fact extraction (in production, use NLP)
        let facts = self.extract_facts(response);
        let mut stored = 0;
        
        for fact in facts {
            let entry = MemoryEntry {
                id: uuid::Uuid::new_v4().to_string(),
                memory_type: MemoryType::Semantic,
                content: fact,
                embedding: None,
                relevance: 0.8,
                access_count: 0,
                created_at: std::time::SystemTime::now(),
                last_accessed: std::time::SystemTime::now(),
                metadata: HashMap::new(),
            };
            
            self.memory_bank.store(entry).await?;
            stored += 1;
        }
        
        Ok(stored)
    }
    
    /// Extract facts from text (placeholder)
    fn extract_facts(&self, text: &str) -> Vec<String> {
        // In production, use NLP to extract facts
        // For now, extract sentences that look like facts
        text.split('.')
            .filter(|s| s.len() > 20 && (s.contains(" is ") || s.contains(" are ")))
            .map(|s| s.trim().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_bank() {
        let bank = MemoryBank::new(MemoryConfig::default());
        
        // Store short-term memory
        let entry = MemoryEntry {
            id: "test-1".to_string(),
            memory_type: MemoryType::ShortTerm,
            content: "Test memory".to_string(),
            embedding: None,
            relevance: 0.5,
            access_count: 0,
            created_at: std::time::SystemTime::now(),
            last_accessed: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };
        
        bank.store(entry).await.unwrap();
        
        // Retrieve memory
        let retrieved = bank.retrieve("test-1").await.unwrap().unwrap();
        assert_eq!(retrieved.content, "Test memory");
        assert_eq!(retrieved.access_count, 1);
    }
    
    #[tokio::test]
    async fn test_memory_search() {
        let bank = MemoryBank::new(MemoryConfig::default());
        
        // Store some memories
        for i in 0..3 {
            let entry = MemoryEntry {
                id: format!("test-{}", i),
                memory_type: MemoryType::Working,
                content: format!("Memory about Rust programming {}", i),
                embedding: None,
                relevance: 0.5 + (i as f32 * 0.1),
                access_count: 0,
                created_at: std::time::SystemTime::now(),
                last_accessed: std::time::SystemTime::now(),
                metadata: HashMap::new(),
            };
            bank.store(entry).await.unwrap();
        }
        
        // Search memories
        let results = bank.search("Rust", vec![MemoryType::Working], 2).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].relevance > results[1].relevance);
    }
}