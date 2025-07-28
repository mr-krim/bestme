//! Circuit breaker pattern implementation for preventing cascading failures
//! 
//! Monitors failure rates and temporarily disables operations when thresholds are exceeded

use crate::ai::{Result, AIError};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use log::{info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed - normal operation
    Closed,
    /// Circuit is open - rejecting requests
    Open,
    /// Circuit is half-open - testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Time to wait before transitioning from open to half-open
    pub reset_timeout: Duration,
    /// Time window for counting failures
    pub window_size: Duration,
    /// Maximum concurrent requests in half-open state
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            reset_timeout: Duration::from_secs(60),
            window_size: Duration::from_secs(60),
            half_open_max_calls: 3,
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    half_open_calls: AtomicU32,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    state_changed_at: Arc<RwLock<Instant>>,
    total_calls: AtomicU64,
    total_failures: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            half_open_calls: AtomicU32::new(0),
            last_failure_time: Arc::new(RwLock::new(None)),
            state_changed_at: Arc::new(RwLock::new(Instant::now())),
            total_calls: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
        }
    }

    /// Check if operation is allowed
    pub fn check_state(&self) -> Result<()> {
        let state = self.current_state();
        
        match state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                // Check if we should transition to half-open
                if self.should_attempt_reset() {
                    info!("Circuit breaker transitioning to half-open state");
                    tokio::spawn({
                        let breaker = self.clone_internal();
                        async move {
                            breaker.transition_to_half_open().await;
                        }
                    });
                    Ok(())
                } else {
                    Err(AIError::InferenceError(
                        "Circuit breaker is open - service unavailable".into()
                    ))
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited calls in half-open state
                let current_calls = self.half_open_calls.fetch_add(1, Ordering::SeqCst);
                if current_calls < self.config.half_open_max_calls {
                    Ok(())
                } else {
                    self.half_open_calls.fetch_sub(1, Ordering::SeqCst);
                    Err(AIError::InferenceError(
                        "Circuit breaker is half-open - limited capacity".into()
                    ))
                }
            }
        }
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        self.total_calls.fetch_add(1, Ordering::SeqCst);
        let state = self.current_state();
        
        match state {
            CircuitState::Closed => {
                // Reset failure count on success in closed state
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                self.half_open_calls.fetch_sub(1, Ordering::SeqCst);
                
                if successes >= self.config.success_threshold {
                    info!("Circuit breaker closing after successful recovery");
                    tokio::spawn({
                        let breaker = self.clone_internal();
                        async move {
                            breaker.transition_to_closed().await;
                        }
                    });
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                warn!("Success recorded in open state - this shouldn't happen");
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        self.total_calls.fetch_add(1, Ordering::SeqCst);
        self.total_failures.fetch_add(1, Ordering::SeqCst);
        
        let state = self.current_state();
        
        match state {
            CircuitState::Closed => {
                // Check if failure is within window
                let should_count = tokio::task::block_in_place(|| {
                    let runtime = tokio::runtime::Handle::current();
                    runtime.block_on(async {
                        let last_failure = self.last_failure_time.read().await;
                        match *last_failure {
                            Some(time) if time.elapsed() > self.config.window_size => false,
                            _ => true,
                        }
                    })
                });

                if !should_count {
                    // Reset counter if outside window
                    self.failure_count.store(1, Ordering::SeqCst);
                } else {
                    let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                    
                    if failures >= self.config.failure_threshold {
                        warn!("Circuit breaker opening due to {} failures", failures);
                        tokio::spawn({
                            let breaker = self.clone_internal();
                            async move {
                                breaker.transition_to_open().await;
                            }
                        });
                    }
                }
                
                // Update last failure time
                tokio::spawn({
                    let last_failure = self.last_failure_time.clone();
                    async move {
                        *last_failure.write().await = Some(Instant::now());
                    }
                });
            }
            CircuitState::HalfOpen => {
                self.half_open_calls.fetch_sub(1, Ordering::SeqCst);
                warn!("Failure in half-open state - reopening circuit");
                tokio::spawn({
                    let breaker = self.clone_internal();
                    async move {
                        breaker.transition_to_open().await;
                    }
                });
            }
            CircuitState::Open => {
                // Already open, just track the failure
            }
        }
    }

    /// Get current state
    pub fn current_state(&self) -> CircuitState {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                *self.state.read().await
            })
        })
    }

    /// Check if we should attempt reset from open state
    fn should_attempt_reset(&self) -> bool {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let state_changed = *self.state_changed_at.read().await;
                state_changed.elapsed() >= self.config.reset_timeout
            })
        })
    }

    /// Transition to open state
    async fn transition_to_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Open;
        *self.state_changed_at.write().await = Instant::now();
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::HalfOpen;
        *self.state_changed_at.write().await = Instant::now();
        self.success_count.store(0, Ordering::SeqCst);
        self.half_open_calls.store(0, Ordering::SeqCst);
    }

    /// Transition to closed state
    async fn transition_to_closed(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        *self.state_changed_at.write().await = Instant::now();
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        *self.last_failure_time.write().await = None;
    }

    /// Reset circuit breaker manually
    pub fn reset(&self) {
        tokio::spawn({
            let breaker = self.clone_internal();
            async move {
                breaker.transition_to_closed().await;
            }
        });
    }

    /// Get circuit breaker statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.current_state(),
            total_calls: self.total_calls.load(Ordering::SeqCst),
            total_failures: self.total_failures.load(Ordering::SeqCst),
            current_failures: self.failure_count.load(Ordering::SeqCst),
            current_successes: self.success_count.load(Ordering::SeqCst),
        }
    }

    /// Internal clone for async operations
    fn clone_internal(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
            failure_count: AtomicU32::new(self.failure_count.load(Ordering::SeqCst)),
            success_count: AtomicU32::new(self.success_count.load(Ordering::SeqCst)),
            half_open_calls: AtomicU32::new(self.half_open_calls.load(Ordering::SeqCst)),
            last_failure_time: self.last_failure_time.clone(),
            state_changed_at: self.state_changed_at.clone(),
            total_calls: AtomicU64::new(self.total_calls.load(Ordering::SeqCst)),
            total_failures: AtomicU64::new(self.total_failures.load(Ordering::SeqCst)),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub total_calls: u64,
    pub total_failures: u64,
    pub current_failures: u32,
    pub current_successes: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed_to_open() {
        let mut config = CircuitBreakerConfig::default();
        config.failure_threshold = 3;
        
        let breaker = CircuitBreaker::new(config);
        
        // Should start closed
        assert_eq!(breaker.current_state(), CircuitState::Closed);
        assert!(breaker.check_state().is_ok());
        
        // Record failures
        for _ in 0..3 {
            breaker.record_failure();
        }
        
        // Wait for state transition
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Should be open now
        assert_eq!(breaker.current_state(), CircuitState::Open);
        assert!(breaker.check_state().is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_to_half_open() {
        let mut config = CircuitBreakerConfig::default();
        config.failure_threshold = 1;
        config.reset_timeout = Duration::from_millis(100);
        
        let breaker = CircuitBreaker::new(config);
        
        // Open the circuit
        breaker.record_failure();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(breaker.current_state(), CircuitState::Open);
        
        // Wait for reset timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should allow attempt (transition to half-open)
        assert!(breaker.check_state().is_ok());
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(breaker.current_state(), CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_to_closed() {
        let mut config = CircuitBreakerConfig::default();
        config.failure_threshold = 1;
        config.success_threshold = 2;
        config.reset_timeout = Duration::from_millis(100);
        
        let breaker = CircuitBreaker::new(config);
        
        // Open then half-open the circuit
        breaker.record_failure();
        tokio::time::sleep(Duration::from_millis(150)).await;
        breaker.check_state().ok();
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Record successes in half-open state
        for _ in 0..2 {
            assert!(breaker.check_state().is_ok());
            breaker.record_success();
        }
        
        // Wait for state transition
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Should be closed now
        assert_eq!(breaker.current_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let mut config = CircuitBreakerConfig::default();
        config.failure_threshold = 2;
        
        let breaker = CircuitBreaker::new(config);
        
        // Record some operations
        breaker.record_success();
        breaker.record_failure();
        breaker.record_failure();
        
        let stats = breaker.stats();
        assert_eq!(stats.total_calls, 3);
        assert_eq!(stats.total_failures, 2);
    }
}