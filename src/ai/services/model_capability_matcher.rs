use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::ai::services::text_analyzer::{TextCharacteristics, TextDomain};
use crate::ai::models::ModelMetadata;

/// Matches text characteristics with model capabilities
#[derive(Debug, Clone)]
pub struct ModelCapabilityMatcher;

/// Model capability scores for different aspects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityScore {
    /// Overall match score (0-1)
    pub overall: f32,
    /// Individual capability scores
    pub scores: HashMap<String, f32>,
    /// Reasons for the score
    pub reasons: Vec<String>,
}

/// Requirements derived from text characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    /// Minimum context window needed
    pub min_context_window: usize,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Preferred capabilities (nice to have)
    pub preferred_capabilities: Vec<String>,
    /// Maximum acceptable latency (ms)
    pub max_latency_ms: u32,
    /// Minimum accuracy requirement (0-1)
    pub min_accuracy: f32,
    /// Whether GPU is recommended
    pub gpu_recommended: bool,
    /// Complexity tier (1-5)
    pub complexity_tier: u8,
}

impl ModelCapabilityMatcher {
    pub fn new() -> Self {
        Self
    }
    
    /// Convert text characteristics into model requirements
    pub fn derive_requirements(&self, characteristics: &TextCharacteristics) -> ModelRequirements {
        let mut required_capabilities = Vec::new();
        let mut preferred_capabilities = Vec::new();
        
        // Context window based on text length
        let min_context_window = if characteristics.length > 10000 {
            8192
        } else if characteristics.length > 5000 {
            4096
        } else {
            2048
        };
        
        // Domain-specific requirements
        match characteristics.domain {
            TextDomain::Code => {
                required_capabilities.push("code_understanding".to_string());
                required_capabilities.push("syntax_awareness".to_string());
                preferred_capabilities.push("code_completion".to_string());
            }
            TextDomain::Medical => {
                required_capabilities.push("medical_terminology".to_string());
                required_capabilities.push("high_accuracy".to_string());
            }
            TextDomain::Legal => {
                required_capabilities.push("legal_terminology".to_string());
                required_capabilities.push("precise_language".to_string());
            }
            TextDomain::Academic | TextDomain::Scientific => {
                required_capabilities.push("technical_language".to_string());
                if characteristics.contains_math {
                    required_capabilities.push("mathematical_reasoning".to_string());
                }
            }
            TextDomain::Technical => {
                required_capabilities.push("technical_language".to_string());
            }
            TextDomain::Creative => {
                preferred_capabilities.push("creative_writing".to_string());
                preferred_capabilities.push("style_adaptation".to_string());
            }
            _ => {}
        }
        
        // Language requirements
        if characteristics.language != "en" {
            required_capabilities.push(format!("language_{}", characteristics.language));
            required_capabilities.push("multilingual".to_string());
        }
        
        // Complexity-based requirements
        let complexity_tier = if characteristics.complexity_score > 0.8 {
            5
        } else if characteristics.complexity_score > 0.6 {
            4
        } else if characteristics.complexity_score > 0.4 {
            3
        } else if characteristics.complexity_score > 0.2 {
            2
        } else {
            1
        };
        
        // Performance requirements based on use case
        let (max_latency_ms, min_accuracy) = if characteristics.domain == TextDomain::Conversational {
            (100, 0.8) // Fast response for conversation
        } else if characteristics.domain == TextDomain::Medical || characteristics.domain == TextDomain::Legal {
            (5000, 0.95) // High accuracy more important than speed
        } else if characteristics.complexity_score > 0.7 {
            (3000, 0.9) // Complex text needs better models
        } else {
            (500, 0.85) // Balanced requirements
        };
        
        // GPU recommendation based on complexity and length
        let gpu_recommended = characteristics.complexity_score > 0.6 || 
                            characteristics.length > 5000 ||
                            characteristics.domain == TextDomain::Code;
        
        // Special content requirements
        if characteristics.contains_urls {
            preferred_capabilities.push("url_understanding".to_string());
        }
        
        ModelRequirements {
            min_context_window,
            required_capabilities,
            preferred_capabilities,
            max_latency_ms,
            min_accuracy,
            gpu_recommended,
            complexity_tier,
        }
    }
    
