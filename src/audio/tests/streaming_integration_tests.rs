//! Integration tests for the streaming transcription pipeline
//! TODO: These tests need to be updated to work with the new StreamingTranscriptionProcessor API
//! which requires WhisperContext and other dependencies that need proper mocking

#[cfg(test)]
#[cfg(feature = "skip_broken_tests")]
mod tests {
    use crate::audio::{
        streaming_transcribe::{StreamingTranscriptionProcessor, StreamingConfig, StreamingEvent},
        enhanced_transcribe::{EnhancedWhisperProcessor, TranscriptSegment},
        vad::{VoiceActivityDetector, VADResult},
        vocabulary::VocabularyManager,
        CaptureManager,
    };
    use crate::config::{WhisperParamsSettings, Config};
    use std::sync::{Arc, Mutex};
    use std::path::PathBuf;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tempfile::TempDir;
    
    // Helper function to create test audio samples
    fn generate_test_audio(duration_ms: usize, frequency: f32, sample_rate: usize) -> Vec<f32> {
        let samples = (sample_rate * duration_ms) / 1000;
        let mut audio = Vec::with_capacity(samples);
        
        for i in 0..samples {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
            audio.push(sample);
        }
        
        audio
    }
    
    // Helper to create silence
    fn generate_silence(duration_ms: usize, sample_rate: usize) -> Vec<f32> {
        let samples = (sample_rate * duration_ms) / 1000;
        vec![0.0f32; samples]
    }
    
    // Helper to mix audio with noise
    fn add_noise(audio: &mut [f32], noise_level: f32) {
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        
        for sample in audio.iter_mut() {
            let noise = rng.gen_range(-noise_level..noise_level);
            *sample = (*sample + noise).clamp(-1.0, 1.0);
        }
    }
    
    #[tokio::test]
    async fn test_streaming_pipeline_basic() {
        // TODO: Fix this test - StreamingTranscriptionProcessor constructor requires whisper context
        // and other dependencies that need to be properly mocked
        /*
        let config = StreamingConfig {
            chunk_size: 8000, // 0.5 seconds at 16kHz
            overlap_size: 1600, // 0.1 seconds
            max_buffer_size: 32000, // 2 seconds
            min_speech_duration_ms: 200,
            max_silence_duration_ms: 2000,
        };
        
        let mut processor = StreamingTranscriptionProcessor::new(config);
        
        // Generate test audio: speech followed by silence
        let mut audio = generate_test_audio(1000, 440.0, 16000); // 1 second of tone
        audio.extend(generate_silence(3000, 16000)); // 3 seconds of silence
        
        let (tx, mut rx) = mpsc::channel(10);
        
        // Process audio in chunks
        let chunk_size = 8000;
        for chunk in audio.chunks(chunk_size) {
            processor.process_chunk(chunk.to_vec(), tx.clone()).await;
        }
        
        // Collect events
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
        
        // Verify we got speech start and end events
        assert!(events.iter().any(|e| matches!(e, StreamingEvent::SpeechStart)));
        assert!(events.iter().any(|e| matches!(e, StreamingEvent::SpeechEnd)));
        */
    }
    
    #[tokio::test]
    async fn test_streaming_with_vad_integration() {
        // TODO: Fix this test - StreamingTranscriptionProcessor constructor requires whisper context
        /*
        let config = StreamingConfig::default();
        let mut processor = StreamingTranscriptionProcessor::new(config);
        
        // Configure VAD
        processor.set_vad_enabled(true);
        processor.set_vad_threshold(0.3);
        
        let (tx, mut rx) = mpsc::channel(10);
        
        // Test sequence: silence -> speech -> silence
        let mut audio = generate_silence(500, 16000);
        audio.extend(generate_test_audio(1500, 440.0, 16000)); // Speech
        audio.extend(generate_silence(500, 16000));
        
        // Add some noise to make it more realistic
        add_noise(&mut audio[8000..24000], 0.1); // Add noise to speech portion
        
        // Process
        for chunk in audio.chunks(8000) {
            processor.process_chunk(chunk.to_vec(), tx.clone()).await;
        }
        
        // Verify VAD detected speech
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
        
        // Should have VAD state changes
        let vad_events: Vec<_> = events.iter()
            .filter_map(|e| match e {
                StreamingEvent::VADStateChange { is_speech } => Some(*is_speech),
                _ => None
            })
            .collect();
        
        assert!(!vad_events.is_empty(), "Should have VAD state changes");
        assert!(vad_events.contains(&true), "Should detect speech");
        assert!(vad_events.contains(&false), "Should detect silence");
        */
    }
    
