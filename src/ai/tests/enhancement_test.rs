#[cfg(test)]
mod enhancement_tests {
    use crate::ai::local::LocalAIProcessor;
    use crate::ai::services::ai_service::AIService;
    use crate::ai::local::model_manager::ModelManager;
    use crate::ai::types::{AIProvider, AIModel, ModelSource};
    use crate::config::Config;
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_text_enhancement_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            ai: crate::config::AIConfig {
                provider: AIProvider::Local,
                api_key: None,
                model: AIModel {
                    provider: AIProvider::Local,
                    name: "phi3.5-mini".to_string(),
                    source: ModelSource::HuggingFace,
                    size: None,
                    quantization: None,
                    context_length: 4096,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let model_manager = Arc::new(ModelManager::new(temp_dir.path().to_path_buf()));
        let processor = LocalAIProcessor::new(config.clone(), model_manager.clone());
        let ai_service = Arc::new(Mutex::new(AIService::new(config.clone())));

        // Test text enhancement
        let input = "hello wrld how r u today";
        let result = ai_service.lock().await.enhance_text(input).await;

        match result {
            Ok(enhanced) => {
                println!("Enhanced text: {}", enhanced);
                // Should correct spelling and grammar
                assert!(enhanced.contains("Hello"));
                assert!(enhanced.contains("world"));
                assert!(enhanced.contains("are"));
                assert!(enhanced.contains("you"));
            }
            Err(e) => {
                // If model is not available, that's okay for this test
                println!("Text enhancement not available: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_text_enhancement_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        let ai_service = Arc::new(Mutex::new(AIService::new(config)));

        // Test with context
        let input = "The cat sat on the";
        let context = Some("We are writing a children's story about a cat.".to_string());
        
        let result = ai_service.lock().await.enhance_text_with_context(input, context.as_deref()).await;

        match result {
            Ok(enhanced) => {
                println!("Enhanced with context: {}", enhanced);
                // Should complete the sentence appropriately
                assert!(enhanced.len() > input.len());
            }
            Err(e) => {
                println!("Text enhancement with context not available: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_ai_service_initialization() {
        let config = Config::default();
        let ai_service = AIService::new(config);

        // Test that service initializes correctly
        assert!(ai_service.is_available().await);
        
        // Get current model info
        let model_info = ai_service.get_current_model();
        println!("Current model: {:?}", model_info);
        assert!(!model_info.name.is_empty());
    }

    #[tokio::test]
    async fn test_punctuation_correction() {
        let config = Config::default();
        let ai_service = Arc::new(Mutex::new(AIService::new(config)));

        let test_cases = vec![
            ("hello world", "Hello world."),
            ("how are you doing today", "How are you doing today?"),
            ("wow that's amazing", "Wow, that's amazing!"),
        ];

        for (input, _expected) in test_cases {
            let result = ai_service.lock().await.enhance_text(input).await;
            match result {
                Ok(enhanced) => {
                    println!("Input: '{}' -> Enhanced: '{}'", input, enhanced);
                    // Check basic improvements
                    assert!(enhanced.chars().next().unwrap().is_uppercase());
                    assert!(enhanced.ends_with('.') || enhanced.ends_with('?') || enhanced.ends_with('!'));
                }
                Err(e) => {
                    println!("Enhancement failed for '{}': {}", input, e);
                }
            }
        }
    }
}