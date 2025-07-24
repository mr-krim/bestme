//! Multi-pass processing for improved transcription accuracy

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::sync::Arc;
use parking_lot::Mutex;

use crate::config::{SpeechSettings, WhisperParamsSettings};
use crate::audio::enhanced_transcribe::{
    EnhancedWhisperProcessor, TranscriptSegment, TranscriptionContext, HallucinationDetector
};

#[cfg(feature = "whisper")]
use whisper_rs::WhisperContext;

/// Multi-pass processing strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessingStrategy {
    /// Single pass (fastest)
    Single,
    
    /// Two passes: quick + refined
    Double,
    
    /// Three passes: quick + refined + verification
    Triple,
    
    /// Adaptive based on confidence
    Adaptive,
}

/// Pass configuration
#[derive(Debug, Clone)]
pub struct PassConfig {
    /// Temperature for this pass
    pub temperature: f32,
    
    /// Beam size (0 for greedy)
    pub beam_size: u32,
    
    /// Best of N samples
    pub best_of: i32,
    
    /// Use previous pass result as prompt
    pub use_previous_as_prompt: bool,
    
    /// Minimum confidence to skip remaining passes
    pub confidence_threshold: f32,
    
    /// Whether to apply hallucination filter
    pub filter_hallucinations: bool,
}

impl PassConfig {
    /// Quick first pass configuration
    pub fn quick() -> Self {
        Self {
            temperature: 0.0,  // Deterministic
            beam_size: 0,      // Greedy decoding
            best_of: 1,
            use_previous_as_prompt: false,
            confidence_threshold: 0.9,
            filter_hallucinations: true,
        }
    }
    
    /// Refined second pass configuration
    pub fn refined() -> Self {
        Self {
            temperature: 0.2,
            beam_size: 5,
            best_of: 3,
            use_previous_as_prompt: true,
            confidence_threshold: 0.95,
            filter_hallucinations: true,
        }
    }
    
    /// Verification third pass configuration
    pub fn verification() -> Self {
        Self {
            temperature: 0.1,
            beam_size: 10,
            best_of: 5,
            use_previous_as_prompt: true,
            confidence_threshold: 0.98,
            filter_hallucinations: true,
        }
    }
}

/// Multi-pass processing result
#[derive(Debug, Clone)]
pub struct MultiPassResult {
    /// Final transcript segment
    pub final_segment: TranscriptSegment,
    
    /// Results from each pass
    pub pass_results: Vec<PassResult>,
    
    /// Total processing time
    pub total_time_ms: u64,
    
    /// Strategy used
    pub strategy: ProcessingStrategy,
}

/// Result from a single pass
#[derive(Debug, Clone)]
pub struct PassResult {
    /// Pass number (1-indexed)
    pub pass_number: usize,
    
    /// Transcript segment from this pass
    pub segment: TranscriptSegment,
    
    /// Processing time for this pass
    pub time_ms: u64,
    
    /// Whether this pass was skipped
    pub skipped: bool,
    
    /// Reason for skipping (if applicable)
    pub skip_reason: Option<String>,
}

/// Multi-pass transcription processor
pub struct MultiPassProcessor {
    /// Whisper context
    #[cfg(feature = "whisper")]
    context: Arc<WhisperContext>,
    
    /// Base speech settings
    speech_settings: SpeechSettings,
    
    /// Base whisper parameters
    base_params: WhisperParamsSettings,
    
    /// Processing strategy
    strategy: ProcessingStrategy,
    
    /// Transcription context
    transcription_context: Arc<Mutex<TranscriptionContext>>,
    
    /// Hallucination detector
    hallucination_detector: Arc<HallucinationDetector>,
    
    /// Pass configurations
    pass_configs: Vec<PassConfig>,
}

impl MultiPassProcessor {
    /// Create a new multi-pass processor
    #[cfg(feature = "whisper")]
    pub fn new(
        context: Arc<WhisperContext>,
        speech_settings: SpeechSettings,
        base_params: WhisperParamsSettings,
        strategy: ProcessingStrategy,
    ) -> Self {
        let pass_configs = match strategy {
            ProcessingStrategy::Single => vec![PassConfig::quick()],
            ProcessingStrategy::Double => vec![
                PassConfig::quick(),
                PassConfig::refined(),
            ],
            ProcessingStrategy::Triple => vec![
                PassConfig::quick(),
                PassConfig::refined(),
                PassConfig::verification(),
            ],
            ProcessingStrategy::Adaptive => vec![
                PassConfig::quick(),
                PassConfig::refined(),
                PassConfig::verification(),
            ],
        };
        
        Self {
            context,
            speech_settings,
            base_params,
            strategy,
            transcription_context: Arc::new(Mutex::new(TranscriptionContext::new())),
            hallucination_detector: Arc::new(HallucinationDetector::new()),
            pass_configs,
        }
    }
    