    /// Calculate capability match score between requirements and model
    pub fn calculate_match_score(
        &self,
        requirements: &ModelRequirements,
        model: &ModelMetadata,
    ) -> CapabilityScore {
        let mut score = CapabilityScore::default();
        let mut total_weight = 0.0;
        let mut weighted_score = 0.0;
        
        // Context window check (weight: 0.2)
        let context_score = if model.context_window >= requirements.min_context_window {
            1.0
        } else {
            (model.context_window as f32 / requirements.min_context_window as f32).max(0.0)
        };
        weighted_score += context_score * 0.2;
        total_weight += 0.2;
        score.scores.insert("context_window".to_string(), context_score);
        
        if context_score < 1.0 {
            score.reasons.push(format!(
                "Context window {} < required {}",
                model.context_window, requirements.min_context_window
            ));
        }
        
        // Required capabilities check (weight: 0.4)
        let required_met = requirements.required_capabilities.iter()
            .filter(|cap| {
                model.capabilities.iter().any(|c| format!("{:?}", c).to_lowercase() == cap.to_lowercase())
            })
            .count();
        let required_score = if requirements.required_capabilities.is_empty() {
            1.0
        } else {
            required_met as f32 / requirements.required_capabilities.len() as f32
        };
        weighted_score += required_score * 0.4;
        total_weight += 0.4;
        score.scores.insert("required_capabilities".to_string(), required_score);
        
        if required_score < 1.0 {
            let missing: Vec<_> = requirements.required_capabilities.iter()
                .filter(|cap| {
                    !model.capabilities.iter().any(|c| format!("{:?}", c).to_lowercase() == cap.to_lowercase())
                })
                .collect();
            score.reasons.push(format!("Missing required capabilities: {:?}", missing));
        }
        
        // Preferred capabilities check (weight: 0.1)
        let preferred_met = requirements.preferred_capabilities.iter()
            .filter(|cap| {
                model.capabilities.iter().any(|c| format!("{:?}", c).to_lowercase() == cap.to_lowercase())
            })
            .count();
        let preferred_score = if requirements.preferred_capabilities.is_empty() {
            1.0
        } else {
            preferred_met as f32 / requirements.preferred_capabilities.len() as f32
        };
        weighted_score += preferred_score * 0.1;
        total_weight += 0.1;
        score.scores.insert("preferred_capabilities".to_string(), preferred_score);
        
        // Performance class match (weight: 0.2)
        let performance_score = match (requirements.complexity_tier, model.performance_class.to_lowercase().as_str()) {
            (1..=2, "low") | (1..=2, "fast") => 1.0,
            (3..=4, "medium") | (3..=4, "balanced") => 1.0,
            (5, "high") | (5, "quality") => 1.0,
            (1..=2, "medium") | (1..=2, "balanced") => 0.8,
            (3..=4, "low") | (3..=4, "fast") => 0.6,
            (3..=4, "high") | (3..=4, "quality") => 0.8,
            (5, "medium") | (5, "balanced") => 0.7,
            (5, "low") | (5, "fast") => 0.4,
            _ => 0.5,
        };
        weighted_score += performance_score * 0.2;
        total_weight += 0.2;
        score.scores.insert("performance_class".to_string(), performance_score);
        
        if performance_score < 0.8 {
            score.reasons.push(format!(
                "Model performance class '{}' not optimal for complexity tier {}",
                model.performance_class, requirements.complexity_tier
            ));
        }
        
        // Hardware requirements match (weight: 0.1)
        let hardware_score = if requirements.gpu_recommended && model.supports_gpu {
            1.0
        } else if !requirements.gpu_recommended {
            1.0
        } else {
            0.7 // Can still work on CPU, just slower
        };
        weighted_score += hardware_score * 0.1;
        total_weight += 0.1;
        score.scores.insert("hardware_match".to_string(), hardware_score);
        
        // Calculate overall score
        score.overall = weighted_score / total_weight;
        
        // Add positive reasons
        if score.overall > 0.9 {
            score.reasons.push("Excellent match for requirements".to_string());
        } else if score.overall > 0.7 {
            score.reasons.push("Good match for requirements".to_string());
        }
        
        score
    }
    
