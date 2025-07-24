//! Streaming transcription with real-time processing

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::{mpsc, oneshot};
use std::collections::VecDeque;

use crate::config::{SpeechSettings, WhisperParamsSettings};
use crate::audio::vad::{VoiceActivityDetector, VADResult};
use crate::audio::enhanced_transcribe::{TranscriptSegment, HallucinationDetector, EnhancedWhisperProcessor};

#[cfg(feature = "whisper")]
use whisper_rs::WhisperContext;

/// Configuration for streaming transcription
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Chunk size in samples (e.g., 0.5 seconds at 16kHz = 8000 samples)
    pub chunk_size: usize,
    
    /// Overlap between chunks in samples (for context)
    pub overlap_size: usize,
    
    /// Maximum buffer size before forcing processing
    pub max_buffer_size: usize,
    
    /// Minimum speech duration before processing (ms)
    pub min_speech_duration_ms: u32,
    
    /// Maximum silence before ending speech segment (ms)
    pub max_silence_duration_ms: u32,
    
    /// Enable partial results
    pub enable_partial_results: bool,
    
    /// Partial result update interval (ms)
    pub partial_update_interval_ms: u32,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 8000,        // 0.5 seconds at 16kHz
            overlap_size: 1600,      // 0.1 seconds overlap
            max_buffer_size: 48000,  // 3 seconds max
            min_speech_duration_ms: 300,
            max_silence_duration_ms: 800,
            enable_partial_results: true,
            partial_update_interval_ms: 100,
        }
    }
}

/// Streaming transcription event
#[derive(Debug, Clone)]
pub enum StreamingEvent {
    /// Speech segment started
    SpeechStart { timestamp_ms: u64 },
    
    /// Speech segment ended
    SpeechEnd { timestamp_ms: u64 },
    
    /// Partial transcription result
    PartialTranscript {
        text: String,
        start_ms: u64,
        is_final: bool,
    },
    
    /// Final transcription result
    FinalTranscript {
        segment: TranscriptSegment,
    },
    
    /// Processing latency update
    LatencyUpdate { latency_ms: u32 },
}

/// Streaming processor state
#[derive(Debug, Clone, PartialEq)]
enum StreamingState {
    Idle,
    Listening,
    Processing,
    Speaking,
}

/// Streaming transcription processor
pub struct StreamingTranscriptionProcessor {
    /// Enhanced Whisper processor
    #[cfg(feature = "whisper")]
    enhanced_processor: Option<EnhancedWhisperProcessor>,
    
    /// Voice activity detector
    vad: VoiceActivityDetector,
    
    /// Streaming configuration
    config: StreamingConfig,
    
    /// Audio buffer for current speech segment
    speech_buffer: VecDeque<f32>,
    
    /// Context buffer for overlap
    context_buffer: VecDeque<f32>,
    
    /// Event sender
    event_sender: mpsc::Sender<StreamingEvent>,
    
    /// Current state
    state: StreamingState,
    
    /// Timestamp of speech start
    speech_start_time: Option<std::time::Instant>,
    
    /// Last partial update time
    last_partial_update: std::time::Instant,
    
    /// Hallucination detector
    hallucination_detector: Arc<HallucinationDetector>,
    
    /// Sample rate
    sample_rate: usize,
}

impl StreamingTranscriptionProcessor {
    /// Create a new streaming processor
    #[cfg(feature = "whisper")]
    pub fn new(
        whisper_context: Arc<WhisperContext>,
        speech_settings: SpeechSettings,
        whisper_params: WhisperParamsSettings,
        streaming_config: StreamingConfig,
        event_sender: mpsc::Sender<StreamingEvent>,
    ) -> Self {
        let sample_rate = 16000;
        
        let vad = VoiceActivityDetector::new(
            whisper_params.vad_threshold,
            streaming_config.min_speech_duration_ms,
            streaming_config.max_silence_duration_ms,
            sample_rate,
        );
        
        let enhanced_processor = Some(EnhancedWhisperProcessor::new(
            whisper_context,
            speech_settings,
            whisper_params,
        ));
        
        Self {
            enhanced_processor,
            vad,
            config: streaming_config,
            speech_buffer: VecDeque::with_capacity(48000),
            context_buffer: VecDeque::with_capacity(8000),
            state: StreamingState::Idle,
            speech_start_time: None,
            last_partial_update: std::time::Instant::now(),
            event_sender,
            hallucination_detector: Arc::new(HallucinationDetector::new()),
            sample_rate,
        }
    }
    
