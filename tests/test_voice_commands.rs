use bestme::audio::voice_commands::{
    VoiceCommand, VoiceCommandConfig, VoiceCommandManager, DeleteScope
};
use std::sync::Arc;
use parking_lot::Mutex;

#[test]
fn test_voice_command_parsing() {
    // Test basic command parsing
    let commands = vec![
        ("delete word", VoiceCommand::Delete(DeleteScope::Word)),
        ("delete line", VoiceCommand::Delete(DeleteScope::Line)),
        ("undo", VoiceCommand::Undo),
        ("redo", VoiceCommand::Redo),
        ("new line", VoiceCommand::NewLine),
        ("new paragraph", VoiceCommand::NewParagraph),
        ("capitalize", VoiceCommand::Capitalize),
        ("uppercase", VoiceCommand::Uppercase),
        ("lowercase", VoiceCommand::Lowercase),
    ];

    for (text, expected) in commands {
        println!("Testing command: '{}'", text);
        let manager = VoiceCommandManager::new(VoiceCommandConfig::default());
        let command = manager.parse_command(text, 0.9);
        assert!(command.is_some(), "Failed to parse: {}", text);
        if let Some(cmd) = command {
            println!("  Parsed as: {:?}", cmd);
        }
    }
}

#[test]
fn test_voice_command_config() {
    let config = VoiceCommandConfig::default();
    
    println!("Voice Command Config:");
    println!("  Enabled: {}", config.enabled);
    println!("  Command prefix: {:?}", config.command_prefix);
    println!("  Require prefix: {}", config.require_prefix);
    println!("  Sensitivity: {}", config.sensitivity);
    
    assert!(config.sensitivity >= 0.0 && config.sensitivity <= 1.0);
}

#[test]
fn test_voice_command_history() {
    let mut manager = VoiceCommandManager::new(VoiceCommandConfig::default());
    
    // Test adding commands to history
    let test_commands = vec![
        VoiceCommand::Undo,
        VoiceCommand::NewLine,
        VoiceCommand::Delete(DeleteScope::Word),
    ];
    
    for cmd in test_commands {
        manager.add_to_history(cmd.clone());
    }
    
    let history = manager.get_history();
    assert!(!history.is_empty());
    println!("Command history has {} entries", history.len());
}

#[test] 
fn test_ai_voice_commands() {
    use bestme::audio::ai_voice_commands::AIVoiceCommandProcessor;
    
    let config = VoiceCommandConfig::default();
    let processor = Arc::new(Mutex::new(AIVoiceCommandProcessor::new()));
    
    // Test AI command detection
    let test_phrases = vec![
        "make this more formal",
        "fix the grammar",
        "translate to spanish",
        "summarize this paragraph",
    ];
    
    for phrase in test_phrases {
        println!("Testing AI command: '{}'", phrase);
        let is_ai_command = processor.lock().is_ai_command(phrase);
        println!("  Is AI command: {}", is_ai_command);
    }
}