use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use serde::{Serialize, Deserialize};

use crate::ai::services::{
    ModelService,
    text_analyzer::{TextAnalyzer, TextCharacteristics},
    model_capability_matcher::{ModelCapabilityMatcher, CapabilityScore, ModelRequirements},
};
use crate::ai::models::ModelMetadata;
use crate::ai::telemetry::metrics::ModelMetricsCollector as MetricsCollector;

/// Automatic model selection service
pub struct ModelSelector {
    model_service: Arc<ModelService>,
    text_analyzer: TextAnalyzer,
    capability_matcher: ModelCapabilityMatcher,
    metrics_collector: Arc<MetricsCollector>,
    selection_cache: Arc<SelectionCache>,
    config: ModelSelectorConfig,
}

/// Configuration for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectorConfig {
    /// Enable performance-based selection
    pub use_performance_metrics: bool,
    /// Minimum score threshold for selection
    pub min_score_threshold: f32,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum cache size
    pub max_cache_entries: usize,
    /// Weight for capability score (0-1)
    pub capability_weight: f32,
    /// Weight for performance score (0-1)
    pub performance_weight: f32,
    /// Prefer loaded models
    pub prefer_loaded_models: bool,
}

impl Default for ModelSelectorConfig {
    fn default() -> Self {
        Self {
            use_performance_metrics: true,
            min_score_threshold: 0.6,
            cache_ttl_seconds: 3600, // 1 hour
            max_cache_entries: 1000,
            capability_weight: 0.6,
            performance_weight: 0.4,
            prefer_loaded_models: true,
        }
    }
}

/// Selection result with detailed scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    pub selected_model: ModelMetadata,
    pub capability_score: CapabilityScore,
    pub performance_score: Option<f32>,
    pub combined_score: f32,
    pub alternatives: Vec<(ModelMetadata, f32)>,
    pub selection_reason: String,
    pub cached: bool,
}

/// Performance metrics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub avg_latency_ms: f32,
    pub p95_latency_ms: f32,
    pub avg_tokens_per_second: f32,
    pub error_rate: f32,
    pub memory_usage_mb: f32,
    #[serde(skip, default = "Instant::now")]
    pub last_updated: Instant,
}

/// Cache for selection decisions
struct SelectionCache {
    cache: DashMap<String, (SelectionResult, Instant)>,
    ttl: Duration,
    max_size: usize,
}

impl SelectionCache {
    fn new(ttl_seconds: u64, max_size: usize) -> Self {
        Self {
            cache: DashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
            max_size,
        }
    }
    
    fn get(&self, key: &str) -> Option<SelectionResult> {
        self.cache.get(key).and_then(|entry| {
            if entry.1.elapsed() < self.ttl {
                let mut result = entry.0.clone();
                result.cached = true;
                Some(result)
            } else {
                // Remove expired entry
                drop(entry);
                self.cache.remove(key);
                None
            }
        })
    }
    
    fn put(&self, key: String, result: SelectionResult) {
        // Simple LRU-like behavior: remove oldest if at capacity
        if self.cache.len() >= self.max_size {
            if let Some(oldest_key) = self.cache.iter()
                .min_by_key(|entry| entry.value().1)
                .map(|entry| entry.key().clone()) {
                self.cache.remove(&oldest_key);
            }
        }
        
        self.cache.insert(key, (result, Instant::now()));
    }
    
    fn clear(&self) {
        self.cache.clear();
    }
}

impl ModelSelector {
    pub fn new(
        model_service: Arc<ModelService>,
        metrics_collector: Arc<MetricsCollector>,
        config: ModelSelectorConfig,
    ) -> Self {
        let cache = Arc::new(SelectionCache::new(
            config.cache_ttl_seconds,
            config.max_cache_entries,
        ));
        
        Self {
            model_service,
            text_analyzer: TextAnalyzer::new(),
            capability_matcher: ModelCapabilityMatcher::new(),
            metrics_collector,
            selection_cache: cache,
            config,
        }
    }
    
