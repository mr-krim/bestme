use bestme::audio::voice_commands::{VoiceCommand, VoiceCommandConfig, DeleteScope};

#[test]
fn test_voice_command_enum() {
    // Test that voice command enums are defined correctly
    let commands = vec![
        VoiceCommand::Delete(DeleteScope::Word),
        VoiceCommand::Delete(DeleteScope::Line),
        VoiceCommand::Undo,
        VoiceCommand::Redo,
        VoiceCommand::NewLine,
        VoiceCommand::NewParagraph,
        VoiceCommand::Capitalize,
        VoiceCommand::Uppercase,
        VoiceCommand::Lowercase,
    ];

    for cmd in commands {
        println!("Voice command: {:?}", cmd);
        
        // Test command matching
        match cmd {
            VoiceCommand::Delete(scope) => {
                println!("  Delete command with scope: {:?}", scope);
            }
            VoiceCommand::Undo => {
                println!("  Undo command");
            }
            VoiceCommand::Redo => {
                println!("  Redo command");
            }
            _ => {
                println!("  Other command");
            }
        }
    }
}

#[test]
fn test_voice_command_config_defaults() {
    let config = VoiceCommandConfig::default();
    
    println!("Default Voice Command Config:");
    println!("  Enabled: {}", config.enabled);
    println!("  Command prefix: {:?}", config.command_prefix);
    println!("  Require prefix: {}", config.require_prefix);
    println!("  Sensitivity: {}", config.sensitivity);
    
    // Test reasonable defaults
    assert!(config.sensitivity >= 0.0 && config.sensitivity <= 1.0);
    assert!(!config.enabled); // Should be disabled by default for safety
}

#[test]
fn test_delete_scope_enum() {
    let scopes = vec![
        DeleteScope::Character,
        DeleteScope::Word,
        DeleteScope::Line,
        DeleteScope::Sentence,
        DeleteScope::Paragraph,
        DeleteScope::All,
    ];
    
    for scope in scopes {
        println!("Delete scope: {:?}", scope);
    }
}