use bestme::config::Config;

#[test]
fn test_ai_config_structure() {
    let config = Config::default();
    
    println!("Chat Model: {}", config.ai.chat_model);
    println!("Has Requesty API Key: {}", config.ai.requesty_api_key.is_some());
    
    // Chat model may be empty by default
    if config.ai.chat_model.is_empty() {
        println!("Note: No default chat model configured");
    }
}

#[test]
fn test_whisper_params() {
    let config = Config::default();
    
    println!("Whisper Parameters:");
    println!("  Best of: {}", config.whisper_params.best_of);
    println!("  Temperature: {}", config.whisper_params.temperature);
    println!("  Compression ratio threshold: {}", config.whisper_params.compression_ratio_threshold);
    println!("  Log prob threshold: {}", config.whisper_params.logprob_threshold);
    
    // Test defaults
    assert!(config.whisper_params.temperature >= 0.0);
    assert!(config.whisper_params.temperature <= 1.0);
}

#[test]
fn test_speech_settings() {
    let config = Config::default();
    
    println!("Speech Settings:");
    println!("  Model size: {:?}", config.audio.speech.model_size);
    println!("  Auto punctuate: {}", config.audio.speech.auto_punctuate);
    println!("  Context formatting: {}", config.audio.speech.context_formatting);
    
    // Model path may be None by default
    if let Some(path) = &config.audio.speech.model_path {
        assert!(!path.is_empty());
    }
}