    /// Select the best model for the given text
    pub async fn select_model(&self, text: &str) -> Result<SelectionResult, String> {
        // Check cache first
        let cache_key = self.generate_cache_key(text);
        if let Some(cached_result) = self.selection_cache.get(&cache_key) {
            return Ok(cached_result);
        }
        
        // Analyze text characteristics
        let characteristics = self.text_analyzer.analyze(text);
        
        // Derive model requirements
        let requirements = self.capability_matcher.derive_requirements(&characteristics);
        
        // Get available models
        let available_models = self.model_service.get_registry()
            .list_downloaded_models()
            .await
            .map_err(|e| format!("Failed to list models: {}", e))?;
        
        if available_models.is_empty() {
            return Err("No models available for selection".to_string());
        }
        
        // Get model metadata
        let mut models_metadata = Vec::new();
        for model_id in &available_models {
            if let Ok(metadata) = self.model_service.get_registry()
                .get_model_metadata(model_id).await {
                models_metadata.push(metadata);
            }
        }
        
        if models_metadata.is_empty() {
            return Err("No model metadata available".to_string());
        }
        
        // Score models
        let scored_models = self.score_models(
            &requirements,
            &characteristics,
            &models_metadata,
        ).await?;
        
        // Select best model
        let best_model = scored_models.first()
            .ok_or("No suitable model found")?;
        
        if best_model.2 < self.config.min_score_threshold {
            return Err(format!(
                "No model meets minimum score threshold of {}",
                self.config.min_score_threshold
            ));
        }
        
        // Create result
        let result = SelectionResult {
            selected_model: best_model.0.clone(),
            capability_score: best_model.1.clone(),
            performance_score: best_model.3,
            combined_score: best_model.2,
            alternatives: scored_models.iter()
                .skip(1)
                .take(3)
                .map(|(model, _, score, _)| (model.clone(), *score))
                .collect(),
            selection_reason: self.generate_selection_reason(
                &best_model.0,
                &requirements,
                &characteristics,
                best_model.2,
            ),
            cached: false,
        };
        
        // Cache the result
        self.selection_cache.put(cache_key, result.clone());
        
        // Record metrics
        // TODO: Add model selection metrics
        // self.metrics_collector.record_model_selection(
        //     &best_model.0.id,
        //     best_model.2,
        // ).await;
        
        Ok(result)
    }
    
    /// Score models based on capabilities and performance
    async fn score_models(
        &self,
        requirements: &ModelRequirements,
        _characteristics: &TextCharacteristics,
        models: &[ModelMetadata],
    ) -> Result<Vec<(ModelMetadata, CapabilityScore, f32, Option<f32>)>, String> {
        let mut scored_models = Vec::new();
        
        for model in models {
            // Calculate capability score
            let capability_score = self.capability_matcher
                .calculate_match_score(requirements, model);
            
            // Calculate performance score if enabled
            let performance_score = if self.config.use_performance_metrics {
                self.calculate_performance_score(model, requirements).await
            } else {
                None
            };
            
            // Calculate combined score
            let mut combined_score = capability_score.overall * self.config.capability_weight;
            
            if let Some(perf_score) = performance_score {
                combined_score += perf_score * self.config.performance_weight;
            } else {
                // If no performance data, use capability score only
                combined_score = capability_score.overall;
            }
            
            // Bonus for already loaded models
            if self.config.prefer_loaded_models {
                let loaded_models = self.model_service.list_loaded_models().await;
                if loaded_models.iter().any(|m| m.model_id == model.id) {
                    combined_score *= 1.1; // 10% bonus
                }
            }
            
            scored_models.push((
                model.clone(),
                capability_score,
                combined_score.min(1.0),
                performance_score,
            ));
        }
        
        // Sort by combined score (descending)
        scored_models.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        
        Ok(scored_models)
    }
    
    /// Calculate performance score for a model
    async fn calculate_performance_score(
        &self,
        model: &ModelMetadata,
        requirements: &ModelRequirements,
    ) -> Option<f32> {
        // Get performance metrics from model service
        // TODO: Implement get_model_performance in ModelService
        // match self.model_service.get_model_performance(&model.id).await {
        //     Ok(metrics) => {
        {
            // Use default performance metrics from model metadata
            let metrics = &model.performance;
                let mut score = 0.0;
                let mut weight_sum = 0.0;
                
                // Latency score (lower is better)
                if metrics.avg_latency_ms > 0.0 {
                    let latency_score = (requirements.max_latency_ms as f32 / metrics.avg_latency_ms as f32)
                        .min(1.0)
                        .max(0.0);
                    score += latency_score * 0.4;
                    weight_sum += 0.4;
                }
                
                // Throughput score (higher is better)
                if metrics.tokens_per_second > 0.0 {
                    let throughput_score = (metrics.tokens_per_second / 100.0).min(1.0) as f32;
                    score += throughput_score * 0.3;
                    weight_sum += 0.3;
                }
                
                // Error rate score (lower is better)
                // TODO: Add error_rate to PerformanceProfile
                // let error_score = 1.0 - metrics.error_rate.min(1.0);
                // score += error_score * 0.3;
                // weight_sum += 0.3;
                
                if weight_sum > 0.0 {
                    Some(score / weight_sum)
                } else {
                    None
                }
        }
        // }
        // Err(_) => None,
        // }
    }
    
