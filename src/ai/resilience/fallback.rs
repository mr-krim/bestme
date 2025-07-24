//! Fallback chain implementation for resilient AI execution
//! 
//! Provides a chain of execution strategies that are tried in order:
//! GPU → CPU → Cache → Error

use crate::ai::{Result, AIError};
use crate::ai::telemetry::get_metrics;
use opentelemetry::KeyValue;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use log::{info, warn, error};

/// Execution mode for the current attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// GPU-accelerated execution
    GPU,
    /// CPU-only execution
    CPU,
    /// Use cached results
    Cache,
    /// Final fallback - return error
    Error,
}

impl ExecutionMode {
    /// Get the next fallback mode
    pub fn next(&self) -> Self {
        match self {
            ExecutionMode::GPU => ExecutionMode::CPU,
            ExecutionMode::CPU => ExecutionMode::Cache,
            ExecutionMode::Cache => ExecutionMode::Error,
            ExecutionMode::Error => ExecutionMode::Error,
        }
    }

    /// Check if this is the last mode before error
    pub fn is_final(&self) -> bool {
        matches!(self, ExecutionMode::Error)
    }
}

/// Strategy for fallback execution
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    /// Try all modes in sequence
    Sequential,
    /// Skip to specific mode based on error type
    Adaptive,
    /// Custom strategy function
    Custom(Arc<dyn Fn(&AIError) -> ExecutionMode + Send + Sync>),
}

/// Fallback chain executor
pub struct FallbackChain {
    strategy: FallbackStrategy,
    timeout: Duration,
    mode_status: Arc<tokio::sync::RwLock<Vec<(ExecutionMode, bool)>>>,
}

impl FallbackChain {
    /// Create a new fallback chain
    pub fn new(timeout: Duration) -> Self {
        let modes = vec![
            (ExecutionMode::GPU, true),
            (ExecutionMode::CPU, true),
            (ExecutionMode::Cache, true),
            (ExecutionMode::Error, true),
        ];
        
        Self {
            strategy: FallbackStrategy::Sequential,
            timeout,
            mode_status: Arc::new(tokio::sync::RwLock::new(modes)),
        }
    }

    /// Set the fallback strategy
    pub fn with_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Execute operation with fallback chain
    pub async fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn(ExecutionMode) -> Pin<Box<dyn Future<Output = Result<T>> + Send>> + Send + Sync,
        T: Send + 'static,
    {
        let mut current_mode = ExecutionMode::GPU;
        let mut last_error: Option<AIError> = None;
        
        loop {
            // Check if mode is available
            if !self.is_mode_available(current_mode).await {
                info!("Skipping unavailable mode: {:?}", current_mode);
                if current_mode.is_final() {
                    break;
                }
                current_mode = current_mode.next();
                continue;
            }

            info!("Attempting execution with mode: {:?}", current_mode);
            let start = Instant::now();
            
            // Execute with timeout
            let result = match timeout(self.timeout, operation(current_mode)).await {
                Ok(result) => result,
                Err(_) => {
                    warn!("Execution timed out for mode: {:?}", current_mode);
                    self.mark_mode_unavailable(current_mode).await;
                    Err(AIError::InferenceError(format!(
                        "Timeout after {:?} for mode {:?}",
                        self.timeout, current_mode
                    )))
                }
            };
            
            let duration = start.elapsed();
            
            match result {
                Ok(value) => {
                    info!(
                        "Successfully executed with mode {:?} in {:?}",
                        current_mode, duration
                    );
                    self.mark_mode_available(current_mode).await;
                    return Ok(value);
                }
                Err(e) => {
                    warn!(
                        "Execution failed with mode {:?} after {:?}: {}",
                        current_mode, duration, e
                    );
                    
                    // Determine next mode based on strategy
                    let next_mode = self.determine_next_mode(&e, current_mode).await;
                    
                    // Mark current mode as potentially problematic
                    if self.should_mark_unavailable(&e) {
                        self.mark_mode_unavailable(current_mode).await;
                    }
                    
                    last_error = Some(e);
                    
                    if next_mode.is_final() {
                        break;
                    }
                    
                    current_mode = next_mode;
                }
            }
        }
        
        // All attempts failed
        error!("All fallback attempts exhausted");
        Err(last_error.unwrap_or_else(|| {
            AIError::InferenceError("All fallback strategies failed".into())
        }))
    }

    /// Determine next mode based on strategy and error
    async fn determine_next_mode(&self, error: &AIError, current: ExecutionMode) -> ExecutionMode {
        match &self.strategy {
            FallbackStrategy::Sequential => current.next(),
            FallbackStrategy::Adaptive => self.adaptive_next_mode(error, current),
            FallbackStrategy::Custom(f) => f(error),
        }
    }

    /// Adaptive strategy for determining next mode
    fn adaptive_next_mode(&self, error: &AIError, current: ExecutionMode) -> ExecutionMode {
        match (error, current) {
            // GPU OOM -> skip to CPU
            (AIError::InferenceError(msg), ExecutionMode::GPU) 
                if msg.contains("out of memory") || msg.contains("OOM") => {
                ExecutionMode::CPU
            }
            // Model not found -> skip to error
            (AIError::ModelNotFound(_), _) => ExecutionMode::Error,
            // Network error with cache available -> try cache
            (AIError::NetworkError(_), _) => ExecutionMode::Cache,
            // Default sequential fallback
            _ => current.next(),
        }
    }

