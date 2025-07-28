use super::{ModelMetadata, ModelSource, get_phi3_mini_metadata, get_llama32_1b_metadata, get_grammar_t5_metadata};
use crate::ai::{Result, AIError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<String, ModelMetadata>>>,
    download_manager: Arc<ModelDownloadManager>,
    models_dir: PathBuf,
}

#[derive(Debug)]
pub struct ModelDownloadManager {
    active_downloads: Arc<RwLock<HashMap<String, DownloadProgress>>>,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub percentage: f32,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Converting,
    Completed,
    Failed,
}

impl ModelRegistry {
    pub async fn new() -> Result<Self> {
        let models_dir = crate::ai::common::ensure_models_directory()
            .map_err(|e| AIError::ConfigError(e.to_string()))?;

        let mut models = HashMap::new();
        
        // Register pre-defined models
        let phi3 = get_phi3_mini_metadata();
        models.insert(phi3.id.clone(), phi3);
        
        let llama = get_llama32_1b_metadata();
        models.insert(llama.id.clone(), llama);
        
        let grammar = get_grammar_t5_metadata();
        models.insert(grammar.id.clone(), grammar);

        let download_manager = Arc::new(ModelDownloadManager::new()?);

        Ok(Self {
            models: Arc::new(RwLock::new(models)),
            download_manager,
            models_dir,
        })
    }

    pub async fn get_metadata(&self, model_id: &str) -> Result<ModelMetadata> {
        let models = self.models.read().await;
        models.get(model_id)
            .cloned()
            .ok_or_else(|| AIError::ModelNotFound(format!("Model '{}' not found in registry", model_id)))
    }

    pub async fn list_available_models(&self) -> Vec<ModelMetadata> {
        let models = self.models.read().await;
        let mut model_list: Vec<_> = models.values().cloned().collect();
        model_list.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        model_list
    }

    pub async fn list_downloaded_models(&self) -> Result<Vec<String>> {
        let mut downloaded = Vec::new();
        
        if !self.models_dir.exists() {
            return Ok(downloaded);
        }

        let entries = std::fs::read_dir(&self.models_dir)
            .map_err(|e| AIError::ConfigError(format!("Failed to read models directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| AIError::ConfigError(e.to_string()))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("onnx") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    // Check if this model is in our registry
                    let models = self.models.read().await;
                    if models.values().any(|m| m.id == name || m.name.contains(name)) {
                        downloaded.push(name.to_string());
                    }
                }
            }
        }

        Ok(downloaded)
    }

    pub async fn download_model(&self, model_id: &str) -> Result<PathBuf> {
        let metadata = self.get_metadata(model_id).await?;
        let model_path = self.get_model_path(model_id);

        // Check if already downloaded
        if model_path.exists() {
            log::info!("Model {} already downloaded at {:?}", model_id, model_path);
            return Ok(model_path);
        }

        // Start download
        self.download_manager.download_model(&metadata, &self.models_dir).await
    }

    pub async fn get_download_progress(&self, model_id: &str) -> Option<DownloadProgress> {
        self.download_manager.get_progress(model_id).await
    }

    pub fn get_model_path(&self, model_id: &str) -> PathBuf {
        self.models_dir.join(model_id).with_extension("onnx")
    }

    pub async fn register_custom_model(&self, metadata: ModelMetadata) -> Result<()> {
        let mut models = self.models.write().await;
        if models.contains_key(&metadata.id) {
            return Err(AIError::ConfigError(format!("Model '{}' already registered", metadata.id)));
        }
        models.insert(metadata.id.clone(), metadata);
        Ok(())
    }

    // Alias for compatibility
    pub async fn get_model_metadata(&self, model_id: &str) -> Result<ModelMetadata> {
        self.get_metadata(model_id).await
    }

    pub async fn update_model_metadata(&self, model_id: &str, metadata: ModelMetadata) -> Result<()> {
        let mut models = self.models.write().await;
        models.insert(model_id.to_string(), metadata);
        Ok(())
    }

    pub async fn remove_custom_model(&self, model_id: &str) -> Result<()> {
        let mut models = self.models.write().await;
        models.remove(model_id)
            .ok_or_else(|| AIError::ModelNotFound(format!("Model '{}' not found", model_id)))?;
        Ok(())
    }
}

