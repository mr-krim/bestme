<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, createEventDispatcher } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  
  const dispatch = createEventDispatcher();
  
  // Model types
  export interface ModelInfo {
    id: string;
    name: string;
    provider: string;
    architecture: string;
    parameters: string;
    size_bytes: number;
    capabilities: string[];
    performance_class: 'fast' | 'balanced' | 'quality';
    hardware_requirements: {
      min_ram_gb: number;
      recommended_vram_gb?: number;
      supports_gpu: boolean;
    };
  }
  
  export interface ModelPerformance {
    model_id: string;
    avg_tokens_per_second: number;
    avg_latency_ms: number;
    memory_usage_mb: number;
    last_updated: Date;
  }
  
  // Props
  export let selectedModelId: string = '';
  export let showPerformance: boolean = true;
  export let filterByCapability: string = '';
  
  // State
  let availableModels: ModelInfo[] = [];
  let downloadedModels: string[] = [];
  let loadedModels: ModelInfo[] = [];
  let modelPerformance: Map<string, ModelPerformance> = new Map();
  let isLoading = false;
  let error = '';
  let downloadProgress: Map<string, number> = new Map();
  
  // Lifecycle
  onMount(async () => {
    await loadModels();
    
    // Listen for download progress events
    const unlistenProgress = await listen('model-download-progress', (event: any) => {
      const { model_id, progress } = event.payload;
      downloadProgress.set(model_id, progress);
      downloadProgress = downloadProgress;
    });
    
    // Listen for download completion
    const unlistenComplete = await listen('model-download-complete', async (event: any) => {
      const { model_id, success } = event.payload;
      if (success) {
        downloadProgress.delete(model_id);
        await loadModels();
      }
    });
    
    return () => {
      unlistenProgress();
      unlistenComplete();
    };
  });
  
  // Load all model data
  async function loadModels() {
    isLoading = true;
    error = '';
    
    try {
      const [available, downloaded, loaded] = await Promise.all([
        invoke<ModelInfo[]>('list_available_models'),
        invoke<string[]>('list_downloaded_models'),
        invoke<ModelInfo[]>('list_loaded_models')
      ]);
      
      availableModels = available;
      downloadedModels = downloaded;
      loadedModels = loaded;
      
      // Load performance data for downloaded models
      for (const modelId of downloaded) {
        try {
          const perf = await invoke<ModelPerformance>('get_model_performance', { modelId });
          if (perf) {
            modelPerformance.set(modelId, perf);
          }
        } catch (e) {
          // Performance data might not be available for all models
        }
      }
      modelPerformance = modelPerformance;
    } catch (e) {
      error = `Failed to load models: ${e}`;
    } finally {
      isLoading = false;
    }
  }
  
  // Download a model
  async function downloadModel(modelId: string) {
    try {
      await invoke('download_model', { modelId });
      downloadProgress.set(modelId, 0);
      downloadProgress = downloadProgress;
    } catch (e) {
      error = `Failed to start download: ${e}`;
    }
  }
  
  // Load/unload a model
  async function toggleModelLoad(modelId: string) {
    const isLoaded = loadedModels.some(m => m.id === modelId);
    
    try {
      if (isLoaded) {
        await invoke('unload_ai_model', { modelId });
      } else {
        await invoke('load_ai_model', { modelId });
      }
      await loadModels();
    } catch (e) {
      error = `Failed to ${isLoaded ? 'unload' : 'load'} model: ${e}`;
    }
  }
  
  // Select a model
  function selectModel(modelId: string) {
    selectedModelId = modelId;
    dispatch('select', { modelId });
  }
  
  // Filter models
  $: filteredModels = availableModels.filter(model => {
    if (!filterByCapability) return true;
    return model.capabilities.includes(filterByCapability);
  });
  
  // Format file size
  function formatSize(bytes: number): string {
    const gb = bytes / 1e9;
    return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / 1e6).toFixed(0)} MB`;
  }
  
  // Get performance class color
  function getPerformanceColor(perfClass: string): string {
    switch (perfClass) {
      case 'fast': return '#28a745';
      case 'balanced': return '#ffc107';
      case 'quality': return '#007bff';
      default: return '#6c757d';
    }
  }
</script>

<div class="model-selector">
  <div class="header">
    <h3>AI Models</h3>
    {#if filterByCapability}
      <span class="filter-badge">
        Filtering: {filterByCapability}
        <button on:click={() => filterByCapability = ''}>×</button>
      </span>
    {/if}
  </div>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  {#if isLoading}
    <div class="loading">Loading models...</div>
  {:else}
    <div class="model-grid">
      {#each filteredModels as model}
        <div 
          class="model-card"
          class:selected={selectedModelId === model.id}
          class:loaded={loadedModels.some(m => m.id === model.id)}
        >
          <div class="model-header">
            <h4>{model.name}</h4>
            <span 
              class="performance-badge"
              style="background-color: {getPerformanceColor(model.performance_class)}"
            >
              {model.performance_class}
            </span>
          </div>
          
          <div class="model-info">
            <div class="info-row">
              <span class="label">Architecture:</span>
              <span>{model.architecture}</span>
            </div>
            <div class="info-row">
              <span class="label">Parameters:</span>
              <span>{model.parameters}</span>
            </div>
            <div class="info-row">
              <span class="label">Size:</span>
              <span>{formatSize(model.size_bytes)}</span>
            </div>
            <div class="info-row">
              <span class="label">RAM Required:</span>
              <span>{model.hardware_requirements.min_ram_gb}GB</span>
            </div>
            {#if model.hardware_requirements.supports_gpu}
              <div class="info-row">
                <span class="label">GPU VRAM:</span>
                <span>{model.hardware_requirements.recommended_vram_gb || 'N/A'}GB</span>
              </div>
            {/if}
          </div>
          
          <div class="capabilities">
            {#each model.capabilities as capability}
              <button 
                class="capability-tag"
                on:click={() => filterByCapability = capability}
                title="Filter by this capability"
              >
                {capability}
              </button>
            {/each}
          </div>
          
          {#if showPerformance && modelPerformance.has(model.id)}
            {@const perf = modelPerformance.get(model.id)}
            <div class="performance-stats">
              <div class="stat">
                <span class="stat-value">{perf.avg_tokens_per_second.toFixed(1)}</span>
                <span class="stat-label">tokens/sec</span>
              </div>
              <div class="stat">
                <span class="stat-value">{perf.avg_latency_ms.toFixed(0)}</span>
                <span class="stat-label">ms latency</span>
              </div>
              <div class="stat">
                <span class="stat-value">{(perf.memory_usage_mb / 1024).toFixed(1)}</span>
                <span class="stat-label">GB memory</span>
              </div>
            </div>
          {/if}
          
          <div class="model-actions">
            {#if downloadProgress.has(model.id)}
              <div class="download-progress">
                <progress value={downloadProgress.get(model.id)} max="100">
                  {downloadProgress.get(model.id)}%
                </progress>
                <span>{downloadProgress.get(model.id)}%</span>
              </div>
            {:else if downloadedModels.includes(model.id)}
              <button 
                class="action-button load"
                on:click={() => toggleModelLoad(model.id)}
              >
                {loadedModels.some(m => m.id === model.id) ? 'Unload' : 'Load'}
              </button>
              <button 
                class="action-button select"
                class:primary={selectedModelId === model.id}
                on:click={() => selectModel(model.id)}
                disabled={!loadedModels.some(m => m.id === model.id)}
              >
                {selectedModelId === model.id ? 'Selected' : 'Select'}
              </button>
            {:else}
              <button 
                class="action-button download"
                on:click={() => downloadModel(model.id)}
              >
                Download
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .model-selector {
    padding: 20px;
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }
  
  .header h3 {
    margin: 0;
  }
  
  .filter-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    background-color: #007bff;
    color: white;
    border-radius: 20px;
    font-size: 0.9em;
  }
  
  .filter-badge button {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 1.2em;
    padding: 0;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .error {
    background-color: #f8d7da;
    color: #721c24;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .loading {
    text-align: center;
    padding: 40px;
    color: #666;
  }
  
  .model-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 20px;
  }
  
  .model-card {
    background: #f8f9fa;
    border-radius: 8px;
    padding: 20px;
    transition: all 0.3s ease;
    border: 2px solid transparent;
  }
  
  .model-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
  
  .model-card.selected {
    border-color: #007bff;
    background: #e7f3ff;
  }
  
  .model-card.loaded {
    border-color: #28a745;
  }
  
  .model-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 15px;
  }
  
  .model-header h4 {
    margin: 0;
    font-size: 1.1em;
  }
  
  .performance-badge {
    padding: 4px 12px;
    border-radius: 12px;
    color: white;
    font-size: 0.8em;
    font-weight: 500;
  }
  
  .model-info {
    margin-bottom: 15px;
  }
  
  .info-row {
    display: flex;
    justify-content: space-between;
    padding: 4px 0;
    font-size: 0.9em;
  }
  
  .info-row .label {
    color: #666;
  }
  
  .capabilities {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 15px;
  }
  
  .capability-tag {
    padding: 4px 8px;
    background: #e9ecef;
    border: 1px solid #dee2e6;
    border-radius: 4px;
    font-size: 0.8em;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .capability-tag:hover {
    background: #dee2e6;
    border-color: #ced4da;
  }
  
  .performance-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
    margin-bottom: 15px;
    padding: 10px;
    background: white;
    border-radius: 4px;
  }
  
  .stat {
    text-align: center;
  }
  
  .stat-value {
    display: block;
    font-size: 1.2em;
    font-weight: bold;
    color: #007bff;
  }
  
  .stat-label {
    display: block;
    font-size: 0.8em;
    color: #666;
  }
  
  .model-actions {
    display: flex;
    gap: 10px;
  }
  
  .action-button {
    flex: 1;
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .action-button.download {
    background: #007bff;
    color: white;
  }
  
  .action-button.download:hover {
    background: #0056b3;
  }
  
  .action-button.load {
    background: #28a745;
    color: white;
  }
  
  .action-button.load:hover {
    background: #218838;
  }
  
  .action-button.select {
    background: #6c757d;
    color: white;
  }
  
  .action-button.select:hover {
    background: #5a6268;
  }
  
  .action-button.select.primary {
    background: #007bff;
  }
  
  .action-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .download-progress {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .download-progress progress {
    flex: 1;
    height: 20px;
  }
  
  .download-progress span {
    font-size: 0.9em;
    font-weight: 500;
  }
  
  @media (prefers-color-scheme: dark) {
    .model-card {
      background: #2a2a2a;
      color: #e0e0e0;
    }
    
    .model-card.selected {
      background: #1a3a52;
    }
    
    .performance-stats {
      background: #1a1a1a;
    }
    
    .capability-tag {
      background: #3a3a3a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .capability-tag:hover {
      background: #4a4a4a;
      border-color: #5a5a5a;
    }
    
    .info-row .label {
      color: #999;
    }
  }
</style>