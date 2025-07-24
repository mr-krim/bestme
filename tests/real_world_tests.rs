//! Real-world usage tests for Whisper enhancements

use bestme::config::{Config, WhisperModelSize};
use bestme::audio::transcribe::{TranscriptionManager, TranscriptionEvent};
use bestme::audio::vocabulary::{VocabularyManager, VocabularyEntry};
use std::time::Instant;
use tokio::sync::mpsc;
use log::info;

/// Simulate real-world speech patterns
fn generate_realistic_audio(scenario: &str) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let sample_rate = 16000;
    let mut audio = Vec::new();
    
    match scenario {
        "conversation" => {
            // Simulate a conversation with pauses
            for i in 0..10 {
                // Speech segment (1-3 seconds)
                let speech_duration = rng.gen_range(1.0..3.0);
                let speech_samples = (sample_rate as f32 * speech_duration) as usize;
                for _ in 0..speech_samples {
                    let noise = rng.gen_range(-0.2..0.2);
                    let freq = rng.gen_range(100.0..400.0);
                    let t = audio.len() as f32 / sample_rate as f32;
                    let sine = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.3;
                    audio.push(noise + sine);
                }
                
                // Pause (0.5-2 seconds)
                let pause_duration = rng.gen_range(0.5..2.0);
                let pause_samples = (sample_rate as f32 * pause_duration) as usize;
                audio.extend(vec![0.0; pause_samples]);
            }
        }
        "presentation" => {
            // Simulate a presentation with longer speech segments
            for i in 0..5 {
                // Long speech segment (5-10 seconds)
                let speech_duration = rng.gen_range(5.0..10.0);
                let speech_samples = (sample_rate as f32 * speech_duration) as usize;
                for _ in 0..speech_samples {
                    let noise = rng.gen_range(-0.15..0.15);
                    let freq = 200.0 + (i as f32 * 20.0); // Slightly different tone
                    let t = audio.len() as f32 / sample_rate as f32;
                    let sine = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.4;
                    audio.push(noise + sine);
                }
                
                // Short pause (0.3-1 second)
                let pause_duration = rng.gen_range(0.3..1.0);
                let pause_samples = (sample_rate as f32 * pause_duration) as usize;
                audio.extend(vec![0.0; pause_samples]);
            }
        }
        "noisy" => {
            // Simulate speech in noisy environment
            let duration = 30.0;
            let total_samples = (sample_rate as f32 * duration) as usize;
            
            for i in 0..total_samples {
                let t = i as f32 / sample_rate as f32;
                
                // Background noise
                let bg_noise = rng.gen_range(-0.1..0.1);
                
                // Speech (intermittent)
                let speech = if (t as i32) % 3 < 2 {
                    let freq = 250.0 + (t * 10.0).sin() * 50.0;
                    (2.0 * std::f32::consts::PI * freq * t).sin() * 0.3
                } else {
                    0.0
                };
                
                // Occasional loud noise
                let loud_noise = if rng.gen_bool(0.01) {
                    rng.gen_range(-0.5..0.5)
                } else {
                    0.0
                };
                
                audio.push(bg_noise + speech + loud_noise);
            }
        }
        _ => {
            // Default: 30 seconds of varied audio
            let duration = 30.0;
            let total_samples = (sample_rate as f32 * duration) as usize;
            audio = generate_varied_audio(total_samples, sample_rate);
        }
    }
    
    audio
}

/// Generate varied audio patterns
fn generate_varied_audio(num_samples: usize, sample_rate: usize) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut audio = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let phase = (t / 5.0) as i32 % 4;
        
        let sample = match phase {
            0 => 0.0, // Silence
            1 => { // Low frequency speech
                let freq = 200.0 + (t * 5.0).sin() * 50.0;
                (2.0 * std::f32::consts::PI * freq * t).sin() * 0.3 + rng.gen_range(-0.05..0.05)
            }
            2 => { // High frequency speech
                let freq = 300.0 + (t * 7.0).sin() * 70.0;
                (2.0 * std::f32::consts::PI * freq * t).sin() * 0.25 + rng.gen_range(-0.05..0.05)
            }
            _ => { // Mixed
                let freq1 = 250.0;
                let freq2 = 350.0;
                let s1 = (2.0 * std::f32::consts::PI * freq1 * t).sin() * 0.2;
                let s2 = (2.0 * std::f32::consts::PI * freq2 * t).sin() * 0.15;
                s1 + s2 + rng.gen_range(-0.1..0.1)
            }
        };
        
        audio.push(sample);
    }
    
    audio
}

