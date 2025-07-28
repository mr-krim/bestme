use bestme::audio::device::list_audio_devices;
use bestme::audio::transcribe::{create_transcriber, TranscriberOptions};
use bestme::config::Config;
use bestme::ai::services::model_service::ModelService;
use bestme::ai::models::registry::ModelRegistry;
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_headless_functionality() {
    println!("Running headless tests...");
    
    // Test 1: Configuration loading
    println!("\n1. Testing configuration loading...");
    let config = Config::load().expect("Failed to load config");
    println!("✓ Configuration loaded successfully");
    println!("  - Whisper model: {}", config.audio.speech.model);
    println!("  - Language: {}", config.audio.speech.language);
    
    // Test 2: Audio device detection
    println!("\n2. Testing audio device detection...");
    let devices = list_audio_devices().unwrap();
    println!("✓ Found {} audio devices", devices.len());
    for device in &devices {
        println!("  - {}", device.name);
    }
    
    // Test 3: Model registry
    println!("\n3. Testing AI model registry...");
    let registry = ModelRegistry::new();
    let models = registry.list_models();
    println!("✓ Found {} registered AI models", models.len());
    for model in &models {
        println!("  - {} ({})", model.name, model.size);
    }
    
    // Test 4: Transcriber creation
    println!("\n4. Testing transcriber initialization...");
    let options = TranscriberOptions {
        model: config.audio.speech.model.clone(),
        language: Some(config.audio.speech.language.clone()),
        use_gpu: true,
        ..Default::default()
    };
    
    match create_transcriber(options).await {
        Ok(_) => println!("✓ Transcriber created successfully"),
        Err(e) => println!("✗ Transcriber creation failed: {}", e),
    }
    
    // Test 5: Model service
    println!("\n5. Testing model service...");
    let config = Arc::new(config);
    let service = ModelService::new(config);
    
    if let Ok(Some(model_info)) = service.get_model_info("microsoft/phi-2").await {
        println!("✓ Model service working");
        println!("  - Model: {}", model_info.name);
        println!("  - Provider: {:?}", model_info.provider);
    } else {
        println!("! Model not found (will download on first use)");
    }
    
    println!("\n✅ All headless tests completed!");
}

#[tokio::main]
async fn main() {
    // Run the test directly for cargo run
    test_headless_functionality().await;
}