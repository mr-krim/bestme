use tauri::{plugin::Plugin, Runtime, State, Manager, Emitter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use bestme::ai::{
    AIProvider, EnhancementOptions, EnhancedText, PrivacyLevel,
    local::LocalAI,
    cloud::{CloudAI, CloudAIConfig, AIProviderType},
    common::ModelConfig,
    services::{
        ModelService, ModelSelector, ModelSelectorConfig,
        ModelUpdateChecker, UpdateCheckerConfig, ModelUpdate, UpdateCheckResult,
        CustomModelManager, CustomModel, ImportOptions, ImportResult,
        ValidationResult as ModelValidationResult,
    },
    models::registry::DownloadProgress,
};

#[derive(Default)]
pub struct AIPlugin {
    local_ai: Arc<RwLock<Option<LocalAI>>>,
    cloud_ai: Arc<RwLock<Option<CloudAI>>>,
    model_service: Arc<RwLock<Option<Arc<ModelService>>>>,
    model_selector: Arc<RwLock<Option<Arc<ModelSelector>>>>,
    update_checker: Arc<RwLock<Option<Arc<ModelUpdateChecker>>>>,
    custom_model_manager: Arc<RwLock<Option<Arc<CustomModelManager>>>>,
}

impl<R: Runtime> Plugin<R> for AIPlugin {
    fn name(&self) -> &'static str {
        "ai"
    }

    fn initialize(&mut self, app: &tauri::AppHandle<R>, _config: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        app.manage(AIState {
            local_ai: self.local_ai.clone(),
            cloud_ai: self.cloud_ai.clone(),
            model_service: self.model_service.clone(),
            model_selector: self.model_selector.clone(),
            update_checker: self.update_checker.clone(),
            custom_model_manager: self.custom_model_manager.clone(),
        });
        Ok(())
    }
}

