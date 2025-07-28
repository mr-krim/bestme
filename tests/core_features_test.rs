//! Core feature integration tests for BestMe

use bestme::audio::{AudioDevice, DeviceManager, AudioCaptureManager, VoiceActivityDetector};
use bestme::config::{Config, SpeechSettings, WhisperParamsSettings};
use bestme::ai::{ModelSelector, EnhancementOptions, PrivacyLevel};
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::runtime::Runtime;

#[test]
fn test_audio_device_detection() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let device_manager = DeviceManager::new().await.unwrap();
        let devices = device_manager.list_devices().await.unwrap();
        
        // Should find at least one audio device
        assert!(!devices.is_empty(), "No audio devices found");
        
        // Each device should have valid properties
        for device in &devices {
            assert!(!device.name.is_empty(), "Device name should not be empty");
            assert!(device.channels > 0, "Device should have at least 1 channel");
            assert!(device.sample_rate > 0, "Sample rate should be positive");
        }
    });
}

#[test]
fn test_config_loading() {
    // Test default config
    let config = Config::default();
    assert_eq!(config.speech.model, "base.en");
    assert_eq!(config.speech.language, "en");
    assert!(!config.speech.enable_translate);
    
    // Test config validation
    assert!(config.speech.sample_rate > 0);
    assert!(config.speech.silence_threshold >= 0.0);
    assert!(config.speech.min_speech_duration_ms > 0);
}

#[test]
fn test_vad_initialization() {
    let vad = VoiceActivityDetector::new(
        0.02, // threshold
        100,  // min speech duration ms
        300,  // max silence duration ms
        16000 // sample rate
    );
    
    // Test with silence
    let silence = vec![0.0f32; 1600]; // 100ms of silence
    let result = vad.process(&silence);
    assert!(result.segments.is_empty(), "Should not detect speech in silence");
    
    // Test with noise
    let noise: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let result = vad.process(&noise);
    
    // VAD should process without panicking
    assert_eq!(result.is_speech, result.segments.iter().any(|s| s.is_speech));
}

#[test]
fn test_model_selector() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let selector = ModelSelector::new(Default::default()).await.unwrap();
        
        // Test model selection for different text types
        let short_text = "Hello world";
        let model = selector.select_model_for_text(short_text).await.unwrap();
        assert!(!model.id.is_empty(), "Should select a model for short text");
        
        let long_text = "This is a much longer text that contains multiple sentences. \
                        It should trigger selection of a more capable model. \
                        The model selector should analyze the complexity and length.";
        let model = selector.select_model_for_text(long_text).await.unwrap();
        assert!(!model.id.is_empty(), "Should select a model for long text");
    });
}

#[test]
fn test_enhancement_options() {
    let options = EnhancementOptions {
        enable_grammar: true,
        enable_style: false,
        enable_clarity: true,
        target_style: None,
        preserve_tone: true,
        aggressiveness: 0.5,
        custom_vocabulary: vec!["BestMe".to_string(), "AI".to_string()],
        privacy_level: PrivacyLevel::Standard,
    };
    
    // Test serialization
    let serialized = serde_json::to_string(&options).unwrap();
    let deserialized: EnhancementOptions = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(options.enable_grammar, deserialized.enable_grammar);
    assert_eq!(options.aggressiveness, deserialized.aggressiveness);
    assert_eq!(options.custom_vocabulary.len(), deserialized.custom_vocabulary.len());
}

#[test]
fn test_audio_capture_manager() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let device_manager = DeviceManager::new().await.unwrap();
        let devices = device_manager.list_devices().await.unwrap();
        
        if let Some(device) = devices.first() {
            let capture_manager = AudioCaptureManager::new(
                device.clone(),
                16000, // sample rate
                1,     // channels
                1024   // buffer size
            ).await;
            
            match capture_manager {
                Ok(manager) => {
                    // Manager should be created successfully
                    assert!(manager.get_sample_rate() == 16000);
                    assert!(manager.get_channels() == 1);
                },
                Err(e) => {
                    // In CI environment, audio might not be available
                    println!("Audio capture not available in test environment: {}", e);
                }
            }
        }
    });
}

#[test]
fn test_whisper_params_settings() {
    let params = WhisperParamsSettings::default();
    
    // Test default values
    assert_eq!(params.n_threads, 4);
    assert_eq!(params.n_max_text_ctx, 16384);
    assert!(!params.translate);
    assert!(!params.no_context);
    assert!(!params.print_special);
    assert!(!params.print_progress);
    assert!(!params.print_timestamps);
    
    // Test builder pattern
    let custom_params = WhisperParamsSettings {
        n_threads: 8,
        translate: true,
        language: Some("es".to_string()),
        ..Default::default()
    };
    
    assert_eq!(custom_params.n_threads, 8);
    assert!(custom_params.translate);
    assert_eq!(custom_params.language.as_deref(), Some("es"));
}

#[test]
fn test_privacy_levels() {
    let levels = vec![
        PrivacyLevel::Maximum,
        PrivacyLevel::Standard,
        PrivacyLevel::Minimum,
    ];
    
    for level in levels {
        // Test serialization
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: PrivacyLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
        
        // Test privacy implications
        match level {
            PrivacyLevel::Maximum => {
                // Should use only local models
                assert!(level.allows_cloud_processing() == false);
            },
            PrivacyLevel::Standard => {
                // May use cloud for non-sensitive data
                assert!(level.allows_cloud_processing() == true);
            },
            PrivacyLevel::Minimum => {
                // Can use any available service
                assert!(level.allows_cloud_processing() == true);
            }
        }
    }
}

// Mock implementation for testing
impl PrivacyLevel {
    fn allows_cloud_processing(&self) -> bool {
        match self {
            PrivacyLevel::Maximum => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod mock_audio_tests {
    use super::*;
    
    #[test]
    fn test_mock_audio_processing() {
        // Create mock audio data
        let sample_rate = 16000;
        let duration_seconds = 1.0;
        let samples = (sample_rate as f32 * duration_seconds) as usize;
        
        // Generate a 440Hz sine wave (A4 note)
        let frequency = 440.0;
        let audio_data: Vec<f32> = (0..samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5
            })
            .collect();
        
        // Test audio properties
        assert_eq!(audio_data.len(), samples);
        assert!(audio_data.iter().all(|&x| x >= -1.0 && x <= 1.0));
        
        // Test RMS calculation
        let rms = calculate_rms(&audio_data);
        assert!(rms > 0.3 && rms < 0.4, "RMS should be around 0.35 for sine wave");
    }
    
    fn calculate_rms(samples: &[f32]) -> f32 {
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_model_paths() {
        let config = Config::default();
        
        // Test model directory creation
        let model_dir = PathBuf::from(&config.model_dir);
        assert!(!config.model_dir.is_empty());
        
        // Test model naming
        let models = vec!["tiny", "base", "small", "medium", "large"];
        for model in models {
            let model_path = model_dir.join(format!("ggml-{}.bin", model));
            // Path should be constructable
            assert!(!model_path.to_string_lossy().is_empty());
        }
    }
}

#[test]
fn test_concurrent_access() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let shared_config = Arc::new(Mutex::new(Config::default()));
        let mut handles = vec![];
        
        // Spawn multiple tasks accessing config
        for i in 0..10 {
            let config = shared_config.clone();
            let handle = tokio::spawn(async move {
                let mut cfg = config.lock();
                cfg.speech.silence_threshold = 0.01 * i as f32;
                cfg.speech.silence_threshold
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        let results: Vec<f32> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        
        // All updates should have succeeded
        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|&x| x >= 0.0 && x < 0.1));
    });
}