    /// Rank models by match score
    pub fn rank_models(
        &self,
        requirements: &ModelRequirements,
        models: &[ModelMetadata],
    ) -> Vec<(ModelMetadata, CapabilityScore)> {
        let mut ranked: Vec<(ModelMetadata, CapabilityScore)> = models.iter()
            .map(|model| {
                let score = self.calculate_match_score(requirements, model);
                (model.clone(), score)
            })
            .collect();
        
        // Sort by overall score (descending)
        ranked.sort_by(|a, b| b.1.overall.partial_cmp(&a.1.overall).unwrap());
        
        ranked
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::services::text_analyzer::TextAnalyzer;
    
    fn create_test_model(name: &str, capabilities: Vec<&str>, performance_class: &str) -> ModelMetadata {
        use crate::ai::models::{ModelFormat, ModelSource, PerformanceProfile, ModelRequirements, Capability};
        
        ModelMetadata {
            id: name.to_string(),
            name: name.to_string(),
            display_name: name.to_string(),
            description: "Test model".to_string(),
            architecture: "transformer".to_string(),
            parameters: "1B".to_string(),
            size_bytes: 1_000_000_000,
            format: ModelFormat::ONNX,
            source: ModelSource::Local { path: std::path::PathBuf::from("test.onnx") },
            capabilities: capabilities.into_iter().map(|s| match s {
                "grammar_correction" => Capability::GrammarCorrection,
                "punctuation" => Capability::Punctuation,
                "summarization" => Capability::Summarization,
                "translation" => Capability::Translation,
                "intent_detection" => Capability::IntentDetection,
                _ => Capability::GrammarCorrection,
            }).collect(),
            performance: PerformanceProfile {
                avg_latency_ms: 50.0,
                tokens_per_second: 100.0,
                memory_usage_mb: 1000,
                supports_batch: false,
                max_batch_size: 1,
            },
            requirements: ModelRequirements {
                min_ram_gb: 1.0,
                min_vram_gb: None,
                supports_cpu: true,
                supports_gpu: false,
                supported_backends: vec!["cpu".to_string()],
            },
            performance_class: performance_class.to_string(),
            context_window: 4096,
            supports_gpu: true,
            download_url: None,
            sha256: None,
        }
    }
    
    #[test]
    fn test_derive_requirements() {
        let analyzer = TextAnalyzer::new();
        let matcher = ModelCapabilityMatcher::new();
        
        // Test code domain
        let code_text = "function calculate() { return x * 2; }";
        let code_chars = analyzer.analyze(code_text);
        let code_reqs = matcher.derive_requirements(&code_chars);
        
        assert!(code_reqs.required_capabilities.contains(&"code_understanding".to_string()));
        assert!(code_reqs.gpu_recommended);
        
        // Test medical domain
        let medical_text = "Patient diagnosis requires careful examination of symptoms.";
        let medical_chars = analyzer.analyze(medical_text);
        let medical_reqs = matcher.derive_requirements(&medical_chars);
        
        assert!(medical_reqs.required_capabilities.contains(&"medical_terminology".to_string()));
        assert_eq!(medical_reqs.min_accuracy, 0.95);
    }
    
    #[test]
    fn test_capability_matching() {
        let matcher = ModelCapabilityMatcher::new();
        
        let requirements = ModelRequirements {
            min_context_window: 2048,
            required_capabilities: vec!["code_understanding".to_string()],
            preferred_capabilities: vec!["code_completion".to_string()],
            max_latency_ms: 1000,
            min_accuracy: 0.9,
            gpu_recommended: true,
            complexity_tier: 3,
        };
        
        let good_model = create_test_model(
            "code-model",
            vec!["code_understanding", "code_completion"],
            "balanced"
        );
        
        let poor_model = create_test_model(
            "general-model",
            vec!["general_understanding"],
            "fast"
        );
        
        let good_score = matcher.calculate_match_score(&requirements, &good_model);
        let poor_score = matcher.calculate_match_score(&requirements, &poor_model);
        
        assert!(good_score.overall > poor_score.overall);
        assert_eq!(good_score.scores["required_capabilities"], 1.0);
        assert_eq!(poor_score.scores["required_capabilities"], 0.0);
    }
    
    #[test]
    fn test_model_ranking() {
        let matcher = ModelCapabilityMatcher::new();
        
        let requirements = ModelRequirements {
            min_context_window: 4096,
            required_capabilities: vec!["technical_language".to_string()],
            preferred_capabilities: vec![],
            max_latency_ms: 2000,
            min_accuracy: 0.85,
            gpu_recommended: false,
            complexity_tier: 2,
        };
        
        let models = vec![
            create_test_model("fast-model", vec!["technical_language"], "fast"),
            create_test_model("balanced-model", vec!["technical_language", "general"], "balanced"),
            create_test_model("quality-model", vec!["general"], "quality"),
        ];
        
        let ranked = matcher.rank_models(&requirements, &models);
        
        assert_eq!(ranked[0].0.name, "fast-model"); // Best match for low complexity
        assert!(ranked[0].1.overall > ranked[2].1.overall);
    }
}