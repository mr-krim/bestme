use crate::ai::{AIProvider, EnhancementOptions, EnhancedText};
use crate::audio::TranscriptionEvent;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// AI Enhancement for transcribed text
pub struct TranscriptionEnhancer {
    ai_provider: Arc<RwLock<Option<Box<dyn AIProvider>>>>,
    options: EnhancementOptions,
    enabled: bool,
}

impl TranscriptionEnhancer {
    pub fn new() -> Self {
        Self {
            ai_provider: Arc::new(RwLock::new(None)),
            options: EnhancementOptions::default(),
            enabled: false,
        }
    }

    pub async fn set_ai_provider(&self, provider: Box<dyn AIProvider>) {
        let mut guard = self.ai_provider.write().await;
        *guard = Some(provider);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_options(&mut self, options: EnhancementOptions) {
        self.options = options;
    }

    pub async fn enhance_transcription_event(
        &self,
        event: TranscriptionEvent,
    ) -> Result<TranscriptionEvent, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(event);
        }

        match event {
            TranscriptionEvent::Transcription(text) => {
                let enhanced_text = self.enhance_text(&text).await?;
                Ok(TranscriptionEvent::Transcription(enhanced_text.enhanced))
            }
            TranscriptionEvent::PartialTranscription(text) => {
                // Don't enhance partial transcriptions
                Ok(TranscriptionEvent::PartialTranscription(text))
            }
            _ => Ok(event), // Pass through other events unchanged
        }
    }

    pub async fn enhance_text(
        &self,
        text: &str,
    ) -> Result<EnhancedText, Box<dyn std::error::Error>> {
        let provider_guard = self.ai_provider.read().await;
        let provider = provider_guard
            .as_ref()
            .ok_or("No AI provider configured")?;

        let enhanced = provider
            .enhance_text(text, &self.options)
            .await?;

        Ok(enhanced)
    }
}

/// Integration with the transcription pipeline
pub struct EnhancedTranscriptionPipeline {
    enhancer: Arc<TranscriptionEnhancer>,
}

impl EnhancedTranscriptionPipeline {
    pub fn new(enhancer: Arc<TranscriptionEnhancer>) -> Self {
        Self { enhancer }
    }

    pub async fn process_event(
        &self,
        event: TranscriptionEvent,
    ) -> Result<TranscriptionEvent, Box<dyn std::error::Error>> {
        // Apply AI enhancement if enabled
        self.enhancer.enhance_transcription_event(event).await
    }

    pub async fn process_streaming_chunk(
        &self,
        text: &str,
        is_final: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if !is_final {
            // Don't enhance partial results
            return Ok(text.to_string());
        }

        // Enhance final chunks
        let enhanced = self.enhancer.enhance_text(text).await?;
        Ok(enhanced.enhanced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{Intent, Correction};

    struct MockAIProvider;

    #[async_trait::async_trait]
    impl AIProvider for MockAIProvider {
        fn name(&self) -> &str {
            "MockAI"
        }

        fn is_available(&self) -> bool {
            true
        }

        async fn initialize(&mut self) -> crate::ai::Result<()> {
            Ok(())
        }

        async fn enhance_text(
            &self,
            text: &str,
            _options: &EnhancementOptions,
        ) -> crate::ai::Result<EnhancedText> {
            // Simple mock enhancement: capitalize and fix "teh" -> "the"
            let enhanced = text.replace("teh", "the");
            let enhanced = if enhanced.chars().next().map_or(false, |c| c.is_lowercase()) {
                format!("{}{}", 
                    enhanced.chars().next().unwrap().to_uppercase(),
                    &enhanced[1..]
                )
            } else {
                enhanced
            };

            Ok(EnhancedText {
                original: text.to_string(),
                enhanced,
                intent: Some(Intent::Dictation),
                confidence: 0.95,
                corrections: vec![
                    Correction {
                        start: 0,
                        end: 3,
                        original: "teh".to_string(),
                        corrected: "the".to_string(),
                        reason: "Spelling correction".to_string(),
                    }
                ],
            })
        }
    }

    #[tokio::test]
    async fn test_transcription_event_enhancement() {
        let mut enhancer = TranscriptionEnhancer::new();
        enhancer.set_ai_provider(Box::new(MockAIProvider)).await;
        enhancer.set_enabled(true);
        
        let event = TranscriptionEvent::Transcription("teh quick brown fox".to_string());
        let enhanced_event = enhancer.enhance_transcription_event(event).await.unwrap();
        
        match enhanced_event {
            TranscriptionEvent::Transcription(text) => {
                assert_eq!(text, "The quick brown fox");
            }
            _ => panic!("Expected Transcription event"),
        }
    }

    #[tokio::test]
    async fn test_streaming_enhancement() {
        let mut enhancer = TranscriptionEnhancer::new();
        enhancer.set_ai_provider(Box::new(MockAIProvider)).await;
        enhancer.set_enabled(true);
        
        let enhancer_arc = Arc::new(enhancer);
        let pipeline = EnhancedTranscriptionPipeline::new(enhancer_arc);
        
        // Partial result should not be enhanced
        let partial = pipeline.process_streaming_chunk("hello", false).await.unwrap();
        assert_eq!(partial, "hello");
        
        // Final result should be enhanced
        let final_text = pipeline.process_streaming_chunk("teh world", true).await.unwrap();
        assert_eq!(final_text, "The world");
    }
}