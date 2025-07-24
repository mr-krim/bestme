//! AI Resilience Module
//! 
//! Provides comprehensive error recovery and fallback mechanisms for AI model failures.
//! Implements patterns like circuit breakers, fallback chains, and automatic recovery
//! to ensure the AI functionality remains available even when facing various failures.

pub mod fallback;
pub mod circuit_breaker;
pub mod recovery;

pub use fallback::{FallbackChain, FallbackStrategy, ExecutionMode};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use recovery::{RecoveryStrategy, RecoveryManager, RecoveryConfig};

use crate::ai::{Result, AIError};
use crate::ai::telemetry::{get_metrics, AIMetrics};
use opentelemetry::KeyValue;
use std::sync::Arc;
use std::time::Duration;

/// Resilience configuration for AI operations
#[derive(Debug, Clone)]
pub struct ResilienceConfig {
    /// Enable circuit breaker pattern
    pub enable_circuit_breaker: bool,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Enable automatic recovery
    pub enable_recovery: bool,
    /// Recovery configuration
    pub recovery: RecoveryConfig,
    /// Fallback chain timeout
    pub fallback_timeout: Duration,
    /// Enable telemetry integration
    pub enable_telemetry: bool,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            enable_circuit_breaker: true,
            circuit_breaker: CircuitBreakerConfig::default(),
            enable_recovery: true,
            recovery: RecoveryConfig::default(),
            fallback_timeout: Duration::from_secs(30),
            enable_telemetry: true,
        }
    }
}

/// Resilience manager that coordinates all resilience mechanisms
pub struct ResilienceManager {
    config: ResilienceConfig,
    fallback_chain: FallbackChain,
    circuit_breaker: Option<CircuitBreaker>,
    recovery_manager: Option<RecoveryManager>,
    metrics: Option<Arc<AIMetrics>>,
}

impl ResilienceManager {
    /// Create a new resilience manager
    pub fn new(config: ResilienceConfig) -> Self {
        let circuit_breaker = if config.enable_circuit_breaker {
            Some(CircuitBreaker::new(config.circuit_breaker.clone()))
        } else {
            None
        };

        let recovery_manager = if config.enable_recovery {
            Some(RecoveryManager::new(config.recovery.clone()))
        } else {
            None
        };

        Self {
            config: config.clone(),
            fallback_chain: FallbackChain::new(config.fallback_timeout),
            circuit_breaker,
            recovery_manager,
            metrics: None,
        }
    }

    /// Initialize with telemetry support
    pub fn with_telemetry(mut self, meter: &opentelemetry::metrics::Meter) -> Self {
        if self.config.enable_telemetry {
            self.metrics = Some(get_metrics(meter));
        }
        self
    }

    /// Execute an operation with full resilience support
    pub async fn execute<F, T>(&self, operation: F, model_id: &str) -> Result<T>
    where
        F: Fn(ExecutionMode) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>> + Send + Sync,
        T: Send + 'static,
    {
        // Check circuit breaker first
        if let Some(ref cb) = self.circuit_breaker {
            cb.check_state()?;
        }

        // Track resilience metrics
        let labels = vec![
            KeyValue::new("model_id", model_id.to_string()),
            KeyValue::new("resilience", "enabled"),
        ];

        // Execute with fallback chain
        let result = self.fallback_chain.execute(operation).await;

        match &result {
            Ok(_) => {
                // Success - update circuit breaker and metrics
                if let Some(ref cb) = self.circuit_breaker {
                    cb.record_success();
                }
                if let Some(ref metrics) = self.metrics {
                    metrics.inference_requests.add(1, &labels);
                }
            }
            Err(e) => {
                // Failure - update circuit breaker and attempt recovery
                if let Some(ref cb) = self.circuit_breaker {
                    cb.record_failure();
                }
                if let Some(ref metrics) = self.metrics {
                    metrics.errors.add(1, &labels);
                }

                // Attempt recovery if enabled
                if let Some(ref recovery) = self.recovery_manager {
                    recovery.handle_failure(e, model_id).await?;
                }
            }
        }

        result
    }

    /// Get current circuit breaker state
    pub fn circuit_state(&self) -> Option<CircuitState> {
        self.circuit_breaker.as_ref().map(|cb| cb.current_state())
    }

    /// Manually reset circuit breaker
    pub fn reset_circuit_breaker(&self) {
        if let Some(ref cb) = self.circuit_breaker {
            cb.reset();
        }
    }

    /// Get fallback chain status
    pub fn fallback_status(&self) -> Vec<(ExecutionMode, bool)> {
        self.fallback_chain.get_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_resilience_manager_creation() {
        let config = ResilienceConfig::default();
        let manager = ResilienceManager::new(config);
        
        assert!(manager.circuit_breaker.is_some());
        assert!(manager.recovery_manager.is_some());
    }

    #[tokio::test]
    async fn test_resilience_execution_success() {
        let config = ResilienceConfig::default();
        let manager = ResilienceManager::new(config);
        
        let call_count = Arc::new(AtomicU32::new(0));
        let count_clone = call_count.clone();
        
        let operation = move |_mode: ExecutionMode| {
            let count = count_clone.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok::<i32, AIError>(42)
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<i32>> + Send>>
        };
        
        let result = manager.execute(operation, "test-model").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_integration() {
        let mut config = ResilienceConfig::default();
        config.circuit_breaker.failure_threshold = 2;
        config.circuit_breaker.reset_timeout = Duration::from_millis(100);
        
        let manager = ResilienceManager::new(config);
        
        // Simulate failures
        for _ in 0..3 {
            let operation = |_mode: ExecutionMode| {
                Box::pin(async move {
                    Err::<i32, AIError>(AIError::InferenceError("Test failure".into()))
                }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<i32>> + Send>>
            };
            
            let _ = manager.execute(operation, "test-model").await;
        }
        
        // Circuit should be open now
        assert_eq!(manager.circuit_state(), Some(CircuitState::Open));
    }
}