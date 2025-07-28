use crate::ai::{Result, AIError};
use crate::ai::common::{ModelInfo, QuantizationType};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ModelManager {
    models_dir: std::path::PathBuf,
    download_progress: Arc<RwLock<HashMap<String, f32>>>,
}

use std::collections::HashMap;

impl ModelManager {
    pub fn new() -> Result<Self> {
        let models_dir = crate::ai::common::ensure_models_directory()
            .map_err(|e| AIError::ConfigError(e.to_string()))?;
        
        Ok(Self {
            models_dir,
            download_progress: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn download_model(&self, model_info: &ModelInfo) -> Result<std::path::PathBuf> {
        let model_path = self.models_dir.join(&model_info.name).with_extension("onnx");
        
        if model_path.exists() {
            log::info!("Model {} already exists at {:?}", model_info.name, model_path);
            return Ok(model_path);
        }

        log::info!("Downloading model {} ({} bytes)", model_info.name, model_info.size_bytes);
        
        // Set initial progress
        {
            let mut progress = self.download_progress.write().await;
            progress.insert(model_info.name.clone(), 0.0);
        }

        // Model-specific download URLs
        let model_url = match model_info.name.as_str() {
            "phi-3-mini" => {
                // Microsoft Phi-3-mini ONNX model
                "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-onnx/resolve/main/onnx/model.onnx"
            }
            "llama-3.2-1b" => {
                // Meta Llama 3.2 1B (we'll need to convert this)
                "https://huggingface.co/meta-llama/Llama-3.2-1B/resolve/main/model.safetensors"
            }
            _ => {
                return Err(AIError::ModelNotFound(
                    format!("Unknown model: {}", model_info.name)
                ));
            }
        };

        // Create a temporary file for download
        let temp_path = model_path.with_extension("tmp");
        
        // Download with progress tracking
        match self.download_file(model_url, &temp_path, &model_info.name).await {
            Ok(_) => {
                // Move to final location
                std::fs::rename(&temp_path, &model_path)
                    .map_err(|e| AIError::ConfigError(format!("Failed to move model file: {}", e)))?;
            }
            Err(e) => {
                // Clean up temp file if it exists
                let _ = std::fs::remove_file(&temp_path);
                return Err(e);
            }
        }

        // Remove from progress tracking
        {
            let mut progress = self.download_progress.write().await;
            progress.remove(&model_info.name);
        }

        log::info!("Model {} downloaded successfully", model_info.name);
        Ok(model_path)
    }

    async fn download_file(
        &self,
        url: &str,
        path: &Path,
        model_name: &str,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        use futures::StreamExt;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AIError::ConfigError(format!("Failed to create directory: {}", e)))?;
        }
        
        // Create HTTP client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout
            .build()
            .map_err(|e| AIError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;
        
        // Start download
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(format!("Failed to start download: {}", e)))?;
        
        // Check response status
        if !response.status().is_success() {
            return Err(AIError::NetworkError(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }
        
        // Get content length for progress tracking
        let total_size = response
            .content_length()
            .unwrap_or(0);
        
        // Create file
        let file = tokio::fs::File::create(path)
            .await
            .map_err(|e| AIError::ConfigError(format!("Failed to create file: {}", e)))?;
        
        let mut writer = tokio::io::BufWriter::new(file);
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        // Download with progress tracking
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| AIError::NetworkError(format!("Download error: {}", e)))?;
            
            writer.write_all(&chunk).await
                .map_err(|e| AIError::ConfigError(format!("Failed to write chunk: {}", e)))?;
            
            downloaded += chunk.len() as u64;
            
            // Update progress
            if total_size > 0 {
                let progress = (downloaded as f32 / total_size as f32) * 100.0;
                let mut progress_map = self.download_progress.write().await;
                progress_map.insert(model_name.to_string(), progress);
                
                log::info!("Download progress for {}: {:.1}%", model_name, progress);
            }
        }
        
        // Flush writer
        writer.flush().await
            .map_err(|e| AIError::ConfigError(format!("Failed to flush file: {}", e)))?;
        
        Ok(())
    }

    pub async fn get_download_progress(&self, model_name: &str) -> Option<f32> {
        let progress = self.download_progress.read().await;
        progress.get(model_name).copied()
    }

    pub fn list_downloaded_models(&self) -> Result<Vec<String>> {
        let mut models = Vec::new();
        
        let entries = std::fs::read_dir(&self.models_dir)
            .map_err(|e| AIError::ConfigError(e.to_string()))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| AIError::ConfigError(e.to_string()))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("onnx") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    models.push(name.to_string());
                }
            }
        }
        
        Ok(models)
    }

    pub fn get_model_path(&self, model_name: &str) -> std::path::PathBuf {
        self.models_dir.join(model_name).with_extension("onnx")
    }
}

pub fn apply_quantization(
    model_path: &Path,
    quantization: QuantizationType,
) -> Result<std::path::PathBuf> {
    let quantized_path = model_path.with_extension(format!("{:?}.onnx", quantization).to_lowercase());
    
    if quantized_path.exists() {
        return Ok(quantized_path);
    }

    // TODO: Implement actual quantization
    // This would involve:
    // 1. Loading the model
    // 2. Applying quantization (Int8, Int4, or FP16)
    // 3. Saving the quantized model
    
    Err(AIError::ConfigError(
        format!("Quantization {:?} not yet implemented", quantization)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_manager_creation() {
        let manager = ModelManager::new();
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_list_models() {
        let manager = ModelManager::new().unwrap();
        let models = manager.list_downloaded_models();
        assert!(models.is_ok());
    }
}