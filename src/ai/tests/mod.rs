mod integration_tests;
mod unit_tests;
mod enhancement_test;

#[cfg(test)]
mod test_helpers {
    use crate::ai::{Result, EnhancementOptions, EnhancedText, Intent, Correction};
    use crate::ai::services::model_service::AIModel;
    use std::sync::Arc;
    use async_trait::async_trait;
    
    /// Mock AI model for testing
    pub struct MockAIModel {
        pub id: String,
        pub memory_usage: u64,
        pub should_fail: bool,
        pub latency_ms: u64,
    }
    
    impl MockAIModel {
        pub fn new(id: String) -> Self {
            Self {
                id,
                memory_usage: 100 * 1024 * 1024, // 100MB
                should_fail: false,
                latency_ms: 50,
            }
        }
        
        pub fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }
        
        pub fn with_latency(mut self, latency_ms: u64) -> Self {
            self.latency_ms = latency_ms;
            self
        }
    }
    
    #[async_trait]
    impl AIModel for MockAIModel {
        fn model_id(&self) -> &str {
            &self.id
        }
        
        fn memory_usage(&self) -> u64 {
            self.memory_usage
        }
        
        async fn enhance_text(
            &self,
            text: &str,
            _options: &EnhancementOptions,
        ) -> Result<EnhancedText> {
            // Simulate latency
            tokio::time::sleep(tokio::time::Duration::from_millis(self.latency_ms)).await;
            
            if self.should_fail {
                return Err(crate::ai::AIError::InferenceError("Mock failure".to_string()));
            }
            
            // Simple mock enhancement
            let enhanced = text
                .replace("grammer", "grammar")
                .replace("sentense", "sentence")
                .replace("  ", " ");
            
            let corrections = if enhanced != text {
                vec![Correction {
                    start: 0,
                    end: text.len(),
                    original: text.to_string(),
                    corrected: enhanced.clone(),
                    reason: "Mock correction".to_string(),
                }]
            } else {
                vec![]
            };
            
            Ok(EnhancedText {
                original: text.to_string(),
                enhanced,
                intent: Some(Intent::Dictation),
                confidence: 0.95,
                corrections,
            })
        }
    }
    
    /// Create a mock model service for testing
    pub async fn create_mock_model_service() -> Arc<crate::ai::services::model_service::ModelService> {
        // This would create a model service with mock models
        // For now, return a placeholder
        unimplemented!("Mock model service creation")
    }
    
    /// Test data generator
    pub struct TestDataGenerator;
    
    impl TestDataGenerator {
        pub fn generate_text_samples(count: usize) -> Vec<String> {
            (0..count)
                .map(|i| format!("Test text sample {} with some errors", i))
                .collect()
        }
        
        pub fn generate_conversation_turns(count: usize) -> Vec<(String, String)> {
            (0..count)
                .map(|i| {
                    (
                        format!("User message {}", i),
                        format!("Assistant response {}", i),
                    )
                })
                .collect()
        }
    }
}