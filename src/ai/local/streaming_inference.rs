use crate::ai::{Result, AIError, EnhancementOptions};
use crate::ai::services::model_service::AIModel;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{Duration, timeout};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// Configuration for streaming inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Minimum text length before triggering inference
    pub min_chunk_size: usize,
    /// Maximum time to wait before processing partial input
    pub max_wait_ms: u64,
    /// Buffer size for streaming chunks
    pub buffer_size: usize,
    /// Enable predictive typing suggestions
    pub enable_predictions: bool,
    /// Number of tokens to predict ahead
    pub prediction_length: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 20,  // Process after 20 characters
            max_wait_ms: 300,    // Or after 300ms of inactivity
            buffer_size: 1024,
            enable_predictions: true,
            prediction_length: 5,
        }
    }
}

/// Streaming inference engine for real-time text enhancement
pub struct StreamingInference {
    model: Arc<dyn AIModel>,
    config: StreamingConfig,
    /// Buffer for incomplete text chunks
    text_buffer: Arc<RwLock<String>>,
    /// Queue of processed segments
    processed_segments: Arc<RwLock<VecDeque<ProcessedSegment>>>,
    /// Channel for receiving text updates
    input_rx: Arc<RwLock<mpsc::Receiver<TextUpdate>>>,
    /// Channel for sending enhanced results
    output_tx: mpsc::Sender<StreamingResult>,
}