    /// Generate cache key for text
    fn generate_cache_key(&self, text: &str) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        
        // Include first 100 chars of text
        let text_prefix = if text.len() > 100 {
            &text[..100]
        } else {
            text
        };
        text_prefix.hash(&mut hasher);
        
        // Include config that affects selection
        self.config.min_score_threshold.to_bits().hash(&mut hasher);
        self.config.capability_weight.to_bits().hash(&mut hasher);
        self.config.performance_weight.to_bits().hash(&mut hasher);
        
        format!("selection_{:x}", hasher.finish())
    }
    
    /// Generate human-readable selection reason
    fn generate_selection_reason(
        &self,
        model: &ModelMetadata,
        requirements: &ModelRequirements,
        characteristics: &TextCharacteristics,
        score: f32,
    ) -> String {
        let mut reasons = Vec::new();
        
        // Score-based reason
        if score > 0.9 {
            reasons.push("Excellent match for your text");
        } else if score > 0.8 {
            reasons.push("Very good match for your text");
        } else if score > 0.7 {
            reasons.push("Good match for your text");
        } else {
            reasons.push("Best available option");
        }
        
        // Domain-specific reason
        match characteristics.domain {
            crate::ai::services::text_analyzer::TextDomain::Code => {
                reasons.push("optimized for code understanding");
            }
            crate::ai::services::text_analyzer::TextDomain::Medical => {
                reasons.push("specialized in medical terminology");
            }
            crate::ai::services::text_analyzer::TextDomain::Legal => {
                reasons.push("trained on legal documents");
            }
            crate::ai::services::text_analyzer::TextDomain::Academic |
            crate::ai::services::text_analyzer::TextDomain::Scientific => {
                reasons.push("excels at technical content");
            }
            _ => {}
        }
        
        // Performance reason
        match model.performance_class.as_str() {
            "fast" => reasons.push("with fast response times"),
            "quality" => reasons.push("with high accuracy"),
            "balanced" => reasons.push("with balanced performance"),
            _ => {}
        }
        
        // Complexity reason
        if requirements.complexity_tier >= 4 {
            reasons.push("capable of handling complex text");
        }
        
        format!("{} - {}", model.name, reasons.join(", "))
    }
    
    /// Get selection history
    pub async fn get_selection_history(&self) -> Vec<(String, String, f32)> {
        // TODO: Implement model selection history in metrics
        // self.metrics_collector.get_model_selection_history().await
        //     .into_iter()
        //     .map(|(model_id, score, _)| {
        vec![] // Return empty for now
        //         let reason = format!("Selected with score {:.2}", score);
        //         (model_id, reason, score)
        //     })
        //     .collect()
    }
    
    /// Clear selection cache
    pub fn clear_cache(&self) {
        self.selection_cache.clear();
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: ModelSelectorConfig) {
        self.config = config;
        // Clear cache when config changes
        self.selection_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::RwLock;
    
    #[tokio::test]
    async fn test_model_selection() {
        // This would require setting up mock services
        // Implementation depends on the actual service structure
    }
    
    #[tokio::test]
    async fn test_cache_key_generation() {
        use opentelemetry::global;
        
        let model_service = Arc::new(ModelService::new().await.unwrap());
        let meter = global::meter("test");
        let metrics = Arc::new(MetricsCollector::new("test-model".to_string(), meter));
        let selector = ModelSelector::new(model_service, metrics, Default::default());
        
        let key1 = selector.generate_cache_key("This is a test");
        let key2 = selector.generate_cache_key("This is a test");
        let key3 = selector.generate_cache_key("Different text");
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}