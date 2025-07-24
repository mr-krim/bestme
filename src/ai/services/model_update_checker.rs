use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use semver::Version;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::ai::models::registry::ModelRegistry;
use crate::ai::models::ModelMetadata;
use crate::ai::services::ModelService;

/// Service for checking and managing model updates
pub struct ModelUpdateChecker {
    model_service: Arc<ModelService>,
    update_cache: Arc<RwLock<UpdateCache>>,
    config: UpdateCheckerConfig,
}

/// Configuration for update checking
#[derive(Debug, Clone)]
pub struct UpdateCheckerConfig {
    /// Check for updates automatically
    pub auto_check_enabled: bool,
    /// Interval between update checks (in seconds)
    pub check_interval_seconds: u64,
    /// Auto-download updates when available
    pub auto_download: bool,
    /// Only download on WiFi/unmetered connections
    pub wifi_only: bool,
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: usize,
}

impl Default for UpdateCheckerConfig {
    fn default() -> Self {
        Self {
            auto_check_enabled: true,
            check_interval_seconds: 86400, // 24 hours
            auto_download: false,
            wifi_only: true,
            max_concurrent_downloads: 2,
        }
    }
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUpdate {
    pub model_id: String,
    pub current_version: String,
    pub new_version: String,
    pub release_date: DateTime<Utc>,
    pub size_bytes: u64,
    pub changelog: String,
    pub improvements: Vec<String>,
    pub download_url: String,
    pub sha256: String,
    pub is_critical: bool,
    pub compatibility_notes: Option<String>,
}

/// Update check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub last_check: DateTime<Utc>,
    pub available_updates: Vec<ModelUpdate>,
    pub failed_checks: Vec<(String, String)>, // (model_id, error)
}

/// Cache for update information
struct UpdateCache {
    results: HashMap<String, UpdateCheckResult>,
    last_full_check: Option<Instant>,
}

impl ModelUpdateChecker {
    pub fn new(
        model_service: Arc<ModelService>,
        config: UpdateCheckerConfig,
    ) -> Self {
        let cache = Arc::new(RwLock::new(UpdateCache {
            results: HashMap::new(),
            last_full_check: None,
        }));
        
        Self {
            model_service,
            update_cache: cache,
            config,
        }
    }
    
    /// Check for updates for all installed models
    pub async fn check_all_updates(&self, force: bool) -> Result<UpdateCheckResult, String> {
        let mut cache = self.update_cache.write().await;
        
        // Check if we should use cached results
        if !force {
            if let Some(last_check) = cache.last_full_check {
                let elapsed = last_check.elapsed();
                if elapsed < Duration::from_secs(self.config.check_interval_seconds) {
                    // Return cached results
                    let mut combined_result = UpdateCheckResult {
                        last_check: Utc::now(),
                        available_updates: Vec::new(),
                        failed_checks: Vec::new(),
                    };
                    
                    for result in cache.results.values() {
                        combined_result.available_updates.extend(result.available_updates.clone());
                        combined_result.failed_checks.extend(result.failed_checks.clone());
                    }
                    
                    return Ok(combined_result);
                }
            }
        }
        
        // Perform fresh check
        let downloaded_models = self.model_service.get_registry()
            .list_downloaded_models()
            .await
            .map_err(|e| format!("Failed to list models: {}", e))?;
        
        let mut available_updates = Vec::new();
        let mut failed_checks = Vec::new();
        
        for model_id in downloaded_models {
            match self.check_model_update(&model_id).await {
                Ok(Some(update)) => {
                    available_updates.push(update);
                }
                Ok(None) => {
                    // No update available
                }
                Err(e) => {
                    failed_checks.push((model_id.clone(), e));
                }
            }
        }
        
        let result = UpdateCheckResult {
            last_check: Utc::now(),
            available_updates,
            failed_checks,
        };
        
        // Update cache
        cache.last_full_check = Some(Instant::now());
        for update in &result.available_updates {
            cache.results.insert(
                update.model_id.clone(),
                UpdateCheckResult {
                    last_check: result.last_check,
                    available_updates: vec![update.clone()],
                    failed_checks: vec![],
                },
            );
        }
        
        Ok(result)
    }
    
    /// Check for updates for a specific model
    pub async fn check_model_update(&self, model_id: &str) -> Result<Option<ModelUpdate>, String> {
        // Get current model metadata
        let current_metadata = self.model_service.get_registry()
            .get_model_metadata(model_id)
            .await
            .map_err(|e| format!("Failed to get model metadata: {}", e))?
            .ok_or_else(|| format!("Model {} not found", model_id))?;
        
        // Get latest metadata from registry (simulated - in production, this would query a remote registry)
        let latest_metadata = self.fetch_latest_metadata(model_id).await?;
        
        // Compare versions
        if let Some(update) = self.compare_versions(&current_metadata, &latest_metadata)? {
            Ok(Some(update))
        } else {
            Ok(None)
        }
    }
    
