use crate::ai::{Result, EnhancementOptions, EnhancedText};
use crate::ai::services::model_service::AIModel;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Configuration for model warmup and caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupConfig {
    /// Number of warmup iterations
    pub warmup_iterations: usize,
    /// Text samples for warmup
    pub warmup_samples: Vec<String>,
    /// Cache size for inference results
    pub cache_size: usize,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable result caching
    pub enable_caching: bool,
    /// Precompute common phrases
    pub precompute_common: bool,
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 3,
            warmup_samples: vec![
                "Hello world".to_string(),
                "The quick brown fox jumps over the lazy dog".to_string(),
                "This is a test sentence for model warmup".to_string(),
            ],
            cache_size: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            enable_caching: true,
            precompute_common: true,
        }
    }
}

/// Cached inference result
#[derive(Debug, Clone)]
struct CachedResult {
    result: EnhancedText,
    timestamp: Instant,
    access_count: usize,
}

/// Model warmup and caching manager
pub struct ModelWarmupCache {
    config: WarmupConfig,
    /// Cache of inference results
    cache: Arc<RwLock<HashMap<String, CachedResult>>>,
    /// LRU tracking
    lru_queue: Arc<Mutex<VecDeque<String>>>,
    /// Common phrases for precomputation
    common_phrases: Vec<String>,
    /// Warmup statistics
    warmup_stats: Arc<RwLock<WarmupStatistics>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WarmupStatistics {
    pub total_warmup_time_ms: u64,
    pub average_latency_ms: f64,
    pub min_latency_ms: u64,
    pub max_latency_ms: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
}

impl ModelWarmupCache {
    /// Create a new warmup cache manager
    pub fn new(config: WarmupConfig) -> Self {
        let common_phrases = if config.precompute_common {
            Self::load_common_phrases()
        } else {
            Vec::new()
        };
        
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            lru_queue: Arc::new(Mutex::new(VecDeque::new())),
            common_phrases,
            warmup_stats: Arc::new(RwLock::new(WarmupStatistics::default())),
        }
    }
    
    /// Perform model warmup
    pub async fn warmup_model(&self, model: &Arc<dyn AIModel>) -> Result<WarmupStatistics> {
        log::info!("Starting model warmup for {}", model.model_id());
        let start_time = Instant::now();
        let mut latencies = Vec::new();
        
        // Default enhancement options for warmup
        let options = EnhancementOptions {
            correct_grammar: true,
            improve_clarity: false,
            preserve_style: true,
            detect_intent: false,
            format_markdown: false,
            improve_punctuation: true,
            confidence_threshold: 0.7,
        };
        
        // Run warmup iterations
        for i in 0..self.config.warmup_iterations {
            for sample in &self.config.warmup_samples {
                let iter_start = Instant::now();
                
                match model.enhance_text(sample, &options).await {
                    Ok(result) => {
                        let latency = iter_start.elapsed().as_millis() as u64;
                        latencies.push(latency);
                        
                        // Cache the result
                        if self.config.enable_caching {
                            self.cache_result(sample.clone(), result).await?;
                        }
                        
                        log::debug!("Warmup iteration {} completed in {}ms", i + 1, latency);
                    }
                    Err(e) => {
                        log::warn!("Warmup iteration {} failed: {}", i + 1, e);
                    }
                }
            }
        }
        
        // Precompute common phrases
        if self.config.precompute_common {
            log::info!("Precomputing {} common phrases", self.common_phrases.len());
            for phrase in &self.common_phrases {
                match model.enhance_text(phrase, &options).await {
                    Ok(result) => {
                        if self.config.enable_caching {
                            self.cache_result(phrase.clone(), result).await?;
                        }
                    }
                    Err(e) => {
                        log::debug!("Failed to precompute phrase: {}", e);
                    }
                }
            }
        }
        
        // Calculate statistics
        let total_time = start_time.elapsed().as_millis() as u64;
        let avg_latency = if !latencies.is_empty() {
            latencies.iter().sum::<u64>() as f64 / latencies.len() as f64
        } else {
            0.0
        };
        let min_latency = latencies.iter().min().copied().unwrap_or(0);
        let max_latency = latencies.iter().max().copied().unwrap_or(0);
        
        let stats = WarmupStatistics {
            total_warmup_time_ms: total_time,
            average_latency_ms: avg_latency,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            cache_hits: 0,
            cache_misses: 0,
            evictions: 0,
        };
        
        // Update stats
        {
            let mut warmup_stats = self.warmup_stats.write().await;
            *warmup_stats = stats.clone();
        }
        
        log::info!(
            "Model warmup completed in {}ms (avg: {:.2}ms, min: {}ms, max: {}ms)",
            total_time, avg_latency, min_latency, max_latency
        );
        
        Ok(stats)
    }
    
