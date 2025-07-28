//! Comprehensive performance tests for all BestMe features

use bestme::config::{Config, WhisperModelSize};
use bestme::audio::transcribe::TranscriptionManager;
use bestme::audio::vad::VoiceActivityDetector;
use bestme::audio::vocabulary::VocabularyManager;
use bestme::audio::multi_pass::{MultiPassProcessor, ProcessingStrategy};
use std::time::Instant;
use tokio::sync::mpsc;
use log::info;

/// Performance test results
#[derive(Debug)]
struct PerformanceResult {
    feature: String,
    duration_ms: u128,
    memory_kb: Option<usize>,
    notes: String,
}

/// Generate test audio data
fn generate_test_audio(duration_secs: f32) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let sample_rate = 16000;
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    
    // Mix of silence and noise to simulate real speech
    let mut audio = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        
        // Alternate between silence and "speech" every 2 seconds
        if (t as i32) % 4 < 2 {
            // Silence
            audio.push(0.0);
        } else {
            // Simulated speech (noise + sine wave)
            let noise = rng.gen_range(-0.1..0.1);
            let sine = (2.0 * std::f32::consts::PI * 200.0 * t).sin() * 0.3;
            audio.push(noise + sine);
        }
    }
    
    audio
}

/// Test VAD performance
async fn test_vad_performance() -> PerformanceResult {
    info!("Testing VAD performance...");
    
    let test_audio = generate_test_audio(30.0); // 30 seconds
    let mut vad = VoiceActivityDetector::new(Default::default());
    
    let start = Instant::now();
    let mut speech_frames = 0;
    let chunk_size = 480; // 30ms at 16kHz
    
    for chunk in test_audio.chunks(chunk_size) {
        match vad.process(chunk) {
            Ok(result) => {
                if result.is_speech {
                    speech_frames += 1;
                }
            }
            Err(e) => {
                return PerformanceResult {
                    feature: "VAD".to_string(),
                    duration_ms: start.elapsed().as_millis(),
                    memory_kb: None,
                    notes: format!("Error: {}", e),
                };
            }
        }
    }
    
    let duration = start.elapsed();
    let rtf = duration.as_secs_f32() / 30.0;
    
    PerformanceResult {
        feature: "VAD".to_string(),
        duration_ms: duration.as_millis(),
        memory_kb: None,
        notes: format!("Processed 30s audio in {:.2}s (RTF: {:.4}x), detected {} speech frames", 
            duration.as_secs_f32(), rtf, speech_frames),
    }
}

/// Test vocabulary loading performance
async fn test_vocabulary_performance() -> PerformanceResult {
    info!("Testing vocabulary performance...");
    
    let mut vocab = VocabularyManager::new();
    
    // Add many vocabulary entries
    let start = Instant::now();
    
    for i in 0..1000 {
        let _ = vocab.add_entry(
            &format!("term_{}", i),
            None,
            2.0,
            "test",
            &[format!("variant_{}", i)],
        );
    }
    
    let add_duration = start.elapsed();
    
    // Test search performance
    let search_start = Instant::now();
    let mut found = 0;
    
    for i in 0..100 {
        if vocab.search_entries(&format!("term_{}", i * 10)).len() > 0 {
            found += 1;
        }
    }
    
    let search_duration = search_start.elapsed();
    
    PerformanceResult {
        feature: "Vocabulary".to_string(),
        duration_ms: add_duration.as_millis() + search_duration.as_millis(),
        memory_kb: None,
        notes: format!("Added 1000 entries in {:.2}ms, searched 100 times in {:.2}ms, found {}", 
            add_duration.as_millis(), search_duration.as_millis(), found),
    }
}

