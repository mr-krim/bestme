//! Enhanced transcription with advanced Whisper features

use anyhow::Result;
use log::debug;
use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Serialize, Deserialize};

use crate::config::{SpeechSettings, WhisperParamsSettings};
use crate::audio::vad::{VoiceActivityDetector, VADResult};
use crate::audio::vocabulary::VocabularyManager;

#[cfg(feature = "whisper")]
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};

/// Enhanced transcription segment with confidence scores
#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    pub text: String,
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
    pub language_probability: f32,
    pub no_speech_prob: f32,
    pub tokens: Vec<TokenInfo>,
}

impl TranscriptSegment {
    /// Get word-level timing information
    pub fn get_words(&self) -> Vec<WordTiming> {
        let mut words = Vec::new();
        let mut seen_words = std::collections::HashSet::new();
        
        for token in &self.tokens {
            if let Some(word) = &token.word {
                let key = format!("{}-{:?}-{:?}", word, token.word_start, token.word_end);
                if !seen_words.contains(&key) {
                    seen_words.insert(key);
                    words.push(WordTiming {
                        word: word.clone(),
                        start_time: token.word_start.unwrap_or(token.timestamp),
                        end_time: token.word_end.unwrap_or(token.timestamp),
                        confidence: token.probability,
                    });
                }
            }
        }
        
        words
    }
}

/// Word-level timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTiming {
    pub word: String,
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
}

/// Token-level information
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: String,
    pub timestamp: f32,
    pub probability: f32,
    pub word: Option<String>,     // Reconstructed word from tokens
    pub word_start: Option<f32>,  // Word start time
    pub word_end: Option<f32>,    // Word end time
}

/// Context buffer for maintaining conversation history
pub struct TranscriptionContext {
    /// Previous transcriptions for context
    history: VecDeque<String>,
    
    /// Maximum history size
    max_history: usize,
    
    /// Custom vocabulary with boost scores
    custom_vocabulary: std::collections::HashMap<String, f32>,
    
    /// Domain-specific prompts
    domain_prompts: std::collections::HashMap<String, String>,
    
    /// Current domain
    current_domain: Option<String>,
    
    /// Vocabulary manager
    vocabulary_manager: Option<Arc<VocabularyManager>>,
}

impl TranscriptionContext {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(10),
            max_history: 10,
            custom_vocabulary: std::collections::HashMap::new(),
            domain_prompts: Self::default_domain_prompts(),
            current_domain: None,
            vocabulary_manager: None,
        }
    }
    
    /// Set vocabulary manager
    pub fn set_vocabulary_manager(&mut self, manager: Arc<VocabularyManager>) {
        self.vocabulary_manager = Some(manager);
    }
    
    /// Add transcription to history
    pub fn add_to_history(&mut self, text: &str) {
        self.history.push_back(text.to_string());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }
    
    /// Get context prompt for conditioning
    pub fn get_context_prompt(&self) -> String {
        let mut prompt = String::new();
        
        // Add domain-specific prompt if set
        if let Some(domain) = &self.current_domain {
            if let Some(domain_prompt) = self.domain_prompts.get(domain) {
                prompt.push_str(domain_prompt);
                prompt.push_str(" ");
            }
        }
        
        // Add recent history
        if !self.history.is_empty() {
            let recent: Vec<&str> = self.history.iter()
                .rev()
                .take(3)
                .map(|s| s.as_str())
                .collect();
            prompt.push_str(&recent.join(" "));
        }
        
        // Add vocabulary hints from vocabulary manager
        if let Some(vocab_manager) = &self.vocabulary_manager {
            let vocab_hints = vocab_manager.generate_prompt_hints(self.current_domain.as_deref());
            if !vocab_hints.is_empty() {
                prompt.push_str(" ");
                prompt.push_str(&vocab_hints);
            }
        } else if !self.custom_vocabulary.is_empty() {
            // Fallback to custom vocabulary if no manager
            prompt.push_str(" Vocabulary: ");
            let vocab_hints: Vec<&str> = self.custom_vocabulary.keys()
                .take(10)
                .map(|s| s.as_str())
                .collect();
            prompt.push_str(&vocab_hints.join(", "));
        }
        
        prompt
    }
    
    /// Default domain prompts for better accuracy
    fn default_domain_prompts() -> std::collections::HashMap<String, String> {
        let mut prompts = std::collections::HashMap::new();
        
        prompts.insert(
            "technical".to_string(),
            "Technical discussion about programming, software development, APIs, frameworks.".to_string()
        );
        
        prompts.insert(
            "medical".to_string(),
            "Medical conversation with terminology, symptoms, diagnoses, treatments.".to_string()
        );
        
        prompts.insert(
            "legal".to_string(),
            "Legal discussion with contracts, agreements, regulations, compliance.".to_string()
        );
        
        prompts.insert(
            "casual".to_string(),
            "Casual conversation, everyday topics, informal speech.".to_string()
        );
        
        prompts
    }
    
    /// Add custom vocabulary
    pub fn add_vocabulary(&mut self, word: String, boost: f32) {
        self.custom_vocabulary.insert(word, boost);
    }
    
    /// Set current domain
    pub fn set_domain(&mut self, domain: Option<String>) {
        self.current_domain = domain;
    }
}

