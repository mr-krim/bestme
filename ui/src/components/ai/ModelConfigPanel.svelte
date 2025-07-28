<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  // Configuration types
  interface ModelConfig {
    model_id: string;
    // Performance settings
    batch_size: number;
    max_tokens: number;
    temperature: number;
    top_p: number;
    // Hardware settings
    use_gpu: boolean;
    gpu_device_id?: number;
    num_threads: number;
    memory_limit_mb?: number;
    // Optimization settings
    quantization?: 'none' | 'int8' | 'int4' | 'fp16';
    enable_caching: boolean;
    cache_size: number;
    // Advanced settings
    context_window: number;
    streaming_enabled: boolean;
    fallback_enabled: boolean;
    timeout_ms: number;
  }
  
  interface GpuDevice {
    id: number;
    name: string;
    memory_mb: number;
    compute_capability?: string;
  }
  
  // Props
  export let modelId: string;
  export let config: ModelConfig | null = null;
  
  // State
  let editedConfig: ModelConfig = getDefaultConfig();
  let availableGpus: GpuDevice[] = [];
  let isLoading = false;
  let isSaving = false;
  let message = '';
  let showAdvanced = false;
  
  // Initialize config
  $: if (config) {
    editedConfig = { ...config };
  }
  
  // Load GPU devices on mount
  async function loadGpuDevices() {
    try {
      const gpuInfo = await invoke<GpuDevice[]>('get_gpu_info');
      availableGpus = gpuInfo;
    } catch (e) {
      console.error('Failed to load GPU info:', e);
    }
  }
  
  // Get default configuration
  function getDefaultConfig(): ModelConfig {
    return {
      model_id: modelId,
      batch_size: 1,
      max_tokens: 2048,
      temperature: 0.7,
      top_p: 0.9,
      use_gpu: true,
      num_threads: 4,
      quantization: 'none',
      enable_caching: true,
      cache_size: 1000,
      context_window: 4096,
      streaming_enabled: true,
      fallback_enabled: true,
      timeout_ms: 30000,
    };
  }
  
  // Load current configuration
  async function loadConfig() {
    if (!modelId) return;
    
    isLoading = true;
    try {
      const currentConfig = await invoke<ModelConfig>('get_model_config', { modelId });
      config = currentConfig;
      editedConfig = { ...currentConfig };
    } catch (e) {
      message = `Failed to load configuration: ${e}`;
    } finally {
      isLoading = false;
    }
  }
  
  // Save configuration
  async function saveConfig() {
    isSaving = true;
    message = '';
    
    try {
      await invoke('update_model_config', { 
        modelId,
        config: editedConfig 
      });
      config = { ...editedConfig };
      message = 'Configuration saved successfully';
      dispatch('save', { config: editedConfig });
    } catch (e) {
      message = `Failed to save configuration: ${e}`;
    } finally {
      isSaving = false;
    }
  }
  
  // Reset to defaults
  function resetToDefaults() {
    editedConfig = getDefaultConfig();
  }
  
  // Validate configuration
  function validateConfig(): boolean {
    if (editedConfig.batch_size < 1) return false;
    if (editedConfig.max_tokens < 1) return false;
    if (editedConfig.temperature < 0 || editedConfig.temperature > 2) return false;
    if (editedConfig.top_p < 0 || editedConfig.top_p > 1) return false;
    if (editedConfig.num_threads < 1) return false;
    if (editedConfig.timeout_ms < 1000) return false;
    return true;
  }
  
  // Calculate estimated memory usage
  $: estimatedMemoryMb = calculateMemoryUsage(editedConfig);
  
  function calculateMemoryUsage(cfg: ModelConfig): number {
    // Rough estimation based on configuration
    let baseMb = 1000; // Base model memory
    
    // Quantization reduces memory
    if (cfg.quantization === 'int8') baseMb *= 0.5;
    else if (cfg.quantization === 'int4') baseMb *= 0.25;
    else if (cfg.quantization === 'fp16') baseMb *= 0.5;
    
    // Context and batch size increase memory
    baseMb += (cfg.context_window / 1024) * 50 * cfg.batch_size;
    
    // Cache adds memory
    if (cfg.enable_caching) {
      baseMb += cfg.cache_size * 0.1;
    }
    
    return Math.round(baseMb);
  }
  
  // Initialize
  loadGpuDevices();
  if (modelId && !config) {
    loadConfig();
  }
</script>

