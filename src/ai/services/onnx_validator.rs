use std::path::Path;
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use ort::{Environment, SessionBuilder};

/// ONNX model validator
pub struct OnnxValidator {
    environment: Arc<Environment>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub model_info: Option<ModelInfo>,
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<String>,
    pub compatibility: CompatibilityInfo,
}

/// Extracted model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
    pub input_shapes: HashMap<String, Vec<i64>>,
    pub output_shapes: HashMap<String, Vec<i64>>,
    pub opset_version: i64,
    pub producer_name: String,
    pub producer_version: String,
    pub graph_name: String,
    pub model_version: i64,
    pub metadata: HashMap<String, String>,
    pub estimated_parameters: u64,
    pub model_size_bytes: u64,
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    FileFormat,
    ModelStructure,
    InputOutput,
    Operators,
    Performance,
    Compatibility,
}

/// Compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub supported_architectures: Vec<String>,
    pub minimum_onnx_version: String,
    pub requires_gpu: bool,
    pub estimated_memory_mb: u64,
    pub supports_dynamic_shapes: bool,
    pub supports_quantization: bool,
}

use std::sync::Arc;

impl OnnxValidator {
    pub fn new() -> Result<Self, String> {
        let environment = Environment::builder()
            .with_name("onnx_validator")
            .build()
            .map_err(|e| format!("Failed to create ONNX environment: {}", e))?;
        
        Ok(Self {
            environment: Arc::new(environment),
        })
    }
    
