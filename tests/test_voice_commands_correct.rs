use bestme::audio::voice_commands::{VoiceCommand, VoiceCommandType, VoiceCommandConfig};

#[test]
fn test_voice_command_types() {
    // Test that voice command types are defined correctly
    let command_types = vec![
        VoiceCommandType::Delete,
        VoiceCommandType::Undo,
        VoiceCommandType::Redo,
        VoiceCommandType::Capitalize,
        VoiceCommandType::Lowercase,
        VoiceCommandType::NewLine,
        VoiceCommandType::NewParagraph,
        VoiceCommandType::Period,
        VoiceCommandType::Comma,
        VoiceCommandType::QuestionMark,
        VoiceCommandType::ExclamationMark,
        VoiceCommandType::Pause,
        VoiceCommandType::Resume,
        VoiceCommandType::Stop,
        VoiceCommandType::Custom("test custom".to_string()),
    ];

    for cmd_type in command_types {
        println!("Voice command type: {:?}", cmd_type);
    }
}

#[test]
fn test_voice_command_creation() {
    // Create some voice commands
    let commands = vec![
        VoiceCommand::new(VoiceCommandType::Delete, "delete word"),
        VoiceCommand::new(VoiceCommandType::Undo, "undo"),
        VoiceCommand::new(VoiceCommandType::NewLine, "new line"),
    ];
    
    for cmd in commands {
        println!("Voice Command:");
        println!("  Type: {:?}", cmd.command_type);
        println!("  Trigger text: {}", cmd.trigger_text);
        println!("  Parameters: {:?}", cmd.parameters);
    }
}

#[test]
fn test_voice_command_with_parameters() {
    let cmd = VoiceCommand::new(VoiceCommandType::Delete, "delete")
        .with_parameters("last 3 words");
    
    assert_eq!(cmd.trigger_text, "delete");
    assert!(cmd.parameters.is_some());
    assert_eq!(cmd.parameters.unwrap(), "last 3 words");
}

#[test]
fn test_voice_command_config() {
    let config = VoiceCommandConfig::default();
    
    println!("Voice Command Config:");
    println!("  Enabled: {}", config.enabled);
    println!("  Command prefix: {:?}", config.command_prefix);
    println!("  Require prefix: {}", config.require_prefix);
    println!("  Sensitivity: {}", config.sensitivity);
    println!("  Custom commands: {} entries", config.custom_commands.len());
    
    // Test reasonable defaults
    assert!(config.sensitivity >= 0.0 && config.sensitivity <= 1.0);
    assert!(config.enabled); // Voice commands are enabled by default
    assert_eq!(config.sensitivity, 0.8); // Default sensitivity is 0.8
}