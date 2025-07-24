use crate::ai::{Result, AIError, EnhancementOptions, EnhancedText};
use crate::ai::services::model_service::AIModel;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Configuration for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum number of segments to process in parallel
    pub max_parallel: usize,
    /// Maximum batch size for model inference
    pub batch_size: usize,
    /// Timeout for processing a single batch (ms)
    pub timeout_ms: u64,
    /// Whether to preserve segment order
    pub preserve_order: bool,
    /// Enable smart batching by text length
    pub smart_batching: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_parallel: 4,
            batch_size: 8,
            timeout_ms: 5000,
            preserve_order: true,
            smart_batching: true,
        }
    }
}

/// Text segment for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    /// Unique identifier for the segment
    pub id: String,
    /// The text content
    pub text: String,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
    /// Priority (higher = process first)
    pub priority: i32,
}

/// Result of batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Segment ID
    pub id: String,
    /// Original text
    pub original: String,
    /// Enhanced text
    pub enhanced: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Whether processing succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Original metadata
    pub metadata: Option<serde_json::Value>,
}

/// Batch processor for efficient multi-segment processing
pub struct BatchProcessor {
    model: Arc<dyn AIModel>,
    config: BatchConfig,
    semaphore: Arc<Semaphore>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(model: Arc<dyn AIModel>, config: BatchConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_parallel));
        
        Self {
            model,
            config,
            semaphore,
        }
    }
    
    /// Process a batch of text segments
    pub async fn process_batch(
        &self,
        segments: Vec<TextSegment>,
        options: &EnhancementOptions,
    ) -> Result<Vec<BatchResult>> {
        if segments.is_empty() {
            return Ok(Vec::new());
        }
        
        // Sort by priority if not preserving order
        let mut segments = segments;
        if !self.config.preserve_order {
            segments.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
        
        // Group segments for efficient batching
        let batches = if self.config.smart_batching {
            self.create_smart_batches(segments)
        } else {
            self.create_simple_batches(segments)
        };
        
        // Process batches
        let results = self.process_batches(batches, options).await?;
        
        // Restore original order if needed
        if self.config.preserve_order {
            Ok(self.restore_order(results))
        } else {
            Ok(results)
        }
    }
    
    /// Create smart batches based on text length similarity
    fn create_smart_batches(&self, segments: Vec<TextSegment>) -> Vec<Vec<TextSegment>> {
        let mut batches = Vec::new();
        let mut sorted_segments = segments;
        
        // Sort by text length for better batching
        sorted_segments.sort_by_key(|s| s.text.len());
        
        // Group similar-length texts
        let mut current_batch = Vec::new();
        let mut current_avg_len = 0;
        
        for segment in sorted_segments {
            let seg_len = segment.text.len();
            
            if current_batch.is_empty() {
                current_avg_len = seg_len;
                current_batch.push(segment);
            } else if current_batch.len() < self.config.batch_size {
                // Check if length is within 50% of average
                let diff_ratio = (seg_len as f64 - current_avg_len as f64).abs() / current_avg_len as f64;
                
                if diff_ratio < 0.5 {
                    // Add to current batch and update average
                    let total_len: usize = current_batch.iter().map(|s| s.text.len()).sum();
                    current_batch.push(segment);
                    current_avg_len = (total_len + seg_len) / current_batch.len();
                } else {
                    // Start new batch
                    batches.push(current_batch);
                    current_batch = vec![segment];
                    current_avg_len = seg_len;
                }
            } else {
                // Batch is full
                batches.push(current_batch);
                current_batch = vec![segment];
                current_avg_len = seg_len;
            }
        }
        
        if !current_batch.is_empty() {
            batches.push(current_batch);
        }
        
        batches
    }
    
    /// Create simple batches by chunking
    fn create_simple_batches(&self, segments: Vec<TextSegment>) -> Vec<Vec<TextSegment>> {
        segments
            .chunks(self.config.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
    
    /// Process multiple batches in parallel
    async fn process_batches(
        &self,
        batches: Vec<Vec<TextSegment>>,
        options: &EnhancementOptions,
    ) -> Result<Vec<BatchResult>> {
        let mut all_results = Vec::new();
        
        // Create a stream of batch processing tasks
        let batch_stream = stream::iter(batches)
            .map(|batch| {
                let model = self.model.clone();
                let semaphore = self.semaphore.clone();
                let options = options.clone();
                let timeout_ms = self.config.timeout_ms;
                
                async move {
                    // Acquire semaphore permit
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    // Process batch with timeout
                    match tokio::time::timeout(
                        tokio::time::Duration::from_millis(timeout_ms),
                        Self::process_single_batch(&model, batch, &options)
                    ).await {
                        Ok(Ok(results)) => results,
                        Ok(Err(e)) => {
                            log::error!("Batch processing error: {}", e);
                            Vec::new()
                        }
                        Err(_) => {
                            log::error!("Batch processing timeout");
                            Vec::new()
                        }
                    }
                }
            })
            .buffer_unordered(self.config.max_parallel);
        
        // Collect all results
        let results: Vec<Vec<BatchResult>> = batch_stream.collect().await;
        
        // Flatten results
        for batch_results in results {
            all_results.extend(batch_results);
        }
        
        Ok(all_results)
    }
    
    /// Process a single batch
    async fn process_single_batch(
        model: &Arc<dyn AIModel>,
        batch: Vec<TextSegment>,
        options: &EnhancementOptions,
    ) -> Result<Vec<BatchResult>> {
        use crate::ai::telemetry::{metrics, tracing};
        
        let batch_size = batch.len();
        let batch_start = Instant::now();
        
        // Start batch tracing span
        let tracer = tracing::get_tracer();
        let mut batch_span = tracer.start_batch_span(model.model_id(), batch_size);
        
        // Get metrics collector
        let meter = opentelemetry::global::meter("bestme-ai");
        let aggregator = metrics::get_aggregator(&meter);
        let metrics_collector = aggregator.get_model_collector(model.model_id(), &meter);
        
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        
        // Process each segment in the batch
        // In a real implementation, this could be optimized to process
        // multiple segments in a single model forward pass
        for segment in batch {
            let start_time = Instant::now();
            
            match model.enhance_text(&segment.text, options).await {
                Ok(enhanced) => {
                    success_count += 1;
                    results.push(BatchResult {
                        id: segment.id,
                        original: segment.text,
                        enhanced: enhanced.enhanced,
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        success: true,
                        error: None,
                        metadata: segment.metadata,
                    });
                }
                Err(e) => {
                    failure_count += 1;
                    results.push(BatchResult {
                        id: segment.id,
                        original: segment.text.clone(),
                        enhanced: segment.text, // Return original on error
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        success: false,
                        error: Some(e.to_string()),
                        metadata: segment.metadata,
                    });
                }
            }
        }
        
        // Record batch metrics
        metrics_collector.record_batch(batch_size as u64, batch_start.elapsed());
        
        // Complete batch span
        batch_span.record_success_count(success_count);
        batch_span.record_failure_count(failure_count);
        batch_span.complete();
        
        Ok(results)
    }
    
    /// Restore original order based on segment IDs
    fn restore_order(&self, mut results: Vec<BatchResult>) -> Vec<BatchResult> {
        results.sort_by(|a, b| {
            // Extract numeric part from IDs for sorting
            let a_num = a.id.chars()
                .filter(|c| c.is_numeric())
                .collect::<String>()
                .parse::<usize>()
                .unwrap_or(0);
            let b_num = b.id.chars()
                .filter(|c| c.is_numeric())
                .collect::<String>()
                .parse::<usize>()
                .unwrap_or(0);
            
            a_num.cmp(&b_num)
        });
        
        results
    }
    
    /// Process a stream of text segments
    pub async fn process_stream(
        &self,
        mut segment_rx: mpsc::Receiver<TextSegment>,
        options: &EnhancementOptions,
    ) -> mpsc::Receiver<BatchResult> {
        let (result_tx, result_rx) = mpsc::channel(100);
        let model = self.model.clone();
        let semaphore = self.semaphore.clone();
        let options = options.clone();
        
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut last_flush = Instant::now();
            
            loop {
                // Receive with timeout to allow periodic flushing
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(100),
                    segment_rx.recv()
                ).await {
                    Ok(Some(segment)) => {
                        buffer.push(segment);
                        
                        // Flush if buffer is full
                        if buffer.len() >= 8 { // Batch size
                            let batch = std::mem::take(&mut buffer);
                            if let Ok(results) = Self::process_single_batch(&model, batch, &options).await {
                                for result in results {
                                    let _ = result_tx.send(result).await;
                                }
                            }
                            last_flush = Instant::now();
                        }
                    }
                    Ok(None) => {
                        // Channel closed, process remaining
                        if !buffer.is_empty() {
                            let batch = std::mem::take(&mut buffer);
                            if let Ok(results) = Self::process_single_batch(&model, batch, &options).await {
                                for result in results {
                                    let _ = result_tx.send(result).await;
                                }
                            }
                        }
                        break;
                    }
                    Err(_) => {
                        // Timeout - check if we should flush
                        if !buffer.is_empty() && last_flush.elapsed() > tokio::time::Duration::from_millis(500) {
                            let batch = std::mem::take(&mut buffer);
                            if let Ok(results) = Self::process_single_batch(&model, batch, &options).await {
                                for result in results {
                                    let _ = result_tx.send(result).await;
                                }
                            }
                            last_flush = Instant::now();
                        }
                    }
                }
            }
        });
        
        result_rx
    }
}