    /// Validate an ONNX model file
    pub fn validate_model(&self, model_path: &Path) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Check if file exists
        if !model_path.exists() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::FileFormat,
                message: "Model file does not exist".to_string(),
                details: Some(format!("Path: {:?}", model_path)),
            });
            return ValidationResult {
                is_valid: false,
                model_info: None,
                issues,
                warnings,
                compatibility: CompatibilityInfo::default(),
            };
        }
        
        // Check file extension
        if model_path.extension().and_then(|s| s.to_str()) != Some("onnx") {
            warnings.push("File does not have .onnx extension".to_string());
        }
        
        // Get file size
        let file_size = match fs::metadata(model_path) {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::FileFormat,
                    message: "Cannot read file metadata".to_string(),
                    details: Some(e.to_string()),
                });
                return ValidationResult {
                    is_valid: false,
                    model_info: None,
                    issues,
                    warnings,
                    compatibility: CompatibilityInfo::default(),
                };
            }
        };
        
        // Check file size
        if file_size == 0 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::FileFormat,
                message: "Model file is empty".to_string(),
                details: None,
            });
        } else if file_size > 10 * 1024 * 1024 * 1024 { // 10GB
            warnings.push(format!("Model file is very large: {:.2} GB", file_size as f64 / 1e9));
        }
        
        // Try to load the model
        let model_info = match self.load_and_inspect_model(model_path, file_size) {
            Ok(info) => {
                // Validate model structure
                self.validate_model_structure(&info, &mut issues, &mut warnings);
                Some(info)
            }
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::ModelStructure,
                    message: "Failed to load ONNX model".to_string(),
                    details: Some(e),
                });
                None
            }
        };
        
        // Determine compatibility
        let compatibility = if let Some(ref info) = model_info {
            self.analyze_compatibility(info, file_size)
        } else {
            CompatibilityInfo::default()
        };
        
        // Check for critical issues
        let has_errors = issues.iter().any(|i| i.severity == IssueSeverity::Error);
        
        ValidationResult {
            is_valid: !has_errors && model_info.is_some(),
            model_info,
            issues,
            warnings,
            compatibility,
        }
    }
    
    /// Load and inspect ONNX model
    fn load_and_inspect_model(&self, model_path: &Path, file_size: u64) -> Result<ModelInfo, String> {
        // Create a session to inspect the model
        let session = SessionBuilder::new(&self.environment)
            .map_err(|e| format!("Failed to create session builder: {}", e))?
            .with_model_from_file(model_path)
            .map_err(|e| format!("Failed to load model: {}", e))?;
        
        // Extract input information
        let mut input_names = Vec::new();
        let mut input_shapes = HashMap::new();
        
        for (i, input) in session.inputs.iter().enumerate() {
            let name = input.name.clone();
            input_names.push(name.clone());
            
            // Get shape
            let shape: Vec<i64> = input.dimensions()
                .map(|d| d.map(|v| v as i64).unwrap_or(-1))
                .collect();
            input_shapes.insert(name, shape);
        }
        
        // Extract output information
        let mut output_names = Vec::new();
        let mut output_shapes = HashMap::new();
        
        for output in session.outputs.iter() {
            let name = output.name.clone();
            output_names.push(name.clone());
            
            // Get shape
            let shape: Vec<i64> = output.dimensions()
                .map(|d| d.map(|v| v as i64).unwrap_or(-1))
                .collect();
            output_shapes.insert(name, shape);
        }
        
        // Extract metadata (simplified - in production, parse ONNX protobuf)
        let metadata = HashMap::new();
        
        // Estimate parameters (very rough estimate based on file size)
        let estimated_parameters = (file_size / 4) as u64; // Assume float32
        
        Ok(ModelInfo {
            input_names,
            output_names,
            input_shapes,
            output_shapes,
            opset_version: 14, // Default, would extract from model
            producer_name: "Unknown".to_string(),
            producer_version: "Unknown".to_string(),
            graph_name: "main_graph".to_string(),
            model_version: 1,
            metadata,
            estimated_parameters,
            model_size_bytes: file_size,
        })
    }
    
    /// Validate model structure
    fn validate_model_structure(
        &self,
        info: &ModelInfo,
        issues: &mut Vec<ValidationIssue>,
        warnings: &mut Vec<String>,
    ) {
        // Check inputs
        if info.input_names.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::InputOutput,
                message: "Model has no inputs".to_string(),
                details: None,
            });
        } else if info.input_names.len() > 10 {
            warnings.push(format!("Model has many inputs: {}", info.input_names.len()));
        }
        
        // Check outputs
        if info.output_names.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::InputOutput,
                message: "Model has no outputs".to_string(),
                details: None,
            });
        }
        
        // Check input shapes
        for (name, shape) in &info.input_shapes {
            // Check for invalid dimensions
            if shape.iter().any(|&d| d == 0) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::InputOutput,
                    message: format!("Input '{}' has zero dimension", name),
                    details: Some(format!("Shape: {:?}", shape)),
                });
            }
            
            // Check for very large dimensions
            if shape.iter().any(|&d| d > 100000) {
                warnings.push(format!("Input '{}' has very large dimension: {:?}", name, shape));
            }
            
            // Check for dynamic dimensions
            if shape.iter().any(|&d| d < 0) {
                warnings.push(format!("Input '{}' has dynamic dimensions", name));
            }
        }
        
        // Check model size vs parameters
        let expected_size = info.estimated_parameters * 4; // float32
        let size_ratio = info.model_size_bytes as f64 / expected_size as f64;
        
        if size_ratio > 2.0 {
            warnings.push("Model file seems larger than expected for parameter count".to_string());
        } else if size_ratio < 0.5 {
            warnings.push("Model file seems smaller than expected for parameter count".to_string());
        }
    }
    
    /// Analyze model compatibility
    fn analyze_compatibility(&self, info: &ModelInfo, file_size: u64) -> CompatibilityInfo {
        // Determine supported architectures based on model characteristics
        let mut supported_architectures = vec!["cpu".to_string()];
        
        // Models under 1GB generally work well on GPU
        if file_size < 1_000_000_000 {
            supported_architectures.push("cuda".to_string());
            supported_architectures.push("metal".to_string());
            supported_architectures.push("directml".to_string());
        }
        
        // Check if model requires GPU based on size
        let requires_gpu = file_size > 4_000_000_000; // >4GB typically needs GPU
        
        // Estimate memory usage (model size + working memory)
        let estimated_memory_mb = (file_size as f64 * 2.5 / 1_000_000.0) as u64;
        
        // Check for dynamic shapes
        let supports_dynamic_shapes = info.input_shapes.values()
            .any(|shape| shape.iter().any(|&d| d < 0));
        
        // Models with certain characteristics support quantization better
        let supports_quantization = info.estimated_parameters > 1_000_000;
        
        CompatibilityInfo {
            supported_architectures,
            minimum_onnx_version: "1.6.0".to_string(),
            requires_gpu,
            estimated_memory_mb,
            supports_dynamic_shapes,
            supports_quantization,
        }
    }
    
    /// Validate model for specific use case
    pub fn validate_for_use_case(
        &self,
        model_path: &Path,
        use_case: ModelUseCase,
    ) -> ValidationResult {
        let mut result = self.validate_model(model_path);
        
        // Add use-case specific validation
        match use_case {
            ModelUseCase::TextGeneration => {
                self.validate_text_generation_model(&mut result);
            }
            ModelUseCase::TextClassification => {
                self.validate_text_classification_model(&mut result);
            }
            ModelUseCase::TokenClassification => {
                self.validate_token_classification_model(&mut result);
            }
            ModelUseCase::Seq2Seq => {
                self.validate_seq2seq_model(&mut result);
            }
        }
        
        result
    }
    
    /// Validate text generation model
    fn validate_text_generation_model(&self, result: &mut ValidationResult) {
        if let Some(ref info) = result.model_info {
            // Check for decoder inputs
            let has_decoder_inputs = info.input_names.iter()
                .any(|name| name.contains("input_ids") || name.contains("token"));
            
            if !has_decoder_inputs {
                result.issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::ModelStructure,
                    message: "Model may not be suitable for text generation".to_string(),
                    details: Some("Expected input names containing 'input_ids' or 'token'".to_string()),
                });
            }
            
            // Check output structure
            let has_logits = info.output_names.iter()
                .any(|name| name.contains("logits") || name.contains("output"));
            
            if !has_logits {
                result.issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::ModelStructure,
                    message: "Model outputs may not be suitable for text generation".to_string(),
                    details: Some("Expected output names containing 'logits'".to_string()),
                });
            }
        }
    }
    
    /// Validate text classification model
    fn validate_text_classification_model(&self, result: &mut ValidationResult) {
        if let Some(ref info) = result.model_info {
            // Check for appropriate outputs
            let output_count = info.output_names.len();
            if output_count != 1 {
                result.warnings.push(format!(
                    "Text classification models typically have 1 output, found {}",
                    output_count
                ));
            }
        }
    }
    
    /// Validate token classification model
    fn validate_token_classification_model(&self, result: &mut ValidationResult) {
        if let Some(ref info) = result.model_info {
            // Check for sequence outputs
            for (name, shape) in &info.output_shapes {
                if shape.len() < 3 {
                    result.warnings.push(format!(
                        "Output '{}' may not be suitable for token classification (expected 3D tensor)",
                        name
                    ));
                }
            }
        }
    }
    
    /// Validate seq2seq model
    fn validate_seq2seq_model(&self, result: &mut ValidationResult) {
        if let Some(ref info) = result.model_info {
            // Check for encoder and decoder inputs
            let has_encoder = info.input_names.iter()
                .any(|name| name.contains("encoder"));
            let has_decoder = info.input_names.iter()
                .any(|name| name.contains("decoder"));
            
            if !has_encoder || !has_decoder {
                result.warnings.push(
                    "Seq2Seq model should have encoder and decoder inputs".to_string()
                );
            }
        }
    }
}

/// Model use case
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelUseCase {
    TextGeneration,
    TextClassification,
    TokenClassification,
    Seq2Seq,
}

impl Default for CompatibilityInfo {
    fn default() -> Self {
        Self {
            supported_architectures: vec!["cpu".to_string()],
            minimum_onnx_version: "1.6.0".to_string(),
            requires_gpu: false,
            estimated_memory_mb: 0,
            supports_dynamic_shapes: false,
            supports_quantization: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_validator_creation() {
        let validator = OnnxValidator::new();
        assert!(validator.is_ok());
    }
    
    #[test]
    fn test_nonexistent_file_validation() {
        let validator = OnnxValidator::new().unwrap();
        let result = validator.validate_model(Path::new("nonexistent.onnx"));
        
        assert!(!result.is_valid);
        assert!(!result.issues.is_empty());
        assert_eq!(result.issues[0].category, IssueCategory::FileFormat);
    }
}