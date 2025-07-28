# BestMe AI Guide

This guide consolidates all AI-related documentation for the BestMe project.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Architecture](#architecture)
4. [Features](#features)
5. [Model Management](#model-management)
6. [API Reference](#api-reference)
7. [Performance Optimization](#performance-optimization)
8. [Troubleshooting](#troubleshooting)

## Overview

BestMe includes a comprehensive AI system that enhances speech-to-text transcription with advanced text processing capabilities. The system supports both local (ONNX) and cloud-based AI providers.

### Key Capabilities

- **Text Enhancement**: Grammar correction, punctuation, style transformation
- **Multi-Provider Support**: Local ONNX models and cloud providers (OpenRouter, OpenAI, Requesty)
- **GPU Acceleration**: CUDA, Metal, and DirectML support
- **Intelligent Model Selection**: Automatic model selection based on text characteristics
- **Custom Models**: Import and manage your own ONNX models
- **Real-time Processing**: Streaming inference with <300ms latency

## Quick Start

### 1. Enable AI Features

```rust
// Initialize local AI with a model
cargo tauri dev -- --ai-model phi-3-mini

// Or configure in settings.json
{
  "ai": {
    "enabled": true,
    "provider": "local",
    "model": "phi-3-mini"
  }
}
```

### 2. Using Cloud Providers

```javascript
// Configure cloud AI in the UI
await invoke('init_cloud_ai', {
  request: {
    provider: 'openrouter',
    api_key: 'your-api-key',
    model: 'meta-llama/llama-3.2-3b-instruct',
    privacy_level: 'balanced'
  }
});
```

### 3. Text Enhancement

```javascript
// Enhance transcribed text
const enhanced = await invoke('enhance_text', {
  request: {
    text: "hello world how are u",
    options: {
      fix_grammar: true,
      add_punctuation: true,
      detect_intent: false
    },
    use_cloud: false
  }
});
// Result: "Hello world, how are you?"
```

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────┐
│                    UI Layer (Svelte)                     │
├─────────────────────────────────────────────────────────┤
│                 Tauri Command Interface                   │
├─────────────────────────────────────────────────────────┤
│                    AI Service Layer                       │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │Model Service│  │Model Selector│  │Update Checker │  │
│  └─────────────┘  └──────────────┘  └───────────────┘  │
├─────────────────────────────────────────────────────────┤
│                  AI Provider Layer                        │
│  ┌──────────┐  ┌──────────┐  ┌────────────────────┐    │
│  │Local ONNX│  │Cloud APIs│  │Custom Model Manager│    │
│  └──────────┘  └──────────┘  └────────────────────┘    │
├─────────────────────────────────────────────────────────┤
│              Hardware Acceleration Layer                  │
│  ┌──────┐  ┌───────┐  ┌──────────┐  ┌─────────────┐   │
│  │ CUDA │  │ Metal │  │ DirectML │  │CPU Fallback │   │
│  └──────┘  └───────┘  └──────────┘  └─────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Key Components

1. **Model Service** (`/src/ai/services/model_service.rs`)
   - Central hub for model management
   - Handles loading, unloading, and configuration
   - Manages model lifecycle and resources

2. **ONNX Runtime** (`/src/ai/local/onnx_runtime.rs`)
   - Executes local AI models
   - Supports multiple architectures (Seq2Seq, CausalLM, Token Classification)
   - Handles tokenization and inference

3. **Model Selector** (`/src/ai/services/model_selector.rs`)
   - Intelligently selects best model for given text
   - Analyzes text characteristics
   - Considers performance metrics and capabilities

4. **Resilience System** (`/src/ai/resilience/`)
   - Multi-level fallback: GPU → CPU → Cache → Error
   - Circuit breaker pattern for fault tolerance
   - Automatic recovery mechanisms

## Features

### 1. Text Enhancement

Enhance transcribed text with various options:

```rust
pub struct EnhancementOptions {
    pub fix_grammar: bool,
    pub add_punctuation: bool,
    pub improve_clarity: bool,
    pub detect_intent: bool,
    pub preserve_style: bool,
    pub target_formality: Option<String>,
}
```

### 2. Model Management

#### Available Pre-configured Models

- **phi-3-mini**: Fast, lightweight model for basic enhancements
- **llama-3.2-1b**: Balanced performance and quality
- **grammar-t5-base**: Specialized for grammar correction
- **flan-t5-small**: Good for various NLP tasks

#### Model Operations

```javascript
// List available models
const models = await invoke('list_available_models');

// Download a model
await invoke('download_model', { modelId: 'phi-3-mini' });

// Load a model into memory
await invoke('load_ai_model', { modelId: 'phi-3-mini' });

// Get model performance metrics
const metrics = await invoke('get_model_performance', { modelId: 'phi-3-mini' });
```

### 3. Custom Model Support

Import your own ONNX models:

```javascript
const result = await invoke('import_custom_model', {
  path: '/path/to/model.onnx',
  options: {
    name: 'My Custom Model',
    description: 'Custom grammar correction model',
    model_type: 'Seq2Seq',
    tags: ['grammar', 'custom'],
    validate_thoroughly: true,
    auto_generate_metadata: true
  }
});
```

### 4. Automatic Model Selection

The system can automatically select the best model based on text characteristics:

```javascript
const selection = await invoke('auto_select_model', {
  text: "Your text here..."
});
// Returns: { selected_model, capability_score, performance_score, selection_reason }
```

### 5. Performance Monitoring

Real-time performance metrics:

```javascript
// Get current metrics
const metrics = await invoke('get_current_ai_metrics', { modelId });

// Get performance summary
const summary = await invoke('get_ai_performance_summary', { modelId });
```

## Model Management

### Model Lifecycle

1. **Discovery**: Models are listed from the registry
2. **Download**: Models are downloaded to local storage
3. **Validation**: ONNX models are validated for compatibility
4. **Loading**: Models are loaded into memory when needed
5. **Optimization**: Models are optimized based on hardware
6. **Caching**: Frequently used phrases are cached

### Storage Locations

Models are stored in platform-specific directories:

- **Windows**: `%APPDATA%\bestme\models\`
- **macOS**: `~/Library/Application Support/bestme/models/`
- **Linux**: `~/.local/share/bestme/models/`

### Model Updates

The system automatically checks for model updates:

```javascript
// Check for updates
const updates = await invoke('check_model_updates', { force: false });

// Download an update
await invoke('download_model_update', { update });
```

## API Reference

### Core Commands

#### Text Enhancement
```typescript
enhance_text(request: {
  text: string,
  options: EnhancementOptions,
  use_cloud: boolean
}): Promise<EnhancedText>
```

#### Model Management
```typescript
// List models
list_available_models(): Promise<ModelInfo[]>
list_downloaded_models(): Promise<string[]>
list_loaded_models(): Promise<ModelInfo[]>

// Model operations
download_model(modelId: string): Promise<void>
load_ai_model(modelId: string): Promise<void>
unload_ai_model(modelId: string): Promise<void>

// Performance
get_model_performance(modelId: string): Promise<ModelPerformance>
get_ai_performance_summary(modelId: string): Promise<PerformanceSummary>
```

#### Cloud AI
```typescript
// Initialize
init_cloud_ai(request: CloudAIRequest): Promise<void>

// Operations
summarize_text(text: string, maxLength: number): Promise<string>
translate_text(text: string, targetLanguage: string): Promise<string>
transform_style(text: string, targetStyle: string): Promise<string>
```

### Events

The system emits various events for real-time updates:

- `model-download-progress`: Download progress updates
- `model-download-complete`: Download completion
- `ai-performance-metrics`: Real-time performance data

## Performance Optimization

### GPU Acceleration

The system automatically detects and uses available GPU:

1. **CUDA** (NVIDIA GPUs)
2. **Metal** (Apple Silicon)
3. **DirectML** (Windows)
4. **CPU Fallback** (when GPU unavailable)

### Optimization Techniques

1. **Model Quantization**: Reduce model size by 30-75%
   - INT8 quantization for balanced quality
   - FP16 for minimal quality loss

2. **Batch Processing**: Process multiple texts efficiently
   ```javascript
   const results = await invoke('batch_enhance_texts', {
     texts: ["text1", "text2", "text3"],
     options: enhancementOptions
   });
   ```

3. **Caching**: LRU cache for common phrases
   - 1000 entry default capacity
   - Configurable TTL
   - 60-80% hit rate for common text

4. **Model Warmup**: Pre-compute common operations
   - Automatic on model load
   - Common phrase precomputation
   - Reduces first-inference latency

### Performance Targets

- **Inference Latency**: <50ms with GPU, <200ms with CPU
- **Throughput**: 100+ tokens/second on GPU
- **Memory Usage**: <4GB for typical usage
- **Model Loading**: <5 seconds for large models

## Troubleshooting

### Common Issues

1. **Model Download Fails**
   - Check internet connection
   - Verify disk space (models can be 100MB-2GB)
   - Check firewall settings

2. **GPU Not Detected**
   - Ensure drivers are updated
   - CUDA toolkit required for NVIDIA
   - Check GPU compatibility

3. **High Memory Usage**
   - Unload unused models
   - Enable quantization
   - Reduce batch size

4. **Slow Performance**
   - Check if GPU is being used
   - Enable model optimization
   - Use smaller models for simple tasks

### Debug Commands

```bash
# Enable debug logging
RUST_LOG=debug cargo tauri dev

# Check GPU status
cargo run -- --check-gpu

# Benchmark models
cargo run -- --benchmark-models
```

### Error Recovery

The system includes automatic error recovery:

1. **Fallback Chain**: GPU → CPU → Cache → Graceful error
2. **Circuit Breaker**: Prevents cascading failures
3. **Automatic Retry**: With exponential backoff
4. **Resource Management**: Automatic cleanup on errors

## Advanced Usage

### Custom Providers

Implement custom AI providers:

```rust
impl AIProvider for MyCustomProvider {
    async fn enhance_text(&self, text: &str, options: &EnhancementOptions) -> Result<EnhancedText> {
        // Your implementation
    }
}
```

### Model Benchmarking

Run comprehensive benchmarks:

```bash
cargo test --release -- --ignored benchmark
```

### Integration with Voice Commands

AI enhancement can be triggered by voice:

```javascript
// Configure in voice command settings
{
  "commands": [
    {
      "phrase": "enhance text",
      "action": "ai_enhance",
      "ai_options": {
        "fix_grammar": true,
        "add_punctuation": true
      }
    }
  ]
}
```

## Future Roadmap

1. **Additional Models**: More specialized models for different domains
2. **Fine-tuning**: Support for model fine-tuning on user data
3. **Multi-language**: Expanded language support
4. **Edge Deployment**: Optimized models for edge devices
5. **Federation**: Distributed model serving

---

For more details on specific components, see the source code documentation in `/src/ai/`.