    /// Process audio chunk
    pub async fn process_chunk(&mut self, audio_chunk: &[f32]) -> Result<()> {
        let process_start = std::time::Instant::now();
        
        // Run VAD on the chunk
        let vad_result = self.vad.process(audio_chunk);
        
        match vad_result {
            VADResult::SpeechStart => {
                self.handle_speech_start().await?;
            }
            VADResult::Speech => {
                self.handle_speech(audio_chunk).await?;
            }
            VADResult::SpeechEnd => {
                self.handle_speech_end().await?;
            }
            VADResult::Silence => {
                self.handle_silence().await?;
            }
        }
        
        // Report latency
        let latency_ms = process_start.elapsed().as_millis() as u32;
        let _ = self.event_sender.send(StreamingEvent::LatencyUpdate { latency_ms }).await;
        
        Ok(())
    }
    
    /// Handle speech start
    async fn handle_speech_start(&mut self) -> Result<()> {
        info!("Speech started");
        self.state = StreamingState::Speaking;
        self.speech_start_time = Some(std::time::Instant::now());
        
        // Clear speech buffer but keep context
        self.speech_buffer.clear();
        
        // Send event
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis() as u64;
        
        let _ = self.event_sender.send(StreamingEvent::SpeechStart { timestamp_ms }).await;
        
        Ok(())
    }
    
    /// Handle ongoing speech
    async fn handle_speech(&mut self, audio_chunk: &[f32]) -> Result<()> {
        // Add to speech buffer
        self.speech_buffer.extend(audio_chunk);
        
        // Limit buffer size
        while self.speech_buffer.len() > self.config.max_buffer_size {
            self.speech_buffer.pop_front();
        }
        
        // Process partial results if enabled
        if self.config.enable_partial_results 
            && self.last_partial_update.elapsed().as_millis() >= self.config.partial_update_interval_ms as u128 {
            self.process_partial().await?;
            self.last_partial_update = std::time::Instant::now();
        }
        
        Ok(())
    }
    
    /// Handle speech end
    async fn handle_speech_end(&mut self) -> Result<()> {
        info!("Speech ended, processing final segment");
        self.state = StreamingState::Processing;
        
        // Process final segment
        self.process_final().await?;
        
        // Update context buffer with end of speech
        self.update_context_buffer();
        
        // Send event
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis() as u64;
        
        let _ = self.event_sender.send(StreamingEvent::SpeechEnd { timestamp_ms }).await;
        
        self.state = StreamingState::Idle;
        self.speech_start_time = None;
        
        Ok(())
    }
    
    /// Handle silence
    async fn handle_silence(&mut self) -> Result<()> {
        // Just maintain idle state
        if self.state == StreamingState::Speaking {
            // This shouldn't happen with proper VAD
            warn!("Unexpected silence during speech");
        }
        Ok(())
    }
    
