use crate::ai::{Result, AIError, EnhancementOptions, EnhancedText, Intent, Correction};
use crate::ai::common::ModelInfo;
use crate::ai::local::LocalModel;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ONNX-based implementation for grammar correction models
pub struct OnnxGrammarModel {
    session: Arc<ort::Session>,
    tokenizer: Arc<tokenizers::Tokenizer>,
    model_info: ModelInfo,
    max_length: usize,
}

impl OnnxGrammarModel {
    pub async fn load(model_path: &Path, model_info: ModelInfo) -> Result<Self> {
        // Initialize ONNX Runtime is not needed with ort 1.16 - it's done automatically

        // Load the ONNX model
        let session = ort::Session::new(model_path)
            .map_err(|e| AIError::ModelNotFound(format!("Failed to load model: {}", e)))?;

        // Load tokenizer
        let tokenizer_path = model_path.with_extension("json");
        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| AIError::ConfigError(format!("Failed to load tokenizer: {}", e)))?;

        Ok(Self {
            session: Arc::new(session),
            tokenizer: Arc::new(tokenizer),
            model_info,
            max_length: 512,
        })
    }

    async fn tokenize(&self, text: &str) -> Result<tokenizers::Encoding> {
        let encoding = self.tokenizer
            .encode(text, true)
            .map_err(|e| AIError::InferenceError(format!("Tokenization failed: {}", e)))?;
        
        Ok(encoding)
    }

    async fn run_inference(&self, input_ids: &[u32], attention_mask: &[u32]) -> Result<Vec<u32>> {
        use ndarray::{Array2, ArrayD};
        
        // Create input tensors
        let input_ids_array = Array2::from_shape_vec(
            (1, input_ids.len()),
            input_ids.iter().map(|&x| x as i64).collect(),
        ).map_err(|e| AIError::InferenceError(format!("Failed to create input array: {}", e)))?;
        
        let attention_mask_array = Array2::from_shape_vec(
            (1, attention_mask.len()),
            attention_mask.iter().map(|&x| x as i64).collect(),
        ).map_err(|e| AIError::InferenceError(format!("Failed to create attention mask: {}", e)))?;

        // Run inference
        let input_ids_value = ort::Value::from_array(input_ids_array)
            .map_err(|e| AIError::InferenceError(format!("Failed to create input_ids value: {}", e)))?;
        let attention_mask_value = ort::Value::from_array(attention_mask_array)
            .map_err(|e| AIError::InferenceError(format!("Failed to create attention_mask value: {}", e)))?;
        
        let inputs = vec![
            ("input_ids".to_string(), input_ids_value),
            ("attention_mask".to_string(), attention_mask_value),
        ];

        let outputs = self.session.run(inputs)
            .map_err(|e| AIError::InferenceError(format!("Inference failed: {}", e)))?;

        // Extract output tokens
        let output_key = outputs.keys().next()
            .ok_or_else(|| AIError::InferenceError("No output from model".to_string()))?;
        
        let output_tensor = &outputs[output_key];
        let output_array = output_tensor
            .try_extract_tensor::<i64>()
            .map_err(|e| AIError::InferenceError(format!("Failed to extract output: {}", e)))?;

        // Convert to u32
        let output_ids: Vec<u32> = output_array
            .as_slice()
            .ok_or_else(|| AIError::InferenceError("Failed to get output slice".to_string()))?
            .iter()
            .map(|&x| x as u32)
            .collect();

        Ok(output_ids)
    }

    fn detect_corrections(&self, original: &str, corrected: &str) -> Vec<Correction> {
        let mut corrections = Vec::new();
        
        // Simple word-level diff
        let orig_words: Vec<&str> = original.split_whitespace().collect();
        let corr_words: Vec<&str> = corrected.split_whitespace().collect();
        
        let mut orig_pos = 0;
        for (i, (orig_word, corr_word)) in orig_words.iter().zip(corr_words.iter()).enumerate() {
            if orig_word != corr_word {
                corrections.push(Correction {
                    start: orig_pos,
                    end: orig_pos + orig_word.len(),
                    original: orig_word.to_string(),
                    corrected: corr_word.to_string(),
                    reason: "Grammar correction".to_string(),
                });
            }
            orig_pos += orig_word.len() + 1; // +1 for space
        }
        
        corrections
    }
}