    /// Compare model versions
    fn compare_versions(
        &self,
        current: &ModelMetadata,
        latest: &ModelMetadata,
    ) -> Result<Option<ModelUpdate>, String> {
        // Extract version from model metadata
        let current_version = self.extract_version(&current.id)?;
        let latest_version = self.extract_version(&latest.id)?;
        
        // Compare versions
        if latest_version > current_version {
            let update = ModelUpdate {
                model_id: current.id.clone(),
                current_version: current_version.to_string(),
                new_version: latest_version.to_string(),
                release_date: Utc::now(), // Would come from registry
                size_bytes: latest.size_bytes,
                changelog: self.generate_changelog(&current_version, &latest_version),
                improvements: self.detect_improvements(current, latest),
                download_url: latest.download_url.clone().unwrap_or_default(),
                sha256: latest.sha256.clone().unwrap_or_default(),
                is_critical: self.is_critical_update(&current_version, &latest_version),
                compatibility_notes: self.check_compatibility(current, latest),
            };
            
            Ok(Some(update))
        } else {
            Ok(None)
        }
    }
    
    /// Extract version from model ID or metadata
    fn extract_version(&self, model_id: &str) -> Result<Version, String> {
        // Try to extract version from model ID
        // Format: "model-name-v1.2.3" or "model-name-1.2.3"
        let parts: Vec<&str> = model_id.split('-').collect();
        
        // Look for version pattern
        for part in parts.iter().rev() {
            // Remove 'v' prefix if present
            let version_str = part.strip_prefix('v').unwrap_or(part);
            
            if let Ok(version) = Version::parse(version_str) {
                return Ok(version);
            }
        }
        
        // Default version if not found
        Ok(Version::new(1, 0, 0))
    }
    
    /// Generate changelog between versions
    fn generate_changelog(&self, current: &Version, latest: &Version) -> String {
        let mut changelog = String::new();
        
        if latest.major > current.major {
            changelog.push_str("Major update with significant improvements and new features.\n");
        } else if latest.minor > current.minor {
            changelog.push_str("Minor update with new features and enhancements.\n");
        } else {
            changelog.push_str("Patch update with bug fixes and performance improvements.\n");
        }
        
        // In production, this would fetch actual changelog from registry
        changelog.push_str("\nChanges:\n");
        changelog.push_str("- Improved accuracy and performance\n");
        changelog.push_str("- Bug fixes and stability improvements\n");
        changelog.push_str("- Updated model weights\n");
        
        changelog
    }
    
    /// Detect improvements between model versions
    fn detect_improvements(&self, current: &ModelMetadata, latest: &ModelMetadata) -> Vec<String> {
        let mut improvements = Vec::new();
        
        // Check parameter count
        if let (Ok(current_params), Ok(latest_params)) = (
            self.parse_parameters(&current.parameters),
            self.parse_parameters(&latest.parameters),
        ) {
            if latest_params > current_params {
                improvements.push(format!(
                    "Increased model capacity ({} → {} parameters)",
                    current.parameters, latest.parameters
                ));
            }
        }
        
        // Check new capabilities
        for capability in &latest.capabilities {
            if !current.capabilities.contains(capability) {
                improvements.push(format!("New capability: {}", capability));
            }
        }
        
        // Check context window
        if latest.context_window > current.context_window {
            improvements.push(format!(
                "Larger context window ({} → {} tokens)",
                current.context_window, latest.context_window
            ));
        }
        
        // Check performance class
        if latest.performance_class != current.performance_class {
            improvements.push(format!(
                "Performance class changed: {} → {}",
                current.performance_class, latest.performance_class
            ));
        }
        
        improvements
    }
    
    /// Parse parameter count from string
    fn parse_parameters(&self, params: &str) -> Result<u64, String> {
        // Parse formats like "7B", "13B", "1.5B", etc.
        let cleaned = params.to_lowercase().replace("parameters", "").trim().to_string();
        
        if let Some(b_pos) = cleaned.find('b') {
            let num_str = &cleaned[..b_pos];
            if let Ok(num) = num_str.parse::<f64>() {
                return Ok((num * 1_000_000_000.0) as u64);
            }
        } else if let Some(m_pos) = cleaned.find('m') {
            let num_str = &cleaned[..m_pos];
            if let Ok(num) = num_str.parse::<f64>() {
                return Ok((num * 1_000_000.0) as u64);
            }
        }
        
        Err(format!("Cannot parse parameters: {}", params))
    }
    
