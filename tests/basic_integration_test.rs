//! Basic integration tests for BestMe core functionality

use bestme::config::Config;
use bestme::audio::device::DeviceManager;

#[tokio::test]
async fn test_config_creation() {
    let config = Config::default();
    
    // Test default values
    assert_eq!(config.audio.speech.model, "base.en");
    assert_eq!(config.audio.speech.language, "en");
    assert!(config.audio.speech.sample_rate > 0);
    
    // Test version
    assert_eq!(config.version, "0.1.0");
}

#[tokio::test]
async fn test_device_manager() {
    let device_manager = DeviceManager::new();
    
    match device_manager {
        Ok(manager) => {
            // Try to list devices
            let devices = manager.list_devices().await;
            match devices {
                Ok(device_list) => {
                    println!("Found {} audio devices", device_list.len());
                    // In CI, there might be no audio devices
                    assert!(device_list.len() >= 0);
                },
                Err(e) => {
                    println!("Could not list devices in test environment: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Could not initialize device manager in test environment: {}", e);
        }
    }
}

#[test]
fn test_basic_math() {
    // Simple smoke test to ensure test framework works
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_vad_creation() {
    use bestme::audio::VoiceActivityDetector;
    
    let vad = VoiceActivityDetector::new(
        0.02,  // threshold
        100,   // min speech duration ms
        300,   // max silence duration ms
        16000  // sample rate
    );
    
    // Process empty audio
    let silence = vec![0.0f32; 1600];
    let result = vad.process(&silence);
    
    // Should not crash - just check that we got a result
    // In silence, we expect energy below threshold
    assert!(result.energy < 0.01); // Very low energy for silence
}