#[derive(Debug, Clone)]
pub enum TextUpdate {
    /// New text appended
    Append(String),
    /// Text at position replaced
    Replace { start: usize, end: usize, text: String },
    /// Clear all text
    Clear,
    /// Finalize current buffer
    Finalize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResult {
    /// Original text segment
    pub original: String,
    /// Enhanced text segment
    pub enhanced: String,
    /// Position in the full text
    pub position: TextPosition,
    /// Suggested completions
    pub suggestions: Vec<String>,
    /// Processing latency in milliseconds
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
struct ProcessedSegment {
    _original: String,
    enhanced: String,
    _position: TextPosition,
    _timestamp: std::time::Instant,
}

impl StreamingInference {
    /// Create a new streaming inference engine
    pub fn new(
        model: Arc<dyn AIModel>,
        config: StreamingConfig,
    ) -> (Self, mpsc::Sender<TextUpdate>, mpsc::Receiver<StreamingResult>) {
        let (input_tx, input_rx) = mpsc::channel(config.buffer_size);
        let (output_tx, output_rx) = mpsc::channel(config.buffer_size);
        
        let engine = Self {
            model,
            config,
            text_buffer: Arc::new(RwLock::new(String::new())),
            processed_segments: Arc::new(RwLock::new(VecDeque::new())),
            input_rx: Arc::new(RwLock::new(input_rx)),
            output_tx,
        };
        
        (engine, input_tx, output_rx)
    }
    
    /// Start the streaming inference loop
    pub async fn start(self) -> Result<()> {
        let buffer = self.text_buffer.clone();
        let segments = self.processed_segments.clone();
        let config = self.config.clone();
        let model = self.model.clone();
        let output_tx = self.output_tx.clone();
        let input_rx = self.input_rx.clone();
        
        // Spawn the main processing loop
        tokio::spawn(async move {
            let mut last_process_time = std::time::Instant::now();
            let mut current_position = 0usize;
            
            loop {
                // Try to receive with timeout
                let update = {
                    let mut rx = input_rx.write().await;
                    match timeout(
                        Duration::from_millis(config.max_wait_ms),
                        rx.recv()
                    ).await {
                        Ok(Some(update)) => Some(update),
                        Ok(None) => break, // Channel closed
                        Err(_) => None, // Timeout
                    }
                };
                
                match update {
                    Some(TextUpdate::Append(text)) => {
                        let mut buffer_guard = buffer.write().await;
                        buffer_guard.push_str(&text);
                        
                        // Check if we should process
                        if buffer_guard.len() >= config.min_chunk_size {
                            drop(buffer_guard); // Release lock
                            if let Err(e) = Self::process_buffer(
                                &model,
                                &buffer,
                                &segments,
                                &output_tx,
                                &config,
                                &mut current_position,
                            ).await {
                                log::error!("Processing error: {}", e);
                            }
                            last_process_time = std::time::Instant::now();
                        }
                    }
                    Some(TextUpdate::Replace { start, end, text }) => {
                        let mut buffer = buffer.write().await;
                        if start <= buffer.len() && end <= buffer.len() && start <= end {
                            buffer.replace_range(start..end, &text);
                        }
                    }
                    Some(TextUpdate::Clear) => {
                        let mut buffer = buffer.write().await;
                        buffer.clear();
                        let mut segments = segments.write().await;
                        segments.clear();
                        current_position = 0;
                    }
                    Some(TextUpdate::Finalize) => {
                        // Force process any remaining buffer
                        if let Err(e) = Self::process_buffer(
                            &model,
                            &buffer,
                            &segments,
                            &output_tx,
                            &config,
                            &mut current_position,
                        ).await {
                            log::error!("Final processing error: {}", e);
                        }
                    }
                    None => {
                        // Timeout - check if we should process partial buffer
                        let buffer_len = buffer.read().await.len();
                        if buffer_len > 0 && 
                           last_process_time.elapsed() > Duration::from_millis(config.max_wait_ms) {
                            if let Err(e) = Self::process_buffer(
                                &model,
                                &buffer,
                                &segments,
                                &output_tx,
                                &config,
                                &mut current_position,
                            ).await {
                                log::error!("Timeout processing error: {}", e);
                            }
                            last_process_time = std::time::Instant::now();
                        }
                    }
                }
            }
            
            log::info!("Streaming inference loop ended");
        });
        
        Ok(())
    }
    
    /// Process the current buffer
    async fn process_buffer(
        model: &Arc<dyn AIModel>,
        buffer: &Arc<RwLock<String>>,
        segments: &Arc<RwLock<VecDeque<ProcessedSegment>>>,
        output_tx: &mpsc::Sender<StreamingResult>,
        config: &StreamingConfig,
        current_position: &mut usize,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Get and clear buffer
        let text = {
            let mut buf = buffer.write().await;
            if buf.is_empty() {
                return Ok(());
            }
            let text = buf.clone();
            buf.clear();
            text
        };
        
        // Create context from previous segments
        let context = Self::build_context(segments, 512).await;
        
        // Prepare text with context
        let input_text = if context.is_empty() {
            text.clone()
        } else {
            format!("{} {}", context, text)
        };
        
        // Run enhancement
        let options = EnhancementOptions {
            correct_grammar: true,
            improve_clarity: true,
            preserve_style: true,
            detect_intent: false,
            format_markdown: false,
            improve_punctuation: true,
            confidence_threshold: 0.7,
        };
        
        let result = model.enhance_text(&input_text, &options).await?;
        
        // Extract only the enhanced part (removing context)
        let enhanced = if context.is_empty() {
            result.enhanced
        } else {
            // Remove context prefix from result
            result.enhanced
                .strip_prefix(&context)
                .unwrap_or(&result.enhanced)
                .trim_start()
                .to_string()
        };
        
        // Generate suggestions if enabled
        let suggestions = if config.enable_predictions {
            Self::generate_suggestions(&enhanced, config.prediction_length).await
        } else {
            Vec::new()
        };
        
        // Store processed segment
        let position = TextPosition {
            start: *current_position,
            end: *current_position + text.len(),
        };
        
        let segment_len = text.len();
        let segment = ProcessedSegment {
            _original: text.clone(),
            enhanced: enhanced.clone(),
            _position: position.clone(),
            _timestamp: std::time::Instant::now(),
        };
        
        {
            let mut segs = segments.write().await;
            segs.push_back(segment);
            // Keep only recent segments (last 10)
            while segs.len() > 10 {
                segs.pop_front();
            }
        }
        
        // Send result
        let streaming_result = StreamingResult {
            original: text,
            enhanced,
            position,
            suggestions,
            latency_ms: start_time.elapsed().as_millis() as u64,
        };
        
        if let Err(e) = output_tx.send(streaming_result).await {
            log::warn!("Failed to send streaming result: {}", e);
        }
        
        *current_position += segment_len;
        
        Ok(())
    }
    
    /// Build context from previous segments
    async fn build_context(
        segments: &Arc<RwLock<VecDeque<ProcessedSegment>>>,
        max_length: usize,
    ) -> String {
        let segs = segments.read().await;
        let mut context = String::new();
        
        // Iterate from most recent backwards
        for segment in segs.iter().rev() {
            let segment_text = &segment.enhanced;
            if context.len() + segment_text.len() > max_length {
                break;
            }
            context = format!("{} {}", segment_text, context);
        }
        
        context.trim().to_string()
    }
    
    /// Generate predictive suggestions
    async fn generate_suggestions(text: &str, length: usize) -> Vec<String> {
        // Simple heuristic-based suggestions
        // In a real implementation, this would use the model
        let mut suggestions = Vec::new();
        
        let last_word = text.split_whitespace().last().unwrap_or("");
        
        // Common completions based on context
        match last_word.to_lowercase().as_str() {
            "the" => suggestions.extend(["quick", "best", "first", "last", "only"].iter().map(|s| s.to_string())),
            "to" => suggestions.extend(["be", "have", "do", "make", "get"].iter().map(|s| s.to_string())),
            "i" => suggestions.extend(["am", "will", "have", "can", "want"].iter().map(|s| s.to_string())),
            "thank" => suggestions.push("you".to_string()),
            _ => {}
        }
        
        suggestions.truncate(length);
        suggestions
    }
}

/// Streaming text processor for real-time enhancement
pub struct StreamingTextProcessor {
    inference: Option<StreamingInference>,
    input_tx: Option<mpsc::Sender<TextUpdate>>,
    output_rx: Option<mpsc::Receiver<StreamingResult>>,
}

impl StreamingTextProcessor {
    /// Create a new streaming processor
    pub fn new(model: Arc<dyn AIModel>, config: StreamingConfig) -> Self {
        let (inference, input_tx, output_rx) = StreamingInference::new(model, config);
        
        Self {
            inference: Some(inference),
            input_tx: Some(input_tx),
            output_rx: Some(output_rx),
        }
    }
    
    /// Start processing
    pub async fn start(&mut self) -> Result<()> {
        if let Some(inference) = self.inference.take() {
            inference.start().await?;
        }
        Ok(())
    }
    
    /// Send text update
    pub async fn send_update(&self, update: TextUpdate) -> Result<()> {
        if let Some(tx) = &self.input_tx {
            tx.send(update).await
                .map_err(|_| AIError::InferenceError("Failed to send update".to_string()))?;
        }
        Ok(())
    }
    
    /// Receive enhanced result
    pub async fn receive_result(&mut self) -> Option<StreamingResult> {
        if let Some(rx) = &mut self.output_rx {
            rx.recv().await
        } else {
            None
        }
    }
    
    /// Append text
    pub async fn append_text(&self, text: &str) -> Result<()> {
        self.send_update(TextUpdate::Append(text.to_string())).await
    }
    
    /// Finalize and get remaining results
    pub async fn finalize(&self) -> Result<()> {
        self.send_update(TextUpdate::Finalize).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.min_chunk_size, 20);
        assert_eq!(config.max_wait_ms, 300);
        assert!(config.enable_predictions);
    }
    
    #[tokio::test]
    async fn test_text_position() {
        let pos = TextPosition { start: 0, end: 10 };
        assert_eq!(pos.end - pos.start, 10);
    }
}