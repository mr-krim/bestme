use crate::ai::{Result, AIError};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Real-time performance tracking for AI models
pub struct PerformanceTracker {
    /// Performance metrics per model
    metrics: Arc<RwLock<HashMap<String, ModelMetrics>>>,
    /// Configuration
    config: TrackerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerConfig {
    /// Window size for rolling metrics
    pub window_size: usize,
    /// Enable detailed tracking
    pub detailed_tracking: bool,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Max acceptable latency in ms
    pub max_latency_ms: u64,
    /// Min acceptable throughput in tps
    pub min_throughput_tps: f64,
    /// Max memory usage in MB
    pub max_memory_mb: u64,
    /// Max error rate percentage
    pub max_error_rate: f64,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            window_size: 1000,
            detailed_tracking: true,
            alert_thresholds: AlertThresholds {
                max_latency_ms: 1000,
                min_throughput_tps: 1.0,
                max_memory_mb: 4096,
                max_error_rate: 5.0,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct ModelMetrics {
    /// Recent latency measurements
    latencies: VecDeque<LatencyMeasurement>,
    /// Error count
    error_count: u64,
    /// Success count
    success_count: u64,
    /// Current memory usage
    memory_usage_mb: u64,
    /// Last update time
    last_update: Instant,
}

#[derive(Debug, Clone)]
struct LatencyMeasurement {
    latency_ms: u64,
    timestamp: Instant,
    input_size: usize,
    output_size: usize,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new(config: TrackerConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Record a successful inference
    pub async fn record_success(
        &self,
        model_id: &str,
        latency: Duration,
        input_size: usize,
        output_size: usize,
    ) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let model_metrics = metrics.entry(model_id.to_string()).or_insert_with(|| ModelMetrics {
            latencies: VecDeque::new(),
            error_count: 0,
            success_count: 0,
            memory_usage_mb: 0,
            last_update: Instant::now(),
        });
        
        // Add latency measurement
        let measurement = LatencyMeasurement {
            latency_ms: latency.as_millis() as u64,
            timestamp: Instant::now(),
            input_size,
            output_size,
        };
        
        model_metrics.latencies.push_back(measurement);
        
        // Maintain window size
        while model_metrics.latencies.len() > self.config.window_size {
            model_metrics.latencies.pop_front();
        }
        
        model_metrics.success_count += 1;
        model_metrics.last_update = Instant::now();
        
        // Check for alerts
        self.check_alerts(model_id, model_metrics).await?;
        
        Ok(())
    }
    
    /// Record a failed inference
    pub async fn record_error(&self, model_id: &str, error: &str) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let model_metrics = metrics.entry(model_id.to_string()).or_insert_with(|| ModelMetrics {
            latencies: VecDeque::new(),
            error_count: 0,
            success_count: 0,
            memory_usage_mb: 0,
            last_update: Instant::now(),
        });
        
        model_metrics.error_count += 1;
        model_metrics.last_update = Instant::now();
        
        log::warn!("Model {} error: {}", model_id, error);
        
        // Check error rate
        let total = model_metrics.success_count + model_metrics.error_count;
        if total > 0 {
            let error_rate = (model_metrics.error_count as f64 / total as f64) * 100.0;
            if error_rate > self.config.alert_thresholds.max_error_rate {
                log::error!("Model {} error rate {:.1}% exceeds threshold", model_id, error_rate);
            }
        }
        
        Ok(())
    }
    
    /// Update memory usage for a model
    pub async fn update_memory_usage(&self, model_id: &str, memory_mb: u64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(model_metrics) = metrics.get_mut(model_id) {
            model_metrics.memory_usage_mb = memory_mb;
            
            if memory_mb > self.config.alert_thresholds.max_memory_mb {
                log::warn!("Model {} memory usage {}MB exceeds threshold", model_id, memory_mb);
            }
        }
        Ok(())
    }
    
    /// Get current performance statistics for a model
    pub async fn get_stats(&self, model_id: &str) -> Option<PerformanceStats> {
        let metrics = self.metrics.read().await;
        let model_metrics = metrics.get(model_id)?;
        
        if model_metrics.latencies.is_empty() {
            return None;
        }
        
        // Calculate statistics
        let latencies: Vec<u64> = model_metrics.latencies.iter()
            .map(|m| m.latency_ms)
            .collect();
        
        let avg_latency = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
        let min_latency = *latencies.iter().min().unwrap_or(&0);
        let max_latency = *latencies.iter().max().unwrap_or(&0);
        
        // Calculate percentiles
        let mut sorted_latencies = latencies.clone();
        sorted_latencies.sort_unstable();
        
        let p50 = percentile(&sorted_latencies, 50.0);
        let p90 = percentile(&sorted_latencies, 90.0);
        let p95 = percentile(&sorted_latencies, 95.0);
        let p99 = percentile(&sorted_latencies, 99.0);
        
        // Calculate throughput
        let total_time_ms: u64 = model_metrics.latencies.iter()
            .map(|m| m.latency_ms)
            .sum();
        let throughput = if total_time_ms > 0 {
            (model_metrics.latencies.len() as f64 * 1000.0) / total_time_ms as f64
        } else {
            0.0
        };
        
        // Calculate error rate
        let total = model_metrics.success_count + model_metrics.error_count;
        let error_rate = if total > 0 {
            (model_metrics.error_count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        Some(PerformanceStats {
            model_id: model_id.to_string(),
            avg_latency_ms: avg_latency,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            p50_latency_ms: p50,
            p90_latency_ms: p90,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            throughput_tps: throughput,
            error_rate_percent: error_rate,
            success_count: model_metrics.success_count,
            error_count: model_metrics.error_count,
            memory_usage_mb: model_metrics.memory_usage_mb,
            last_update: model_metrics.last_update.elapsed().as_secs(),
        })
    }
    
    /// Get statistics for all tracked models
    pub async fn get_all_stats(&self) -> Vec<PerformanceStats> {
        let metrics = self.metrics.read().await;
        let mut all_stats = Vec::new();
        
        for model_id in metrics.keys() {
            if let Some(stats) = self.get_stats(model_id).await {
                all_stats.push(stats);
            }
        }
        
        all_stats
    }
    
    /// Check for performance alerts
    async fn check_alerts(&self, model_id: &str, metrics: &ModelMetrics) -> Result<()> {
        if metrics.latencies.is_empty() {
            return Ok(());
        }
        
        // Check average latency
        let avg_latency = metrics.latencies.iter()
            .map(|m| m.latency_ms)
            .sum::<u64>() as f64 / metrics.latencies.len() as f64;
        
        if avg_latency > self.config.alert_thresholds.max_latency_ms as f64 {
            log::warn!(
                "Model {} average latency {:.0}ms exceeds threshold",
                model_id, avg_latency
            );
        }
        
        // Check throughput
        let throughput = (metrics.latencies.len() as f64 * 1000.0) / 
                        metrics.latencies.iter().map(|m| m.latency_ms).sum::<u64>() as f64;
        
        if throughput < self.config.alert_thresholds.min_throughput_tps {
            log::warn!(
                "Model {} throughput {:.2}tps below threshold",
                model_id, throughput
            );
        }
        
        Ok(())
    }
    
    /// Clear metrics for a model
    pub async fn clear_metrics(&self, model_id: &str) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.remove(model_id);
        Ok(())
    }
    
    /// Export metrics to JSON
    pub async fn export_metrics(&self) -> Result<String> {
        let all_stats = self.get_all_stats().await;
        serde_json::to_string_pretty(&all_stats)
            .map_err(|e| AIError::ConfigError(format!("Failed to serialize metrics: {}", e)))
    }
}

/// Performance statistics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub model_id: String,
    pub avg_latency_ms: f64,
    pub min_latency_ms: u64,
    pub max_latency_ms: u64,
    pub p50_latency_ms: u64,
    pub p90_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub throughput_tps: f64,
    pub error_rate_percent: f64,
    pub success_count: u64,
    pub error_count: u64,
    pub memory_usage_mb: u64,
    pub last_update: u64,
}

/// Calculate percentile from sorted data
fn percentile(sorted_data: &[u64], p: f64) -> u64 {
    if sorted_data.is_empty() {
        return 0;
    }
    
    let index = ((p / 100.0) * (sorted_data.len() - 1) as f64) as usize;
    sorted_data[index]
}

/// Global performance tracker instance
static GLOBAL_TRACKER: once_cell::sync::OnceCell<Arc<PerformanceTracker>> = once_cell::sync::OnceCell::new();

/// Get or create the global performance tracker
pub fn get_global_tracker() -> Arc<PerformanceTracker> {
    GLOBAL_TRACKER.get_or_init(|| {
        Arc::new(PerformanceTracker::new(TrackerConfig::default()))
    }).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_tracking() {
        let tracker = PerformanceTracker::new(TrackerConfig::default());
        
        // Record some measurements
        tracker.record_success("test_model", Duration::from_millis(100), 100, 120).await.unwrap();
        tracker.record_success("test_model", Duration::from_millis(150), 100, 120).await.unwrap();
        tracker.record_success("test_model", Duration::from_millis(200), 100, 120).await.unwrap();
        
        // Get stats
        let stats = tracker.get_stats("test_model").await.unwrap();
        assert_eq!(stats.avg_latency_ms, 150.0);
        assert_eq!(stats.min_latency_ms, 100);
        assert_eq!(stats.max_latency_ms, 200);
    }
}