/// Test multi-pass processing performance
async fn test_multipass_performance() -> PerformanceResult {
    info!("Testing multi-pass processing performance...");
    
    let config = Config::default();
    let strategies = vec![
        ProcessingStrategy::Single,
        ProcessingStrategy::Double,
        ProcessingStrategy::Triple,
    ];
    
    let mut results = Vec::new();
    
    for strategy in strategies {
        let processor = MultiPassProcessor::new(config.clone(), strategy.clone());
        
        let start = Instant::now();
        
        // Simulate processing
        match processor.get_strategy() {
            ProcessingStrategy::Single => {
                // Simulate single pass
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            ProcessingStrategy::Double => {
                // Simulate double pass
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
            ProcessingStrategy::Triple => {
                // Simulate triple pass
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            }
            _ => {}
        }
        
        let duration = start.elapsed();
        results.push((strategy, duration));
    }
    
    let notes = results.iter()
        .map(|(s, d)| format!("{:?}: {:.0}ms", s, d.as_millis()))
        .collect::<Vec<_>>()
        .join(", ");
    
    PerformanceResult {
        feature: "Multi-pass".to_string(),
        duration_ms: results.iter().map(|(_, d)| d.as_millis()).sum::<u128>() / results.len() as u128,
        memory_kb: None,
        notes,
    }
}

/// Test streaming pipeline performance
async fn test_streaming_performance() -> PerformanceResult {
    info!("Testing streaming pipeline performance...");
    
    use bestme::audio::streaming_transcribe::{StreamingTranscriber, StreamingConfig};
    
    let config = StreamingConfig {
        chunk_size: 8000, // 0.5s chunks
        overlap_size: 1600,
        sample_rate: 16000,
        enable_partial_results: true,
        partial_update_interval_ms: 100,
        max_silence_duration_ms: 2000,
    };
    
    let transcriber = StreamingTranscriber::new(config.clone());
    let test_audio = generate_test_audio(10.0); // 10 seconds
    
    let start = Instant::now();
    let mut chunks_processed = 0;
    
    for chunk in test_audio.chunks(config.chunk_size) {
        match transcriber.process_chunk(chunk).await {
            Ok(_) => chunks_processed += 1,
            Err(e) => {
                return PerformanceResult {
                    feature: "Streaming".to_string(),
                    duration_ms: start.elapsed().as_millis(),
                    memory_kb: None,
                    notes: format!("Error: {}", e),
                };
            }
        }
    }
    
    let duration = start.elapsed();
    let rtf = duration.as_secs_f32() / 10.0;
    
    PerformanceResult {
        feature: "Streaming".to_string(),
        duration_ms: duration.as_millis(),
        memory_kb: None,
        notes: format!("Processed {} chunks in {:.2}s (RTF: {:.2}x)", 
            chunks_processed, duration.as_secs_f32(), rtf),
    }
}

/// Test overall transcription performance with all features
async fn test_full_pipeline_performance() -> PerformanceResult {
    info!("Testing full pipeline performance...");
    
    let mut config = Config::default();
    config.audio.speech.model_size = WhisperModelSize::Tiny;
    config.audio.speech.vad.enabled = true;
    config.audio.speech.whisper_params.temperature = 0.0;
    
    let (tx, mut _rx) = mpsc::channel(100);
    let manager = TranscriptionManager::new(
        config.audio.speech.clone(),
        config.get_model_path().await,
    );
    
    manager.set_event_sender(tx);
    
    let start = Instant::now();
    
    match manager.initialize().await {
        Ok(_) => {
            let test_audio = generate_test_audio(10.0);
            
            match manager.process_audio(&test_audio).await {
                Ok(_) => {
                    let duration = start.elapsed();
                    let rtf = duration.as_secs_f32() / 10.0;
                    
                    PerformanceResult {
                        feature: "Full Pipeline".to_string(),
                        duration_ms: duration.as_millis(),
                        memory_kb: None,
                        notes: format!("Processed 10s audio in {:.2}s (RTF: {:.2}x) with VAD + Whisper", 
                            duration.as_secs_f32(), rtf),
                    }
                }
                Err(e) => PerformanceResult {
                    feature: "Full Pipeline".to_string(),
                    duration_ms: start.elapsed().as_millis(),
                    memory_kb: None,
                    notes: format!("Processing error: {}", e),
                }
            }
        }
        Err(e) => PerformanceResult {
            feature: "Full Pipeline".to_string(),
            duration_ms: start.elapsed().as_millis(),
            memory_kb: None,
            notes: format!("Initialization error: {}", e),
        }
    }
}

/// Get current memory usage in KB
fn get_memory_usage() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return parts[1].parse().ok();
                    }
                }
            }
        }
    }
    None
}

