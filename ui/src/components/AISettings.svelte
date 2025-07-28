<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { ModelSelector, PerformanceMonitor, ModelDownloadProgress, ModelConfigPanel, AutoModelSelector, ModelUpdateManager, CustomModelImporter, CustomModelManager } from './ai';

  // AI Provider types
  type AIProvider = 'openrouter' | 'requesty' | 'openai' | 'local';
  type PrivacyLevel = 'strict' | 'balanced' | 'permissive';

  // State
  let selectedProvider: AIProvider = 'local';
  let apiKey = '';
  let selectedModel = '';
  let privacyLevel: PrivacyLevel = 'balanced';
  let savedProviders: string[] = [];
  let availableModels: any[] = [];
  let downloadedModels: string[] = [];
  let isLoading = false;
  let message = '';
  let downloadProgress: { [key: string]: number } = {};
  let activeTab: 'provider' | 'models' | 'performance' | 'config' | 'auto' | 'updates' | 'custom' = 'provider';
  let showAdvancedComponents = false;

  // Privacy level descriptions
  const privacyDescriptions = {
    strict: 'All processing happens locally. No data is sent to cloud services.',
    balanced: 'Sensitive information is anonymized before cloud processing.',
    permissive: 'Data may be sent to cloud services with minimal restrictions.'
  };

  onMount(async () => {
    await loadSavedProviders();
    await loadAvailableModels();
    await loadDownloadedModels();
  });

  async function loadSavedProviders() {
    try {
      savedProviders = await invoke('list_saved_api_keys');
    } catch (error) {
      console.error('Failed to load saved providers:', error);
    }
  }

  async function loadAvailableModels() {
    try {
      availableModels = await invoke('list_available_models');
    } catch (error) {
      console.error('Failed to load available models:', error);
    }
  }

  async function loadDownloadedModels() {
    try {
      downloadedModels = await invoke('list_downloaded_models');
    } catch (error) {
      console.error('Failed to load downloaded models:', error);
    }
  }

  async function saveApiKey() {
    if (!apiKey || selectedProvider === 'local') {
      message = 'Please enter an API key';
      return;
    }

    isLoading = true;
    try {
      await invoke('save_api_key', {
        provider: selectedProvider,
        apiKey
      });
      message = 'API key saved securely';
      apiKey = '';
      await loadSavedProviders();
    } catch (error) {
      message = `Failed to save API key: ${error}`;
    } finally {
      isLoading = false;
    }
  }

  async function deleteApiKey(provider: string) {
    if (!confirm(`Delete API key for ${provider}?`)) {
      return;
    }

    try {
      await invoke('delete_api_key', { provider });
      await loadSavedProviders();
      message = `API key for ${provider} deleted`;
    } catch (error) {
      message = `Failed to delete API key: ${error}`;
    }
  }

  async function initializeAI() {
    isLoading = true;
    try {
      if (selectedProvider === 'local') {
        if (!selectedModel) {
          message = 'Please select a model';
          return;
        }
        await invoke('init_local_ai', {
          request: {
            model_name: selectedModel,
            use_gpu: true,
            quantization: null
          }
        });
        message = 'Local AI initialized successfully';
      } else {
        if (!savedProviders.includes(selectedProvider)) {
          message = 'Please save an API key first';
          return;
        }
        await invoke('init_cloud_ai', {
          request: {
            provider: selectedProvider,
            api_key: '', // Will be loaded from keychain
            model: getDefaultModel(selectedProvider),
            privacy_level: privacyLevel,
            base_url: null
          }
        });
        message = 'Cloud AI initialized successfully';
      }
    } catch (error) {
      message = `Failed to initialize AI: ${error}`;
    } finally {
      isLoading = false;
    }
  }

  async function downloadModel(modelName: string) {
    isLoading = true;
    try {
      await invoke('download_model', { modelName });
      message = `Downloading ${modelName}...`;
      
      // Poll for download progress
      const interval = setInterval(async () => {
        try {
          // In a real implementation, you'd get progress from the backend
          downloadProgress[modelName] = (downloadProgress[modelName] || 0) + 10;
          if (downloadProgress[modelName] >= 100) {
            clearInterval(interval);
            await loadDownloadedModels();
            message = `${modelName} downloaded successfully`;
            delete downloadProgress[modelName];
          }
        } catch (error) {
          clearInterval(interval);
          message = `Download failed: ${error}`;
        }
      }, 1000);
    } catch (error) {
      message = `Failed to start download: ${error}`;
    } finally {
      isLoading = false;
    }
  }

  function getDefaultModel(provider: AIProvider): string {
    switch (provider) {
      case 'openrouter':
        return 'anthropic/claude-3-sonnet';
      case 'requesty':
        return 'gpt-4-turbo';
      case 'openai':
        return 'gpt-4-turbo-preview';
      default:
        return '';
    }
  }

  async function testEnhancement() {
    try {
      const result = await invoke('enhance_text', {
        request: {
          text: 'this is a test sentense with erors',
          options: {
            correct_grammar: true,
            improve_punctuation: true,
            detect_intent: true,
            preserve_style: true,
            confidence_threshold: 0.7
          },
          use_cloud: selectedProvider !== 'local'
        }
      });
      message = `Enhanced: ${JSON.stringify(result)}`;
    } catch (error) {
      message = `Enhancement failed: ${error}`;
    }
  }
