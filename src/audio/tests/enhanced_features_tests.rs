//! Unit tests for enhanced Whisper features

#[cfg(test)]
mod tests {
    use crate::audio::{
        vad::{VoiceActivityDetector, VADResult},
        enhanced_transcribe::{TranscriptionContext, HallucinationDetector, TokenInfo, TranscriptSegment},
        vocabulary::{VocabularyManager, VocabularyEntry},
        multi_pass::{MultiPassProcessor, ProcessingStrategy, PassConfig, RetryConfig},
        streaming_transcribe::{StreamingConfig, StreamingEvent},
    };
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    // VAD Tests
    #[test]
    fn test_vad_initialization() {
        let vad = VoiceActivityDetector::new(0.5, 250, 2000, 16000);
        assert_eq!(vad.get_threshold(), 0.5);
    }
    
    #[test]
    fn test_vad_speech_detection() {
        let mut vad = VoiceActivityDetector::new(0.3, 250, 2000, 16000);
        
        // Test silence
        let silence = vec![0.0f32; 16000];
        let result = vad.process(&silence);
        assert!(matches!(result, VADResult::Silence));
        
        // Test speech (simulate loud audio)
        let speech = vec![0.8f32; 16000];
        let result = vad.process(&speech);
        assert!(matches!(result, VADResult::SpeechStart | VADResult::Speech));
    }
    
    #[test]
    fn test_vad_adaptive_threshold() {
        let mut vad = VoiceActivityDetector::new(0.5, 250, 2000, 16000);
        
        // Process some background noise
        let noise = vec![0.1f32; 16000];
        for _ in 0..10 {
            vad.process(&noise);
        }
        
        // Threshold should adapt
        assert!(vad.get_threshold() > 0.1);
    }
    
    // Transcription Context Tests
    #[test]
    fn test_transcription_context_history() {
        let mut ctx = TranscriptionContext::new();
        
        ctx.add_to_history("Hello world");
        ctx.add_to_history("How are you");
        ctx.add_to_history("Fine thanks");
        
        let prompt = ctx.get_context_prompt();
        assert!(prompt.contains("Fine thanks"));
        assert!(prompt.contains("How are you"));
    }
    
    #[test]
    fn test_transcription_context_vocabulary() {
        let mut ctx = TranscriptionContext::new();
        
        ctx.add_vocabulary("API".to_string(), 2.0);
        ctx.add_vocabulary("REST".to_string(), 1.5);
        
        let prompt = ctx.get_context_prompt();
        assert!(prompt.contains("API"));
        assert!(prompt.contains("REST"));
    }
    
    #[test]
    fn test_transcription_context_domain() {
        let mut ctx = TranscriptionContext::new();
        
        ctx.set_domain(Some("technical".to_string()));
        let prompt = ctx.get_context_prompt();
        assert!(prompt.contains("Technical discussion"));
        assert!(prompt.contains("programming"));
    }
    
    // Hallucination Detector Tests
    #[test]
    fn test_hallucination_detection() {
        let detector = HallucinationDetector::new();
        
        // Normal text
        assert!(!detector.is_hallucination("This is a normal sentence.", 0.8));
        
        // Repetitive text
        assert!(detector.is_hallucination("hello hello hello hello hello", 0.8));
        
        // Low confidence
        assert!(detector.is_hallucination("Maybe this text", 0.2));
        
        // YouTube hallucination
        assert!(detector.is_hallucination("Thank you for watching and please subscribe", 0.9));
        
        // Just dots
        assert!(detector.is_hallucination(". . . . . .", 0.8));
    }
    
    #[test]
    fn test_hallucination_repetition_detection() {
        let detector = HallucinationDetector::new();
        
        // High repetition ratio
        let repetitive = "the the the quick brown fox the the the";
        assert!(detector.is_hallucination(repetitive, 0.8));
        
        // Normal repetition
        let normal = "the quick brown fox jumps over the lazy dog";
        assert!(!detector.is_hallucination(normal, 0.8));
    }
    
    // Vocabulary Manager Tests
    #[test]
    fn test_vocabulary_manager_basic() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocabulary.json");
        
        let manager = VocabularyManager::new(vocab_path).unwrap();
        
        // Add entry
        let entry = VocabularyEntry::simple("Rust".to_string(), 2.0);
        manager.add_entry(entry.clone()).unwrap();
        
        // Get entry
        let retrieved = manager.get_entry("Rust").unwrap();
        assert_eq!(retrieved.term, "Rust");
        assert_eq!(retrieved.boost, 2.0);
        
