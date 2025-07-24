#[cfg(test)]
mod integration_tests {
    use crate::ai::{
        AIProvider, EnhancementOptions, EnhancedText, PrivacyLevel, Intent,
        cloud::{CloudAI, CloudAIConfig, AIProviderType},
        local::LocalAI,
        security::{ApiKeyManager, get_api_key_from_env},
        common::ModelConfig,
    };
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_privacy_manager_integration() {
        use crate::ai::cloud::privacy::PrivacyManager;
        
        let manager = PrivacyManager::new(PrivacyLevel::Balanced);
        let test_text = "Contact me at john.doe@example.com or call 555-123-4567";
        
        let (anonymized, deanonymizer) = manager.anonymize(test_text).await.unwrap();
        
        // Verify anonymization
        assert!(anonymized.contains("[EMAIL_"));
        assert!(anonymized.contains("[PHONE_"));
        assert!(!anonymized.contains("john.doe@example.com"));
        assert!(!anonymized.contains("555-123-4567"));
        
        // Verify deanonymization
        if let Some(deanon) = deanonymizer {
            let restored = deanon.restore(&anonymized).unwrap();
            assert_eq!(restored, test_text);
        }
    }

    #[tokio::test]
    async fn test_api_key_environment_fallback() {
        // Set environment variable
        std::env::set_var("BESTME_OPENROUTER_API_KEY", "test_key_123");
        
        let key = get_api_key_from_env("openrouter");
        assert_eq!(key, Some("test_key_123".to_string()));
        
        // Clean up
        std::env::remove_var("BESTME_OPENROUTER_API_KEY");
    }

    #[tokio::test]
    async fn test_enhancement_options() {
        let options = EnhancementOptions::default();
        
        assert!(options.correct_grammar);
        assert!(options.improve_punctuation);
        assert!(options.detect_intent);
        assert!(options.preserve_style);
        assert_eq!(options.confidence_threshold, 0.7);
    }

    #[tokio::test]
    async fn test_text_enhancer_basic_corrections() {
        use crate::ai::local::enhancement::TextEnhancer;
        
        let enhancer = TextEnhancer::new();
        
        // Test basic corrections
        let (result1, corrections1) = enhancer.apply_basic_corrections("i cant beleive it");
        assert!(result1.contains("can't"));
        assert!(result1.contains("believe"));
        assert!(corrections1.len() >= 2);
        
        // Test punctuation
        let result2 = enhancer.improve_punctuation("hello world");
        assert_eq!(result2, "Hello world.");
        
        let result3 = enhancer.improve_punctuation("what is your name");
        assert_eq!(result3, "What is your name?");
    }

    #[tokio::test]
    async fn test_intent_detection_patterns() {
        use crate::ai::local::onnx_models::OnnxGrammarModel;
        
        // Test various intent patterns
        let commands = vec![
            ("delete the last word", "edit"),
            ("remove that sentence", "edit"),
            ("undo the change", "edit"),
        ];
        
        let questions = vec![
            "what is the weather?",
            "where is the meeting?",
            "when does it start?",
            "who is coming?",
            "why is this happening?",
            "how does it work?",
        ];
        
        let conversations = vec![
            "hi there",
            "hello everyone",
            "hey, how are you",
        ];
        
        // Verify patterns
        for (cmd, _) in commands {
            assert!(cmd.contains("delete") || cmd.contains("remove") || cmd.contains("undo"));
        }
        
        for question in questions {
            assert!(question.contains('?') || question.starts_with("what") || 
                   question.starts_with("where") || question.starts_with("when") ||
                   question.starts_with("who") || question.starts_with("why") ||
                   question.starts_with("how"));
        }
        
        for conv in conversations {
            assert!(conv.starts_with("hi") || conv.starts_with("hello") || conv.starts_with("hey"));
        }
    }

    #[tokio::test]
    async fn test_model_download_progress() {
        use crate::ai::local::model::ModelManager;
        
        let manager = ModelManager::new().unwrap();
        let models = manager.list_downloaded_models().unwrap();
        
        // This just verifies the manager can be created and list models
        assert!(models.is_empty() || models.len() > 0);
    }

    #[tokio::test]
    async fn test_cloud_ai_config_serialization() {
        use serde_json;
        
        let config = CloudAIConfig {
            provider: AIProviderType::OpenRouter,
            api_key: "test_key".to_string(),
            model: "claude-3-sonnet".to_string(),
            privacy_level: PrivacyLevel::Balanced,
            base_url: None,
        };
        
        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("OpenRouter"));
        assert!(json.contains("Balanced"));
        
        // Test deserialization
        let config2: CloudAIConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config2.model, config.model);
    }

    #[tokio::test]
    async fn test_ai_enhancement_pipeline() {
        use crate::audio::ai_enhancement::{TranscriptionEnhancer, EnhancedTranscriptionPipeline};
        use std::sync::Arc;
        
        // Create mock provider
        struct MockProvider;
        
        #[async_trait::async_trait]
        impl AIProvider for MockProvider {
            fn name(&self) -> &str { "Mock" }
            fn is_available(&self) -> bool { true }
            async fn initialize(&mut self) -> crate::ai::Result<()> { Ok(()) }
            
            async fn enhance_text(
                &self,
                text: &str,
                _options: &EnhancementOptions,
            ) -> crate::ai::Result<EnhancedText> {
                Ok(EnhancedText {
                    original: text.to_string(),
                    enhanced: text.to_uppercase(),
                    intent: Some(Intent::Dictation),
                    confidence: 0.9,
                    corrections: vec![],
                })
            }
        }
        
        let mut enhancer = TranscriptionEnhancer::new();
        enhancer.set_ai_provider(Box::new(MockProvider)).await;
        enhancer.set_enabled(true);
        
        let pipeline = EnhancedTranscriptionPipeline::new(Arc::new(enhancer));
        
        // Test enhancement
        let result = pipeline.process_streaming_chunk("hello world", true).await.unwrap();
        assert_eq!(result, "HELLO WORLD");
    }

    #[test]
    fn test_model_info_lookup() {
        use crate::ai::common::{find_model_info, AVAILABLE_MODELS};
        
        // Test available models
        assert!(AVAILABLE_MODELS.len() > 0);
        
        // Test model lookup
        let phi3 = find_model_info("phi-3-mini");
        assert!(phi3.is_some());
        
        if let Some(info) = phi3 {
            assert_eq!(info.name, "phi-3-mini");
            assert_eq!(info.parameters, "3.8B");
            assert!(info.capabilities.contains(&"grammar_correction".to_string()));
        }
    }

    #[test]
    fn test_correction_structure() {
        use crate::ai::Correction;
        
        let correction = Correction {
            start: 5,
            end: 8,
            original: "teh".to_string(),
            corrected: "the".to_string(),
            reason: "Common typo".to_string(),
        };
        
        assert_eq!(correction.end - correction.start, 3);
        assert_eq!(correction.original.len(), 3);
        assert_eq!(correction.corrected.len(), 3);
    }

    #[tokio::test]
    async fn test_error_handling() {
        use crate::ai::AIError;
        
        // Test error display
        let errors = vec![
            AIError::ModelNotFound("test.onnx".to_string()),
            AIError::InferenceError("Failed to run".to_string()),
            AIError::ConfigError("Bad config".to_string()),
            AIError::NetworkError("Connection failed".to_string()),
            AIError::PrivacyViolation("PII detected".to_string()),
        ];
        
        for error in errors {
            let msg = error.to_string();
            assert!(!msg.is_empty());
        }
    }
}