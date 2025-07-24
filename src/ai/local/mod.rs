pub mod model;
pub mod inference;
pub mod enhancement;
pub mod onnx_models;
pub mod onnx_runtime;
pub mod model_optimizer;
pub mod streaming_inference;
pub mod batch_processor;
pub mod model_warmup;

use crate::ai::{AIProvider, EnhancementOptions, EnhancedText, Result, AIError};
use crate::ai::common::{ModelConfig, ModelInfo};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct LocalAI {
    config: ModelConfig,
    model: Arc<RwLock<Option<Box<dyn LocalModel>>>>,
    processor: Arc<dyn crate::ai::common::TextProcessor>,
}

impl LocalAI {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            model: Arc::new(RwLock::new(None)),
            processor: Arc::new(crate::ai::common::StandardTextProcessor),
        }
    }

    pub async fn load_model(&self, model_name: &str) -> Result<()> {
        let model_info = crate::ai::common::find_model_info(model_name)
            .ok_or_else(|| AIError::ModelNotFound(model_name.to_string()))?;

        // Check if model file exists, download if necessary
        let model_path = self.ensure_model_downloaded(model_info).await?;
        
        // Load the model based on the backend
        let model = self.create_model_instance(&model_path, model_info).await?;
        
        let mut model_guard = self.model.write().await;
        *model_guard = Some(model);
        
        Ok(())
    }

    async fn ensure_model_downloaded(&self, model_info: &ModelInfo) -> Result<std::path::PathBuf> {
        let models_dir = crate::ai::common::ensure_models_directory()
            .map_err(|e| AIError::ConfigError(e.to_string()))?;
        
        let model_path = models_dir.join(&model_info.name).with_extension("onnx");
        
        if !model_path.exists() {
            // TODO: Implement model downloading
            return Err(AIError::ModelNotFound(
                format!("Model {} not found at {:?}. Download not implemented yet.", 
                    model_info.name, model_path)
            ));
        }
        
        Ok(model_path)
    }

    async fn create_model_instance(
        &self,
        model_path: &std::path::Path,
        model_info: &ModelInfo,
    ) -> Result<Box<dyn LocalModel>> {
        // Load ONNX model based on the model type
        if model_info.name.contains("t5") || model_info.name.contains("T5") {
            let model = onnx_models::T5GrammarModel::new(model_path, model_info.clone()).await?;
            Ok(Box::new(model))
        } else {
            let model = onnx_models::OnnxGrammarModel::load(model_path, model_info.clone()).await?;
            Ok(Box::new(model))
        }
    }
}

#[async_trait::async_trait]
impl AIProvider for LocalAI {
    fn name(&self) -> &str {
        "Local AI"
    }

    fn is_available(&self) -> bool {
        true // Local AI is always available
    }

    async fn initialize(&mut self) -> Result<()> {
        // Load default model if configured
        if let Some(model_name) = std::env::var("BESTME_LOCAL_MODEL").ok() {
            self.load_model(&model_name).await?;
        }
        Ok(())
    }

    async fn enhance_text(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Result<EnhancedText> {
        let model_guard = self.model.read().await;
        let model = model_guard.as_ref()
            .ok_or_else(|| AIError::ConfigError("No model loaded".to_string()))?;

        // Preprocess text
        let preprocessed = self.processor.preprocess(text);
        
        // Run inference
        let enhanced = model.infer(&preprocessed, options).await?;
        
        // Postprocess
        let final_text = self.processor.postprocess(&enhanced.enhanced);
        
        Ok(EnhancedText {
            enhanced: final_text,
            ..enhanced
        })
    }
}

#[async_trait::async_trait]
pub trait LocalModel: Send + Sync {
    async fn infer(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Result<EnhancedText>;
    
    fn model_info(&self) -> &ModelInfo;
}