use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::ai::services::{OnnxValidator, ValidationResult, ModelInfo as ValidatorModelInfo};
use crate::ai::models::registry::ModelRegistry;
use crate::ai::models::ModelMetadata;

/// Custom model manager for handling user-imported models
pub struct CustomModelManager {
    registry: Arc<ModelRegistry>,
    validator: Arc<OnnxValidator>,
    custom_models: Arc<RwLock<HashMap<String, CustomModel>>>,
    storage_path: PathBuf,
}

/// Custom model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomModel {
    pub id: String,
    pub name: String,
    pub original_filename: String,
    pub description: String,
    pub model_type: CustomModelType,
    pub metadata: ModelMetadata,
    pub validation_result: ValidationResult,
    pub import_date: chrono::DateTime<Utc>,
    pub tags: Vec<String>,
    pub is_active: bool,
    pub usage_count: u64,
    pub last_used: Option<chrono::DateTime<Utc>>,
}

/// Type of custom model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomModelType {
    TextGeneration,
    TextClassification,
    TokenClassification,
    Seq2Seq,
    Unknown,
}

/// Import result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub model_id: Option<String>,
    pub validation_result: ValidationResult,
    pub error: Option<String>,
}

/// Import options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub name: String,
    pub description: String,
    pub model_type: CustomModelType,
    pub tags: Vec<String>,
    pub validate_thoroughly: bool,
    pub auto_generate_metadata: bool,
}