        // Search
        let results = manager.search("Ru");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].term, "Rust");
        
        // Remove entry
        manager.remove_entry("Rust").unwrap();
        assert!(manager.get_entry("Rust").is_none());
    }
    
    #[test]
    fn test_vocabulary_categories() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocabulary.json");
        
        let manager = VocabularyManager::new(vocab_path).unwrap();
        
        // Default technical category should exist
        let tech_entries = manager.get_category_entries("technical");
        assert!(!tech_entries.is_empty());
        assert!(tech_entries.iter().any(|e| e.term == "API"));
        
        // Disable category
        manager.set_category_enabled("technical", false).unwrap();
        let enabled = manager.get_enabled_entries();
        assert!(enabled.is_empty());
    }
    
    #[test]
    fn test_vocabulary_variants() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocabulary.json");
        
        let manager = VocabularyManager::new(vocab_path).unwrap();
        
        // Add entry with variants
        let mut entry = VocabularyEntry::simple("colour".to_string(), 1.5);
        entry.variants = vec!["color".to_string()];
        manager.add_entry(entry).unwrap();
        
        // Should find by variant
        assert!(manager.get_entry("color").is_some());
        assert_eq!(manager.get_entry("color").unwrap().term, "colour");
    }
    
    #[test]
    fn test_vocabulary_prompt_generation() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocabulary.json");
        
        let manager = VocabularyManager::new(vocab_path).unwrap();
        
        let prompt = manager.generate_prompt_hints(None);
        assert!(prompt.contains("Vocabulary hints"));
        assert!(prompt.contains("technical:"));
        assert!(prompt.contains("API"));
    }
    
    // Multi-pass Processing Tests
    #[test]
    fn test_pass_configurations() {
        let quick = PassConfig::quick();
        assert_eq!(quick.temperature, 0.0);
        assert_eq!(quick.beam_size, 0);
        assert!(quick.filter_hallucinations);
        
        let refined = PassConfig::refined();
        assert_eq!(refined.temperature, 0.2);
        assert_eq!(refined.beam_size, 5);
        assert!(refined.use_previous_as_prompt);
        
        let verification = PassConfig::verification();
        assert_eq!(verification.temperature, 0.1);
        assert_eq!(verification.beam_size, 10);
        assert_eq!(verification.confidence_threshold, 0.98);
    }
    
    #[test]
    fn test_processing_strategy_selection() {
        use crate::audio::multi_pass::MultiPassProcessor;
        
        // Short audio - single pass
        let strategy = MultiPassProcessor::suggest_strategy(500, 0.1, None);
        assert_eq!(strategy, ProcessingStrategy::Single);
        
        // High noise - triple pass
        let strategy = MultiPassProcessor::suggest_strategy(3000, 0.7, None);
        assert_eq!(strategy, ProcessingStrategy::Triple);
        
        // Low previous confidence - triple pass
        let strategy = MultiPassProcessor::suggest_strategy(3000, 0.2, Some(0.4));
        assert_eq!(strategy, ProcessingStrategy::Triple);
        
        // Medium duration, high confidence - double pass
        let strategy = MultiPassProcessor::suggest_strategy(3000, 0.2, Some(0.9));
        assert_eq!(strategy, ProcessingStrategy::Double);
        
        // Default - adaptive
        let strategy = MultiPassProcessor::suggest_strategy(10000, 0.3, Some(0.7));
        assert_eq!(strategy, ProcessingStrategy::Adaptive);
    }
    
    #[test]
    fn test_retry_configuration() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.min_confidence, 0.6);
        assert_eq!(config.temperature_increment, 0.1);
        assert_eq!(config.beam_increment, 2);
    }
    
    // Streaming Configuration Tests
    #[test]
    fn test_streaming_config() {
        let config = StreamingConfig::default();
        assert_eq!(config.chunk_size, 8000); // 0.5 seconds at 16kHz
        assert_eq!(config.overlap_size, 1600); // 0.1 seconds
        assert!(config.enable_partial_results);
        assert_eq!(config.partial_update_interval_ms, 100);
    }
    
    // Word-level Timestamp Tests
    #[test]
    fn test_word_reconstruction() {
        use crate::audio::enhanced_transcribe::EnhancedWhisperProcessor;
        
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
    fn test_transcript_segment_words() {
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
    
    // Integration test for context + vocabulary
    #[test]
    fn test_context_with_vocabulary_manager() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocabulary.json");
        
        let vocab_manager = VocabularyManager::new(vocab_path).unwrap();
        let mut ctx = TranscriptionContext::new();
        
        // Set vocabulary manager
        ctx.set_vocabulary_manager(std::sync::Arc::new(vocab_manager));
        
        // Set technical domain
        ctx.set_domain(Some("technical".to_string()));
        
        // Add history
        ctx.add_to_history("Working with REST APIs");
        
        let prompt = ctx.get_context_prompt();
        
        // Should contain domain prompt
        assert!(prompt.contains("Technical discussion"));
        
        // Should contain history
        assert!(prompt.contains("Working with REST APIs"));
        
        // Should contain vocabulary hints
        assert!(prompt.contains("Vocabulary hints"));
        assert!(prompt.contains("API"));
    }
}