/// Enhanced Whisper processor with advanced features
pub struct EnhancedWhisperProcessor {
    #[cfg(feature = "whisper")]
    context: Arc<WhisperContext>,
    
    speech_settings: SpeechSettings,
    whisper_params: WhisperParamsSettings,
    transcription_context: Arc<Mutex<TranscriptionContext>>,
    vad: Option<VoiceActivityDetector>,
}

impl EnhancedWhisperProcessor {
    /// Create a new enhanced processor
    #[cfg(feature = "whisper")]
    pub fn new(
        context: Arc<WhisperContext>,
        speech_settings: SpeechSettings,
        whisper_params: WhisperParamsSettings,
    ) -> Self {
        let vad = if whisper_params.vad_enabled {
            Some(VoiceActivityDetector::new(
                whisper_params.vad_threshold,
                whisper_params.min_speech_duration_ms,
                whisper_params.max_silence_duration_ms,
                16000, // Sample rate
            ))
        } else {
            None
        };
        
        Self {
            context,
            speech_settings,
            whisper_params,
            transcription_context: Arc::new(Mutex::new(TranscriptionContext::new())),
            vad,
        }
    }
    
    /// Process audio with enhanced features
    #[cfg(feature = "whisper")]
    pub async fn process_enhanced(
        &mut self,
        audio_data: &[f32],
    ) -> Result<Option<TranscriptSegment>> {
        // Apply VAD if enabled
        if let Some(vad) = &mut self.vad {
            match vad.process(audio_data) {
                VADResult::Silence => {
                    debug!("VAD: Silence detected, skipping processing");
                    return Ok(None);
                }
                VADResult::SpeechEnd => {
                    debug!("VAD: Speech ended, processing accumulated audio");
                }
                _ => {}
            }
        }
        
        // Get context prompt
        let context_prompt = {
            let ctx = self.transcription_context.lock();
            ctx.get_context_prompt()
        };
        
        // Clone necessary data for the blocking task
        let context = Arc::clone(&self.context);
        let audio_data = audio_data.to_vec();
        let language = self.speech_settings.language.clone();
        let translate = self.speech_settings.translate_to_english;
        let params_config = self.whisper_params.clone();
        let initial_prompt = if context_prompt.is_empty() {
            params_config.initial_prompt.clone()
        } else {
            Some(context_prompt)
        };
        
        // Run transcription in blocking task
        let result = tokio::task::spawn_blocking(move || {
            Self::transcribe_with_params(
                &context,
                &audio_data,
                &language,
                translate,
                &params_config,
                initial_prompt,
            )
        }).await??;
        
        // Update context if we got a result
        if let Some(ref segment) = result {
            let mut ctx = self.transcription_context.lock();
            ctx.add_to_history(&segment.text);
        }
        
        Ok(result)
    }
    
    /// Reconstruct words from tokens
    #[doc(hidden)]
    pub fn reconstruct_words(tokens: &mut Vec<TokenInfo>) {
        let mut current_word = String::new();
        let mut word_start: Option<f32> = None;
        let mut word_tokens: Vec<usize> = Vec::new();
        
        for i in 0..tokens.len() {
            let token_str = tokens[i].token.clone();
            let token_timestamp = tokens[i].timestamp;
            let is_word_boundary = token_str.starts_with(' ') 
                || token_str.starts_with('\n')
                || token_str.is_empty()
                || (i > 0 && tokens[i-1].token.ends_with('.'))
                || (i > 0 && tokens[i-1].token.ends_with(','))
                || (i > 0 && tokens[i-1].token.ends_with('!'))
                || (i > 0 && tokens[i-1].token.ends_with('?'));
            
            if is_word_boundary && !current_word.is_empty() {
                // Complete the current word
                let word_end = token_timestamp;
                let word_tokens_copy = word_tokens.clone();
                
                // Update all tokens that form this word
                for idx in word_tokens_copy {
                    tokens[idx].word = Some(current_word.trim().to_string());
                    tokens[idx].word_start = word_start;
                    tokens[idx].word_end = Some(word_end);
                }
                
                // Reset for next word
                current_word.clear();
                word_start = None;
                word_tokens.clear();
            }
            
            // Add token to current word
            if !token_str.is_empty() && !token_str.trim().is_empty() {
                if word_start.is_none() {
                    word_start = Some(token_timestamp);
                }
                current_word.push_str(&token_str);
                word_tokens.push(i);
            }
        }
        
        // Handle last word if any
        if !current_word.is_empty() && !word_tokens.is_empty() {
            let word_end = tokens.last().map(|t| t.timestamp).unwrap_or(0.0);
            for idx in word_tokens {
                tokens[idx].word = Some(current_word.trim().to_string());
                tokens[idx].word_start = word_start;
                tokens[idx].word_end = Some(word_end);
            }
        }
    }
    