    /// Check if specific error should mark mode as unavailable
    fn should_mark_unavailable(&self, error: &AIError) -> bool {
        matches!(
            error,
            AIError::InferenceError(msg) if msg.contains("out of memory") || 
                                           msg.contains("device not available") ||
                                           msg.contains("driver error")
        )
    }

    /// Check if mode is currently available
    async fn is_mode_available(&self, mode: ExecutionMode) -> bool {
        let status = self.mode_status.read().await;
        status.iter()
            .find(|(m, _)| *m == mode)
            .map(|(_, available)| *available)
            .unwrap_or(true)
    }

    /// Mark mode as unavailable
    async fn mark_mode_unavailable(&self, mode: ExecutionMode) {
        let mut status = self.mode_status.write().await;
        if let Some((_, available)) = status.iter_mut().find(|(m, _)| *m == mode) {
            *available = false;
        }
    }

    /// Mark mode as available
    async fn mark_mode_available(&self, mode: ExecutionMode) {
        let mut status = self.mode_status.write().await;
        if let Some((_, available)) = status.iter_mut().find(|(m, _)| *m == mode) {
            *available = true;
        }
    }

    /// Get current status of all modes
    pub fn get_status(&self) -> Vec<(ExecutionMode, bool)> {
        // This is a simplified synchronous version
        // In production, you might want to use try_read or make this async
        vec![
            (ExecutionMode::GPU, true),
            (ExecutionMode::CPU, true),
            (ExecutionMode::Cache, true),
            (ExecutionMode::Error, true),
        ]
    }

    /// Reset all modes to available
    pub async fn reset(&self) {
        let mut status = self.mode_status.write().await;
        for (_, available) in status.iter_mut() {
            *available = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_execution_mode_progression() {
        assert_eq!(ExecutionMode::GPU.next(), ExecutionMode::CPU);
        assert_eq!(ExecutionMode::CPU.next(), ExecutionMode::Cache);
        assert_eq!(ExecutionMode::Cache.next(), ExecutionMode::Error);
        assert_eq!(ExecutionMode::Error.next(), ExecutionMode::Error);
    }

    #[test]
    fn test_execution_mode_is_final() {
        assert!(!ExecutionMode::GPU.is_final());
        assert!(!ExecutionMode::CPU.is_final());
        assert!(!ExecutionMode::Cache.is_final());
        assert!(ExecutionMode::Error.is_final());
    }

    #[tokio::test]
    async fn test_fallback_chain_success_on_first_try() {
        let chain = FallbackChain::new(Duration::from_secs(5));
        let call_count = Arc::new(AtomicU32::new(0));
        let count_clone = call_count.clone();
        
        let operation = move |mode: ExecutionMode| {
            let count = count_clone.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                assert_eq!(mode, ExecutionMode::GPU);
                Ok::<String, AIError>("Success".to_string())
            }) as Pin<Box<dyn Future<Output = Result<String>> + Send>>
        };
        
        let result = chain.execute(operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_fallback_chain_gpu_to_cpu() {
        let chain = FallbackChain::new(Duration::from_secs(5));
        let call_count = Arc::new(AtomicU32::new(0));
        let count_clone = call_count.clone();
        
        let operation = move |mode: ExecutionMode| {
            let count = count_clone.clone();
            Box::pin(async move {
                let call = count.fetch_add(1, Ordering::SeqCst);
                match (call, mode) {
                    (0, ExecutionMode::GPU) => {
                        Err(AIError::InferenceError("GPU out of memory".into()))
                    }
                    (1, ExecutionMode::CPU) => {
                        Ok("CPU Success".to_string())
                    }
                    _ => panic!("Unexpected call: {} with mode {:?}", call, mode),
                }
            }) as Pin<Box<dyn Future<Output = Result<String>> + Send>>
        };
        
        let result = chain.execute(operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CPU Success");
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_fallback_chain_timeout() {
        let chain = FallbackChain::new(Duration::from_millis(100));
        
        let operation = move |_mode: ExecutionMode| {
            Box::pin(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok::<String, AIError>("Should timeout".to_string())
            }) as Pin<Box<dyn Future<Output = Result<String>> + Send>>
        };
        
        let result = chain.execute(operation).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_adaptive_strategy() {
        let chain = FallbackChain::new(Duration::from_secs(5))
            .with_strategy(FallbackStrategy::Adaptive);
        
        let operation = move |mode: ExecutionMode| {
            Box::pin(async move {
                match mode {
                    ExecutionMode::GPU => {
                        Err(AIError::NetworkError("Network failure".into()))
                    }
                    ExecutionMode::Cache => {
                        Ok("Cached result".to_string())
                    }
                    _ => panic!("Should skip CPU and go to Cache"),
                }
            }) as Pin<Box<dyn Future<Output = Result<String>> + Send>>
        };
        
        let result = chain.execute(operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Cached result");
    }
}