    /// Check if update is critical
    fn is_critical_update(&self, current: &Version, latest: &Version) -> bool {
        // Major version changes are considered critical
        latest.major > current.major
    }
    
    /// Check compatibility between versions
    fn check_compatibility(&self, current: &ModelMetadata, latest: &ModelMetadata) -> Option<String> {
        let mut notes = Vec::new();
        
        // Check architecture changes
        if current.architecture != latest.architecture {
            notes.push(format!(
                "Architecture changed from {} to {}",
                current.architecture, latest.architecture
            ));
        }
        
        // Check hardware requirements
        if latest.supports_gpu && !current.supports_gpu {
            notes.push("New version requires GPU support".to_string());
        }
        
        if notes.is_empty() {
            None
        } else {
            Some(notes.join(". "))
        }
    }
    
    /// Fetch latest metadata from registry (simulated)
    async fn fetch_latest_metadata(&self, model_id: &str) -> Result<ModelMetadata, String> {
        // In production, this would make an HTTP request to a model registry
        // For now, we'll simulate by modifying the current metadata
        
        let mut metadata = self.model_service.get_registry()
            .get_model_metadata(model_id)
            .await
            .map_err(|e| format!("Failed to get metadata: {}", e))?
            .ok_or_else(|| format!("Model {} not found", model_id))?;
        
        // Simulate a newer version
        if let Ok(version) = self.extract_version(&metadata.id) {
            let new_version = Version::new(version.major, version.minor + 1, 0);
            metadata.id = metadata.id.replace(&version.to_string(), &new_version.to_string());
        }
        
        Ok(metadata)
    }
    
    /// Auto-download updates
    pub async fn auto_download_updates(&self) -> Result<Vec<String>, String> {
        if !self.config.auto_download {
            return Ok(vec![]);
        }
        
        let update_result = self.check_all_updates(false).await?;
        let mut downloaded = Vec::new();
        
        for update in update_result.available_updates {
            if update.is_critical || self.config.auto_download {
                match self.download_update(&update).await {
                    Ok(()) => {
                        downloaded.push(update.model_id);
                    }
                    Err(e) => {
                        log::error!("Failed to download update for {}: {}", update.model_id, e);
                    }
                }
            }
        }
        
        Ok(downloaded)
    }
    
    /// Download a specific update
    pub async fn download_update(&self, update: &ModelUpdate) -> Result<(), String> {
        // Use the model service to download the new version
        self.model_service.get_registry()
            .download_model(&update.model_id)
            .await
            .map_err(|e| format!("Failed to download update: {}", e))?;
        
        Ok(())
    }
    
    /// Get update history
    pub async fn get_update_history(&self) -> Vec<ModelUpdate> {
        let cache = self.update_cache.read().await;
        let mut history = Vec::new();
        
        for result in cache.results.values() {
            history.extend(result.available_updates.clone());
        }
        
        // Sort by release date (newest first)
        history.sort_by(|a, b| b.release_date.cmp(&a.release_date));
        
        history
    }
    
    /// Clear update cache
    pub async fn clear_cache(&self) {
        let mut cache = self.update_cache.write().await;
        cache.results.clear();
        cache.last_full_check = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_extraction() {
        let checker = ModelUpdateChecker::new(
            Arc::new(ModelService::new().await.unwrap()),
            Default::default(),
        );
        
        assert_eq!(
            checker.extract_version("phi-3-mini-v1.2.3").unwrap(),
            Version::new(1, 2, 3)
        );
        
        assert_eq!(
            checker.extract_version("llama-3.2-1b-2.0.0").unwrap(),
            Version::new(2, 0, 0)
        );
        
        assert_eq!(
            checker.extract_version("model-without-version").unwrap(),
            Version::new(1, 0, 0)
        );
    }
    
    #[test]
    fn test_parameter_parsing() {
        let checker = ModelUpdateChecker::new(
            Arc::new(ModelService::new().await.unwrap()),
            Default::default(),
        );
        
        assert_eq!(checker.parse_parameters("7B").unwrap(), 7_000_000_000);
        assert_eq!(checker.parse_parameters("1.5B").unwrap(), 1_500_000_000);
        assert_eq!(checker.parse_parameters("350M").unwrap(), 350_000_000);
        assert_eq!(checker.parse_parameters("13B parameters").unwrap(), 13_000_000_000);
    }
}