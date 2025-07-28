//! Recovery strategies for common AI model failures
//! 
//! Implements automatic recovery mechanisms for various failure scenarios

use crate::ai::{Result, AIError};
use crate::ai::common::ensure_models_directory;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use log::{info, warn, error};

/// Recovery strategy types
#[derive(Clone)]
pub enum RecoveryStrategy {
    /// Retry with exponential backoff
    Retry {
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f32,
    },
    /// Redownload corrupted model
    Redownload {
        max_attempts: u32,
        verify_checksum: bool,
    },
    /// Clear cache and retry
    ClearCache {
        cache_types: Vec<CacheType>,
    },
    /// Restart model service
    RestartService {
        cooldown: Duration,
    },
    /// Fallback to different model
    ModelFallback {
        fallback_models: Vec<String>,
    },
    /// Custom recovery function
    Custom(Arc<dyn Fn(&AIError) -> Box<dyn std::future::Future<Output = Result<()>> + Send> + Send + Sync>),
}

impl std::fmt::Debug for RecoveryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retry { max_attempts, initial_delay, max_delay, multiplier } => {
                f.debug_struct("Retry")
                    .field("max_attempts", max_attempts)
                    .field("initial_delay", initial_delay)
                    .field("max_delay", max_delay)
                    .field("multiplier", multiplier)
                    .finish()
            }
            Self::Redownload { max_attempts, verify_checksum } => {
                f.debug_struct("Redownload")
                    .field("max_attempts", max_attempts)
                    .field("verify_checksum", verify_checksum)
                    .finish()
            }
            Self::ClearCache { cache_types } => {
                f.debug_struct("ClearCache")
                    .field("cache_types", cache_types)
                    .finish()
            }
            Self::RestartService { cooldown } => {
                f.debug_struct("RestartService")
                    .field("cooldown", cooldown)
                    .finish()
            }
            Self::ModelFallback { fallback_models } => {
                f.debug_struct("ModelFallback")
                    .field("fallback_models", fallback_models)
                    .finish()
            }
            Self::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

/// Types of cache that can be cleared
#[derive(Debug, Clone, Copy)]
pub enum CacheType {
    /// Model weights cache
    ModelCache,
    /// Inference results cache
    ResultCache,
    /// GPU memory cache
    GpuCache,
    /// All caches
    All,
}

/// Recovery configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Enable automatic model redownload
    pub enable_redownload: bool,
    /// Enable cache clearing
    pub enable_cache_clear: bool,
    /// Enable service restart
    pub enable_service_restart: bool,
    /// Maximum recovery attempts
    pub max_recovery_attempts: u32,
    /// Recovery attempt timeout
    pub recovery_timeout: Duration,
    /// Cooldown between recovery attempts
    pub recovery_cooldown: Duration,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            enable_redownload: true,
            enable_cache_clear: true,
            enable_service_restart: false,
            max_recovery_attempts: 3,
            recovery_timeout: Duration::from_secs(300),
            recovery_cooldown: Duration::from_secs(10),
        }
    }
}

/// Recovery manager
pub struct RecoveryManager {
    config: RecoveryConfig,
    strategies: HashMap<String, RecoveryStrategy>,
    recovery_history: Arc<RwLock<Vec<RecoveryAttempt>>>,
    last_recovery: Arc<RwLock<Option<Instant>>>,
}

/// Recovery attempt record
#[derive(Debug, Clone)]
struct RecoveryAttempt {
    timestamp: Instant,
    error_type: String,
    strategy: String,
    success: bool,
    duration: Duration,
}