    /// Process audio with multiple passes
    #[cfg(feature = "whisper")]
    pub async fn process(&mut self, audio_data: &[f32]) -> Result<MultiPassResult> {
        let start_time = std::time::Instant::now();
        let mut pass_results = Vec::new();
        let mut best_segment: Option<TranscriptSegment> = None;
        let mut previous_text: Option<String> = None;
        
        for (i, pass_config) in self.pass_configs.iter().enumerate() {
            let pass_start = std::time::Instant::now();
            let pass_number = i + 1;
            
            // Check if we should skip based on confidence
            if let Some(ref segment) = best_segment {
                if segment.confidence >= pass_config.confidence_threshold {
                    info!("Skipping pass {} due to high confidence ({:.2})", pass_number, segment.confidence);
                    
                    pass_results.push(PassResult {
                        pass_number,
                        segment: segment.clone(),
                        time_ms: 0,
                        skipped: true,
                        skip_reason: Some(format!("Confidence {:.2} exceeds threshold", segment.confidence)),
                    });
                    
                    if self.strategy == ProcessingStrategy::Adaptive {
                        break;
                    }
                    continue;
                }
            }
            
            // Create pass-specific parameters
            let mut params = self.base_params.clone();
            params.temperature = pass_config.temperature;
            params.beam_size = pass_config.beam_size;
            params.best_of = pass_config.best_of;
            
            // Use previous result as prompt if configured
            if pass_config.use_previous_as_prompt && previous_text.is_some() {
                params.initial_prompt = previous_text.clone();
            }
            
            // Create processor for this pass
            let mut processor = EnhancedWhisperProcessor::new(
                Arc::clone(&self.context),
                self.speech_settings.clone(),
                params,
            );
            
            // Process audio
            match processor.process_enhanced(audio_data).await {
                Ok(Some(mut segment)) => {
                    // Apply hallucination filter if enabled
                    if pass_config.filter_hallucinations {
                        if self.hallucination_detector.is_hallucination(&segment.text, segment.confidence) {
                            warn!("Pass {} produced hallucination, discarding", pass_number);
                            
                            pass_results.push(PassResult {
                                pass_number,
                                segment: segment.clone(),
                                time_ms: pass_start.elapsed().as_millis() as u64,
                                skipped: false,
                                skip_reason: Some("Hallucination detected".to_string()),
                            });
                            continue;
                        }
                    }
                    
                    // Compare with previous best
                    if best_segment.is_none() || segment.confidence > best_segment.as_ref().unwrap().confidence {
                        info!("Pass {} improved confidence: {:.2}", pass_number, segment.confidence);
                        previous_text = Some(segment.text.clone());
                        best_segment = Some(segment.clone());
                    }
                    
                    pass_results.push(PassResult {
                        pass_number,
                        segment,
                        time_ms: pass_start.elapsed().as_millis() as u64,
                        skipped: false,
                        skip_reason: None,
                    });
                }
                Ok(None) => {
                    debug!("Pass {} produced no result", pass_number);
                    
                    pass_results.push(PassResult {
                        pass_number,
                        segment: TranscriptSegment {
                            text: String::new(),
                            start_time: 0.0,
                            end_time: 0.0,
                            confidence: 0.0,
                            language_probability: 0.0,
                            no_speech_prob: 1.0,
                            tokens: Vec::new(),
                        },
                        time_ms: pass_start.elapsed().as_millis() as u64,
                        skipped: false,
                        skip_reason: Some("No speech detected".to_string()),
                    });
                }
                Err(e) => {
                    warn!("Pass {} failed: {}", pass_number, e);
                    
                    pass_results.push(PassResult {
                        pass_number,
                        segment: TranscriptSegment {
                            text: String::new(),
                            start_time: 0.0,
                            end_time: 0.0,
                            confidence: 0.0,
                            language_probability: 0.0,
                            no_speech_prob: 1.0,
                            tokens: Vec::new(),
                        },
                        time_ms: pass_start.elapsed().as_millis() as u64,
                        skipped: false,
                        skip_reason: Some(format!("Error: {}", e)),
                    });
                }
            }
        }
        
        // Get final result
        let final_segment = best_segment.ok_or_else(|| {
            anyhow::anyhow!("No successful transcription in any pass")
        })?;
        
        // Update context
        {
            let mut ctx = self.transcription_context.lock();
            ctx.add_to_history(&final_segment.text);
        }
        
        Ok(MultiPassResult {
            final_segment,
            pass_results,
            total_time_ms: start_time.elapsed().as_millis() as u64,
            strategy: self.strategy,
        })
    }
    
    /// Set transcription context
    pub fn set_context(&mut self, context: Arc<Mutex<TranscriptionContext>>) {
        self.transcription_context = context;
    }
    
