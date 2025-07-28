use opentelemetry::{global, trace::{Span, SpanKind, Status, TraceContextExt, Tracer}};
use opentelemetry::{Context, KeyValue};
use std::sync::Arc;
use std::time::Instant;

/// Distributed tracing for AI operations
pub struct AITracer {
    tracer: global::BoxedTracer,
}

impl AITracer {
    /// Create a new AI tracer
    pub fn new() -> Self {
        let tracer = global::tracer("bestme-ai");
        Self {
            tracer,
        }
    }
    
    /// Start a new inference span
    pub fn start_inference_span(&self, model_id: &str, operation: &str) -> InferenceSpan {
        let span = self.tracer
            .span_builder(format!("ai.inference.{}", operation))
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("model.id", model_id.to_string()),
                KeyValue::new("operation", operation.to_string()),
            ])
            .start(&self.tracer);
        
        InferenceSpan {
            span,
            start_time: Instant::now(),
        }
    }
    
    /// Start a model loading span
    pub fn start_model_load_span(&self, model_id: &str) -> ModelLoadSpan {
        let span = self.tracer
            .span_builder("ai.model.load")
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("model.id", model_id.to_string()),
            ])
            .start(&self.tracer);
        
        ModelLoadSpan {
            span,
            start_time: Instant::now(),
        }
    }
    
    /// Start a batch processing span
    pub fn start_batch_span(&self, model_id: &str, batch_size: usize) -> BatchSpan {
        let span = self.tracer
            .span_builder("ai.batch.process")
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("model.id", model_id.to_string()),
                KeyValue::new("batch.size", batch_size as i64),
            ])
            .start(&self.tracer);
        
        BatchSpan {
            span,
            start_time: Instant::now(),
            batch_size,
        }
    }
    
    /// Create a child span for the current context
    pub fn child_span(&self, name: &str) -> ChildSpan {
        let cx = Context::current();
        let span = self.tracer
            .span_builder(name.to_string())
            .with_kind(SpanKind::Internal)
            .start_with_context(&self.tracer, &cx);
        
        ChildSpan { span }
    }
}

/// Span for tracking inference operations
pub struct InferenceSpan {
    span: global::BoxedSpan,
    start_time: Instant,
}

impl InferenceSpan {
    /// Record inference details
    pub fn record_details(&mut self, input_tokens: u64, output_tokens: u64) {
        self.span.set_attributes(vec![
            KeyValue::new("input.tokens", input_tokens as i64),
            KeyValue::new("output.tokens", output_tokens as i64),
        ]);
    }
    
    /// Record cache hit
    pub fn record_cache_hit(&mut self, hit: bool) {
        self.span.set_attribute(KeyValue::new("cache.hit", hit));
    }
    
    /// Record error
    pub fn record_error(&mut self, error: &str) {
        // Convert string to a proper error type
        let err = std::io::Error::new(std::io::ErrorKind::Other, error);
        self.span.record_error(&err);
        self.span.set_status(Status::error(error.to_string()));
    }
    
    /// Complete the span successfully
    pub fn complete(mut self) {
        let duration = self.start_time.elapsed();
        self.span.set_attribute(KeyValue::new("duration.ms", duration.as_millis() as i64));
        self.span.set_status(Status::Ok);
        self.span.end();
    }
}

impl Drop for InferenceSpan {
    fn drop(&mut self) {
        if !self.span.is_recording() {
            return;
        }
        self.span.end();
    }
}

/// Span for tracking model loading
pub struct ModelLoadSpan {
    span: global::BoxedSpan,
    start_time: Instant,
}

impl ModelLoadSpan {
    /// Record model size
    pub fn record_model_size(&mut self, size_mb: f64) {
        self.span.set_attribute(KeyValue::new("model.size_mb", size_mb));
    }
    
    /// Record optimization applied
    pub fn record_optimization(&mut self, optimization_type: &str) {
        self.span.set_attribute(KeyValue::new("optimization.type", optimization_type.to_string()));
    }
    
