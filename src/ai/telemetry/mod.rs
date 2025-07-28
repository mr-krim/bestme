pub mod metrics;
pub mod tracing;
pub mod exporter;

use crate::ai::{Result, AIError};
use opentelemetry::{global, KeyValue};
use opentelemetry::metrics::{Counter, Histogram, Meter};
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::runtime;
use std::sync::Arc;
use std::time::Duration;

/// Telemetry configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Service name for telemetry
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable distributed tracing
    pub enable_tracing: bool,
    /// Metrics export interval
    pub export_interval: Duration,
    /// OTLP endpoint (if using OTLP exporter)
    pub otlp_endpoint: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "bestme-ai".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            enable_metrics: true,
            enable_tracing: true,
            export_interval: Duration::from_secs(60),
            otlp_endpoint: None,
        }
    }
}

/// Initialize telemetry system
pub fn init_telemetry(config: TelemetryConfig) -> Result<TelemetryHandle> {
    // Initialize metrics if enabled
    let meter = if config.enable_metrics {
        init_metrics(&config)?
    } else {
        None
    };
    
    // Initialize tracing if enabled
    if config.enable_tracing {
        init_tracing(&config)?;
    }
    
    Ok(TelemetryHandle {
        config,
        meter,
    })
}

/// Initialize metrics collection
fn init_metrics(config: &TelemetryConfig) -> Result<Option<Meter>> {
    let _reader = if let Some(endpoint) = &config.otlp_endpoint {
        // Use OTLP exporter
        let exporter_builder = opentelemetry_otlp::new_exporter()
            .tonic();
        let exporter_builder = opentelemetry_otlp::WithExportConfig::with_endpoint(exporter_builder, endpoint);
        let exporter = exporter_builder
            .build_metrics_exporter(
                Box::new(opentelemetry_sdk::metrics::reader::DefaultAggregationSelector::new()),
                Box::new(opentelemetry_sdk::metrics::reader::DefaultTemporalitySelector::new()),
            )
            .map_err(|e| AIError::ConfigError(format!("Failed to create OTLP exporter: {}", e)))?;
        PeriodicReader::builder(exporter, runtime::Tokio)
            .with_interval(config.export_interval)
            .build()
    } else {
        // Use stdout exporter for development
        let exporter = opentelemetry_stdout::MetricsExporter::default();
        PeriodicReader::builder(exporter, runtime::Tokio)
            .with_interval(config.export_interval)
            .build()
    };
    
    // In newer OpenTelemetry SDK, the meter provider is created differently
    // For now, we'll use the global meter directly
    
    let meter = global::meter_with_version(
        config.service_name.clone(),
        Some(config.service_version.clone()),
        None::<&str>,
        None,
    );
    
    Ok(Some(meter))
}

/// Initialize distributed tracing
fn init_tracing(config: &TelemetryConfig) -> Result<()> {
    
    use opentelemetry_sdk::trace::{self, Sampler};
    
    let tracer_provider = if let Some(endpoint) = &config.otlp_endpoint {
        // Use OTLP exporter
        let exporter_builder = opentelemetry_otlp::new_exporter()
            .tonic();
        let exporter_builder = opentelemetry_otlp::WithExportConfig::with_endpoint(exporter_builder, endpoint);
        let exporter = exporter_builder
            .build_span_exporter()
            .map_err(|e| AIError::ConfigError(format!("Failed to create span exporter: {}", e)))?;
        
        trace::TracerProvider::builder()
            .with_batch_exporter(exporter, runtime::Tokio)
            // .with_id_generator(RandomIdGenerator::default()) // Not available in this version
            .with_config(
                trace::Config::default()
                    .with_sampler(Sampler::AlwaysOn)
            )
            .build()
    } else {
        // Use stdout for development
        let exporter = opentelemetry_stdout::SpanExporter::default();
        
        trace::TracerProvider::builder()
            .with_simple_exporter(exporter)
            // .with_id_generator(RandomIdGenerator::default()) // Not available in this version
            .with_config(
                trace::Config::default()
                    .with_sampler(Sampler::AlwaysOn)
            )
            .build()
    };
    
    global::set_tracer_provider(tracer_provider);
    
    Ok(())
}

/// Handle for telemetry system
pub struct TelemetryHandle {
    config: TelemetryConfig,
    meter: Option<Meter>,
}

impl TelemetryHandle {
    /// Get the metrics meter
    pub fn meter(&self) -> Option<&Meter> {
        self.meter.as_ref()
    }
    
    /// Shutdown telemetry gracefully
    pub fn shutdown(self) -> Result<()> {
        // Note: In newer versions of OpenTelemetry, shutdown is handled differently
        // The providers are automatically shut down when dropped
        
        if self.config.enable_tracing {
            global::shutdown_tracer_provider();
        }
        
        Ok(())
    }
}

/// Common metric labels
pub fn common_labels(model_id: &str) -> Vec<KeyValue> {
    vec![
        KeyValue::new("model_id", model_id.to_string()),
        KeyValue::new("service", "bestme-ai"),
    ]
}

/// Create standard AI metrics
pub struct AIMetrics {
    /// Inference request counter
    pub inference_requests: Counter<u64>,
    /// Inference latency histogram
    pub inference_latency: Histogram<f64>,
    /// Model load time histogram
    pub model_load_time: Histogram<f64>,
    /// Token throughput histogram
    pub token_throughput: Histogram<f64>,
    /// Error counter
    pub errors: Counter<u64>,
    /// Cache hit counter
    pub cache_hits: Counter<u64>,
    /// Cache miss counter
    pub cache_misses: Counter<u64>,
    /// Memory usage histogram
    pub memory_usage: Histogram<f64>,
}

impl AIMetrics {
    /// Create standard AI metrics
    pub fn new(meter: &Meter) -> Self {
        Self {
            inference_requests: meter
                .u64_counter("ai.inference.requests")
                .with_description("Total number of inference requests")
                .init(),
            
            inference_latency: meter
                .f64_histogram("ai.inference.latency")
                .with_unit("ms")
                .with_description("Inference latency in milliseconds")
                .init(),
            
            model_load_time: meter
                .f64_histogram("ai.model.load_time")
                .with_unit("ms")
                .with_description("Model load time in milliseconds")
                .init(),
            
            token_throughput: meter
                .f64_histogram("ai.token.throughput")
                .with_unit("tokens/s")
                .with_description("Token processing throughput")
                .init(),
            
            errors: meter
                .u64_counter("ai.errors")
                .with_description("Total number of AI errors")
                .init(),
            
            cache_hits: meter
                .u64_counter("ai.cache.hits")
                .with_description("Number of cache hits")
                .init(),
            
            cache_misses: meter
                .u64_counter("ai.cache.misses")
                .with_description("Number of cache misses")
                .init(),
            
            memory_usage: meter
                .f64_histogram("ai.memory.usage")
                .with_unit("MB")
                .with_description("Memory usage in megabytes")
                .init(),
        }
    }
}

/// Global metrics instance
static METRICS: once_cell::sync::OnceCell<Arc<AIMetrics>> = once_cell::sync::OnceCell::new();

/// Get or create global AI metrics
pub fn get_metrics(meter: &Meter) -> Arc<AIMetrics> {
    METRICS.get_or_init(|| Arc::new(AIMetrics::new(meter))).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert_eq!(config.service_name, "bestme-ai");
        assert!(config.enable_metrics);
        assert!(config.enable_tracing);
    }
    
    #[test]
    fn test_common_labels() {
        let labels = common_labels("test-model");
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].key.as_str(), "model_id");
        assert_eq!(labels[0].value.as_str(), "test-model");
    }
}