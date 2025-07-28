use opentelemetry::{KeyValue, Context};
use opentelemetry::metrics::{Counter, Histogram, Meter, UpDownCounter};
use std::time::{Duration, Instant};
use std::sync::Arc;

/// Model-specific metrics collector
pub struct ModelMetricsCollector {
    model_id: String,
    meter: Meter,
    
    // Counters
    inference_count: Counter<u64>,
    error_count: Counter<u64>,
    cache_hits: Counter<u64>,
    cache_misses: Counter<u64>,
    
    // Histograms
    latency_histogram: Histogram<f64>,
    token_histogram: Histogram<u64>,
    memory_histogram: Histogram<f64>,
    batch_size_histogram: Histogram<u64>,
    
    // Gauges (UpDownCounters)
    active_requests: UpDownCounter<i64>,
    loaded_models: UpDownCounter<i64>,
}

impl ModelMetricsCollector {
    /// Create a new metrics collector for a model
    pub fn new(model_id: String, meter: Meter) -> Self {
        let _labels = vec![KeyValue::new("model_id", model_id.clone())];
        
        Self {
            model_id: model_id.clone(),
            meter: meter.clone(),
            
            inference_count: meter
                .u64_counter("ai.model.inference.count")
                .with_description("Number of inference requests")
                .init(),
            
            error_count: meter
                .u64_counter("ai.model.error.count")
                .with_description("Number of errors")
                .init(),
            
            cache_hits: meter
                .u64_counter("ai.model.cache.hits")
                .with_description("Cache hit count")
                .init(),
            
            cache_misses: meter
                .u64_counter("ai.model.cache.misses")
                .with_description("Cache miss count")
                .init(),
            
            latency_histogram: meter
                .f64_histogram("ai.model.latency")
                .with_unit("ms")
                .with_description("Inference latency distribution")
                .init(),
            
            token_histogram: meter
                .u64_histogram("ai.model.tokens")
                .with_description("Token count distribution")
                .init(),
            
            memory_histogram: meter
                .f64_histogram("ai.model.memory")
                .with_unit("MB")
                .with_description("Memory usage distribution")
                .init(),
            
            batch_size_histogram: meter
                .u64_histogram("ai.model.batch_size")
                .with_description("Batch size distribution")
                .init(),
            
            active_requests: meter
                .i64_up_down_counter("ai.model.active_requests")
                .with_description("Number of active requests")
                .init(),
            
            loaded_models: meter
                .i64_up_down_counter("ai.models.loaded")
                .with_description("Number of loaded models")
                .init(),
        }
    }
    
    /// Record an inference request
    pub fn record_inference(&self, latency: Duration, tokens: u64, success: bool) {
        let _cx = Context::current();
        let labels = self.labels();
        
        // Record request count
        self.inference_count.add(1, &labels);
        
        // Record latency
        self.latency_histogram.record(latency.as_millis() as f64, &labels);
        
        // Record tokens
        self.token_histogram.record(tokens, &labels);
        
        // Record error if failed
        if !success {
            self.error_count.add(1, &labels);
        }
        
        // Calculate and record throughput
        if latency.as_secs_f64() > 0.0 {
            let throughput = tokens as f64 / latency.as_secs_f64();
            self.meter
                .f64_histogram("ai.model.throughput")
                .with_unit("tokens/s")
                .with_description("Token throughput")
                .init()
                .record(throughput, &labels);
        }
    }
    
    /// Record cache hit
    pub fn record_cache_hit(&self) {
        let _cx = Context::current();
        self.cache_hits.add(1, &self.labels());
    }
    
    /// Record cache miss
    pub fn record_cache_miss(&self) {
        let _cx = Context::current();
        self.cache_misses.add(1, &self.labels());
    }
    
    /// Record memory usage
    pub fn record_memory_usage(&self, memory_mb: f64) {
        let _cx = Context::current();
        self.memory_histogram.record(memory_mb, &self.labels());
    }
    
    /// Record batch processing
    pub fn record_batch(&self, batch_size: u64, total_latency: Duration) {
        let _cx = Context::current();
        let labels = self.labels();
        
        self.batch_size_histogram.record(batch_size, &labels);
        
        // Record average latency per item
        if batch_size > 0 {
            let avg_latency = total_latency.as_millis() as f64 / batch_size as f64;
            self.meter
                .f64_histogram("ai.model.batch.avg_latency")
                .with_unit("ms")
                .with_description("Average latency per item in batch")
                .init()
                .record(avg_latency, &labels);
        }
    }
    
    /// Track active request
    pub fn track_request(&self) -> RequestTracker {
        let _cx = Context::current();
        self.active_requests.add(1, &self.labels());
        
        RequestTracker {
            collector: self,
            start_time: Instant::now(),
        }
    }
    
    /// Record model loaded
    pub fn record_model_loaded(&self, load_time: Duration) {
        let _cx = Context::current();
        let labels = self.labels();
        
        self.loaded_models.add(1, &labels);
        
        self.meter
            .f64_histogram("ai.model.load_time")
            .with_unit("ms")
            .with_description("Model load time")
            .init()
            .record(load_time.as_millis() as f64, &labels);
    }
    
    /// Record model unloaded
    pub fn record_model_unloaded(&self) {
        let _cx = Context::current();
        self.loaded_models.add(-1, &self.labels());
    }
    
    /// Get labels for this model
    fn labels(&self) -> Vec<KeyValue> {
        vec![KeyValue::new("model_id", self.model_id.clone())]
    }
}

