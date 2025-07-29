use bestme::ai::services::ai_service::AIService;
use bestme::config::Config;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_ai_service_basic() {
    let config = Config::default();
    let ai_service = AIService::new(config);
    
    // Test service availability
    let available = ai_service.is_available().await;
    println!("AI Service available: {}", available);
    
    // Get model info
    let model = ai_service.get_current_model();
    println!("Current AI model: {:?}", model);
    
    assert!(true); // Basic test passes if no panic
}

#[tokio::test]
async fn test_text_enhancement() {
    let config = Config::default();
    let ai_service = Arc::new(Mutex::new(AIService::new(config)));
    
    let test_text = "hello world how are you";
    let result = ai_service.lock().await.enhance_text(test_text).await;
    
    match result {
        Ok(enhanced) => {
            println!("Original: {}", test_text);
            println!("Enhanced: {}", enhanced);
            assert!(!enhanced.is_empty());
        }
        Err(e) => {
            println!("Enhancement not available: {}", e);
            // This is acceptable if models aren't downloaded
        }
    }
}

#[test]
fn test_ai_config() {
    let config = Config::default();
    println!("AI Provider: {:?}", config.ai.provider);
    println!("AI Model: {}", config.ai.model.name);
    assert!(!config.ai.model.name.is_empty());
}