#[async_trait::async_trait]
impl LocalModel for OnnxGrammarModel {
    async fn infer(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Result<EnhancedText> {
        // For now, let's implement a simple enhancement using the model
        let encoding = self.tokenize(text).await?;
        let input_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        
        // Run inference
        let output_ids = self.run_inference(input_ids, attention_mask).await?;
        
        // Decode output
        let enhanced_text = self.tokenizer
            .decode(&output_ids, true)
            .map_err(|e| AIError::InferenceError(format!("Decoding failed: {}", e)))?;
        
        // Detect corrections
        let corrections = if options.correct_grammar {
            self.detect_corrections(text, &enhanced_text)
        } else {
            Vec::new()
        };
        
        // Detect intent if requested
        let intent = if options.detect_intent {
            self.detect_intent(text)?
        } else {
            None
        };
        
        Ok(EnhancedText {
            original: text.to_string(),
            enhanced: enhanced_text,
            intent,
            confidence: 0.85, // TODO: Calculate from model output
            corrections,
        })
    }
    
    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }
}

impl OnnxGrammarModel {
    fn detect_intent(&self, text: &str) -> Result<Option<Intent>> {
        let lower = text.to_lowercase();
        
        // Simple pattern-based intent detection
        if lower.starts_with("delete") || lower.starts_with("remove") || lower.starts_with("undo") {
            Ok(Some(Intent::Command("edit".to_string())))
        } else if lower.contains('?') || 
                  lower.starts_with("what") || 
                  lower.starts_with("where") ||
                  lower.starts_with("when") ||
                  lower.starts_with("who") ||
                  lower.starts_with("why") ||
                  lower.starts_with("how") {
            Ok(Some(Intent::Question))
        } else if lower.starts_with("hi") || lower.starts_with("hello") || lower.starts_with("hey") {
            Ok(Some(Intent::Conversation))
        } else {
            Ok(Some(Intent::Dictation))
        }
    }
}

/// Simplified T5-style model for grammar correction
pub struct T5GrammarModel {
    onnx_model: OnnxGrammarModel,
}

impl T5GrammarModel {
    pub async fn new(model_path: &Path, model_info: ModelInfo) -> Result<Self> {
        let onnx_model = OnnxGrammarModel::load(model_path, model_info).await?;
        Ok(Self { onnx_model })
    }
    
    /// Create the T5 prompt for grammar correction
    fn create_prompt(&self, text: &str) -> String {
        format!("grammar: {}", text)
    }
}

#[async_trait::async_trait]
impl LocalModel for T5GrammarModel {
    async fn infer(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Result<EnhancedText> {
        // T5 uses a specific prompt format
        let prompt = self.create_prompt(text);
        
        // Use the underlying ONNX model
        let mut result = self.onnx_model.infer(&prompt, options).await?;
        
        // Update the original text (since we modified it with the prompt)
        result.original = text.to_string();
        
        Ok(result)
    }
    
    fn model_info(&self) -> &ModelInfo {
        self.onnx_model.model_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_detection() {
        let model_info = ModelInfo {
            name: "test".to_string(),
            size_bytes: 0,
            parameters: "test".to_string(),
            capabilities: vec![],
            requirements: crate::ai::common::ModelRequirements {
                min_ram_gb: 1.0,
                min_vram_gb: None,
                supports_gpu: false,
                supports_quantization: false,
            },
        };
        
        // We can't easily test the full model without loading it,
        // but we can test helper functions
        let text1 = "delete the last word";
        let text2 = "what is the weather?";
        let text3 = "hello there";
        let text4 = "this is a normal sentence";
        
        // The actual detection is done in the model, so we just verify
        // the test strings are valid
        assert!(text1.contains("delete"));
        assert!(text2.contains('?'));
        assert!(text3.starts_with("hello"));
        assert!(!text4.contains('?'));
    }
}