    /// Get cached result if available
    pub async fn get_cached(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Option<EnhancedText> {
        if !self.config.enable_caching {
            return None;
        }
        
        let cache_key = self.generate_cache_key(text, options);
        
        let mut cache = self.cache.write().await;
        if let Some(cached) = cache.get_mut(&cache_key) {
            // Check TTL
            if cached.timestamp.elapsed() > Duration::from_secs(self.config.cache_ttl_seconds) {
                // Expired
                cache.remove(&cache_key);
                self.update_stats_miss().await;
                return None;
            }
            
            // Update access count and LRU
            cached.access_count += 1;
            self.update_lru(&cache_key).await;
            self.update_stats_hit().await;
            
            Some(cached.result.clone())
        } else {
            self.update_stats_miss().await;
            None
        }
    }
    
    /// Cache an inference result
    pub async fn cache_result(
        &self,
        text: String,
        result: EnhancedText,
    ) -> Result<()> {
        if !self.config.enable_caching {
            return Ok(());
        }
        
        let cache_key = self.generate_cache_key(&text, &EnhancementOptions::default());
        
        // Check cache size and evict if necessary
        {
            let cache = self.cache.read().await;
            if cache.len() >= self.config.cache_size {
                drop(cache);
                self.evict_lru().await?;
            }
        }
        
        // Add to cache
        let cached = CachedResult {
            result,
            timestamp: Instant::now(),
            access_count: 1,
        };
        
        let mut cache = self.cache.write().await;
        cache.insert(cache_key.clone(), cached);
        
        // Update LRU
        self.update_lru(&cache_key).await;
        
        Ok(())
    }
    
    /// Generate cache key from text and options
    fn generate_cache_key(&self, text: &str, options: &EnhancementOptions) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        options.correct_grammar.hash(&mut hasher);
        options.improve_clarity.hash(&mut hasher);
        options.preserve_style.hash(&mut hasher);
        options.detect_intent.hash(&mut hasher);
        options.format_markdown.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Update LRU queue
    async fn update_lru(&self, key: &str) {
        let mut queue = self.lru_queue.lock().await;
        
        // Remove if exists
        queue.retain(|k| k != key);
        
        // Add to front
        queue.push_front(key.to_string());
    }
    
    /// Evict least recently used entry
    async fn evict_lru(&self) -> Result<()> {
        let mut queue = self.lru_queue.lock().await;
        
        if let Some(key) = queue.pop_back() {
            let mut cache = self.cache.write().await;
            cache.remove(&key);
            
            let mut stats = self.warmup_stats.write().await;
            stats.evictions += 1;
            
            log::debug!("Evicted cache entry: {}", key);
        }
        
        Ok(())
    }
    
    /// Update cache hit statistics
    async fn update_stats_hit(&self) {
        let mut stats = self.warmup_stats.write().await;
        stats.cache_hits += 1;
    }
    
    /// Update cache miss statistics
    async fn update_stats_miss(&self) {
        let mut stats = self.warmup_stats.write().await;
        stats.cache_misses += 1;
    }
    
    /// Get current statistics
    pub async fn get_statistics(&self) -> WarmupStatistics {
        self.warmup_stats.read().await.clone()
    }
    
    /// Clear cache
    pub async fn clear_cache(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.clear();
        
        let mut queue = self.lru_queue.lock().await;
        queue.clear();
        
        log::info!("Cache cleared");
        Ok(())
    }
    
    /// Load common phrases for precomputation
    fn load_common_phrases() -> Vec<String> {
        vec![
            // Common greetings
            "Hello", "Hi", "Good morning", "Good afternoon", "Good evening",
            "How are you", "Nice to meet you", "Thank you", "Thanks",
            
            // Common questions
            "What is", "Where is", "When is", "How do I", "Can you",
            "Could you please", "Would you mind", "Is it possible",
            
            // Common statements
            "I think", "I believe", "In my opinion", "I agree", "I disagree",
            "Let me know", "Please let me know", "Looking forward to",
            
            // Business phrases
            "Best regards", "Kind regards", "Sincerely", "Yours truly",
            "Please find attached", "As discussed", "Following up on",
            
            // Technical phrases
            "The error is", "The problem is", "The solution is",
            "How to fix", "How to solve", "How to implement",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }
}

/// Cached model wrapper that adds caching to any AI model
pub struct CachedModel {
    model: Arc<dyn AIModel>,
    cache: Arc<ModelWarmupCache>,
}

impl CachedModel {
    /// Create a new cached model
    pub async fn new(
        model: Arc<dyn AIModel>,
        config: WarmupConfig,
    ) -> Result<Self> {
        let cache = Arc::new(ModelWarmupCache::new(config));
        
        // Perform warmup
        cache.warmup_model(&model).await?;
        
        Ok(Self { model, cache })
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> WarmupStatistics {
        self.cache.get_statistics().await
    }
    
    /// Clear cache
    pub async fn clear_cache(&self) -> Result<()> {
        self.cache.clear_cache().await
    }
}

#[async_trait::async_trait]
impl AIModel for CachedModel {
    fn model_id(&self) -> &str {
        self.model.model_id()
    }
    
    fn memory_usage(&self) -> u64 {
        // Add cache memory estimate
        let cache_overhead = 1024 * 1024; // 1MB estimate for cache
        self.model.memory_usage() + cache_overhead
    }
    
    async fn enhance_text(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Result<EnhancedText> {
        use crate::ai::telemetry::metrics;
        
        // Get metrics collector for cache tracking
        let meter = opentelemetry::global::meter("bestme-ai");
        let aggregator = metrics::get_aggregator(&meter);
        let metrics_collector = aggregator.get_model_collector(self.model_id(), &meter);
        
        // Check cache first
        if let Some(cached) = self.cache.get_cached(text, options).await {
            log::debug!("Cache hit for text: {}", text);
            metrics_collector.record_cache_hit();
            return Ok(cached);
        }
        
        // Cache miss - run inference
        metrics_collector.record_cache_miss();
        let result = self.model.enhance_text(text, options).await?;
        
        // Cache the result
        self.cache.cache_result(text.to_string(), result.clone()).await?;
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_warmup_config_default() {
        let config = WarmupConfig::default();
        assert_eq!(config.warmup_iterations, 3);
        assert_eq!(config.cache_size, 1000);
        assert!(config.enable_caching);
    }
    
    #[tokio::test]
    async fn test_cache_key_generation() {
        let cache = ModelWarmupCache::new(WarmupConfig::default());
        let options = EnhancementOptions::default();
        
        let key1 = cache.generate_cache_key("test", &options);
        let key2 = cache.generate_cache_key("test", &options);
        let key3 = cache.generate_cache_key("different", &options);
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}