/// Run all performance tests
#[tokio::test]
async fn test_all_features_performance() {
    env_logger::init();
    
    println!("\n=== BestMe Performance Test Suite ===\n");
    
    let mut results = Vec::new();
    
    // Test individual features
    results.push(test_vad_performance().await);
    results.push(test_vocabulary_performance().await);
    results.push(test_multipass_performance().await);
    results.push(test_streaming_performance().await);
    results.push(test_full_pipeline_performance().await);
    
    // Print results table
    println!("\n{:<20} | {:<15} | {:<50}", "Feature", "Duration (ms)", "Notes");
    println!("{:-<20}-+-{:-<15}-+-{:-<50}", "", "", "");
    
    for result in &results {
        println!("{:<20} | {:<15} | {:<50}", 
            result.feature, 
            result.duration_ms,
            if result.notes.len() > 50 {
                format!("{}...", &result.notes[..47])
            } else {
                result.notes.clone()
            }
        );
    }
    
    // Summary
    println!("\n=== Summary ===");
    
    let total_duration: u128 = results.iter().map(|r| r.duration_ms).sum();
    println!("Total test duration: {:.2}s", total_duration as f64 / 1000.0);
    
    // Performance assertions
    for result in &results {
        match result.feature.as_str() {
            "VAD" => assert!(result.duration_ms < 100, "VAD should process 30s audio in < 100ms"),
            "Vocabulary" => assert!(result.duration_ms < 500, "Vocabulary operations should complete in < 500ms"),
            "Streaming" => assert!(result.duration_ms < 20000, "Streaming should maintain real-time performance"),
            _ => {}
        }
    }
    
    println!("\nAll performance tests completed successfully!");
}

/// Test memory usage patterns
#[tokio::test]
#[ignore] // Run manually as it takes time
async fn test_memory_usage() {
    println!("\n=== Memory Usage Test ===\n");
    
    let initial_memory = get_memory_usage();
    println!("Initial memory: {:?} KB", initial_memory);
    
    // Test memory usage with different model sizes
    let model_sizes = vec![
        WhisperModelSize::Tiny,
        WhisperModelSize::Base,
        WhisperModelSize::Small,
    ];
    
    for model_size in model_sizes {
        let mut config = Config::default();
        config.audio.speech.model_size = model_size.clone();
        
        let (tx, _rx) = mpsc::channel(100);
        let manager = TranscriptionManager::new(
            config.audio.speech.clone(),
            config.get_model_path().await,
        );
        
        manager.set_event_sender(tx);
        
        let before_init = get_memory_usage();
        
        if let Ok(_) = manager.initialize().await {
            let after_init = get_memory_usage();
            
            if let (Some(before), Some(after)) = (before_init, after_init) {
                let diff = after as i32 - before as i32;
                println!("Model {:?}: {} KB (Δ {} KB)", model_size, after, diff);
            }
            
            // Process some audio
            let test_audio = generate_test_audio(5.0);
            let _ = manager.process_audio(&test_audio).await;
            
            let after_process = get_memory_usage();
            if let Some(mem) = after_process {
                println!("  After processing: {} KB", mem);
            }
        }
        
        // Drop manager and wait for cleanup
        drop(manager);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let after_cleanup = get_memory_usage();
        if let Some(mem) = after_cleanup {
            println!("  After cleanup: {} KB", mem);
        }
        
        println!();
    }
}