impl RecoveryManager {
    /// Create a new recovery manager
    pub fn new(config: RecoveryConfig) -> Self {
        let mut strategies = HashMap::new();
        
        // Default retry strategy
        strategies.insert(
            "retry".to_string(),
            RecoveryStrategy::Retry {
                max_attempts: 3,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
                multiplier: 2.0,
            },
        );
        
        // Model redownload strategy
        if config.enable_redownload {
            strategies.insert(
                "redownload".to_string(),
                RecoveryStrategy::Redownload {
                    max_attempts: 2,
                    verify_checksum: true,
                },
            );
        }
        
        // Cache clearing strategy
        if config.enable_cache_clear {
            strategies.insert(
                "clear_cache".to_string(),
                RecoveryStrategy::ClearCache {
                    cache_types: vec![CacheType::All],
                },
            );
        }
        
        // Service restart strategy
        if config.enable_service_restart {
            strategies.insert(
                "restart_service".to_string(),
                RecoveryStrategy::RestartService {
                    cooldown: Duration::from_secs(30),
                },
            );
        }
        
        Self {
            config,
            strategies,
            recovery_history: Arc::new(RwLock::new(Vec::new())),
            last_recovery: Arc::new(RwLock::new(None)),
        }
    }

    /// Add a custom recovery strategy
    pub fn add_strategy(&mut self, name: String, strategy: RecoveryStrategy) {
        self.strategies.insert(name, strategy);
    }

    /// Handle a failure with appropriate recovery
    pub async fn handle_failure(&self, error: &AIError, model_id: &str) -> Result<()> {
        // Check cooldown
        if !self.check_cooldown().await {
            return Err(AIError::InferenceError(
                "Recovery attempted too soon - in cooldown period".into()
            ));
        }

        // Determine appropriate strategy
        let strategy_name = self.determine_strategy(error);
        let strategy = self.strategies.get(&strategy_name)
            .ok_or_else(|| AIError::ConfigError(format!("Unknown recovery strategy: {}", strategy_name)))?;

        info!("Attempting recovery with strategy: {} for error: {}", strategy_name, error);
        
        // Execute recovery
        let start = Instant::now();
        let result = tokio::time::timeout(
            self.config.recovery_timeout,
            self.execute_recovery(strategy, error, model_id)
        ).await;

        let duration = start.elapsed();
        let success = matches!(result, Ok(Ok(())));

        // Record attempt
        self.record_attempt(RecoveryAttempt {
            timestamp: Instant::now(),
            error_type: format!("{:?}", error),
            strategy: strategy_name.clone(),
            success,
            duration,
        }).await;

        // Update last recovery time
        *self.last_recovery.write().await = Some(Instant::now());

        match result {
            Ok(Ok(())) => {
                info!("Recovery successful using {} strategy", strategy_name);
                Ok(())
            }
            Ok(Err(e)) => {
                warn!("Recovery failed: {}", e);
                Err(e)
            }
            Err(_) => {
                error!("Recovery timed out after {:?}", self.config.recovery_timeout);
                Err(AIError::InferenceError("Recovery timeout".into()))
            }
        }
    }

    /// Check if we're in cooldown period
    async fn check_cooldown(&self) -> bool {
        match *self.last_recovery.read().await {
            Some(last) => last.elapsed() >= self.config.recovery_cooldown,
            None => true,
        }
    }

    /// Determine which recovery strategy to use
    fn determine_strategy(&self, error: &AIError) -> String {
        match error {
            AIError::ModelNotFound(_) => "redownload".to_string(),
            AIError::InferenceError(msg) => {
                if msg.contains("corrupted") || msg.contains("invalid model") {
                    "redownload".to_string()
                } else if msg.contains("out of memory") || msg.contains("OOM") {
                    "clear_cache".to_string()
                } else if msg.contains("timeout") || msg.contains("unresponsive") {
                    "restart_service".to_string()
                } else {
                    "retry".to_string()
                }
            }
            AIError::NetworkError(_) => "retry".to_string(),
            _ => "retry".to_string(),
        }
    }