impl CustomModelManager {
    pub fn new(
        registry: Arc<ModelRegistry>,
        storage_path: PathBuf,
    ) -> Result<Self, String> {
        // Create storage directory if it doesn't exist
        std::fs::create_dir_all(&storage_path)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        
        let validator = Arc::new(OnnxValidator::new()
            .map_err(|e| format!("Failed to create validator: {}", e))?);
        
        let custom_models = Arc::new(RwLock::new(HashMap::new()));
        
        // Load existing custom models
        let manager = Arc::new(Self {
            registry,
            validator,
            custom_models: custom_models.clone(),
            storage_path: storage_path.clone(),
        });
        
        // Load persisted custom models
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            if let Err(e) = manager_clone.load_custom_models().await {
                log::error!("Failed to load custom models: {}", e);
            }
        });
        
        // Return the manager, extracting it from the Arc
        Arc::try_unwrap(manager)
            .map_err(|_| "Failed to extract manager from Arc".to_string())
    }
    
    /// Import a custom ONNX model
    pub async fn import_model(
        &self,
        model_path: &Path,
        options: ImportOptions,
    ) -> ImportResult {
        // Validate the model
        let validation_result = if options.validate_thoroughly {
            self.validator.validate_for_use_case(
                model_path,
                options.model_type.into(),
            )
        } else {
            self.validator.validate_model(model_path)
        };
        
        if !validation_result.is_valid {
            return ImportResult {
                success: false,
                model_id: None,
                validation_result,
                error: Some("Model validation failed".to_string()),
            };
        }
        
        // Generate model ID
        let model_id = format!("custom_{}", Uuid::new_v4());
        
        // Create metadata
        let metadata = if options.auto_generate_metadata {
            self.generate_metadata(
                &model_id,
                &options.name,
                &validation_result,
                options.model_type,
            ).await
        } else {
            self.create_basic_metadata(&model_id, &options.name, &validation_result)
        };
        
        // Copy model to storage
        let storage_filename = format!("{}.onnx", model_id);
        let storage_path = self.storage_path.join(&storage_filename);
        
        match std::fs::copy(model_path, &storage_path) {
            Ok(_) => {}
            Err(e) => {
                return ImportResult {
                    success: false,
                    model_id: None,
                    validation_result,
                    error: Some(format!("Failed to copy model: {}", e)),
                };
            }
        }
        
        // Create custom model entry
        let custom_model = CustomModel {
            id: model_id.clone(),
            name: options.name.clone(),
            original_filename: model_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.onnx")
                .to_string(),
            description: options.description,
            model_type: options.model_type,
            metadata: metadata.clone(),
            validation_result: validation_result.clone(),
            import_date: Utc::now(),
            tags: options.tags,
            is_active: true,
            usage_count: 0,
            last_used: None,
        };
        
        // Store custom model
        self.custom_models.write().await.insert(model_id.clone(), custom_model.clone());
        
        // Register with model registry
        if let Err(e) = self.registry.register_custom_model(metadata).await {
            log::error!("Failed to register custom model: {}", e);
        }
        
        // Persist custom models
        if let Err(e) = self.save_custom_models().await {
            log::error!("Failed to save custom models: {}", e);
        }
        
        ImportResult {
            success: true,
            model_id: Some(model_id),
            validation_result,
            error: None,
        }
    }
    
    /// Generate comprehensive metadata for the model
    async fn generate_metadata(
        &self,
        model_id: &str,
        name: &str,
        validation_result: &ValidationResult,
        model_type: CustomModelType,
    ) -> ModelMetadata {
        let model_info = validation_result.model_info.as_ref().unwrap();
        let compatibility = &validation_result.compatibility;
        
        // Detect capabilities based on model type and structure
        let capabilities = self.detect_capabilities(model_info, model_type);
        
        // Determine performance class based on size and parameters
        let performance_class = if model_info.estimated_parameters < 100_000_000 {
            "fast"
        } else if model_info.estimated_parameters < 1_000_000_000 {
            "balanced"
        } else {
            "quality"
        };
        
        // Estimate context window (for text models)
        let context_window = self.estimate_context_window(model_info, model_type);
        
        ModelMetadata {
            id: model_id.to_string(),
            name: name.to_string(),
            display_name: name.to_string(),
            description: format!("Custom {} model", model_type),
            size_bytes: model_info.model_size_bytes,
            format: crate::ai::models::ModelFormat::ONNX,
            source: crate::ai::models::ModelSource::Local { path: self.storage_path.join(format!("{}.onnx", model_id)) },
            capabilities: capabilities.into_iter()
                .filter_map(|cap| match cap.as_str() {
                    "grammar_correction" | "text_generation" => Some(crate::ai::models::Capability::GrammarCorrection),
                    "punctuation" => Some(crate::ai::models::Capability::Punctuation),
                    "sentiment_analysis" | "text_classification" => Some(crate::ai::models::Capability::IntentDetection),
                    "translation" => Some(crate::ai::models::Capability::Translation),
                    "summarization" => Some(crate::ai::models::Capability::Summarization),
                    _ => None,
                })
                .collect(),
            performance: crate::ai::models::PerformanceProfile {
                avg_latency_ms: 100.0, // Default, will be updated after benchmarking
                tokens_per_second: 50.0, // Default
                memory_usage_mb: (model_info.model_size_bytes / (1024 * 1024)) as u64,
                supports_batch: true,
                max_batch_size: 8,
            },
            requirements: crate::ai::models::ModelRequirements {
                min_ram_gb: (model_info.model_size_bytes as f32 / 1_073_741_824.0) * 2.0, // 2x model size
                min_vram_gb: if compatibility.requires_gpu { 
                    Some((model_info.model_size_bytes as f32 / 1_073_741_824.0) * 1.5)
                } else { 
                    None 
                },
                supports_cpu: !compatibility.requires_gpu,
                supports_gpu: true,
                supported_backends: vec!["ONNX".to_string()],
            },
            architecture: model_type.to_string(),
            parameters: "Unknown".to_string(),
            performance_class: "Custom".to_string(),
            context_window,
            supports_gpu: true,
            download_url: None,
            sha256: None,
        }
    }
    
    /// Create basic metadata
    fn create_basic_metadata(
        &self,
        model_id: &str,
        name: &str,
        validation_result: &ValidationResult,
    ) -> ModelMetadata {
        let model_info = validation_result.model_info.as_ref().unwrap();
        
        ModelMetadata {
            id: model_id.to_string(),
            name: name.to_string(),
            display_name: name.to_string(),
            description: "Custom AI model".to_string(),
            size_bytes: model_info.model_size_bytes,
            format: crate::ai::models::ModelFormat::ONNX,
            source: crate::ai::models::ModelSource::Local { path: self.storage_path.join(format!("{}.onnx", model_id)) },
            capabilities: vec![crate::ai::models::Capability::GrammarCorrection], // Default capability
            performance: crate::ai::models::PerformanceProfile::default(),
            requirements: crate::ai::models::ModelRequirements {
                min_ram_gb: (model_info.model_size_bytes as f32 / 1_073_741_824.0) * 2.0,
                min_vram_gb: None,
                supports_cpu: true,
                supports_gpu: true,
                supported_backends: vec!["ONNX".to_string()],
            },
            architecture: "Unknown".to_string(),
            parameters: "Unknown".to_string(),
            performance_class: "Custom".to_string(),
            context_window: 512, // Default
            supports_gpu: true,
            download_url: None,
            sha256: None,
        }
    }
    
    /// Detect model capabilities
    fn detect_capabilities(
        &self,
        model_info: &ValidatorModelInfo,
        model_type: CustomModelType,
    ) -> Vec<String> {
        let mut capabilities = Vec::new();
        
        // Add base capability
        capabilities.push("custom".to_string());
        
        // Type-specific capabilities
        match model_type {
            CustomModelType::TextGeneration => {
                capabilities.push("text_generation".to_string());
                capabilities.push("completion".to_string());
            }
            CustomModelType::TextClassification => {
                capabilities.push("text_classification".to_string());
                capabilities.push("sentiment_analysis".to_string());
            }
            CustomModelType::TokenClassification => {
                capabilities.push("token_classification".to_string());
                capabilities.push("ner".to_string());
            }
            CustomModelType::Seq2Seq => {
                capabilities.push("translation".to_string());
                capabilities.push("summarization".to_string());
            }
            CustomModelType::Unknown => {}
        }
        
        // Detect based on input/output structure
        if model_info.input_names.iter().any(|n| n.contains("mask")) {
            capabilities.push("masked_attention".to_string());
        }
        
        if model_info.output_names.len() > 1 {
            capabilities.push("multi_output".to_string());
        }
        
        capabilities
    }
    
    /// Estimate context window
    fn estimate_context_window(
        &self,
        model_info: &ValidatorModelInfo,
        model_type: CustomModelType,
    ) -> usize {
        // Look for sequence length in input shapes
        for (name, shape) in &model_info.input_shapes {
            if name.contains("input") || name.contains("ids") {
                // Usually the second dimension is sequence length
                if shape.len() >= 2 && shape[1] > 0 {
                    return shape[1] as usize;
                }
            }
        }
        
        // Default based on model type
        match model_type {
            CustomModelType::TextGeneration => 2048,
            CustomModelType::Seq2Seq => 512,
            _ => 512,
        }
    }
    
    /// Format parameter count
    fn format_parameters(&self, count: u64) -> String {
        if count >= 1_000_000_000 {
            format!("{:.1}B", count as f64 / 1_000_000_000.0)
        } else if count >= 1_000_000 {
            format!("{:.1}M", count as f64 / 1_000_000.0)
        } else if count >= 1_000 {
            format!("{:.1}K", count as f64 / 1_000.0)
        } else {
            count.to_string()
        }
    }
    
    /// List all custom models
    pub async fn list_custom_models(&self) -> Vec<CustomModel> {
        self.custom_models.read().await.values().cloned().collect()
    }
    
    /// Get a specific custom model
    pub async fn get_custom_model(&self, model_id: &str) -> Option<CustomModel> {
        self.custom_models.read().await.get(model_id).cloned()
    }
    
    /// Update custom model metadata
    pub async fn update_model(
        &self,
        model_id: &str,
        updates: ModelUpdateRequest,
    ) -> Result<(), String> {
        let mut models = self.custom_models.write().await;
        
        if let Some(model) = models.get_mut(model_id) {
            if let Some(name) = updates.name {
                model.name = name.clone();
                model.metadata.name = name.clone();
                model.metadata.display_name = name;
            }
            
            if let Some(description) = updates.description {
                model.description = description.clone();
                model.metadata.description = description;
            }
            
            if let Some(tags) = updates.tags {
                model.tags = tags;
            }
            
            if let Some(capabilities) = updates.capabilities {
                // Convert string capabilities to enum variants
                model.metadata.capabilities = capabilities.into_iter()
                    .filter_map(|cap| match cap.as_str() {
                        "grammar_correction" => Some(crate::ai::models::Capability::GrammarCorrection),
                        "punctuation" => Some(crate::ai::models::Capability::Punctuation),
                        "intent_detection" => Some(crate::ai::models::Capability::IntentDetection),
                        "style_transformation" => Some(crate::ai::models::Capability::StyleTransformation),
                        "summarization" => Some(crate::ai::models::Capability::Summarization),
                        "translation" => Some(crate::ai::models::Capability::Translation),
                        "question_answering" => Some(crate::ai::models::Capability::QuestionAnswering),
                        _ => None,
                    })
                    .collect();
            }
            
            // Note: supported_languages is not part of ModelMetadata structure
            
            // Update in registry
            self.registry.update_model_metadata(&model_id, model.metadata.clone()).await
                .map_err(|e| e.to_string())?;
            
            // Save changes
            drop(models);
            self.save_custom_models().await?;
            
            Ok(())
        } else {
            Err("Model not found".to_string())
        }
    }
    
    /// Delete a custom model
    pub async fn delete_model(&self, model_id: &str) -> Result<(), String> {
        let mut models = self.custom_models.write().await;
        
        if let Some(model) = models.remove(model_id) {
            // Remove from registry
            self.registry.remove_custom_model(model_id).await
                .map_err(|e| e.to_string())?;
            
            // Delete model file
            let model_path = self.storage_path.join(format!("{}.onnx", model_id));
            if model_path.exists() {
                std::fs::remove_file(model_path)
                    .map_err(|e| format!("Failed to delete model file: {}", e))?;
            }
            
            // Save changes
            drop(models);
            self.save_custom_models().await?;
            
            Ok(())
        } else {
            Err("Model not found".to_string())
        }
    }
    
    /// Export a custom model
    pub async fn export_model(
        &self,
        model_id: &str,
        export_path: &Path,
    ) -> Result<(), String> {
        let models = self.custom_models.read().await;
        
        if let Some(model) = models.get(model_id) {
            let model_path = self.storage_path.join(format!("{}.onnx", model_id));
            
            if !model_path.exists() {
                return Err("Model file not found".to_string());
            }
            
            // Copy model file
            std::fs::copy(&model_path, export_path)
                .map_err(|e| format!("Failed to export model: {}", e))?;
            
            // Export metadata
            let metadata_path = export_path.with_extension("json");
            let metadata_json = serde_json::to_string_pretty(&model)
                .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
            
            std::fs::write(metadata_path, metadata_json)
                .map_err(|e| format!("Failed to write metadata: {}", e))?;
            
            Ok(())
        } else {
            Err("Model not found".to_string())
        }
    }
    
    /// Record model usage
    pub async fn record_usage(&self, model_id: &str) -> Result<(), String> {
        let mut models = self.custom_models.write().await;
        
        if let Some(model) = models.get_mut(model_id) {
            model.usage_count += 1;
            model.last_used = Some(Utc::now());
            
            drop(models);
            self.save_custom_models().await?;
            
            Ok(())
        } else {
            Err("Model not found".to_string())
        }
    }
    
    /// Load custom models from disk
    async fn load_custom_models(&self) -> Result<(), String> {
        let metadata_path = self.storage_path.join("custom_models.json");
        
        if !metadata_path.exists() {
            return Ok(());
        }
        
        let data = tokio::fs::read_to_string(&metadata_path).await
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        
        let models: HashMap<String, CustomModel> = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse metadata: {}", e))?;
        
        *self.custom_models.write().await = models;
        
        Ok(())
    }
    
    /// Save custom models to disk
    async fn save_custom_models(&self) -> Result<(), String> {
        let metadata_path = self.storage_path.join("custom_models.json");
        let models = self.custom_models.read().await;
        
        let data = serde_json::to_string_pretty(&*models)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
        
        tokio::fs::write(&metadata_path, data).await
            .map_err(|e| format!("Failed to write metadata: {}", e))?;
        
        Ok(())
    }
}

/// Model update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub capabilities: Option<Vec<String>>,
    pub supported_languages: Option<Vec<String>>,
}

impl From<CustomModelType> for crate::ai::services::onnx_validator::ModelUseCase {
    fn from(model_type: CustomModelType) -> Self {
        use crate::ai::services::onnx_validator::ModelUseCase;
        
        match model_type {
            CustomModelType::TextGeneration => ModelUseCase::TextGeneration,
            CustomModelType::TextClassification => ModelUseCase::TextClassification,
            CustomModelType::TokenClassification => ModelUseCase::TokenClassification,
            CustomModelType::Seq2Seq => ModelUseCase::Seq2Seq,
            CustomModelType::Unknown => ModelUseCase::TextGeneration, // Default
        }
    }
}

impl std::fmt::Display for CustomModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomModelType::TextGeneration => write!(f, "text-generation"),
            CustomModelType::TextClassification => write!(f, "text-classification"),
            CustomModelType::TokenClassification => write!(f, "token-classification"),
            CustomModelType::Seq2Seq => write!(f, "seq2seq"),
            CustomModelType::Unknown => write!(f, "unknown"),
        }
    }
}