    /// Process partial transcription
    async fn process_partial(&mut self) -> Result<()> {
        if self.speech_buffer.len() < self.config.chunk_size {
            return Ok(());
        }
        
        #[cfg(feature = "whisper")]
        if let Some(processor) = &mut self.enhanced_processor {
            // Combine context and current speech
            let mut audio_data = Vec::with_capacity(self.context_buffer.len() + self.speech_buffer.len());
            audio_data.extend(&self.context_buffer);
            audio_data.extend(&self.speech_buffer);
            
            // Process with enhanced features
            match processor.process_enhanced(&audio_data).await {
                Ok(Some(segment)) => {
                    // Check for hallucination
                    if !self.hallucination_detector.is_hallucination(&segment.text, segment.confidence) {
                        let start_ms = self.speech_start_time
                            .map(|t| t.elapsed().as_millis() as u64)
                            .unwrap_or(0);
                        
                        let _ = self.event_sender.send(StreamingEvent::PartialTranscript {
                            text: segment.text,
                            start_ms,
                            is_final: false,
                        }).await;
                    }
                }
                Ok(None) => {
                    debug!("No partial result from enhanced processor");
                }
                Err(e) => {
                    warn!("Error processing partial transcription: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Process final transcription
    async fn process_final(&mut self) -> Result<()> {
        if self.speech_buffer.is_empty() {
            return Ok(());
        }
        
        #[cfg(feature = "whisper")]
        if let Some(processor) = &mut self.enhanced_processor {
            // Combine context and speech for final processing
            let mut audio_data = Vec::with_capacity(self.context_buffer.len() + self.speech_buffer.len());
            audio_data.extend(&self.context_buffer);
            audio_data.extend(&self.speech_buffer);
            
            // Process with enhanced features
            match processor.process_enhanced(&audio_data).await {
                Ok(Some(segment)) => {
                    // Check for hallucination
                    if !self.hallucination_detector.is_hallucination(&segment.text, segment.confidence) {
                        let _ = self.event_sender.send(StreamingEvent::FinalTranscript {
                            segment,
                        }).await;
                    } else {
                        info!("Filtered out hallucinated text");
                    }
                }
                Ok(None) => {
                    debug!("No final result from enhanced processor");
                }
                Err(e) => {
                    warn!("Error processing final transcription: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Update context buffer with recent audio
    fn update_context_buffer(&mut self) {
        // Keep last overlap_size samples as context
        self.context_buffer.clear();
        
        let start_idx = self.speech_buffer.len().saturating_sub(self.config.overlap_size);
        for sample in self.speech_buffer.range(start_idx..) {
            self.context_buffer.push_back(*sample);
        }
        
        // Clear speech buffer
        self.speech_buffer.clear();
    }
    
    /// Force processing of current buffer
    pub async fn force_process(&mut self) -> Result<()> {
        if self.state == StreamingState::Speaking && !self.speech_buffer.is_empty() {
            info!("Forcing processing of current speech buffer");
            self.handle_speech_end().await?;
        }
        Ok(())
    }
    
    /// Reset the processor state
    pub fn reset(&mut self) {
        self.state = StreamingState::Idle;
        self.speech_buffer.clear();
        self.context_buffer.clear();
        self.speech_start_time = None;
        self.vad.reset();
    }
}

/// Streaming transcription manager
pub struct StreamingTranscriptionManager {
    /// Processor handle
    processor: Arc<Mutex<Option<StreamingTranscriptionProcessor>>>,
    
    /// Control channel
    control_tx: mpsc::Sender<StreamingControl>,
    control_rx: Arc<Mutex<mpsc::Receiver<StreamingControl>>>,
    
    /// Event sender (for passing to processor)
    event_tx: mpsc::Sender<StreamingEvent>,
    
    /// Processing task handle
    task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug)]
enum StreamingControl {
    Start,
    Stop,
    Reset,
    ForceProcess,
}

impl StreamingTranscriptionManager {
    /// Create a new streaming manager
    pub fn new() -> (Self, mpsc::Receiver<StreamingEvent>) {
        let (event_tx, event_rx) = mpsc::channel(100);
        let (control_tx, control_rx) = mpsc::channel(10);
        
        let manager = Self {
            processor: Arc::new(Mutex::new(None)),
            control_tx,
            control_rx: Arc::new(Mutex::new(control_rx)),
            event_tx,
            task_handle: Arc::new(Mutex::new(None)),
        };
        
        (manager, event_rx)
    }
    
    /// Initialize with Whisper context
    #[cfg(feature = "whisper")]
    pub fn initialize(
        &mut self,
        whisper_context: Arc<WhisperContext>,
        speech_settings: SpeechSettings,
        whisper_params: WhisperParamsSettings,
        streaming_config: StreamingConfig,
    ) -> Result<()> {
        let processor = StreamingTranscriptionProcessor::new(
            whisper_context,
            speech_settings,
            whisper_params,
            streaming_config,
            self.event_tx.clone(),
        );
        
        *self.processor.lock() = Some(processor);
        Ok(())
    }
    
    /// Start streaming
    pub async fn start(&mut self) -> Result<()> {
        self.control_tx.send(StreamingControl::Start).await?;
        Ok(())
    }
    
    /// Stop streaming
    pub async fn stop(&mut self) -> Result<()> {
        self.control_tx.send(StreamingControl::Stop).await?;
        Ok(())
    }
    
    /// Force process current buffer
    pub async fn force_process(&mut self) -> Result<()> {
        self.control_tx.send(StreamingControl::ForceProcess).await?;
        Ok(())
    }
    
    /// Reset state
    pub async fn reset(&mut self) -> Result<()> {
        self.control_tx.send(StreamingControl::Reset).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_streaming_config() {
        let config = StreamingConfig::default();
        assert_eq!(config.chunk_size, 8000);
        assert_eq!(config.overlap_size, 1600);
        assert!(config.enable_partial_results);
    }
    
    #[test]
    fn test_vad_integration() {
        let vad = VoiceActivityDetector::new(0.1, 300, 800, 16000);
        
        // Generate test audio
        let silence = vec![0.0f32; 8000];
        let speech = vec![0.5f32; 8000];
        
        // VAD should detect silence
        let result = vad.process(&silence);
        assert!(matches!(result, VADResult::Silence));
    }
}