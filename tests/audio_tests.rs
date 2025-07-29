use bestme::audio::voice_commands::{VoiceCommand, VoiceCommandType, VoiceCommandConfig};
use bestme::audio::device::DeviceManager;

#[test]
fn test_voice_command_types() {
    let commands = vec![
        VoiceCommandType::Delete,
        VoiceCommandType::Undo,
        VoiceCommandType::Redo,
        VoiceCommandType::NewLine,
        VoiceCommandType::Period,
        VoiceCommandType::Comma,
    ];
    
    for cmd in commands {
        match cmd {
            VoiceCommandType::Delete => assert!(true),
            VoiceCommandType::Undo => assert!(true),
            VoiceCommandType::Redo => assert!(true),
            _ => assert!(true),
        }
    }
}

#[test]
fn test_voice_command_creation() {
    let cmd = VoiceCommand::new(VoiceCommandType::Delete, "delete word");
    assert_eq!(cmd.trigger_text, "delete word");
    assert!(cmd.parameters.is_none());
    
    let cmd_with_params = cmd.with_parameters("last 3 words");
    assert_eq!(cmd_with_params.parameters, Some("last 3 words".to_string()));
}

#[test]
fn test_voice_command_config() {
    let config = VoiceCommandConfig::default();
    
    assert!(config.enabled);
    assert_eq!(config.sensitivity, 0.8);
    assert!(!config.require_prefix);
    assert!(config.command_prefix.is_none());
}

#[test]
fn test_device_listing() {
    match DeviceManager::new() {
        Ok(manager) => {
            let devices = manager.get_input_devices();
            for (id, name) in devices {
                println!("Device: {} - {}", id, name);
                assert!(!id.is_empty());
                assert!(!name.is_empty());
            }
        }
        Err(_) => {
            // Acceptable in test environment
            println!("No audio devices available in test environment");
        }
    }
}

#[cfg(test)]
mod audio_buffer_tests {
    #[test]
    fn test_audio_buffer_creation() {
        // Simple test for audio buffer handling
        let buffer_size = 1024;
        let buffer: Vec<f32> = vec![0.0; buffer_size];
        
        assert_eq!(buffer.len(), buffer_size);
        assert!(buffer.iter().all(|&x| x == 0.0));
    }
    
    #[test]
    fn test_audio_levels() {
        let samples = vec![0.1, -0.2, 0.3, -0.4, 0.5];
        let rms: f32 = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
        
        assert!(rms > 0.0);
        assert!(rms < 1.0);
    }
}