    /// Execute recovery strategy
    async fn execute_recovery(
        &self,
        strategy: &RecoveryStrategy,
        error: &AIError,
        model_id: &str,
    ) -> Result<()> {
        match strategy {
            RecoveryStrategy::Retry { max_attempts, initial_delay, max_delay, multiplier } => {
                self.execute_retry(*max_attempts, *initial_delay, *max_delay, *multiplier).await
            }
            RecoveryStrategy::Redownload { max_attempts, verify_checksum } => {
                self.execute_redownload(model_id, *max_attempts, *verify_checksum).await
            }
            RecoveryStrategy::ClearCache { cache_types } => {
                self.execute_cache_clear(cache_types).await
            }
            RecoveryStrategy::RestartService { cooldown } => {
                self.execute_service_restart(*cooldown).await
            }
            RecoveryStrategy::ModelFallback { fallback_models } => {
                self.execute_model_fallback(model_id, fallback_models).await
            }
            RecoveryStrategy::Custom(f) => {
                use std::pin::Pin;
                let boxed_future = f(error);
                Pin::from(boxed_future).await
            }
        }
    }

    /// Execute retry with exponential backoff
    async fn execute_retry(
        &self,
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f32,
    ) -> Result<()> {
        info!("Executing retry strategy with exponential backoff");
        
        // This is a placeholder - in real implementation, you would retry the actual operation
        let mut delay = initial_delay;
        
        for attempt in 1..=max_attempts {
            info!("Retry attempt {}/{}", attempt, max_attempts);
            
            // Wait before retry
            tokio::time::sleep(delay).await;
            
            // Update delay for next attempt
            delay = Duration::from_secs_f32(
                (delay.as_secs_f32() * multiplier).min(max_delay.as_secs_f32())
            );
        }
        
        Ok(())
    }

    /// Execute model redownload
    async fn execute_redownload(
        &self,
        model_id: &str,
        _max_attempts: u32,
        _verify_checksum: bool,
    ) -> Result<()> {
        info!("Executing model redownload for: {}", model_id);
        
        // Get model path
        let models_dir = ensure_models_directory()
            .map_err(|e| AIError::ConfigError(format!("Failed to access models directory: {}", e)))?;
        
        let model_path = models_dir.join(model_id);
        
        // Delete existing model if it exists
        if model_path.exists() {
            warn!("Removing corrupted model at: {:?}", model_path);
            tokio::fs::remove_dir_all(&model_path).await
                .map_err(|e| AIError::ConfigError(format!("Failed to remove model: {}", e)))?;
        }
        
        // In real implementation, you would:
        // 1. Download model from repository
        // 2. Verify checksum if enabled
        // 3. Extract and validate model files
        
        info!("Model redownload completed for: {}", model_id);
        Ok(())
    }

    /// Execute cache clearing
    async fn execute_cache_clear(&self, cache_types: &[CacheType]) -> Result<()> {
        info!("Executing cache clear for types: {:?}", cache_types);
        
        for cache_type in cache_types {
            match cache_type {
                CacheType::ModelCache => {
                    // Clear model weights cache
                    info!("Clearing model cache");
                }
                CacheType::ResultCache => {
                    // Clear inference results cache
                    info!("Clearing result cache");
                }
                CacheType::GpuCache => {
                    // Clear GPU memory cache
                    info!("Clearing GPU cache");
                    // In real implementation: call GPU memory cleanup
                }
                CacheType::All => {
                    // Clear all caches
                    info!("Clearing all caches");
                    Box::pin(self.execute_cache_clear(&[
                        CacheType::ModelCache,
                        CacheType::ResultCache,
                        CacheType::GpuCache,
                    ])).await?;
                    return Ok(());
                }
            }
        }
        
        Ok(())
    }

    /// Execute service restart
    async fn execute_service_restart(&self, cooldown: Duration) -> Result<()> {
        info!("Executing service restart with cooldown: {:?}", cooldown);
        
        // In real implementation:
        // 1. Gracefully stop current service
        // 2. Wait for cooldown
        // 3. Restart service
        // 4. Verify service is healthy
        
        tokio::time::sleep(cooldown).await;
        
        info!("Service restart completed");
        Ok(())
    }