    /// Internal transcription with full parameter control
    #[cfg(feature = "whisper")]
    fn transcribe_with_params(
        context: &WhisperContext,
        audio_data: &[f32],
        language: &str,
        translate: bool,
        params_config: &WhisperParamsSettings,
        initial_prompt: Option<String>,
    ) -> Result<Option<TranscriptSegment>> {
        // Create parameters based on configuration
        let mut params = if params_config.beam_size > 0 {
            FullParams::new(SamplingStrategy::BeamSearch {
                beam_size: params_config.beam_size as i32,
                patience: params_config.patience,
            })
        } else {
            FullParams::new(SamplingStrategy::Greedy {
                best_of: params_config.best_of,
            })
        };
        
        // Configure language
        if language.is_empty() || language == "auto" {
            params.set_language(None);
        } else {
            params.set_language(Some(language));
        }
        
        // Set translation
        params.set_translate(translate);
        
        // Set temperature
        params.set_temperature(params_config.temperature);
        
        // Set initial prompt if provided
        if let Some(prompt) = initial_prompt {
            params.set_initial_prompt(&prompt);
        }
        
        // Advanced parameters
        params.set_no_speech_thold(params_config.no_speech_threshold);
        
        // Enable token timestamps for detailed analysis
        params.set_token_timestamps(true);
        
        // Create state and run inference
        let mut state = context.create_state()?;
        state.full(params, audio_data)?;
        
        // Extract segments with detailed information
        let num_segments = state.full_n_segments()?;
        if num_segments == 0 {
            return Ok(None);
        }
        
        let mut full_text = String::new();
        let mut tokens = Vec::new();
        let mut total_confidence = 0.0;
        let mut segment_count = 0;
        
        for i in 0..num_segments {
            if let Ok(text) = state.full_get_segment_text(i) {
                full_text.push_str(&text);
                full_text.push(' ');
                
                // Get token-level information if available
                if let Ok(n_tokens) = state.full_n_tokens(i) {
                    for j in 0..n_tokens {
                        if let Ok(token_text) = state.full_get_token_text(i, j) {
                            if let Ok(token_data) = state.full_get_token_data(i, j) {
                                tokens.push(TokenInfo {
                                    token: token_text,
                                    timestamp: token_data.t0 as f32 / 100.0,
                                    probability: token_data.p,
                                    word: None,
                                    word_start: None,
                                    word_end: None,
                                });
                                total_confidence += token_data.p;
                                segment_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        let text = full_text.trim().to_string();
        if text.is_empty() {
            return Ok(None);
        }
        
        // Reconstruct words from tokens
        Self::reconstruct_words(&mut tokens);
        
        // Calculate average confidence
        let confidence = if segment_count > 0 {
            total_confidence / segment_count as f32
        } else {
            0.5 // Default confidence if no token data
        };
        
        // Get timing information
        let start_time = if let Ok(t) = state.full_get_segment_t0(0) {
            t as f32 / 100.0
        } else {
            0.0
        };
        
        let end_time = if let Ok(t) = state.full_get_segment_t1(num_segments - 1) {
            t as f32 / 100.0
        } else {
            audio_data.len() as f32 / 16000.0
        };
        
        Ok(Some(TranscriptSegment {
            text,
            start_time,
            end_time,
            confidence,
            language_probability: 0.9, // TODO: Get from Whisper
            no_speech_prob: 0.1, // TODO: Get from Whisper
            tokens,
        }))
    }
}

/// Hallucination detector for filtering out repetitive or nonsensical output
pub struct HallucinationDetector {
    /// Common hallucination patterns
    patterns: Vec<regex::Regex>,
    
    /// Repetition threshold
    repetition_threshold: f32,
}

impl HallucinationDetector {
    pub fn new() -> Self {
        let patterns = vec![
            // Note: Backreferences not supported, so we'll check repetitions differently
            regex::Regex::new(r"^(\s*\.\s*){3,}$").unwrap(), // Just dots
            regex::Regex::new(r"^(\s*,\s*){3,}$").unwrap(), // Just commas
            regex::Regex::new(r"thank you for watching").unwrap(), // Common YouTube hallucination
            regex::Regex::new(r"please subscribe").unwrap(), // Another YouTube pattern
            regex::Regex::new(r"(\w+\s+){3,}(\w+\s+){3,}").unwrap(), // Repeated word patterns
        ];
        
        Self {
            patterns,
            repetition_threshold: 0.5,
        }
    }
    
    /// Check if text is likely a hallucination
    pub fn is_hallucination(&self, text: &str, confidence: f32) -> bool {
        // Low confidence is suspicious
        if confidence < 0.3 {
            return true;
        }
        
        // Check patterns
        for pattern in &self.patterns {
            if pattern.is_match(text) {
                return true;
            }
        }
        
        // Check for excessive repetition
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() > 10 {
            let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
            let repetition_ratio = 1.0 - (unique_words.len() as f32 / words.len() as f32);
            if repetition_ratio > self.repetition_threshold {
                return true;
            }
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transcription_context() {
        let mut ctx = TranscriptionContext::new();
        
        // Test history
        ctx.add_to_history("Hello world");
        ctx.add_to_history("How are you");
        
        let prompt = ctx.get_context_prompt();
        assert!(prompt.contains("How are you"));
        
        // Test vocabulary
        ctx.add_vocabulary("API".to_string(), 2.0);
        ctx.add_vocabulary("Rust".to_string(), 1.5);
        
        let prompt = ctx.get_context_prompt();
        assert!(prompt.contains("API"));
        assert!(prompt.contains("Rust"));
    }
    
    #[test]
    fn test_hallucination_detector() {
        let detector = HallucinationDetector::new();
        
        // Normal text
        assert!(!detector.is_hallucination("This is a normal sentence.", 0.8));
        
        // Repetitive text
        assert!(detector.is_hallucination("hello hello hello hello hello", 0.8));
        
        // Low confidence
        assert!(detector.is_hallucination("Maybe this text", 0.2));
        
        // YouTube hallucination
        assert!(detector.is_hallucination("Thank you for watching and please subscribe", 0.9));
    }
    
    #[test]
    fn test_word_reconstruction() {
        let mut tokens = vec![
            TokenInfo {
                token: "Hello".to_string(),
                timestamp: 0.0,
                probability: 0.9,
                word: None,
                word_start: None,
                word_end: None,
            },
            TokenInfo {
                token: " ".to_string(),
                timestamp: 0.5,
                probability: 0.95,
                word: None,
                word_start: None,
                word_end: None,
            },
            TokenInfo {
                token: "world".to_string(),
                timestamp: 0.6,
                probability: 0.85,
                word: None,
                word_start: None,
                word_end: None,
            },
        ];
        
        EnhancedWhisperProcessor::reconstruct_words(&mut tokens);
        
        // Check first token (part of "Hello")
        assert_eq!(tokens[0].word, Some("Hello".to_string()));
        assert_eq!(tokens[0].word_start, Some(0.0));
        assert_eq!(tokens[0].word_end, Some(0.5));
        
        // Check last token (part of "world")
        assert_eq!(tokens[2].word, Some("world".to_string()));
        assert_eq!(tokens[2].word_start, Some(0.6));
        assert_eq!(tokens[2].word_end, Some(0.6));
    }
    
    #[test]
    fn test_get_words_from_segment() {
        let segment = TranscriptSegment {
            text: "Hello world".to_string(),
            start_time: 0.0,
            end_time: 1.0,
            confidence: 0.9,
            language_probability: 0.95,
            no_speech_prob: 0.05,
            tokens: vec![
                TokenInfo {
                    token: "Hello".to_string(),
                    timestamp: 0.0,
                    probability: 0.9,
                    word: Some("Hello".to_string()),
                    word_start: Some(0.0),
                    word_end: Some(0.5),
                },
                TokenInfo {
                    token: " world".to_string(),
                    timestamp: 0.6,
                    probability: 0.85,
                    word: Some("world".to_string()),
                    word_start: Some(0.6),
                    word_end: Some(1.0),
                },
            ],
        };
        
        let words = segment.get_words();
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].word, "Hello");
        assert_eq!(words[0].start_time, 0.0);
        assert_eq!(words[0].end_time, 0.5);
        assert_eq!(words[1].word, "world");
        assert_eq!(words[1].start_time, 0.6);
        assert_eq!(words[1].end_time, 1.0);
    }
}