impl ModelDownloadManager {
    pub fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(600)) // 10 minute timeout
            .user_agent("BestMe/1.0")
            .build()
            .map_err(|e| AIError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            http_client,
        })
    }

    pub async fn download_model(&self, metadata: &ModelMetadata, models_dir: &Path) -> Result<PathBuf> {
        let model_path = models_dir.join(&metadata.id).with_extension("onnx");
        
        // Initialize progress
        let progress = DownloadProgress {
            model_id: metadata.id.clone(),
            total_bytes: metadata.size_bytes,
            downloaded_bytes: 0,
            percentage: 0.0,
            status: DownloadStatus::Pending,
        };
        
        {
            let mut downloads = self.active_downloads.write().await;
            downloads.insert(metadata.id.clone(), progress);
        }

        let result = match &metadata.source {
            ModelSource::HuggingFace { repo, file } => {
                self.download_from_huggingface(repo, file, &model_path, &metadata.id).await
            }
            ModelSource::Direct { url } => {
                self.download_from_url(url, &model_path, &metadata.id).await
            }
            ModelSource::Local { path } => {
                // Copy local file
                tokio::fs::copy(path, &model_path).await
                    .map_err(|e| AIError::ConfigError(format!("Failed to copy local model: {}", e)))?;
                Ok(model_path)
            }
        };

        // Update status based on result
        {
            let mut downloads = self.active_downloads.write().await;
            if let Some(progress) = downloads.get_mut(&metadata.id) {
                progress.status = if result.is_ok() {
                    DownloadStatus::Completed
                } else {
                    DownloadStatus::Failed
                };
            }
        }

        // Remove from active downloads after a delay
        let model_id = metadata.id.clone();
        let downloads = self.active_downloads.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let mut downloads = downloads.write().await;
            downloads.remove(&model_id);
        });

        result
    }

    async fn download_from_huggingface(
        &self,
        repo: &str,
        file: &str,
        output_path: &Path,
        model_id: &str,
    ) -> Result<PathBuf> {
        // Construct Hugging Face URL
        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            repo, file
        );

        self.download_from_url(&url, output_path, model_id).await
    }

    async fn download_from_url(
        &self,
        url: &str,
        output_path: &Path,
        model_id: &str,
    ) -> Result<PathBuf> {
        use futures::StreamExt;
        use tokio::io::AsyncWriteExt;

        log::info!("Downloading model from: {}", url);

        // Update status to downloading
        {
            let mut downloads = self.active_downloads.write().await;
            if let Some(progress) = downloads.get_mut(model_id) {
                progress.status = DownloadStatus::Downloading;
            }
        }

        // Create parent directory
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| AIError::ConfigError(format!("Failed to create directory: {}", e)))?;
        }

        // Start download
        let response = self.http_client
            .get(url)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(format!("Failed to start download: {}", e)))?;

        if !response.status().is_success() {
            return Err(AIError::NetworkError(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);

        // Create temporary file
        let temp_path = output_path.with_extension("tmp");
        let file = tokio::fs::File::create(&temp_path).await
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
                let percentage = (downloaded as f32 / total_size as f32) * 100.0;
                let mut downloads = self.active_downloads.write().await;
                if let Some(progress) = downloads.get_mut(model_id) {
                    progress.downloaded_bytes = downloaded;
                    progress.percentage = percentage;
                    
                    if downloaded % (1024 * 1024 * 10) == 0 { // Log every 10MB
                        log::info!("Download progress for {}: {:.1}%", model_id, percentage);
                    }
                }
            }
        }

        writer.flush().await
            .map_err(|e| AIError::ConfigError(format!("Failed to flush file: {}", e)))?;

        // Move to final location
        tokio::fs::rename(&temp_path, output_path).await
            .map_err(|e| AIError::ConfigError(format!("Failed to move file: {}", e)))?;

        log::info!("Model {} downloaded successfully", model_id);
        Ok(output_path.to_path_buf())
    }

    pub async fn get_progress(&self, model_id: &str) -> Option<DownloadProgress> {
        let downloads = self.active_downloads.read().await;
        downloads.get(model_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ModelRegistry::new().await;
        assert!(registry.is_ok());
    }

    #[tokio::test]
    async fn test_list_available_models() {
        let registry = ModelRegistry::new().await.unwrap();
        let models = registry.list_available_models().await;
        assert!(models.len() >= 3); // Should have at least the pre-defined models
        
        // Check for specific models
        assert!(models.iter().any(|m| m.id == "phi-3-mini"));
        assert!(models.iter().any(|m| m.id == "llama-3.2-1b"));
        assert!(models.iter().any(|m| m.id == "grammar-t5-base"));
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let registry = ModelRegistry::new().await.unwrap();
        
        let phi3 = registry.get_metadata("phi-3-mini").await;
        assert!(phi3.is_ok());
        let phi3 = phi3.unwrap();
        assert_eq!(phi3.id, "phi-3-mini");
        // Check capabilities exist
        assert!(!phi3.capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_model_path() {
        let registry = ModelRegistry::new().await.unwrap();
        let path = registry.get_model_path("phi-3-mini");
        assert!(path.to_string_lossy().contains("phi-3-mini.onnx"));
    }
}