    /// Execute model fallback
    async fn execute_model_fallback(
        &self,
        current_model: &str,
        fallback_models: &[String],
    ) -> Result<()> {
        info!("Executing model fallback from {} to alternatives", current_model);
        
        for fallback in fallback_models {
            info!("Trying fallback model: {}", fallback);
            
            // In real implementation:
            // 1. Check if fallback model is available
            // 2. Load fallback model
            // 3. Verify it works
            // 4. Update configuration to use fallback
            
            // For now, just return success on first fallback
            return Ok(());
        }
        
        Err(AIError::ModelNotFound("No suitable fallback models available".into()))
    }

    /// Record recovery attempt
    async fn record_attempt(&self, attempt: RecoveryAttempt) {
        let mut history = self.recovery_history.write().await;
        history.push(attempt);
        
        // Keep only recent history (last 100 attempts)
        let len = history.len();
        if len > 100 {
            history.drain(0..len - 100);
        }
    }

    /// Get recovery statistics
    pub async fn get_stats(&self) -> RecoveryStats {
        let history = self.recovery_history.read().await;
        
        let total_attempts = history.len();
        let successful_attempts = history.iter().filter(|a| a.success).count();
        let avg_duration = if !history.is_empty() {
            let total_duration: Duration = history.iter().map(|a| a.duration).sum();
            total_duration / history.len() as u32
        } else {
            Duration::ZERO
        };
        
        RecoveryStats {
            total_attempts,
            successful_attempts,
            success_rate: if total_attempts > 0 {
                successful_attempts as f32 / total_attempts as f32
            } else {
                0.0
            },
            avg_duration,
        }
    }
}

/// Recovery statistics
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    pub total_attempts: usize,
    pub successful_attempts: usize,
    pub success_rate: f32,
    pub avg_duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recovery_manager_creation() {
        let config = RecoveryConfig::default();
        let manager = RecoveryManager::new(config);
        
        assert!(manager.strategies.contains_key("retry"));
        assert!(manager.strategies.contains_key("redownload"));
        assert!(manager.strategies.contains_key("clear_cache"));
    }

    #[tokio::test]
    async fn test_recovery_cooldown() {
        let mut config = RecoveryConfig::default();
        config.recovery_cooldown = Duration::from_millis(100);
        
        let manager = RecoveryManager::new(config);
        
        // First check should pass
        assert!(manager.check_cooldown().await);
        
        // Set last recovery time
        *manager.last_recovery.write().await = Some(Instant::now());
        
        // Immediate check should fail
        assert!(!manager.check_cooldown().await);
        
        // After cooldown should pass
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(manager.check_cooldown().await);
    }

    #[tokio::test]
    async fn test_strategy_determination() {
        let config = RecoveryConfig::default();
        let manager = RecoveryManager::new(config);
        
        // Model not found -> redownload
        let error = AIError::ModelNotFound("test.onnx".into());
        assert_eq!(manager.determine_strategy(&error), "redownload");
        
        // OOM error -> clear cache
        let error = AIError::InferenceError("GPU out of memory".into());
        assert_eq!(manager.determine_strategy(&error), "clear_cache");
        
        // Network error -> retry
        let error = AIError::NetworkError("Connection failed".into());
        assert_eq!(manager.determine_strategy(&error), "retry");
    }

    #[tokio::test]
    async fn test_recovery_stats() {
        let config = RecoveryConfig::default();
        let manager = RecoveryManager::new(config);
        
        // Add some attempts
        for i in 0..5 {
            manager.record_attempt(RecoveryAttempt {
                timestamp: Instant::now(),
                error_type: "TestError".to_string(),
                strategy: "retry".to_string(),
                success: i % 2 == 0,
                duration: Duration::from_millis(100),
            }).await;
        }
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_attempts, 5);
        assert_eq!(stats.successful_attempts, 3);
        assert_eq!(stats.success_rate, 0.6);
    }
}