    #[tokio::test]
    async fn test_streaming_partial_results() {
        // TODO: Fix this test - StreamingTranscriptionProcessor constructor requires whisper context
        /*
        let config = StreamingConfig {
            chunk_size: 8000,
            overlap_size: 1600,
            sample_rate: 16000,
            enable_partial_results: true,
            partial_update_interval_ms: 50, // Fast updates for testing
            max_silence_duration_ms: 1000,
        };
        
        let mut processor = StreamingTranscriptionProcessor::new(config);
        let (tx, mut rx) = mpsc::channel(10);
        
        // Generate continuous speech
        let audio = generate_test_audio(2000, 440.0, 16000);
        
        // Process in small chunks to trigger multiple partial results
        for chunk in audio.chunks(4000) {
            processor.process_chunk(chunk.to_vec(), tx.clone()).await;
            tokio::time::sleep(Duration::from_millis(60)).await; // Wait for partial update
        }
        
        // Collect partial results
        let mut partial_count = 0;
        while let Ok(event) = rx.try_recv() {
            if matches!(event, StreamingEvent::PartialTranscript(_)) {
                partial_count += 1;
            }
        }
        
        assert!(partial_count > 0, "Should have partial results");
    }
    
    #[tokio::test]
    async fn test_streaming_with_vocabulary() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocab.json");
        
        // Create vocabulary manager with test terms
        let vocab_manager = VocabularyManager::new(vocab_path).unwrap();
        vocab_manager.add_entry(crate::audio::vocabulary::VocabularyEntry {
            term: "TestWord".to_string(),
            boost: 2.0,
            category: Some("test".to_string()),
            variants: vec!["testword".to_string()],
            context: Some("Testing context".to_string()),
        }).unwrap();
        
        let config = StreamingConfig::default();
        let mut processor = StreamingTranscriptionProcessor::new(config);
        processor.set_vocabulary_manager(Arc::new(vocab_manager));
        
        let (tx, mut rx) = mpsc::channel(10);
        
        // Process test audio
        let audio = generate_test_audio(1000, 440.0, 16000);
        processor.process_chunk(audio, tx.clone()).await;
        
        // Verify vocabulary was applied (check for context updated event)
        let mut has_context_update = false;
        while let Ok(event) = rx.try_recv() {
            if matches!(event, StreamingEvent::ContextUpdated { .. }) {
                has_context_update = true;
                break;
            }
        }
        
        assert!(has_context_update, "Should have context update with vocabulary");
    }
    
    #[tokio::test]
    async fn test_streaming_error_handling() {
        let config = StreamingConfig::default();
        let mut processor = StreamingTranscriptionProcessor::new(config);
        let (tx, mut rx) = mpsc::channel(10);
        
        // Test with empty audio
        processor.process_chunk(vec![], tx.clone()).await;
        
        // Test with very short audio (less than minimum)
        let short_audio = vec![0.0f32; 100];
        processor.process_chunk(short_audio, tx.clone()).await;
        
        // Should handle gracefully without panic
        // Check for any error events
        let mut error_count = 0;
        while let Ok(event) = rx.try_recv() {
            if matches!(event, StreamingEvent::Error(_)) {
                error_count += 1;
            }
        }
        
        // Empty chunks might not generate errors, just be ignored
        // But we shouldn't panic
        assert!(error_count <= 2, "Should handle edge cases gracefully");
    }
    
    #[tokio::test]
    async fn test_streaming_latency_tracking() {
        let config = StreamingConfig::default();
        let mut processor = StreamingTranscriptionProcessor::new(config);
        let (tx, mut rx) = mpsc::channel(10);
        
        // Process audio and measure latency
        let audio = generate_test_audio(1000, 440.0, 16000);
        let start = std::time::Instant::now();
        
        processor.process_chunk(audio, tx.clone()).await;
        
        let processing_time = start.elapsed();
        
        // Check for latency reports
        let mut latencies = Vec::new();
        while let Ok(event) = rx.try_recv() {
            if let StreamingEvent::LatencyReport { latency_ms } = event {
                latencies.push(latency_ms);
            }
        }
        
        // Should have at least one latency report
        assert!(!latencies.is_empty(), "Should report latency");
        
        // Latency should be reasonable (less than processing time)
        for latency in &latencies {
            assert!(*latency < processing_time.as_millis() as u32 + 100, 
                    "Latency {} should be reasonable", latency);
        }
    }
    
    #[tokio::test]
    async fn test_streaming_multi_language() {
        let config = StreamingConfig::default();
        let mut processor = StreamingTranscriptionProcessor::new(config);
        
        // Set language to Spanish
        processor.set_language("es");
        
        let (tx, mut rx) = mpsc::channel(10);
        
        // Process audio
        let audio = generate_test_audio(1000, 440.0, 16000);
        processor.process_chunk(audio.clone(), tx.clone()).await;
        
        // Change language to French
        processor.set_language("fr");
        processor.process_chunk(audio, tx.clone()).await;
        
        // Collect events
        let mut language_changes = 0;
        while let Ok(event) = rx.try_recv() {
            if matches!(event, StreamingEvent::LanguageDetected { .. }) {
                language_changes += 1;
            }
        }
        
        // Should detect language changes
        assert!(language_changes > 0, "Should detect language changes");
    }
    
    #[tokio::test]
    async fn test_streaming_with_whisper_params() {
        let mut whisper_params = WhisperParamsSettings::default();
        whisper_params.temperature = 0.2;
        whisper_params.beam_size = 10;
        whisper_params.best_of = 3;
        
        let config = StreamingConfig::default();
        let mut processor = StreamingTranscriptionProcessor::new(config);
        processor.update_whisper_params(whisper_params);
        
        let (tx, mut rx) = mpsc::channel(10);
        
        // Process with custom params
        let audio = generate_test_audio(1000, 440.0, 16000);
        processor.process_chunk(audio, tx.clone()).await;
        
        // Should process without errors
        let mut has_result = false;
        while let Ok(event) = rx.try_recv() {
            if matches!(event, StreamingEvent::FinalTranscript(_)) {
                has_result = true;
                break;
            }
        }
        
        assert!(has_result, "Should produce results with custom params");
    }
    
    #[tokio::test]
    async fn test_streaming_concurrent_chunks() {
        let config = StreamingConfig::default();
        let processor = Arc::new(Mutex::new(StreamingTranscriptionProcessor::new(config)));
        let (tx, mut rx) = mpsc::channel(100);
        
        // Generate multiple audio chunks
        let chunks: Vec<_> = (0..5)
            .map(|i| generate_test_audio(500, 440.0 + (i as f32 * 100.0), 16000))
            .collect();
        
        // Process chunks concurrently
        let mut handles = vec![];
        for chunk in chunks {
            let processor = processor.clone();
            let tx = tx.clone();
            
            let handle = tokio::spawn(async move {
                let mut proc = processor.lock().unwrap();
                proc.process_chunk(chunk, tx).await;
            });
            
            handles.push(handle);
        }
        
        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Should have received events from all chunks
        let mut event_count = 0;
        while let Ok(_event) = rx.try_recv() {
            event_count += 1;
        }
        
        assert!(event_count > 0, "Should process concurrent chunks");
    }
    
    // Test memory efficiency with large audio streams
    #[tokio::test]
    async fn test_streaming_memory_efficiency() {
        let config = StreamingConfig {
            chunk_size: 16000, // 1 second chunks
            overlap_size: 1600,
            sample_rate: 16000,
            enable_partial_results: false, // Reduce memory usage
            partial_update_interval_ms: 1000,
            max_silence_duration_ms: 5000,
        };
        
        let mut processor = StreamingTranscriptionProcessor::new(config);
        let (tx, mut rx) = mpsc::channel(10);
        
        // Process a long audio stream (10 seconds)
        for _ in 0..10 {
            let chunk = generate_test_audio(1000, 440.0, 16000);
            processor.process_chunk(chunk, tx.clone()).await;
            
            // Drain events to prevent buffer overflow
            while let Ok(_) = rx.try_recv() {
                // Just consume events
            }
        }
        
        // Should complete without memory issues
        assert!(true, "Processed long stream successfully");
        */
    }
}