<div class="config-panel">
  <div class="header">
    <h3>Model Configuration</h3>
    {#if modelId}
      <span class="model-id">{modelId}</span>
    {/if}
  </div>
  
  {#if message}
    <div class="message" class:success={message.includes('success')}>
      {message}
    </div>
  {/if}
  
  {#if isLoading}
    <div class="loading">Loading configuration...</div>
  {:else}
    <div class="config-sections">
      <!-- Performance Settings -->
      <section class="config-section">
        <h4>Performance Settings</h4>
        
        <div class="form-group">
          <label for="batch-size">Batch Size</label>
          <input 
            id="batch-size"
            type="number"
            bind:value={editedConfig.batch_size}
            min="1"
            max="32"
          />
          <span class="help-text">Number of requests to process in parallel</span>
        </div>
        
        <div class="form-group">
          <label for="max-tokens">Max Tokens</label>
          <input 
            id="max-tokens"
            type="number"
            bind:value={editedConfig.max_tokens}
            min="1"
            max="8192"
          />
          <span class="help-text">Maximum number of tokens to generate</span>
        </div>
        
        <div class="form-group">
          <label for="temperature">Temperature</label>
          <div class="slider-group">
            <input 
              id="temperature"
              type="range"
              bind:value={editedConfig.temperature}
              min="0"
              max="2"
              step="0.1"
            />
            <span class="slider-value">{editedConfig.temperature}</span>
          </div>
          <span class="help-text">Controls randomness (0 = deterministic, 2 = very random)</span>
        </div>
        
        <div class="form-group">
          <label for="top-p">Top P</label>
          <div class="slider-group">
            <input 
              id="top-p"
              type="range"
              bind:value={editedConfig.top_p}
              min="0"
              max="1"
              step="0.05"
            />
            <span class="slider-value">{editedConfig.top_p}</span>
          </div>
          <span class="help-text">Nucleus sampling threshold</span>
        </div>
      </section>
      
      <!-- Hardware Settings -->
      <section class="config-section">
        <h4>Hardware Settings</h4>
        
        <div class="form-group">
          <label>
            <input 
              type="checkbox"
              bind:checked={editedConfig.use_gpu}
            />
            Use GPU Acceleration
          </label>
        </div>
        
        {#if editedConfig.use_gpu && availableGpus.length > 0}
          <div class="form-group">
            <label for="gpu-device">GPU Device</label>
            <select id="gpu-device" bind:value={editedConfig.gpu_device_id}>
              <option value={undefined}>Auto-select</option>
              {#each availableGpus as gpu}
                <option value={gpu.id}>
                  {gpu.name} ({(gpu.memory_mb / 1024).toFixed(1)}GB)
                </option>
              {/each}
            </select>
          </div>
        {/if}
        
        <div class="form-group">
          <label for="num-threads">CPU Threads</label>
          <input 
            id="num-threads"
            type="number"
            bind:value={editedConfig.num_threads}
            min="1"
            max="32"
          />
          <span class="help-text">Number of CPU threads to use</span>
        </div>
        
        <div class="form-group">
          <label for="memory-limit">Memory Limit (MB)</label>
          <input 
            id="memory-limit"
            type="number"
            bind:value={editedConfig.memory_limit_mb}
            min="512"
            placeholder="No limit"
          />
          <span class="help-text">Leave empty for no limit</span>
        </div>
      </section>
      
      <!-- Optimization Settings -->
      <section class="config-section">
        <h4>Optimization Settings</h4>
        
        <div class="form-group">
          <label for="quantization">Quantization</label>
          <select id="quantization" bind:value={editedConfig.quantization}>
            <option value="none">None (Full precision)</option>
            <option value="fp16">FP16 (Half precision)</option>
            <option value="int8">INT8 (8-bit)</option>
            <option value="int4">INT4 (4-bit)</option>
          </select>
          <span class="help-text">Reduces model size and memory usage</span>
        </div>
        
        <div class="form-group">
          <label>
            <input 
              type="checkbox"
              bind:checked={editedConfig.enable_caching}
            />
            Enable Response Caching
          </label>
        </div>
        
        {#if editedConfig.enable_caching}
          <div class="form-group">
            <label for="cache-size">Cache Size</label>
            <input 
              id="cache-size"
              type="number"
              bind:value={editedConfig.cache_size}
              min="100"
              max="10000"
            />
            <span class="help-text">Number of cached responses</span>
          </div>
        {/if}
      </section>
      
      <!-- Advanced Settings -->
      <section class="config-section">
        <button 
          class="toggle-advanced"
          on:click={() => showAdvanced = !showAdvanced}
        >
          {showAdvanced ? '▼' : '▶'} Advanced Settings
        </button>
        
        {#if showAdvanced}
          <div class="advanced-settings">
            <div class="form-group">
              <label for="context-window">Context Window</label>
              <input 
                id="context-window"
                type="number"
                bind:value={editedConfig.context_window}
                min="512"
                max="32768"
              />
              <span class="help-text">Maximum context length in tokens</span>
            </div>
            
            <div class="form-group">
              <label>
                <input 
                  type="checkbox"
                  bind:checked={editedConfig.streaming_enabled}
                />
                Enable Streaming
              </label>
              <span class="help-text">Stream responses as they're generated</span>
            </div>
            
            <div class="form-group">
              <label>
                <input 
                  type="checkbox"
                  bind:checked={editedConfig.fallback_enabled}
                />
                Enable Fallback
              </label>
              <span class="help-text">Fallback to CPU if GPU fails</span>
            </div>
            
            <div class="form-group">
              <label for="timeout">Timeout (ms)</label>
              <input 
                id="timeout"
                type="number"
                bind:value={editedConfig.timeout_ms}
                min="1000"
                max="300000"
                step="1000"
              />
              <span class="help-text">Request timeout in milliseconds</span>
            </div>
          </div>
        {/if}
      </section>
      
      <!-- Memory Estimation -->
      <div class="memory-estimation">
        <h4>Estimated Memory Usage</h4>
        <div class="memory-value">{(estimatedMemoryMb / 1024).toFixed(1)} GB</div>
        <div class="memory-bar">
          <div 
            class="memory-fill"
            style="width: {Math.min(100, (estimatedMemoryMb / 8192) * 100)}%"
          ></div>
        </div>
      </div>
      
      <!-- Actions -->
      <div class="actions">
        <button 
          class="save-button"
          on:click={saveConfig}
          disabled={isSaving || !validateConfig()}
        >
          {isSaving ? 'Saving...' : 'Save Configuration'}
        </button>
        <button 
          class="reset-button"
          on:click={resetToDefaults}
        >
          Reset to Defaults
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .config-panel {
    padding: 20px;
    max-width: 800px;
  }
  
  .header {
    display: flex;
    align-items: center;
    gap: 15px;
    margin-bottom: 20px;
  }
  
  .header h3 {
    margin: 0;
  }
  
  .model-id {
    font-size: 0.9em;
    color: #666;
    background: #f0f0f0;
    padding: 4px 12px;
    border-radius: 20px;
  }
  
  .message {
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
    background: #f8d7da;
    color: #721c24;
  }
  
  .message.success {
    background: #d4edda;
    color: #155724;
  }
  
  .loading {
    text-align: center;
    padding: 40px;
    color: #666;
  }
  
  .config-sections {
    display: flex;
    flex-direction: column;
    gap: 30px;
  }
  
  .config-section {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
  }
  
  .config-section h4 {
    margin: 0 0 20px 0;
    color: #333;
  }
  
  .form-group {
    margin-bottom: 20px;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: 500;
    color: #333;
  }
  
  .form-group input[type="number"],
  .form-group select {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 1em;
  }
  
  .form-group input[type="checkbox"] {
    margin-right: 8px;
  }
  
  .help-text {
    display: block;
    margin-top: 5px;
    font-size: 0.85em;
    color: #6c757d;
  }
  
  .slider-group {
    display: flex;
    align-items: center;
    gap: 15px;
  }
  
  .slider-group input[type="range"] {
    flex: 1;
  }
  
  .slider-value {
    min-width: 40px;
    text-align: right;
    font-weight: 500;
  }
  
  .toggle-advanced {
    background: none;
    border: none;
    color: #007bff;
    cursor: pointer;
    font-size: 1em;
    padding: 5px 0;
    margin-bottom: 15px;
  }
  
  .toggle-advanced:hover {
    text-decoration: underline;
  }
  
  .advanced-settings {
    margin-top: 15px;
  }
  
  .memory-estimation {
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 2px solid #dee2e6;
  }
  
  .memory-estimation h4 {
    margin: 0 0 10px 0;
    color: #333;
  }
  
  .memory-value {
    font-size: 2em;
    font-weight: bold;
    color: #007bff;
    margin-bottom: 10px;
  }
  
  .memory-bar {
    height: 20px;
    background: #e9ecef;
    border-radius: 10px;
    overflow: hidden;
  }
  
  .memory-fill {
    height: 100%;
    background: #007bff;
    transition: width 0.3s ease;
  }
  
  .actions {
    display: flex;
    gap: 15px;
    margin-top: 30px;
  }
  
  .save-button,
  .reset-button {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    font-size: 1em;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .save-button {
    background: #28a745;
    color: white;
  }
  
  .save-button:hover:not(:disabled) {
    background: #218838;
  }
  
  .save-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .reset-button {
    background: #6c757d;
    color: white;
  }
  
  .reset-button:hover {
    background: #5a6268;
  }
  
  @media (prefers-color-scheme: dark) {
    .model-id {
      background: #3a3a3a;
      color: #e0e0e0;
    }
    
    .config-section {
      background: #2a2a2a;
      color: #e0e0e0;
    }
    
    .config-section h4 {
      color: #e0e0e0;
    }
    
    .form-group label {
      color: #e0e0e0;
    }
    
    .form-group input,
    .form-group select {
      background: #1a1a1a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .memory-estimation {
      background: #1a1a1a;
      border-color: #4a4a4a;
    }
    
    .memory-estimation h4 {
      color: #e0e0e0;
    }
    
    .memory-bar {
      background: #3a3a3a;
    }
  }
</style>