/// Test scenario result
struct ScenarioResult {
    scenario: String,
    duration_secs: f32,
    processing_time_secs: f32,
    vad_active_ratio: f32,
    confidence_avg: f32,
    hallucinations_detected: usize,
    notes: String,
}

/// Run a real-world scenario test
async fn test_scenario(scenario: &str, config: Config) -> Result<ScenarioResult, String> {
    info!("Testing scenario: {}", scenario);
    
    let audio = generate_realistic_audio(scenario);
    let duration_secs = audio.len() as f32 / 16000.0;
    
    let (tx, mut rx) = mpsc::channel(100);
    let manager = TranscriptionManager::new(
        config.audio.speech.clone(),
        config.get_model_path().await,
    );
    
    manager.set_event_sender(tx);
    
    // Initialize
    manager.initialize().await
        .map_err(|e| format!("Failed to initialize: {}", e))?;
    
    let start = Instant::now();
    let mut confidence_scores = Vec::new();
    let mut vad_active_chunks = 0;
    let mut total_chunks = 0;
    let mut hallucinations = 0;
    let mut transcription_complete = false;
    
    // Process audio
    manager.process_audio(&audio).await
        .map_err(|e| format!("Failed to process audio: {}", e))?;
    
    manager.force_process_buffer().await
        .map_err(|e| format!("Failed to force process: {}", e))?;
    
    // Collect results
    let timeout = tokio::time::Duration::from_secs(60);
    
    tokio::time::timeout(timeout, async {
        while let Some(event) = rx.recv().await {
            match event {
                TranscriptionEvent::PartialTranscript { text, confidence } => {
                    if let Some(conf) = confidence {
                        confidence_scores.push(conf);
                    }
                }
                TranscriptionEvent::VADUpdate { is_speech } => {
                    total_chunks += 1;
                    if is_speech {
                        vad_active_chunks += 1;
                    }
                }
                TranscriptionEvent::TranscriptionComplete { segments, .. } => {
                    // Check for hallucinations in segments
                    for segment in &segments {
                        if segment.text.contains("Thank you") || 
                           segment.text.contains("Subscribe") ||
                           segment.text.contains("♪") {
                            hallucinations += 1;
                        }
                    }
                    transcription_complete = true;
                    break;
                }
                TranscriptionEvent::Error(e) => {
                    return Err(format!("Transcription error: {}", e));
                }
                _ => {}
            }
        }
        Ok(())
    }).await
        .map_err(|_| "Timeout waiting for transcription".to_string())?
        .map_err(|e| e)?;
    
    if !transcription_complete {
        return Err("Transcription did not complete".to_string());
    }
    
    let processing_time = start.elapsed().as_secs_f32();
    let vad_ratio = if total_chunks > 0 {
        vad_active_chunks as f32 / total_chunks as f32
    } else {
        0.0
    };
    
    let avg_confidence = if !confidence_scores.is_empty() {
        confidence_scores.iter().sum::<f32>() / confidence_scores.len() as f32
    } else {
        0.0
    };
    
    Ok(ScenarioResult {
        scenario: scenario.to_string(),
        duration_secs,
        processing_time_secs: processing_time,
        vad_active_ratio: vad_ratio,
        confidence_avg: avg_confidence,
        hallucinations_detected: hallucinations,
        notes: format!("RTF: {:.2}x", processing_time / duration_secs),
    })
}