    /// Get optimal strategy based on audio characteristics
    pub fn suggest_strategy(
        audio_duration_ms: u64,
        noise_level: f32,
        previous_confidence: Option<f32>,
    ) -> ProcessingStrategy {
        // Short audio: single pass
        if audio_duration_ms < 1000 {
            return ProcessingStrategy::Single;
        }
        
        // High noise or low previous confidence: triple pass
        if noise_level > 0.5 || previous_confidence.map(|c| c < 0.6).unwrap_or(false) {
            return ProcessingStrategy::Triple;
        }
        
        // Medium duration or moderate confidence: double pass
        if audio_duration_ms < 5000 || previous_confidence.map(|c| c > 0.8).unwrap_or(false) {
            return ProcessingStrategy::Double;
        }
        
        // Default to adaptive
        ProcessingStrategy::Adaptive
    }
}

/// Configuration for confidence-based retry
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: usize,
    
    /// Minimum confidence to accept result
    pub min_confidence: f32,
    
    /// Temperature increment per retry
    pub temperature_increment: f32,
    
    /// Beam size increment per retry
    pub beam_increment: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            min_confidence: 0.6,
            temperature_increment: 0.1,
            beam_increment: 2,
        }
    }
}

/// Confidence-based retry mechanism
pub struct ConfidenceRetryProcessor {
    /// Base processor
    processor: MultiPassProcessor,
    
    /// Retry configuration
    retry_config: RetryConfig,
}

impl ConfidenceRetryProcessor {
    /// Create new retry processor
    #[cfg(feature = "whisper")]
    pub fn new(
        context: Arc<WhisperContext>,
        speech_settings: SpeechSettings,
        base_params: WhisperParamsSettings,
        retry_config: RetryConfig,
    ) -> Self {
        let processor = MultiPassProcessor::new(
            context,
            speech_settings,
            base_params,
            ProcessingStrategy::Adaptive,
        );
        
        Self {
            processor,
            retry_config,
        }
    }
    
    /// Process with confidence-based retry
    #[cfg(feature = "whisper")]
    pub async fn process_with_retry(&mut self, audio_data: &[f32]) -> Result<MultiPassResult> {
        let mut best_result: Option<MultiPassResult> = None;
        let mut retry_count = 0;
        
        while retry_count <= self.retry_config.max_retries {
            // Adjust parameters for retry
            if retry_count > 0 {
                self.processor.base_params.temperature += self.retry_config.temperature_increment;
                self.processor.base_params.beam_size += self.retry_config.beam_increment;
                
                info!("Retry {}: temp={:.2}, beam={}", 
                    retry_count, 
                    self.processor.base_params.temperature,
                    self.processor.base_params.beam_size
                );
            }
            
            // Process
            match self.processor.process(audio_data).await {
                Ok(result) => {
                    let confidence = result.final_segment.confidence;
                    
                    // Check if we meet confidence threshold
                    if confidence >= self.retry_config.min_confidence {
                        info!("Achieved required confidence {:.2} on attempt {}", confidence, retry_count + 1);
                        return Ok(result);
                    }
                    
                    // Keep best result
                    if best_result.is_none() || 
                       confidence > best_result.as_ref().unwrap().final_segment.confidence {
                        best_result = Some(result);
                    }
                    
                    retry_count += 1;
                }
                Err(e) => {
                    warn!("Retry {} failed: {}", retry_count, e);
                    retry_count += 1;
                }
            }
        }
        
        // Return best result or error
        best_result.ok_or_else(|| {
            anyhow::anyhow!("Failed to achieve minimum confidence after {} retries", self.retry_config.max_retries)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pass_configs() {
        let quick = PassConfig::quick();
        assert_eq!(quick.temperature, 0.0);
        assert_eq!(quick.beam_size, 0);
        
        let refined = PassConfig::refined();
        assert_eq!(refined.temperature, 0.2);
        assert_eq!(refined.beam_size, 5);
        
        let verification = PassConfig::verification();
        assert_eq!(verification.temperature, 0.1);
        assert_eq!(verification.beam_size, 10);
    }
    
    #[test]
    fn test_strategy_selection() {
        // Short audio
        let strategy = MultiPassProcessor::suggest_strategy(500, 0.1, None);
        assert_eq!(strategy, ProcessingStrategy::Single);
        
        // High noise
        let strategy = MultiPassProcessor::suggest_strategy(3000, 0.7, None);
        assert_eq!(strategy, ProcessingStrategy::Triple);
        
        // Low previous confidence
        let strategy = MultiPassProcessor::suggest_strategy(3000, 0.2, Some(0.4));
        assert_eq!(strategy, ProcessingStrategy::Triple);
        
        // Medium duration
        let strategy = MultiPassProcessor::suggest_strategy(3000, 0.2, Some(0.9));
        assert_eq!(strategy, ProcessingStrategy::Double);
        
        // Default case
        let strategy = MultiPassProcessor::suggest_strategy(10000, 0.3, Some(0.7));
        assert_eq!(strategy, ProcessingStrategy::Adaptive);
    }
    
    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.min_confidence, 0.6);
        assert_eq!(config.temperature_increment, 0.1);
        assert_eq!(config.beam_increment, 2);
    }
}