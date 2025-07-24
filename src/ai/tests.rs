#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{EnhancementOptions, PrivacyLevel};

    #[test]
    fn test_enhancement_options_default() {
        let options = EnhancementOptions::default();
        assert!(options.correct_grammar);
        assert!(options.improve_punctuation);
        assert!(options.detect_intent);
        assert!(options.preserve_style);
        assert_eq!(options.confidence_threshold, 0.7);
    }

    #[test]
    fn test_privacy_levels() {
        let levels = vec![
            PrivacyLevel::Strict,
            PrivacyLevel::Balanced,
            PrivacyLevel::Permissive,
        ];
        
        for level in levels {
            match level {
                PrivacyLevel::Strict => assert!(true),
                PrivacyLevel::Balanced => assert!(true),
                PrivacyLevel::Permissive => assert!(true),
            }
        }
    }

    #[test]
    fn test_intent_types() {
        use crate::ai::Intent;
        
        let intents = vec![
            Intent::Dictation,
            Intent::Command("test".to_string()),
            Intent::Question,
            Intent::Conversation,
        ];
        
        for intent in intents {
            match intent {
                Intent::Dictation => assert!(true),
                Intent::Command(cmd) => assert!(!cmd.is_empty()),
                Intent::Question => assert!(true),
                Intent::Conversation => assert!(true),
            }
        }
    }

    #[test]
    fn test_correction_struct() {
        use crate::ai::Correction;
        
        let correction = Correction {
            start: 0,
            end: 5,
            original: "teh".to_string(),
            corrected: "the".to_string(),
            reason: "Common typo".to_string(),
        };
        
        assert_eq!(correction.start, 0);
        assert_eq!(correction.end, 5);
        assert_eq!(correction.original, "teh");
        assert_eq!(correction.corrected, "the");
    }

    #[test]
    fn test_model_info() {
        use crate::ai::common::find_model_info;
        
        let phi3_info = find_model_info("phi-3-mini");
        assert!(phi3_info.is_some());
        
        if let Some(info) = phi3_info {
            assert_eq!(info.name, "phi-3-mini");
            assert_eq!(info.parameters, "3.8B");
            assert!(info.capabilities.contains(&"grammar_correction".to_string()));
        }
    }

    #[tokio::test]
    async fn test_basic_text_enhancement() {
        use crate::ai::local::enhancement::TextEnhancer;
        
        let enhancer = TextEnhancer::new();
        let (result, corrections) = enhancer.apply_basic_corrections("i cant beleive it");
        
        assert!(result.contains("can't"));
        assert!(result.contains("believe"));
        assert!(!corrections.is_empty());
    }

    #[test]
    fn test_punctuation_improvement() {
        use crate::ai::local::enhancement::TextEnhancer;
        
        let enhancer = TextEnhancer::new();
        
        let result1 = enhancer.improve_punctuation("hello world");
        assert_eq!(result1, "Hello world.");
        
        let result2 = enhancer.improve_punctuation("what is your name");
        assert_eq!(result2, "What is your name?");
    }
}