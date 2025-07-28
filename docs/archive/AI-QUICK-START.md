# AI Quick Start Guide

Get started with BestMe's AI features in just a few minutes!

## Prerequisites

- BestMe application installed
- (Optional) API keys for cloud providers
- (Optional) CUDA/Metal drivers for GPU acceleration

## Basic Setup

### 1. Initialize Local AI (Offline)

```javascript
// In your Svelte component
import { invoke } from '@tauri-apps/api/core';

// Initialize with a small, fast model
await invoke('init_local_ai', {
  request: {
    model_name: 'whisper-tiny-en',
    use_gpu: true,
    quantization: null
  }
});
```

### 2. Enhance Transcribed Text

```javascript
// Simple text enhancement
const result = await invoke('enhance_text', {
  request: {
    text: "this is my transcribed text with some erors",
    options: {
      correct_grammar: true,
      improve_punctuation: true,
      detect_intent: false,
      preserve_style: true,
      confidence_threshold: 0.7
    },
    use_cloud: false
  }
});

console.log(result.enhanced); // "This is my transcribed text with some errors."
```

## Common Use Cases

### Real-time Transcription Enhancement

```javascript
// Listen for transcription updates
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('transcription:segment', async (event) => {
  const { text, is_final } = event.payload;
  
  if (is_final) {
    // Enhance final segments only
    const enhanced = await invoke('enhance_text', {
      request: {
        text,
        options: {
          correct_grammar: true,
          improve_punctuation: true,
          preserve_style: true
        },
        use_cloud: false
      }
    });
    
    // Update UI with enhanced text
    updateTranscription(enhanced.enhanced);
  }
});
```

### Automatic Model Selection

```javascript
// Let AI choose the best model for your text
const selection = await invoke('auto_select_model', {
  text: "Complex technical documentation about machine learning...",
  performanceTarget: 'balanced'
});

// Load the selected model
await invoke('load_ai_model', { modelId: selection.model_id });

// Now enhance with the optimal model
const enhanced = await invoke('enhance_text', {
  request: { text, options, use_cloud: false }
});
```

### Cloud AI Integration

```javascript
// 1. Save API key (one-time setup)
await invoke('save_api_key', {
  provider: 'openrouter',
  apiKey: 'your-api-key-here'
});

// 2. Initialize cloud AI
await invoke('init_cloud_ai', {
  request: {
    provider: 'openrouter',
    api_key: '', // Will use saved key
    model: 'anthropic/claude-3-sonnet',
    privacy_level: 'balanced',
    base_url: null
  }
});

// 3. Use cloud enhancement
const cloudResult = await invoke('enhance_text', {
  request: {
    text: "your text here",
    options: defaultOptions,
    use_cloud: true
  }
});
```

## UI Integration

### Using the AISettings Component

```svelte
<script>
  import AISettings from './components/AISettings.svelte';
  
  let showAISettings = false;
</script>

<button on:click={() => showAISettings = true}>
  AI Settings
</button>

{#if showAISettings}
  <AISettings />
{/if}
```

### Custom Model Selector

```svelte
<script>
  import { ModelSelector } from './components/ai';
  
  let selectedModelId = '';
  
  function handleModelSelect(event) {
    selectedModelId = event.detail.modelId;
    // Load the selected model
    invoke('load_ai_model', { modelId: selectedModelId });
  }
</script>

<ModelSelector 
  bind:selectedModelId
  on:select={handleModelSelect}
/>
```

### Performance Monitor

```svelte
<script>
  import { PerformanceMonitor } from './components/ai';
  
  let currentModelId = 'whisper-small-en';
</script>

<!-- Show real-time performance metrics -->
<PerformanceMonitor modelId={currentModelId} />
```

## Performance Tips

### 1. Choose the Right Model

```javascript
// For real-time processing (fast)
await invoke('load_ai_model', { modelId: 'whisper-tiny-en' });

// For accuracy (slower)
await invoke('load_ai_model', { modelId: 'whisper-small-en' });

// For best quality (slowest)
await invoke('load_ai_model', { modelId: 'bert-base-uncased' });
```

### 2. Enable GPU Acceleration

```javascript
// Check GPU availability
const gpuInfo = await invoke('get_gpu_info');
console.log('GPU available:', gpuInfo.available);

// Initialize with GPU
await invoke('init_local_ai', {
  request: {
    model_name: 'whisper-small-en',
    use_gpu: true,
    quantization: null
  }
});
```

### 3. Use Caching

```javascript
// The system automatically caches:
// - Model selection decisions
// - Frequently enhanced phrases
// - Performance metrics

// Clear cache if needed
await invoke('clear_model_selection_cache');
```

## Custom Model Import

### Quick Import

```javascript
// 1. Select ONNX file
const { open } = await import('@tauri-apps/api/dialog');
const modelPath = await open({
  filters: [{ name: 'ONNX Model', extensions: ['onnx'] }]
});

// 2. Import with auto-config
if (modelPath) {
  const result = await invoke('import_custom_model', {
    path: modelPath,
    options: {
      name: 'My Custom Model',
      description: 'Fine-tuned for my use case',
      model_type: 'TextGeneration',
      tags: ['custom'],
      validate_thoroughly: true,
      auto_generate_metadata: true
    }
  });
  
  if (result.success) {
    console.log('Model imported:', result.model_id);
  }
}
```

## Error Handling

```javascript
try {
  const result = await invoke('enhance_text', {
    request: { text, options, use_cloud: false }
  });
  // Handle success
} catch (error) {
  console.error('Enhancement failed:', error);
  
  // Common errors:
  // - "Model not loaded" - Load a model first
  // - "GPU not available" - Fall back to CPU
  // - "Out of memory" - Use smaller model
  
  // Fallback to original text
  displayText(text);
}
```

## Best Practices

### 1. Initialize Once

```javascript
// Store initialization state
let aiInitialized = false;

async function initializeAI() {
  if (aiInitialized) return;
  
  await invoke('init_local_ai', {
    request: {
      model_name: 'whisper-tiny-en',
      use_gpu: true,
      quantization: null
    }
  });
  
  aiInitialized = true;
}
```

### 2. Handle Offline Mode

```javascript
// Check if using local AI
const settings = await invoke('get_settings');
const useCloud = settings.ai.default_provider !== 'local';

// Enhance with fallback
let enhanced;
try {
  enhanced = await invoke('enhance_text', {
    request: { text, options, use_cloud: useCloud }
  });
} catch (error) {
  if (useCloud) {
    // Fallback to local
    enhanced = await invoke('enhance_text', {
      request: { text, options, use_cloud: false }
    });
  } else {
    throw error;
  }
}
```

### 3. Monitor Performance

```javascript
// Get metrics after processing
const metrics = await invoke('get_current_ai_metrics');

if (metrics.inference_time_ms > 100) {
  console.warn('Slow inference detected:', metrics);
  // Consider using a faster model
}
```

## Next Steps

1. **Explore Advanced Features**
   - Read the [full AI documentation](./AI-FEATURES.md)
   - Check the [API reference](./AI-API-REFERENCE.md)

2. **Optimize Performance**
   - Enable GPU acceleration
   - Use quantized models
   - Implement batch processing

3. **Customize**
   - Import your own ONNX models
   - Configure model selection rules
   - Set up cloud provider fallbacks

## Need Help?

- Check the [troubleshooting section](./AI-FEATURES.md#troubleshooting)
- View [example code](https://github.com/mr-krim/bestme/tree/main/examples)
- Report issues on [GitHub](https://github.com/mr-krim/bestme/issues)