/// Statistics for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatistics {
    pub total_segments: usize,
    pub successful_segments: usize,
    pub failed_segments: usize,
    pub total_time_ms: u64,
    pub average_time_ms: f64,
    pub min_time_ms: u64,
    pub max_time_ms: u64,
}

impl BatchStatistics {
    /// Calculate statistics from batch results
    pub fn from_results(results: &[BatchResult], total_time_ms: u64) -> Self {
        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;
        
        let times: Vec<u64> = results.iter()
            .map(|r| r.processing_time_ms)
            .collect();
        
        let min_time = times.iter().min().copied().unwrap_or(0);
        let max_time = times.iter().max().copied().unwrap_or(0);
        let avg_time = if !times.is_empty() {
            times.iter().sum::<u64>() as f64 / times.len() as f64
        } else {
            0.0
        };
        
        Self {
            total_segments: results.len(),
            successful_segments: successful,
            failed_segments: failed,
            total_time_ms,
            average_time_ms: avg_time,
            min_time_ms: min_time,
            max_time_ms: max_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_parallel, 4);
        assert_eq!(config.batch_size, 8);
        assert!(config.preserve_order);
    }
    
    #[test]
    fn test_text_segment() {
        let segment = TextSegment {
            id: "test-1".to_string(),
            text: "Hello world".to_string(),
            metadata: None,
            priority: 5,
        };
        assert_eq!(segment.priority, 5);
    }
    
    #[test]
    fn test_batch_statistics() {
        let results = vec![
            BatchResult {
                id: "1".to_string(),
                original: "test".to_string(),
                enhanced: "test".to_string(),
                processing_time_ms: 100,
                success: true,
                error: None,
                metadata: None,
            },
            BatchResult {
                id: "2".to_string(),
                original: "test2".to_string(),
                enhanced: "test2".to_string(),
                processing_time_ms: 200,
                success: true,
                error: None,
                metadata: None,
            },
        ];
        
        let stats = BatchStatistics::from_results(&results, 300);
        assert_eq!(stats.total_segments, 2);
        assert_eq!(stats.successful_segments, 2);
        assert_eq!(stats.min_time_ms, 100);
        assert_eq!(stats.max_time_ms, 200);
        assert_eq!(stats.average_time_ms, 150.0);
    }
}