/// Request tracker for automatic duration measurement
pub struct RequestTracker<'a> {
    collector: &'a ModelMetricsCollector,
    start_time: Instant,
}

impl<'a> Drop for RequestTracker<'a> {
    fn drop(&mut self) {
        let _cx = Context::current();
        self.collector.active_requests.add(-1, &self.collector.labels());
    }
}

/// System-wide metrics collector
pub struct SystemMetricsCollector {
    meter: Meter,
    
    // System metrics
    cpu_usage: Histogram<f64>,
    memory_usage: Histogram<f64>,
    gpu_usage: Histogram<f64>,
    gpu_memory: Histogram<f64>,
    
    // Queue metrics
    queue_size: UpDownCounter<i64>,
    queue_latency: Histogram<f64>,
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new(meter: Meter) -> Self {
        Self {
            meter: meter.clone(),
            
            cpu_usage: meter
                .f64_histogram("system.cpu.usage")
                .with_unit("%")
                .with_description("CPU usage percentage")
                .init(),
            
            memory_usage: meter
                .f64_histogram("system.memory.usage")
                .with_unit("MB")
                .with_description("System memory usage")
                .init(),
            
            gpu_usage: meter
                .f64_histogram("system.gpu.usage")
                .with_unit("%")
                .with_description("GPU usage percentage")
                .init(),
            
            gpu_memory: meter
                .f64_histogram("system.gpu.memory")
                .with_unit("MB")
                .with_description("GPU memory usage")
                .init(),
            
            queue_size: meter
                .i64_up_down_counter("ai.queue.size")
                .with_description("Current queue size")
                .init(),
            
            queue_latency: meter
                .f64_histogram("ai.queue.latency")
                .with_unit("ms")
                .with_description("Queue waiting time")
                .init(),
        }
    }
    
    /// Record system metrics
    pub fn record_system_metrics(&self, cpu: f64, memory_mb: f64) {
        let _cx = Context::current();
        let labels = vec![];
        
        self.cpu_usage.record(cpu, &labels);
        self.memory_usage.record(memory_mb, &labels);
    }
    
    /// Record GPU metrics
    pub fn record_gpu_metrics(&self, gpu_id: u32, usage: f64, memory_mb: f64) {
        let _cx = Context::current();
        let labels = vec![KeyValue::new("gpu_id", gpu_id as i64)];
        
        self.gpu_usage.record(usage, &labels);
        self.gpu_memory.record(memory_mb, &labels);
    }
    
    /// Record queue metrics
    pub fn record_queue_metrics(&self, size: i64, wait_time: Duration) {
        let _cx = Context::current();
        let labels = vec![];
        
        self.queue_size.add(size, &labels);
        self.queue_latency.record(wait_time.as_millis() as f64, &labels);
    }
}

/// Metrics aggregator for reporting
pub struct MetricsAggregator {
    collectors: Arc<dashmap::DashMap<String, Arc<ModelMetricsCollector>>>,
    system_collector: Arc<SystemMetricsCollector>,
}

impl MetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new(meter: Meter) -> Self {
        Self {
            collectors: Arc::new(dashmap::DashMap::new()),
            system_collector: Arc::new(SystemMetricsCollector::new(meter)),
        }
    }
    
    /// Get or create a model metrics collector
    pub fn get_model_collector(&self, model_id: &str, meter: &Meter) -> Arc<ModelMetricsCollector> {
        self.collectors
            .entry(model_id.to_string())
            .or_insert_with(|| Arc::new(ModelMetricsCollector::new(model_id.to_string(), meter.clone())))
            .clone()
    }
    
    /// Get the system metrics collector
    pub fn system_collector(&self) -> &Arc<SystemMetricsCollector> {
        &self.system_collector
    }
    
    /// Generate a metrics summary
    pub async fn generate_summary(&self) -> MetricsSummary {
        // This would aggregate metrics from all collectors
        // For now, return a placeholder
        MetricsSummary {
            total_requests: 0,
            total_errors: 0,
            average_latency_ms: 0.0,
            cache_hit_rate: 0.0,
            models_loaded: self.collectors.len(),
        }
    }
}

/// Metrics summary for reporting
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsSummary {
    pub total_requests: u64,
    pub total_errors: u64,
    pub average_latency_ms: f64,
    pub cache_hit_rate: f64,
    pub models_loaded: usize,
}

/// Create a global metrics aggregator
static AGGREGATOR: once_cell::sync::OnceCell<Arc<MetricsAggregator>> = once_cell::sync::OnceCell::new();

/// Get the global metrics aggregator
pub fn get_aggregator(meter: &Meter) -> Arc<MetricsAggregator> {
    AGGREGATOR
        .get_or_init(|| Arc::new(MetricsAggregator::new(meter.clone())))
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_metrics_collector() {
        let meter = opentelemetry::global::meter("test");
        let collector = ModelMetricsCollector::new("test-model".to_string(), meter);
        
        // Test recording inference
        collector.record_inference(Duration::from_millis(100), 50, true);
        collector.record_cache_hit();
        collector.record_memory_usage(256.0);
    }
    
    #[test]
    fn test_request_tracker() {
        let meter = opentelemetry::global::meter("test");
        let collector = ModelMetricsCollector::new("test-model".to_string(), meter);
        
        {
            let _tracker = collector.track_request();
            // Request is being tracked
        }
        // Request tracker dropped, active requests decremented
    }
}