#[derive(Clone)]
pub struct AIState {
    local_ai: Arc<RwLock<Option<LocalAI>>>,
    cloud_ai: Arc<RwLock<Option<CloudAI>>>,
    model_service: Arc<RwLock<Option<Arc<ModelService>>>>,
    model_selector: Arc<RwLock<Option<Arc<ModelSelector>>>>,
    update_checker: Arc<RwLock<Option<Arc<ModelUpdateChecker>>>>,
    custom_model_manager: Arc<RwLock<Option<Arc<CustomModelManager>>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitLocalAIRequest {
    model_name: String,
    use_gpu: bool,
    quantization: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitCloudAIRequest {
    provider: String,
    api_key: String,
    model: String,
    privacy_level: String,
    base_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhanceTextRequest {
    text: String,
    options: EnhancementOptions,
    use_cloud: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDownloadProgress {
    model_name: String,
    progress: f32,
    status: String,
}

#[tauri::command]
pub async fn init_local_ai(
    state: State<'_, AIState>,
    request: InitLocalAIRequest,
) -> Result<String, String> {
    let config = ModelConfig {
        model_path: std::path::PathBuf::new(), // Will be set by model manager
        use_gpu: request.use_gpu,
        quantization: request.quantization.and_then(|q| match q.as_str() {
            "int8" => Some(bestme::ai::common::QuantizationType::Int8),
            "int4" => Some(bestme::ai::common::QuantizationType::Int4),
            "fp16" => Some(bestme::ai::common::QuantizationType::FP16),
            _ => None,
        }),
        ..Default::default()
    };

    let local_ai = LocalAI::new(config);
    
    // Load the model
    if let Err(e) = local_ai.load_model(&request.model_name).await {
        return Err(format!("Failed to load model: {}", e));
    }

    let mut ai_guard = state.local_ai.write().await;
    *ai_guard = Some(local_ai);

    Ok("Local AI initialized successfully".to_string())
}

#[tauri::command]
pub async fn init_cloud_ai(
    state: State<'_, AIState>,
    request: InitCloudAIRequest,
) -> Result<String, String> {
    let provider = match request.provider.as_str() {
        "openrouter" => AIProviderType::OpenRouter,
        "requesty" => AIProviderType::Requesty,
        "openai" => AIProviderType::OpenAI,
        _ => return Err("Invalid provider".to_string()),
    };

    let privacy_level = match request.privacy_level.as_str() {
        "strict" => PrivacyLevel::Strict,
        "balanced" => PrivacyLevel::Balanced,
        "permissive" => PrivacyLevel::Permissive,
        _ => return Err("Invalid privacy level".to_string()),
    };

    let config = CloudAIConfig {
        provider,
        api_key: request.api_key,
        model: request.model,
        privacy_level,
        base_url: request.base_url,
    };

    let mut cloud_ai = CloudAI::new(config)
        .map_err(|e| format!("Failed to create cloud AI: {}", e))?;

    // Validate API key
    if let Err(e) = cloud_ai.initialize().await {
        return Err(format!("Failed to validate API key: {}", e));
    }

    let mut ai_guard = state.cloud_ai.write().await;
    *ai_guard = Some(cloud_ai);

    Ok("Cloud AI initialized successfully".to_string())
}

#[tauri::command]
pub async fn enhance_text(
    state: State<'_, AIState>,
    request: EnhanceTextRequest,
) -> Result<EnhancedText, String> {
    if request.use_cloud {
        let cloud_guard = state.cloud_ai.read().await;
        let cloud_ai = cloud_guard.as_ref()
            .ok_or_else(|| "Cloud AI not initialized".to_string())?;
        
        cloud_ai.enhance_text(&request.text, &request.options).await
            .map_err(|e| format!("Enhancement failed: {}", e))
    } else {
        let local_guard = state.local_ai.read().await;
        let local_ai = local_guard.as_ref()
            .ok_or_else(|| "Local AI not initialized".to_string())?;
        
        local_ai.enhance_text(&request.text, &request.options).await
            .map_err(|e| format!("Enhancement failed: {}", e))
    }
}

#[tauri::command]
pub async fn list_available_models(
    state: State<'_, AIState>,
) -> Result<Vec<serde_json::Value>, String> {
    // Initialize model service if needed
    let mut service_guard = state.model_service.write().await;
    if service_guard.is_none() {
        let service = ModelService::new().await
            .map_err(|e| format!("Failed to create model service: {}", e))?;
        *service_guard = Some(Arc::new(service));
    }
    drop(service_guard);
    
    let service_guard = state.model_service.read().await;
    let service = service_guard.as_ref().unwrap();
    
    let models = service.get_registry().list_available_models().await;
    let json_models = models.into_iter()
        .map(|m| serde_json::to_value(m).unwrap())
        .collect();
    
    Ok(json_models)
}

#[tauri::command]
pub async fn download_model(
    state: State<'_, AIState>,
    model_id: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    // Initialize model service if needed
    let mut service_guard = state.model_service.write().await;
    if service_guard.is_none() {
        let service = ModelService::new().await
            .map_err(|e| format!("Failed to create model service: {}", e))?;
        *service_guard = Some(Arc::new(service));
    }
    drop(service_guard);
    
    let service_guard = state.model_service.read().await;
    let service = service_guard.as_ref().unwrap().clone();
    
    // Start download in background
    tokio::spawn(async move {
        match service.get_registry().download_model(&model_id).await {
            Ok(path) => {
                log::info!("Model {} downloaded to: {:?}", model_id, path);
                // Emit success event
                let _ = app.emit("model-download-complete", serde_json::json!({
                    "model_id": model_id,
                    "path": path.to_string_lossy(),
                    "success": true
                }));
            }
            Err(e) => {
                log::error!("Model {} download failed: {}", model_id, e);
                // Emit error event
                let _ = app.emit("model-download-error", serde_json::json!({
                    "model_id": model_id,
                    "error": e.to_string(),
                    "success": false
                }));
            }
        }
    });

    Ok("Download started".to_string())
}

#[tauri::command]
pub async fn list_downloaded_models(
    state: State<'_, AIState>,
) -> Result<Vec<String>, String> {
    // Initialize model service if needed
    let mut service_guard = state.model_service.write().await;
    if service_guard.is_none() {
        let service = ModelService::new().await
            .map_err(|e| format!("Failed to create model service: {}", e))?;
        *service_guard = Some(Arc::new(service));
    }
    drop(service_guard);
    
    let service_guard = state.model_service.read().await;
    let service = service_guard.as_ref().unwrap();
    
    service.get_registry().list_downloaded_models().await
        .map_err(|e| format!("Failed to list models: {}", e))
}

#[tauri::command]
pub async fn summarize_text(
    state: State<'_, AIState>,
    text: String,
    max_length: usize,
) -> Result<String, String> {
    let cloud_guard = state.cloud_ai.read().await;
    let cloud_ai = cloud_guard.as_ref()
        .ok_or_else(|| "Cloud AI not initialized for summarization".to_string())?;
    
    cloud_ai.summarize_text(&text, max_length).await
        .map_err(|e| format!("Summarization failed: {}", e))
}

#[tauri::command]
pub async fn transform_style(
    state: State<'_, AIState>,
    text: String,
    target_style: String,
) -> Result<String, String> {
    let cloud_guard = state.cloud_ai.read().await;
    let cloud_ai = cloud_guard.as_ref()
        .ok_or_else(|| "Cloud AI not initialized for style transformation".to_string())?;
    
    cloud_ai.transform_style(&text, &target_style).await
        .map_err(|e| format!("Style transformation failed: {}", e))
}

#[tauri::command]
pub async fn translate_text(
    state: State<'_, AIState>,
    text: String,
    target_language: String,
    source_language: Option<String>,
) -> Result<String, String> {
    let cloud_guard = state.cloud_ai.read().await;
    let cloud_ai = cloud_guard.as_ref()
        .ok_or_else(|| "Cloud AI not initialized for translation".to_string())?;
    
    cloud_ai.translate_text(&text, &target_language, source_language.as_deref()).await
        .map_err(|e| format!("Translation failed: {}", e))
}

#[tauri::command]
pub async fn get_cloud_usage(
    state: State<'_, AIState>,
) -> Result<bestme::ai::cloud::CloudAIUsage, String> {
    let cloud_guard = state.cloud_ai.read().await;
    let cloud_ai = cloud_guard.as_ref()
        .ok_or_else(|| "Cloud AI not initialized".to_string())?;
    
    cloud_ai.get_usage().await
        .map_err(|e| format!("Failed to get usage: {}", e))
}

#[tauri::command]
pub async fn save_api_key(
    state: State<'_, AIState>,
    provider: String,
    api_key: String,
) -> Result<String, String> {
    // Create a temporary CloudAI instance to save the key
    let provider_type = match provider.as_str() {
        "openrouter" => bestme::ai::cloud::AIProviderType::OpenRouter,
        "requesty" => bestme::ai::cloud::AIProviderType::Requesty,
        "openai" => bestme::ai::cloud::AIProviderType::OpenAI,
        _ => return Err("Invalid provider".to_string()),
    };
    
    let config = bestme::ai::cloud::CloudAIConfig {
        provider: provider_type,
        api_key: api_key.clone(),
        model: String::new(),
        privacy_level: bestme::ai::PrivacyLevel::Balanced,
        base_url: None,
    };
    
    let cloud_ai = CloudAI::new(config)
        .map_err(|e| format!("Failed to create cloud AI: {}", e))?;
    
    cloud_ai.save_api_key(&api_key).await
        .map_err(|e| format!("Failed to save API key: {}", e))?;
    
    Ok("API key saved securely".to_string())
}

#[tauri::command]
pub async fn list_saved_api_keys() -> Result<Vec<String>, String> {
    let mut key_manager = bestme::ai::security::ApiKeyManager::new();
    key_manager.load_metadata()
        .map_err(|e| format!("Failed to load metadata: {}", e))?;
    
    let keys = key_manager.list_stored_keys()
        .into_iter()
        .map(|k| k.provider.clone())
        .collect();
    
    Ok(keys)
}

#[tauri::command]
pub async fn delete_api_key(provider: String) -> Result<String, String> {
    let mut key_manager = bestme::ai::security::ApiKeyManager::new();
    key_manager.load_metadata()
        .map_err(|e| format!("Failed to load metadata: {}", e))?;
    
    key_manager.delete_api_key(&provider).await
        .map_err(|e| format!("Failed to delete API key: {}", e))?;
    
    Ok("API key deleted".to_string())
}

#[tauri::command]
pub async fn get_gpu_info(
    state: State<'_, AIState>,
) -> Result<Vec<serde_json::Value>, String> {
    // Initialize model service if needed
    let mut service_guard = state.model_service.write().await;
    if service_guard.is_none() {
        let service = ModelService::new().await
            .map_err(|e| format!("Failed to create model service: {}", e))?;
        *service_guard = Some(Arc::new(service));
    }
    drop(service_guard);
    
    let service_guard = state.model_service.read().await;
    let service = service_guard.as_ref().unwrap();
    
    let gpu_infos = service.get_gpu_info().await;
    let json_infos = gpu_infos.into_iter()
        .map(|info| serde_json::to_value(info).unwrap())
        .collect();
    
    Ok(json_infos)
}

#[tauri::command]
pub async fn get_model_download_progress(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<Option<DownloadProgress>, String> {
    let service_guard = state.model_service.read().await;
    if let Some(service) = service_guard.as_ref() {
        Ok(service.get_registry().get_download_progress(&model_id).await)
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn get_model_performance(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<serde_json::Value, String> {
    // TODO: Implement get_model_performance when method is available
    // For now, return placeholder data
    Ok(serde_json::json!({
        "model_id": model_id,
        "avg_latency_ms": 50.0,
        "throughput_tokens_per_sec": 1000.0,
        "memory_usage_mb": 512
    }))
}

#[tauri::command]
pub async fn get_current_ai_metrics(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<serde_json::Value, String> {
    // TODO: Implement get_current_metrics when method is available
    // For now, return placeholder data
    Ok(serde_json::json!({
        "model_id": model_id,
        "current_memory_mb": 512,
        "gpu_utilization": 0.65,
        "requests_per_minute": 120
    }))
}

#[tauri::command]
pub async fn get_ai_performance_summary(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<serde_json::Value, String> {
    // TODO: Implement get_performance_summary when method is available
    // For now, return placeholder data
    Ok(serde_json::json!({
        "model_id": model_id,
        "total_requests": 1000,
        "avg_response_time_ms": 45.5,
        "cache_hit_rate": 0.75,
        "error_rate": 0.01
    }))
}

#[tauri::command]
pub async fn get_active_downloads(
    state: State<'_, AIState>,
) -> Result<Vec<serde_json::Value>, String> {
    // TODO: Implement get_active_downloads when method is available
    // For now, return empty list
    Ok(vec![])
}

#[tauri::command]
pub async fn cancel_model_download(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<String, String> {
    // TODO: Implement cancel_download when method is available
    log::warn!("Cancel download not yet implemented for model: {}", model_id);
    Ok("Download cancellation not yet implemented".to_string())
}

#[tauri::command]
pub async fn get_model_config(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<serde_json::Value, String> {
    // TODO: Implement get_model_config when method is available
    // For now, return placeholder config
    Ok(serde_json::json!({
        "model_id": model_id,
        "use_gpu": true,
        "quantization": "int8",
        "batch_size": 1,
        "max_tokens": 512
    }))
}

#[tauri::command]
pub async fn update_model_config(
    state: State<'_, AIState>,
    model_id: String,
    config: serde_json::Value,
) -> Result<String, String> {
    // TODO: Implement update_model_config when method is available
    log::info!("Model config update requested for {}: {:?}", model_id, config);
    Ok("Configuration update not yet implemented".to_string())
}

#[tauri::command]
pub async fn load_ai_model(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<String, String> {
    // Initialize model service if needed
    let mut service_guard = state.model_service.write().await;
    if service_guard.is_none() {
        let service = ModelService::new().await
            .map_err(|e| format!("Failed to create model service: {}", e))?;
        *service_guard = Some(Arc::new(service));
    }
    drop(service_guard);
    
    let service_guard = state.model_service.read().await;
    let service = service_guard.as_ref().unwrap();
    
    service.load_model(&model_id).await
        .map_err(|e| format!("Failed to load model: {}", e))?;
    
    Ok(format!("Model {} loaded successfully", model_id))
}

#[tauri::command]
pub async fn unload_ai_model(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<String, String> {
    let service_guard = state.model_service.read().await;
    if let Some(service) = service_guard.as_ref() {
        service.unload_model(&model_id).await
            .map_err(|e| format!("Failed to unload model: {}", e))?;
        Ok(format!("Model {} unloaded", model_id))
    } else {
        Err("Model service not initialized".to_string())
    }
}

#[tauri::command]
pub async fn list_loaded_models(
    state: State<'_, AIState>,
) -> Result<Vec<serde_json::Value>, String> {
    let service_guard = state.model_service.read().await;
    if let Some(service) = service_guard.as_ref() {
        let models = service.list_loaded_models().await;
        let json_models = models.into_iter()
            .map(|m| serde_json::to_value(m).unwrap())
            .collect();
        Ok(json_models)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn auto_select_model(
    state: State<'_, AIState>,
    text: String,
) -> Result<serde_json::Value, String> {
    // Ensure model service is initialized
    let mut service_guard = state.model_service.write().await;
    if service_guard.is_none() {
        let service = ModelService::new().await
            .map_err(|e| format!("Failed to create model service: {}", e))?;
        *service_guard = Some(Arc::new(service));
    }
    let model_service = service_guard.as_ref().unwrap().clone();
    drop(service_guard);
    
    // Initialize model selector if needed
    let mut selector_guard = state.model_selector.write().await;
    if selector_guard.is_none() {
        // Create metrics aggregator
        let meter = opentelemetry::global::meter("bestme-ai");
        let metrics_aggregator = Arc::new(bestme::ai::telemetry::metrics::MetricsAggregator::new(meter.clone()));
        // Get a model collector for the selector
        let model_collector = metrics_aggregator.get_model_collector("model-selector", &meter);
        let selector = ModelSelector::new(
            model_service.clone(),
            model_collector,
            ModelSelectorConfig::default(),
        );
        *selector_guard = Some(Arc::new(selector));
    }
    drop(selector_guard);
    
    let selector_guard = state.model_selector.read().await;
    let selector = selector_guard.as_ref().unwrap();
    
    match selector.select_model(&text).await {
        Ok(result) => Ok(serde_json::to_value(result).unwrap()),
        Err(e) => Err(format!("Model selection failed: {}", e)),
    }
}

#[tauri::command]
pub async fn analyze_text_characteristics(
    text: String,
) -> Result<serde_json::Value, String> {
    let analyzer = bestme::ai::services::TextAnalyzer::new();
    let characteristics = analyzer.analyze(&text);
    Ok(serde_json::to_value(characteristics).unwrap())
}

#[tauri::command]
pub async fn get_model_requirements(
    text: String,
) -> Result<serde_json::Value, String> {
    let analyzer = bestme::ai::services::TextAnalyzer::new();
    let characteristics = analyzer.analyze(&text);
    
    let matcher = bestme::ai::services::ModelCapabilityMatcher::new();
    let requirements = matcher.derive_requirements(&characteristics);
    
    Ok(serde_json::to_value(requirements).unwrap())
}

#[tauri::command]
pub async fn update_model_selector_config(
    state: State<'_, AIState>,
    config: serde_json::Value,
) -> Result<String, String> {
    let selector_config: ModelSelectorConfig = serde_json::from_value(config)
        .map_err(|e| format!("Invalid configuration: {}", e))?;
    
    // Initialize selector if needed with new config
    let mut selector_guard = state.model_selector.write().await;
    
    let model_service = if let Some(service) = state.model_service.read().await.as_ref() {
        service.clone()
    } else {
        return Err("Model service not initialized".to_string());
    };
    
    let meter = opentelemetry::global::meter("bestme-ai");
    let metrics_aggregator = Arc::new(bestme::ai::telemetry::metrics::MetricsAggregator::new(meter.clone()));
    let model_collector = metrics_aggregator.get_model_collector("model-selector", &meter);
    let selector = ModelSelector::new(model_service, model_collector, selector_config);
    *selector_guard = Some(Arc::new(selector));
    
    Ok("Model selector configuration updated".to_string())
}

#[tauri::command]
pub async fn get_model_selection_history(
    state: State<'_, AIState>,
) -> Result<Vec<serde_json::Value>, String> {
    let selector_guard = state.model_selector.read().await;
    if let Some(selector) = selector_guard.as_ref() {
        let history = selector.get_selection_history().await;
        let json_history = history.into_iter()
            .map(|(model_id, reason, score)| {
                serde_json::json!({
                    "model_id": model_id,
                    "reason": reason,
                    "score": score,
                })
            })
            .collect();
        Ok(json_history)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn clear_model_selection_cache(
    state: State<'_, AIState>,
) -> Result<String, String> {
    let selector_guard = state.model_selector.read().await;
    if let Some(selector) = selector_guard.as_ref() {
        selector.clear_cache();
        Ok("Model selection cache cleared".to_string())
    } else {
        Err("Model selector not initialized".to_string())
    }
}

#[tauri::command]
pub async fn check_model_updates(
    state: State<'_, AIState>,
    force: bool,
) -> Result<UpdateCheckResult, String> {
    // Ensure model service is initialized
    let mut service_guard = state.model_service.write().await;
    if service_guard.is_none() {
        let service = ModelService::new().await
            .map_err(|e| format!("Failed to create model service: {}", e))?;
        *service_guard = Some(Arc::new(service));
    }
    let model_service = service_guard.as_ref().unwrap().clone();
    drop(service_guard);
    
    // Initialize update checker if needed
    let mut checker_guard = state.update_checker.write().await;
    if checker_guard.is_none() {
        let checker = ModelUpdateChecker::new(
            model_service.clone(),
            UpdateCheckerConfig::default(),
        );
        *checker_guard = Some(Arc::new(checker));
    }
    drop(checker_guard);
    
    let checker_guard = state.update_checker.read().await;
    let checker = checker_guard.as_ref().unwrap();
    
    checker.check_all_updates(force).await
}

#[tauri::command]
pub async fn check_single_model_update(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<Option<ModelUpdate>, String> {
    let checker_guard = state.update_checker.read().await;
    if let Some(checker) = checker_guard.as_ref() {
        checker.check_model_update(&model_id).await
    } else {
        // Initialize if needed
        drop(checker_guard);
        
        let mut service_guard = state.model_service.write().await;
        if service_guard.is_none() {
            let service = ModelService::new().await
                .map_err(|e| format!("Failed to create model service: {}", e))?;
            *service_guard = Some(Arc::new(service));
        }
        let model_service = service_guard.as_ref().unwrap().clone();
        drop(service_guard);
        
        let mut checker_guard = state.update_checker.write().await;
        let checker = ModelUpdateChecker::new(
            model_service.clone(),
            UpdateCheckerConfig::default(),
        );
        let arc_checker = Arc::new(checker);
        *checker_guard = Some(arc_checker.clone());
        drop(checker_guard);
        
        arc_checker.check_model_update(&model_id).await
    }
}

#[tauri::command]
pub async fn download_model_update(
    state: State<'_, AIState>,
    update: ModelUpdate,
) -> Result<String, String> {
    let checker_guard = state.update_checker.read().await;
    if let Some(checker) = checker_guard.as_ref() {
        checker.download_update(&update).await?;
        Ok("Update downloaded successfully".to_string())
    } else {
        Err("Update checker not initialized".to_string())
    }
}

#[tauri::command]
pub async fn configure_update_checker(
    state: State<'_, AIState>,
    config: UpdateCheckerConfig,
) -> Result<String, String> {
    let mut service_guard = state.model_service.write().await;
    if service_guard.is_none() {
        let service = ModelService::new().await
            .map_err(|e| format!("Failed to create model service: {}", e))?;
        *service_guard = Some(Arc::new(service));
    }
    let model_service = service_guard.as_ref().unwrap().clone();
    drop(service_guard);
    
    let mut checker_guard = state.update_checker.write().await;
    let checker = ModelUpdateChecker::new(model_service, config);
    *checker_guard = Some(Arc::new(checker));
    
    Ok("Update checker configured".to_string())
}

#[tauri::command]
pub async fn get_update_history(
    state: State<'_, AIState>,
) -> Result<Vec<ModelUpdate>, String> {
    let checker_guard = state.update_checker.read().await;
    if let Some(checker) = checker_guard.as_ref() {
        Ok(checker.get_update_history().await)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn clear_update_cache(
    state: State<'_, AIState>,
) -> Result<String, String> {
    let checker_guard = state.update_checker.read().await;
    if let Some(checker) = checker_guard.as_ref() {
        checker.clear_cache().await;
        Ok("Update cache cleared".to_string())
    } else {
        Err("Update checker not initialized".to_string())
    }
}

#[tauri::command]
pub async fn validate_onnx_model(
    path: String,
) -> Result<ModelValidationResult, String> {
    let validator = bestme::ai::services::OnnxValidator::new()
        .map_err(|e| format!("Failed to create validator: {}", e))?;
    
    Ok(validator.validate_model(std::path::Path::new(&path)))
}

#[tauri::command]
pub async fn import_custom_model(
    state: State<'_, AIState>,
    path: String,
    options: ImportOptions,
    app: tauri::AppHandle,
) -> Result<ImportResult, String> {
    // Initialize custom model manager if needed
    let mut manager_guard = state.custom_model_manager.write().await;
    if manager_guard.is_none() {
        // Get app data directory
        let app_dir = app.path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
        let custom_models_dir = app_dir.join("custom_models");
        
        // Initialize model service if needed
        let mut service_guard = state.model_service.write().await;
        if service_guard.is_none() {
            let service = ModelService::new().await
                .map_err(|e| format!("Failed to create model service: {}", e))?;
            *service_guard = Some(Arc::new(service));
        }
        let model_service = service_guard.as_ref().unwrap().clone();
        drop(service_guard);
        
        let registry = model_service.get_registry().clone();
        let manager = CustomModelManager::new(registry, custom_models_dir)
            .map_err(|e| format!("Failed to create custom model manager: {}", e))?;
        *manager_guard = Some(Arc::new(manager));
    }
    drop(manager_guard);
    
    let manager_guard = state.custom_model_manager.read().await;
    let manager = manager_guard.as_ref().unwrap();
    
    Ok(manager.import_model(std::path::Path::new(&path), options).await)
}

#[tauri::command]
pub async fn list_custom_models(
    state: State<'_, AIState>,
) -> Result<Vec<CustomModel>, String> {
    let manager_guard = state.custom_model_manager.read().await;
    if let Some(manager) = manager_guard.as_ref() {
        Ok(manager.list_custom_models().await)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn get_custom_model(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<Option<CustomModel>, String> {
    let manager_guard = state.custom_model_manager.read().await;
    if let Some(manager) = manager_guard.as_ref() {
        Ok(manager.get_custom_model(&model_id).await)
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn update_custom_model(
    state: State<'_, AIState>,
    model_id: String,
    updates: serde_json::Value,
) -> Result<String, String> {
    let manager_guard = state.custom_model_manager.read().await;
    if let Some(manager) = manager_guard.as_ref() {
        let update_request = serde_json::from_value(updates)
            .map_err(|e| format!("Invalid update request: {}", e))?;
        manager.update_model(&model_id, update_request).await?;
        Ok("Model updated successfully".to_string())
    } else {
        Err("Custom model manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn delete_custom_model(
    state: State<'_, AIState>,
    model_id: String,
) -> Result<String, String> {
    let manager_guard = state.custom_model_manager.read().await;
    if let Some(manager) = manager_guard.as_ref() {
        manager.delete_model(&model_id).await?;
        Ok("Model deleted successfully".to_string())
    } else {
        Err("Custom model manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn export_custom_model(
    state: State<'_, AIState>,
    model_id: String,
    export_path: String,
) -> Result<String, String> {
    let manager_guard = state.custom_model_manager.read().await;
    if let Some(manager) = manager_guard.as_ref() {
        manager.export_model(&model_id, std::path::Path::new(&export_path)).await?;
        Ok("Model exported successfully".to_string())
    } else {
        Err("Custom model manager not initialized".to_string())
    }
}

// Note: Commands are registered directly in main.rs using tauri::generate_handler!
// This function is kept for reference of available commands