</script>

<div class="ai-settings">
  <h2>AI Settings</h2>
  
  <!-- Tab Navigation -->
  <div class="tabs">
    <button 
      class="tab" 
      class:active={activeTab === 'provider'}
      on:click={() => activeTab = 'provider'}
    >
      Provider Settings
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'models'}
      on:click={() => activeTab = 'models'}
    >
      Model Management
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'performance'}
      on:click={() => activeTab = 'performance'}
    >
      Performance
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'config'}
      on:click={() => activeTab = 'config'}
    >
      Configuration
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'auto'}
      on:click={() => activeTab = 'auto'}
    >
      Auto-Selection
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'updates'}
      on:click={() => activeTab = 'updates'}
    >
      Updates
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'custom'}
      on:click={() => activeTab = 'custom'}
    >
      Custom Models
    </button>
  </div>
  
  {#if activeTab === 'provider'}
  <div class="provider-section">
    <h3>AI Provider</h3>
    <select bind:value={selectedProvider}>
      <option value="local">Local (Offline)</option>
      <option value="openrouter">OpenRouter</option>
      <option value="requesty">Requesty</option>
      <option value="openai">OpenAI</option>
    </select>
  </div>

  {#if selectedProvider !== 'local'}
    <div class="api-key-section">
      <h3>API Key</h3>
      <div class="api-key-input">
        <input
          type="password"
          bind:value={apiKey}
          placeholder="Enter API key"
          disabled={isLoading}
        />
        <button on:click={saveApiKey} disabled={isLoading}>
          Save Securely
        </button>
      </div>
      
      {#if savedProviders.length > 0}
        <div class="saved-providers">
          <h4>Saved Keys</h4>
          {#each savedProviders as provider}
            <div class="saved-provider">
              <span>{provider}</span>
              <button on:click={() => deleteApiKey(provider)}>Delete</button>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="privacy-section">
      <h3>Privacy Level</h3>
      <select bind:value={privacyLevel}>
        <option value="strict">Strict</option>
        <option value="balanced">Balanced</option>
        <option value="permissive">Permissive</option>
      </select>
      <p class="privacy-description">{privacyDescriptions[privacyLevel]}</p>
    </div>
  {:else}
    <div class="model-section">
      <h3>Local Models</h3>
      
      <div class="model-selection">
        <select bind:value={selectedModel}>
          <option value="">Select a model</option>
          {#each availableModels as model}
            <option value={model.name}>
              {model.name} ({model.parameters})
            </option>
          {/each}
        </select>
      </div>

      <div class="model-list">
        <h4>Available Models</h4>
        {#each availableModels as model}
          <div class="model-item">
            <div class="model-info">
              <strong>{model.name}</strong>
              <span>{model.parameters} parameters</span>
              <span>{(model.size_bytes / 1e9).toFixed(1)}GB</span>
            </div>
            {#if downloadedModels.includes(model.name)}
              <span class="downloaded">✓ Downloaded</span>
            {:else if downloadProgress[model.name]}
              <div class="download-progress">
                <progress value={downloadProgress[model.name]} max="100">
                  {downloadProgress[model.name]}%
                </progress>
              </div>
            {:else}
              <button on:click={() => downloadModel(model.name)} disabled={isLoading}>
                Download
              </button>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
  {/if}
  
  {#if activeTab === 'models'}
    <ModelSelector 
      bind:selectedModelId={selectedModel}
      on:select={(e) => selectedModel = e.detail.modelId}
    />
    <ModelDownloadProgress />
  {/if}
  
  {#if activeTab === 'performance' && selectedModel}
    <PerformanceMonitor modelId={selectedModel} />
  {:else if activeTab === 'performance'}
    <div class="empty-state">
      <p>Please select a model first to view performance metrics.</p>
    </div>
  {/if}
  
  {#if activeTab === 'config' && selectedModel}
    <ModelConfigPanel modelId={selectedModel} />
  {:else if activeTab === 'config'}
    <div class="empty-state">
      <p>Please select a model first to configure settings.</p>
    </div>
  {/if}
  
  {#if activeTab === 'auto'}
    <AutoModelSelector 
      onModelSelected={(modelId) => {
        selectedModel = modelId;
        activeTab = 'models';
      }}
      sampleText="This is a complex technical document about machine learning algorithms and neural network architectures. It contains mathematical formulas and code examples."
    />
  {/if}
  
  {#if activeTab === 'updates'}
    <ModelUpdateManager />
  {/if}
  
  {#if activeTab === 'custom'}
    <div class="custom-models-container">
      <CustomModelImporter />
      <div class="divider"></div>
      <CustomModelManager />
    </div>
  {/if}

  {#if activeTab === 'provider'}
  <div class="actions">
    <button on:click={initializeAI} disabled={isLoading} class="primary">
      Initialize AI
    </button>
    <button on:click={testEnhancement} disabled={isLoading}>
      Test Enhancement
    </button>
  </div>

  {#if message && activeTab === 'provider'}
    <div class="message" class:error={message.includes('Failed')}>
      {message}
    </div>
  {/if}
  {/if}
</div>

<style>
  .ai-settings {
    padding: 20px;
    max-width: 1200px;
  }
  
  .tabs {
    display: flex;
    gap: 10px;
    margin-bottom: 30px;
    border-bottom: 2px solid #e9ecef;
  }
  
  .tab {
    padding: 10px 20px;
    background: none;
    border: none;
    border-bottom: 3px solid transparent;
    color: #666;
    font-size: 1em;
    cursor: pointer;
    transition: all 0.3s;
  }
  
  .tab:hover {
    color: #333;
  }
  
  .tab.active {
    color: #007bff;
    border-bottom-color: #007bff;
  }
  
  .empty-state {
    text-align: center;
    padding: 60px 20px;
    color: #666;
  }
  
  .custom-models-container {
    display: flex;
    flex-direction: column;
    gap: 40px;
  }
  
  .divider {
    height: 2px;
    background: #e9ecef;
    margin: 20px 0;
  }

  h2 {
    margin-bottom: 20px;
  }

  h3 {
    margin-top: 20px;
    margin-bottom: 10px;
  }

  select, input {
    width: 100%;
    padding: 8px;
    margin-bottom: 10px;
    border: 1px solid #ccc;
    border-radius: 4px;
  }

  button {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    background-color: #007bff;
    color: white;
    cursor: pointer;
  }

  button:hover {
    background-color: #0056b3;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  button.primary {
    background-color: #28a745;
  }

  button.primary:hover {
    background-color: #218838;
  }

  .api-key-input {
    display: flex;
    gap: 10px;
  }

  .api-key-input input {
    flex: 1;
  }

  .saved-providers {
    margin-top: 20px;
  }

  .saved-provider {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px;
    background-color: #f8f9fa;
    margin-bottom: 5px;
    border-radius: 4px;
  }

  .privacy-description {
    font-size: 0.9em;
    color: #666;
    margin-top: 5px;
  }

  .model-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px;
    background-color: #f8f9fa;
    margin-bottom: 10px;
    border-radius: 4px;
  }

  .model-info {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .model-info span {
    font-size: 0.9em;
    color: #666;
  }

  .downloaded {
    color: #28a745;
    font-weight: bold;
  }

  .download-progress {
    width: 150px;
  }

  progress {
    width: 100%;
  }

  .actions {
    margin-top: 30px;
    display: flex;
    gap: 10px;
  }

  .message {
    margin-top: 20px;
    padding: 10px;
    border-radius: 4px;
    background-color: #d4edda;
    color: #155724;
  }

  .message.error {
    background-color: #f8d7da;
    color: #721c24;
  }
  
  @media (prefers-color-scheme: dark) {
    .tabs {
      border-bottom-color: #3a3a3a;
    }
    
    .tab {
      color: #999;
    }
    
    .tab:hover {
      color: #e0e0e0;
    }
    
    .tab.active {
      color: #4dabf7;
      border-bottom-color: #4dabf7;
    }
    
    .empty-state {
      color: #999;
    }
    
    .divider {
      background: #3a3a3a;
    }
  }
</style>