    /// Record error
    pub fn record_error(&mut self, error: &str) {
        // Convert string to a proper error type
        let err = std::io::Error::new(std::io::ErrorKind::Other, error);
        self.span.record_error(&err);
        self.span.set_status(Status::error(error.to_string()));
    }
    
    /// Complete the span
    pub fn complete(mut self, memory_usage_mb: f64) {
        let duration = self.start_time.elapsed();
        self.span.set_attributes(vec![
            KeyValue::new("duration.ms", duration.as_millis() as i64),
            KeyValue::new("memory.usage_mb", memory_usage_mb),
        ]);
        self.span.set_status(Status::Ok);
        self.span.end();
    }
}

/// Span for tracking batch processing
pub struct BatchSpan {
    span: global::BoxedSpan,
    start_time: Instant,
    batch_size: usize,
}

impl BatchSpan {
    /// Record successful items
    pub fn record_success_count(&mut self, count: usize) {
        self.span.set_attribute(KeyValue::new("batch.success_count", count as i64));
    }
    
    /// Record failed items
    pub fn record_failure_count(&mut self, count: usize) {
        self.span.set_attribute(KeyValue::new("batch.failure_count", count as i64));
    }
    
    /// Complete the span
    pub fn complete(mut self) {
        let duration = self.start_time.elapsed();
        let avg_latency = if self.batch_size > 0 {
            duration.as_millis() as f64 / self.batch_size as f64
        } else {
            0.0
        };
        
        self.span.set_attributes(vec![
            KeyValue::new("duration.ms", duration.as_millis() as i64),
            KeyValue::new("batch.avg_latency_ms", avg_latency),
        ]);
        self.span.set_status(Status::Ok);
        self.span.end();
    }
}

/// Generic child span
pub struct ChildSpan {
    span: global::BoxedSpan,
}

impl ChildSpan {
    /// Add an attribute
    pub fn set_attribute(&mut self, key: &str, value: impl Into<opentelemetry::Value>) {
        self.span.set_attribute(KeyValue::new(key.to_string(), value.into()));
    }
    
    /// Record an event
    pub fn add_event(&mut self, name: &str, attributes: Vec<KeyValue>) {
        self.span.add_event(name.to_string(), attributes);
    }
    
    /// Complete the span
    pub fn complete(mut self) {
        self.span.set_status(Status::Ok);
        self.span.end();
    }
}

/// Tracing context propagation
pub struct TracingContext;

impl TracingContext {
    /// Extract context from the current thread
    pub fn current() -> Context {
        Context::current()
    }
    
    /// Run a closure with a specific context
    pub fn with_context<F, R>(context: Context, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = context.attach();
        f()
    }
    
    /// Create a new root context
    pub fn new_root() -> Context {
        Context::new()
    }
}

/// Global tracer instance
static TRACER: once_cell::sync::OnceCell<Arc<AITracer>> = once_cell::sync::OnceCell::new();

/// Get the global AI tracer
pub fn get_tracer() -> Arc<AITracer> {
    TRACER.get_or_init(|| Arc::new(AITracer::new())).clone()
}

/// Macro for instrumenting functions
#[macro_export]
macro_rules! ai_instrument {
    ($name:expr) => {
        let _span = $crate::ai::telemetry::tracing::get_tracer()
            .child_span($name);
    };
    ($name:expr, $($key:expr => $value:expr),*) => {
        let tracer = $crate::ai::telemetry::tracing::get_tracer();
        let mut span = tracer.child_span($name);
        $(
            span.set_attribute($key, $value);
        )*
        let _span = span;
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inference_span() {
        let tracer = AITracer::new();
        let mut span = tracer.start_inference_span("test-model", "enhance_text");
        
        span.record_details(100, 120);
        span.record_cache_hit(true);
        span.complete();
    }
    
    #[test]
    fn test_batch_span() {
        let tracer = AITracer::new();
        let mut span = tracer.start_batch_span("test-model", 10);
        
        span.record_success_count(8);
        span.record_failure_count(2);
        span.complete();
    }
}