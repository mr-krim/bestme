# AI Features Documentation

## Overview

BestMe includes comprehensive AI capabilities for text enhancement, model management, and intelligent transcription processing. The AI system supports both local (offline) and cloud-based models, with automatic selection based on text characteristics and performance requirements.

## Table of Contents

1. [Architecture](#architecture)
2. [Core Features](#core-features)
3. [API Reference](#api-reference)
4. [UI Components](#ui-components)
5. [Configuration](#configuration)
6. [Usage Examples](#usage-examples)
7. [Custom Models](#custom-models)
8. [Performance Optimization](#performance-optimization)

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (Svelte)                      │
├─────────────────────────────────────────────────────────────┤
│                    Tauri Command Interface                    │
├─────────────────────────────────────────────────────────────┤
│                      AI Plugin (Rust)                         │
├────────────────┬────────────────┬────────────────┬──────────┤
│   Local AI     │   Cloud AI     │ Model Service  │ Custom   │
│   (ONNX)       │   Providers    │   Registry     │ Models   │
└────────────────┴────────────────┴────────────────┴──────────┘
```

### Key Services

1. **AIProvider** - Abstract interface for AI implementations
2. **LocalAI** - ONNX Runtime-based local inference
3. **CloudAI** - Multi-provider cloud API integration
4. **ModelService** - Model lifecycle management
5. **ModelSelector** - Intelligent model selection
6. **CustomModelManager** - Custom ONNX model support

## Core Features

### 1. Text Enhancement

Enhance transcribed text with AI-powered corrections and improvements.

**Capabilities:**
- Grammar correction
- Punctuation improvement
- Intent detection
- Style preservation
- Confidence scoring

**Example:**
```javascript
const result = await invoke('enhance_text', {
  request: {
    text: "this is a test sentense with erors",
    options: {
      correct_grammar: true,
      improve_punctuation: true,
      detect_intent: true,
      preserve_style: true,
      confidence_threshold: 0.7
    },
    use_cloud: false
  }
});
// Returns: {
//   original: "this is a test sentense with erors",
//   enhanced: "This is a test sentence with errors.",
//   confidence: 0.92,
//   intent: "statement",
//   changes: [...]
// }
```

### 2. Model Management

#### Available Models

**Local Models:**
- `whisper-tiny-en` - 39M parameters, fastest
- `whisper-base-en` - 74M parameters, balanced
- `whisper-small-en` - 244M parameters, accurate
- `bert-base-uncased` - 110M parameters, NLP tasks
- `t5-small` - 60M parameters, text generation

**Cloud Providers:**
- OpenRouter (Claude, GPT-4, etc.)
- Requesty (Multiple models)
- OpenAI (GPT-3.5, GPT-4)

#### Model Operations

```javascript
// List available models
const models = await invoke('list_available_models');

// Download a model
await invoke('download_model', { modelName: 'whisper-small-en' });

// Load model into memory
await invoke('load_ai_model', { modelId: 'whisper-small-en' });

// Get model performance metrics
const metrics = await invoke('get_model_performance', { 
  modelId: 'whisper-small-en' 
});
```

### 3. Automatic Model Selection

The system can automatically select the best model based on text characteristics:

```javascript
const result = await invoke('auto_select_model', {
  text: "Complex technical document about quantum computing...",
  performanceTarget: 'balanced' // 'fast', 'balanced', or 'quality'
});
// Returns: {
//   model_id: "bert-base-uncased",
//   reason: "Technical content with domain-specific terminology",
//   confidence_score: 0.85
// }
```

### 4. Performance Monitoring

Track AI performance metrics in real-time:

```javascript
// Get current metrics
const metrics = await invoke('get_current_ai_metrics');
// Returns: {
//   inference_time_ms: 45.2,
//   tokens_processed: 156,
//   memory_usage_mb: 234.5,
//   gpu_utilization: 0.65,
//   cache_hit_rate: 0.72
// }

// Get performance summary
const summary = await invoke('get_ai_performance_summary', {
  modelId: 'whisper-small-en',
  timeRange: 'last_hour' // 'last_hour', 'last_day', 'last_week'
});
```

### 5. Model Updates

Keep models up-to-date with automatic version checking:

```javascript
// Check for updates
const updates = await invoke('check_model_updates', { force: false });
// Returns: {
//   available_updates: [...],
//   last_check: "2024-01-15T10:30:00Z",
//   next_check: "2024-01-16T10:30:00Z"
// }

// Download update
await invoke('download_model_update', {
  update: {
    model_id: "whisper-small-en",
    current_version: "1.0.0",
    latest_version: "1.1.0",
    // ...
  }
});
```

## API Reference

### Tauri Commands

#### AI Initialization

```rust
// Initialize local AI
init_local_ai(request: InitLocalAIRequest) -> Result<String, String>

// Initialize cloud AI
init_cloud_ai(request: InitCloudAIRequest) -> Result<String, String>
```

#### Text Processing

```rust
// Enhance text
enhance_text(request: EnhanceTextRequest) -> Result<EnhancedText, String>

// Summarize text
summarize_text(text: String, max_length: Option<usize>) -> Result<String, String>

// Transform text style
transform_style(text: String, style: String) -> Result<String, String>
```

#### Model Management

```rust
// List models
list_available_models() -> Result<Vec<ModelMetadata>, String>
list_downloaded_models() -> Result<Vec<String>, String>
list_loaded_models() -> Result<Vec<String>, String>

// Model operations
download_model(model_name: String) -> Result<String, String>
load_ai_model(model_id: String) -> Result<String, String>
unload_ai_model(model_id: String) -> Result<String, String>

// Model configuration
get_model_config(model_id: String) -> Result<Value, String>
update_model_config(model_id: String, config: Value) -> Result<String, String>
```

#### Custom Models

```rust
// Validate ONNX model
validate_onnx_model(path: String) -> Result<ValidationResult, String>

// Import custom model
import_custom_model(path: String, options: ImportOptions) -> Result<ImportResult, String>

// Manage custom models
list_custom_models() -> Result<Vec<CustomModel>, String>
update_custom_model(model_id: String, updates: Value) -> Result<String, String>
delete_custom_model(model_id: String) -> Result<String, String>
export_custom_model(model_id: String, export_path: String) -> Result<String, String>
```

## UI Components

### AISettings Component

Main settings interface with tabs for different AI features:

```svelte
<AISettings>
  <Tab name="provider">Provider Settings</Tab>
  <Tab name="models">Model Management</Tab>
  <Tab name="performance">Performance Monitor</Tab>
  <Tab name="config">Configuration</Tab>
  <Tab name="auto">Auto-Selection</Tab>
  <Tab name="updates">Update Manager</Tab>
  <Tab name="custom">Custom Models</Tab>
</AISettings>
```

### Individual Components

1. **ModelSelector** - Interactive grid for model selection
2. **PerformanceMonitor** - Real-time performance metrics
3. **ModelDownloadProgress** - Download tracking
4. **ModelConfigPanel** - Model-specific settings
5. **AutoModelSelector** - Automatic selection interface
6. **ModelUpdateManager** - Update checking and management
7. **CustomModelImporter** - ONNX import wizard
8. **CustomModelManager** - Custom model management

## Configuration

### Config Structure

```toml
[ai]
# Default provider: 'local', 'openrouter', 'requesty', 'openai'
default_provider = "local"

# Privacy level: 'strict', 'balanced', 'permissive'
privacy_level = "balanced"

# Model selection
[ai.model_selection]
auto_select = true
performance_target = "balanced"
cache_selections = true

# Local AI settings
[ai.local]
model_path = "~/.bestme/models"
use_gpu = true
max_memory_mb = 4096
thread_count = 4

# Cloud providers
[ai.cloud.openrouter]
api_key = "" # Stored securely in keychain
base_url = "https://openrouter.ai/api/v1"
model = "anthropic/claude-3-sonnet"

[ai.cloud.requesty]
api_key = ""
model = "gpt-4-turbo"

[ai.cloud.openai]
api_key = ""
model = "gpt-4-turbo-preview"
```

### Environment Variables

```bash
# Enable debug logging
RUST_LOG=bestme::ai=debug

# Force CPU inference
BESTME_AI_FORCE_CPU=1

# Custom model directory
BESTME_MODEL_PATH=/path/to/models

# Disable telemetry
BESTME_AI_TELEMETRY=0
```

## Usage Examples

### Basic Text Enhancement

```javascript
// Simple enhancement
const enhanced = await invoke('enhance_text', {
  request: {
    text: transcribedText,
    options: {
      correct_grammar: true,
      improve_punctuation: true
    },
    use_cloud: false
  }
});

// With cloud provider
const cloudEnhanced = await invoke('enhance_text', {
  request: {
    text: transcribedText,
    options: {
      correct_grammar: true,
      improve_punctuation: true,
      detect_intent: true
    },
    use_cloud: true
  }
});
```

### Model Selection Workflow

```javascript
// 1. Analyze text characteristics
const characteristics = await invoke('analyze_text_characteristics', {
  text: inputText
});

// 2. Get model requirements
const requirements = await invoke('get_model_requirements', {
  text: inputText
});

// 3. Auto-select best model
const selection = await invoke('auto_select_model', {
  text: inputText,
  performanceTarget: 'quality'
});

// 4. Load selected model
await invoke('load_ai_model', {
  modelId: selection.model_id
});

// 5. Process text
const result = await invoke('enhance_text', {
  request: {
    text: inputText,
    options: defaultOptions,
    use_cloud: false
  }
});
```

### Custom Model Import

```javascript
// 1. Validate ONNX model
const validation = await invoke('validate_onnx_model', {
  path: '/path/to/model.onnx'
});

if (validation.is_valid) {
  // 2. Import model
  const importResult = await invoke('import_custom_model', {
    path: '/path/to/model.onnx',
    options: {
      name: 'My Custom Model',
      description: 'Fine-tuned BERT for domain-specific tasks',
      model_type: 'TextClassification',
      tags: ['custom', 'bert', 'domain-specific'],
      validate_thoroughly: true,
      auto_generate_metadata: true
    }
  });

  // 3. Load and use
  if (importResult.success) {
    await invoke('load_ai_model', {
      modelId: importResult.model_id
    });
  }
}
```

## Custom Models

### Supported Formats

- ONNX models (`.onnx` files)
- Opset version 11-17
- Dynamic or static input shapes
- Quantized models (INT8, FP16)

### Model Types

1. **Text Generation** - Decoder-only or encoder-decoder models
2. **Text Classification** - Sentiment, category, intent classification
3. **Token Classification** - NER, POS tagging
4. **Seq2Seq** - Translation, summarization

### Validation Process

1. **File validation** - Size, format, structure
2. **Model inspection** - Inputs, outputs, operators
3. **Compatibility check** - ONNX version, operators, memory
4. **Performance estimate** - Size, parameters, memory usage

### Best Practices

1. **Model Size** - Keep under 1GB for best performance
2. **Input Names** - Use standard names (input_ids, attention_mask)
3. **Quantization** - Use INT8 for 4x smaller size
4. **Metadata** - Include model card with capabilities

## Performance Optimization

### GPU Acceleration

Supported backends:
- CUDA (NVIDIA GPUs)
- Metal (Apple Silicon)
- DirectML (Windows)
- ROCm (AMD GPUs)

Enable GPU:
```javascript
await invoke('init_local_ai', {
  request: {
    model_name: 'whisper-small-en',
    use_gpu: true,
    quantization: null
  }
});
```

### Memory Management

```javascript
// Configure memory limits
await invoke('update_model_config', {
  modelId: 'whisper-small-en',
  config: {
    max_memory_mb: 2048,
    enable_memory_pattern: true,
    graph_optimization_level: 'all'
  }
});
```

### Caching

The system automatically caches:
- Model selection decisions
- Frequently enhanced phrases
- Performance metrics
- Validation results

Clear cache:
```javascript
await invoke('clear_model_selection_cache');
await invoke('clear_update_cache');
```

### Batch Processing

For multiple texts:
```javascript
const texts = ["text1", "text2", "text3"];
const results = await Promise.all(
  texts.map(text => 
    invoke('enhance_text', {
      request: { text, options, use_cloud: false }
    })
  )
);
```

## Troubleshooting

### Common Issues

1. **Model download fails**
   - Check internet connection
   - Verify disk space (models can be large)
   - Check firewall settings

2. **GPU not detected**
   - Install CUDA/Metal drivers
   - Check GPU compatibility
   - Try `BESTME_AI_FORCE_CPU=1`

3. **High memory usage**
   - Reduce max_memory_mb
   - Use smaller models
   - Enable quantization

4. **Slow inference**
   - Enable GPU acceleration
   - Use quantized models
   - Check model selection settings

### Debug Commands

```bash
# Check AI system status
cargo run -- --debug-ai

# List GPU devices
cargo run -- --list-gpus

# Validate model file
cargo run -- --validate-model /path/to/model.onnx

# Benchmark performance
cargo run -- --benchmark-ai
```

## Future Enhancements

1. **Additional AI Capabilities**
   - Real-time translation
   - Speaker diarization
   - Emotion detection
   - Code detection and formatting

2. **Model Improvements**
   - Fine-tuning interface
   - Model compression tools
   - Federated learning support
   - Model ensemble support

3. **Integration Features**
   - Plugin API for custom processors
   - Export to popular formats
   - Integration with external AI services
   - Webhook support for processing

4. **Performance Optimizations**
   - Model pruning
   - Distributed inference
   - Edge deployment
   - WebAssembly support