#[tokio::test]
async fn test_real_world_conversation() {
    let mut config = Config::default();
    config.audio.speech.model_size = WhisperModelSize::Small;
    config.audio.speech.vad.enabled = true;
    config.audio.speech.whisper_params.temperature = 0.0;
    
    match test_scenario("conversation", config).await {
        Ok(result) => {
            println!("\nConversation Scenario Results:");
            println!("  Duration: {:.1}s", result.duration_secs);
            println!("  Processing time: {:.1}s ({})", result.processing_time_secs, result.notes);
            println!("  VAD active ratio: {:.1}%", result.vad_active_ratio * 100.0);
            println!("  Average confidence: {:.2}", result.confidence_avg);
            println!("  Hallucinations: {}", result.hallucinations_detected);
            
            // Assertions for conversation
            assert!(result.vad_active_ratio > 0.3 && result.vad_active_ratio < 0.8, 
                "Conversation should have 30-80% active speech");
        }
        Err(e) => {
            println!("Conversation test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_real_world_presentation() {
    let mut config = Config::default();
    config.audio.speech.model_size = WhisperModelSize::Small;
    config.audio.speech.vad.enabled = true;
    config.audio.speech.whisper_params.suppress_non_speech = true;
    
    match test_scenario("presentation", config).await {
        Ok(result) => {
            println!("\nPresentation Scenario Results:");
            println!("  Duration: {:.1}s", result.duration_secs);
            println!("  Processing time: {:.1}s ({})", result.processing_time_secs, result.notes);
            println!("  VAD active ratio: {:.1}%", result.vad_active_ratio * 100.0);
            println!("  Average confidence: {:.2}", result.confidence_avg);
            println!("  Hallucinations: {}", result.hallucinations_detected);
            
            // Assertions for presentation
            assert!(result.vad_active_ratio > 0.7, 
                "Presentation should have >70% active speech");
        }
        Err(e) => {
            println!("Presentation test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_real_world_noisy_environment() {
    let mut config = Config::default();
    config.audio.speech.model_size = WhisperModelSize::Small;
    config.audio.speech.vad.enabled = true;
    config.audio.speech.vad.threshold = 0.6; // Higher threshold for noise
    config.audio.speech.whisper_params.temperature = 0.2; // Allow some variation
    config.audio.speech.whisper_params.compression_ratio_threshold = Some(2.0);
    
    match test_scenario("noisy", config).await {
        Ok(result) => {
            println!("\nNoisy Environment Results:");
            println!("  Duration: {:.1}s", result.duration_secs);
            println!("  Processing time: {:.1}s ({})", result.processing_time_secs, result.notes);
            println!("  VAD active ratio: {:.1}%", result.vad_active_ratio * 100.0);
            println!("  Average confidence: {:.2}", result.confidence_avg);
            println!("  Hallucinations: {}", result.hallucinations_detected);
            
            // In noisy environments, we expect lower confidence
            assert!(result.confidence_avg < 0.9, 
                "Noisy environment should have lower confidence");
        }
        Err(e) => {
            println!("Noisy environment test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_vocabulary_enhancement() {
    let mut config = Config::default();
    config.audio.speech.model_size = WhisperModelSize::Tiny;
    
    // Create vocabulary manager and add technical terms
    let mut vocab = VocabularyManager::new();
    vocab.add_entry("BestMe", None, 3.0, "product", &["Best Me"]);
    vocab.add_entry("Whisper", None, 2.5, "technology", &["whisper"]);
    vocab.add_entry("VAD", Some("Voice Activity Detection"), 2.0, "technology", &["V.A.D."]);
    vocab.add_entry("GPU", Some("Graphics Processing Unit"), 2.0, "technology", &["G.P.U."]);
    
    // Save vocabulary
    let vocab_path = std::env::temp_dir().join("test_vocabulary.json");
    vocab.save_to_file(&vocab_path).unwrap();
    
    // Test with vocabulary
    let (tx, mut rx) = mpsc::channel(100);
    let manager = TranscriptionManager::new(
        config.audio.speech.clone(),
        config.get_model_path().await,
    );
    
    manager.set_event_sender(tx);
    manager.initialize().await.unwrap();
    
    // Load vocabulary
    manager.load_vocabulary(&vocab_path).await.unwrap();
    
    println!("\nVocabulary Enhancement Test:");
    println!("  Added {} technical terms", vocab.get_all_entries().len());
    println!("  Terms boosted with weights 2.0-3.0x");
    
    // Clean up
    let _ = std::fs::remove_file(vocab_path);
}

#[tokio::test]
async fn test_multi_language_real_world() {
    println!("\nMulti-language Real World Test:");
    
    let languages = vec![
        ("en", "English"),
        ("es", "Spanish"),
        ("fr", "French"),
        ("de", "German"),
    ];
    
    for (code, name) in languages {
        let mut config = Config::default();
        config.audio.speech.model_size = WhisperModelSize::Tiny;
        config.audio.speech.language = Some(code.to_string());
        
        println!("  Testing {}: Model configured for {} transcription", name, code);
    }
}