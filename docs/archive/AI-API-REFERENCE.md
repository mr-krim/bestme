# AI API Reference

## Rust API

### Core Traits

#### AIProvider

The main trait that all AI implementations must implement:

```rust
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Initialize the AI provider
    async fn initialize(&mut self) -> Result<(), AIError>;
    
    /// Enhance text with AI processing
    async fn enhance_text(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Result<EnhancedText, AIError>;
    
    /// Get provider information
    fn get_info(&self) -> ProviderInfo;
    
    /// Check if provider is available
    async fn is_available(&self) -> bool;
    
    /// Get current usage metrics
    async fn get_usage(&self) -> Result<UsageMetrics, AIError>;
}
```

### Data Structures

#### EnhancementOptions

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementOptions {
    pub correct_grammar: bool,
    pub improve_punctuation: bool,
    pub detect_intent: bool,
    pub preserve_style: bool,
    pub confidence_threshold: f32,
    pub max_corrections: Option<usize>,
    pub target_formality: Option<FormalityLevel>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FormalityLevel {
    Casual,
    Neutral,
    Formal,
    Academic,
}
```

#### EnhancedText

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedText {
    pub original: String,
    pub enhanced: String,
    pub confidence: f32,
    pub intent: Option<Intent>,
    pub changes: Vec<TextChange>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChange {
    pub start: usize,
    pub end: usize,
    pub original: String,
    pub replacement: String,
    pub change_type: ChangeType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChangeType {
    Grammar,
    Punctuation,
    Spelling,
    Capitalization,
    Style,
    Clarity,
}
```

### Services

#### ModelService

Core service for model management:

```rust
pub struct ModelService {
    registry: Arc<ModelRegistry>,
    cache_dir: PathBuf,
    active_models: Arc<RwLock<HashMap<String, ModelHandle>>>,
    download_manager: Arc<DownloadManager>,
    metrics: Arc<MetricsCollector>,
}

impl ModelService {
    /// Create new model service
    pub async fn new() -> Result<Self, String>;
    
    /// List all available models
    pub async fn list_available_models() -> Vec<ModelMetadata>;
    
    /// Download a model
    pub async fn download_model(&self, model_name: &str) -> Result<(), String>;
    
    /// Load model into memory
    pub async fn load_model(&self, model_id: &str) -> Result<(), String>;
    
    /// Unload model from memory
    pub async fn unload_model(&self, model_id: &str) -> Result<(), String>;
    
    /// Get model performance metrics
    pub async fn get_model_performance(
        &self, 
        model_id: &str
    ) -> Result<PerformanceMetrics, String>;
}
```

#### TextAnalyzer

Analyzes text characteristics for model selection:

```rust
pub struct TextAnalyzer;

impl TextAnalyzer {
    /// Analyze text characteristics
    pub fn analyze(&self, text: &str) -> TextCharacteristics;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCharacteristics {
    pub length: usize,
    pub complexity_score: f32,
    pub domain: TextDomain,
    pub language: Option<String>,
    pub contains_code: bool,
    pub contains_math: bool,
    pub contains_technical_terms: bool,
    pub avg_word_length: f32,
    pub sentence_count: usize,
    pub unique_words: usize,
    pub formality_score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDomain {
    General,
    Technical,
    Medical,
    Legal,
    Academic,
    Conversational,
    Creative,
}
```

#### ModelSelector

Intelligent model selection based on text and requirements:

```rust
pub struct ModelSelector {
    model_service: Arc<ModelService>,
    capability_matcher: ModelCapabilityMatcher,
    metrics_collector: Arc<MetricsCollector>,
    config: ModelSelectorConfig,
    cache: Arc<RwLock<LruCache<u64, SelectionResult>>>,
}

impl ModelSelector {
    /// Select best model for given text
    pub async fn select_model(&self, text: &str) -> Result<SelectionResult, String>;
    
    /// Select with specific performance target
    pub async fn select_with_target(
        &self,
        text: &str,
        target: PerformanceTarget,
    ) -> Result<SelectionResult, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    pub model_id: String,
    pub reason: String,
    pub confidence_score: f32,
    pub alternatives: Vec<AlternativeModel>,
    pub expected_performance: ExpectedPerformance,
}
```

#### CustomModelManager

Manages user-imported ONNX models:

```rust
pub struct CustomModelManager {
    registry: Arc<ModelRegistry>,
    validator: Arc<OnnxValidator>,
    custom_models: Arc<RwLock<HashMap<String, CustomModel>>>,
    storage_path: PathBuf,
}

impl CustomModelManager {
    /// Import a custom ONNX model
    pub async fn import_model(
        &self,
        model_path: &Path,
        options: ImportOptions,
    ) -> ImportResult;
    
    /// Update model metadata
    pub async fn update_model(
        &self,
        model_id: &str,
        updates: ModelUpdateRequest,
    ) -> Result<(), String>;
    
    /// Export model to file
    pub async fn export_model(
        &self,
        model_id: &str,
        export_path: &Path,
    ) -> Result<(), String>;
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    #[error("Initialization failed: {0}")]
    InitializationError(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Inference failed: {0}")]
    InferenceError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("Resource exhausted: {0}")]
    ResourceError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## TypeScript/JavaScript API

### Types

```typescript
// Enhancement options
interface EnhancementOptions {
  correct_grammar: boolean;
  improve_punctuation: boolean;
  detect_intent: boolean;
  preserve_style: boolean;
  confidence_threshold: number;
  max_corrections?: number;
  target_formality?: 'casual' | 'neutral' | 'formal' | 'academic';
}

// Enhanced text result
interface EnhancedText {
  original: string;
  enhanced: string;
  confidence: number;
  intent?: Intent;
  changes: TextChange[];
  metadata: Record<string, any>;
}

// Text change
interface TextChange {
  start: number;
  end: number;
  original: string;
  replacement: string;
  change_type: 'grammar' | 'punctuation' | 'spelling' | 'capitalization' | 'style' | 'clarity';
  confidence: number;
}

// Model metadata
interface ModelMetadata {
  id: string;
  name: string;
  provider: string;
  architecture: string;
  parameters: string;
  size_bytes: number;
  capabilities: string[];
  performance_class: 'fast' | 'balanced' | 'quality';
  context_window: number;
  supports_gpu: boolean;
  supported_languages: string[];
}

// Custom model
interface CustomModel {
  id: string;
  name: string;
  original_filename: string;
  description: string;
  model_type: 'TextGeneration' | 'TextClassification' | 'TokenClassification' | 'Seq2Seq' | 'Unknown';
  import_date: string;
  tags: string[];
  is_active: boolean;
  usage_count: number;
  validation_result: ValidationResult;
}
```

### Tauri Commands

```typescript
import { invoke } from '@tauri-apps/api/core';

// Initialize AI
await invoke('init_local_ai', {
  request: {
    model_name: string,
    use_gpu: boolean,
    quantization: string | null
  }
});

await invoke('init_cloud_ai', {
  request: {
    provider: string,
    api_key: string,
    model: string,
    privacy_level: string,
    base_url: string | null
  }
});

// Enhance text
const enhanced = await invoke<EnhancedText>('enhance_text', {
  request: {
    text: string,
    options: EnhancementOptions,
    use_cloud: boolean
  }
});

// Model management
const models = await invoke<ModelMetadata[]>('list_available_models');
const downloaded = await invoke<string[]>('list_downloaded_models');
await invoke('download_model', { modelName: string });
await invoke('load_ai_model', { modelId: string });
await invoke('unload_ai_model', { modelId: string });

// Performance monitoring
const metrics = await invoke<PerformanceMetrics>('get_model_performance', {
  modelId: string
});

const currentMetrics = await invoke<CurrentMetrics>('get_current_ai_metrics');

// Model selection
const result = await invoke<SelectionResult>('auto_select_model', {
  text: string,
  performanceTarget: 'fast' | 'balanced' | 'quality'
});

// Custom models
const validation = await invoke<ValidationResult>('validate_onnx_model', {
  path: string
});

const importResult = await invoke<ImportResult>('import_custom_model', {
  path: string,
  options: ImportOptions
});

const customModels = await invoke<CustomModel[]>('list_custom_models');

await invoke('update_custom_model', {
  modelId: string,
  updates: ModelUpdateRequest
});

await invoke('delete_custom_model', { modelId: string });

await invoke('export_custom_model', {
  modelId: string,
  exportPath: string
});
```

### Events

```typescript
import { listen } from '@tauri-apps/api/event';

// Model download progress
await listen('model:download-progress', (event) => {
  const { modelId, progress, total, speed } = event.payload;
  console.log(`Download ${modelId}: ${progress}/${total} at ${speed} MB/s`);
});

// AI metrics update
await listen('ai:metrics-update', (event) => {
  const metrics = event.payload;
  console.log('Current metrics:', metrics);
});

// Model loaded/unloaded
await listen('model:loaded', (event) => {
  console.log('Model loaded:', event.payload.modelId);
});

await listen('model:unloaded', (event) => {
  console.log('Model unloaded:', event.payload.modelId);
});
```

## Configuration API

### Rust Configuration

```rust
use bestme::config::{AIConfig, ModelConfig};

// Load configuration
let config = AIConfig::load()?;

// Update configuration
config.set_default_provider(AIProviderType::Local);
config.set_privacy_level(PrivacyLevel::Balanced);
config.save()?;

// Model-specific configuration
let model_config = ModelConfig {
    max_memory_mb: 2048,
    thread_count: 4,
    use_gpu: true,
    optimization_level: OptimizationLevel::Maximum,
    quantization: Some(QuantizationType::Dynamic),
};
```

### JavaScript Configuration

```typescript
// Get current configuration
const config = await invoke('get_ai_config');

// Update configuration
await invoke('update_ai_config', {
  config: {
    default_provider: 'local',
    privacy_level: 'balanced',
    auto_select_models: true,
    max_memory_mb: 4096
  }
});

// Model-specific config
await invoke('update_model_config', {
  modelId: 'whisper-small-en',
  config: {
    max_memory_mb: 1024,
    thread_count: 4,
    use_gpu: true,
    enable_profiling: false
  }
});
```

## Plugin Development

### Creating Custom AI Provider

```rust
use bestme::ai::{AIProvider, AIError, EnhancedText, EnhancementOptions};
use async_trait::async_trait;

pub struct MyCustomProvider {
    // Your fields
}

#[async_trait]
impl AIProvider for MyCustomProvider {
    async fn initialize(&mut self) -> Result<(), AIError> {
        // Initialize your provider
        Ok(())
    }
    
    async fn enhance_text(
        &self,
        text: &str,
        options: &EnhancementOptions,
    ) -> Result<EnhancedText, AIError> {
        // Implement text enhancement
        Ok(EnhancedText {
            original: text.to_string(),
            enhanced: text.to_string(),
            confidence: 1.0,
            intent: None,
            changes: vec![],
            metadata: HashMap::new(),
        })
    }
    
    fn get_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "My Custom Provider".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["enhance".to_string()],
        }
    }
    
    async fn is_available(&self) -> bool {
        true
    }
    
    async fn get_usage(&self) -> Result<UsageMetrics, AIError> {
        Ok(UsageMetrics::default())
    }
}
```

### Registering Custom Provider

```rust
use bestme::ai::registry::ProviderRegistry;

let registry = ProviderRegistry::global();
registry.register_provider(
    "my_custom",
    Box::new(MyCustomProvider::new())
).await?;
```

## Performance Considerations

### Memory Management

```rust
// Configure memory limits
ModelConfig {
    max_memory_mb: 2048,
    enable_memory_pattern: true,
    memory_growth_factor: 1.5,
    ..Default::default()
}
```

### Batch Processing

```rust
// Process multiple texts efficiently
let texts = vec!["text1", "text2", "text3"];
let results = futures::future::join_all(
    texts.into_iter().map(|text| {
        provider.enhance_text(text, &options)
    })
).await;
```

### Caching

```rust
// Enable caching for model selection
ModelSelectorConfig {
    enable_cache: true,
    cache_ttl_seconds: 3600,
    max_cache_entries: 1000,
    ..Default::default()
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_text_enhancement() {
        let provider = LocalAI::new().unwrap();
        provider.initialize().await.unwrap();
        
        let result = provider.enhance_text(
            "this is test",
            &EnhancementOptions::default()
        ).await.unwrap();
        
        assert_eq!(result.enhanced, "This is a test.");
        assert!(result.confidence > 0.8);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_model_import_workflow() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let manager = CustomModelManager::new(
            Arc::new(ModelRegistry::new()),
            PathBuf::from("./test_models")
        ).unwrap();
        
        let result = manager.import_model(
            Path::new("test_model.onnx"),
            ImportOptions::default()
        ).await;
        
        assert!(result.success);
    });
}
```

## Debugging

### Enable Debug Logging

```rust
env_logger::Builder::from_env(
    env_logger::Env::default()
        .default_filter_or("bestme::ai=debug")
).init();
```

### Performance Profiling

```rust
// Enable profiling
let config = ModelConfig {
    enable_profiling: true,
    profile_output_path: Some("/tmp/ai_profile.json".into()),
    ..Default::default()
};

// Get profiling results
let profile = model_service.get_profiling_data(model_id).await?;
```