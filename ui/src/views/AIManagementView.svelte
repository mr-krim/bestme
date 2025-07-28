<script lang="ts">
  import { ModelSelector, PerformanceMonitor, ModelDownloadProgress, ModelConfigPanel } from '../components/ai';
  
  let selectedModelId = '';
  let showConfig = false;
  
  function handleModelSelect(event: CustomEvent) {
    selectedModelId = event.detail.modelId;
    showConfig = false;
  }
</script>

<div class="ai-management-view">
  <div class="header">
    <h1>AI Model Management</h1>
    <p class="subtitle">Manage your AI models, monitor performance, and configure settings</p>
  </div>
  
  <div class="layout">
    <div class="left-section">
      <div class="section">
        <h2>Available Models</h2>
        <ModelSelector 
          bind:selectedModelId={selectedModelId}
          on:select={handleModelSelect}
        />
      </div>
      
      <div class="section">
        <h2>Downloads</h2>
        <ModelDownloadProgress />
      </div>
    </div>
    
    <div class="right-section">
      {#if selectedModelId}
        <div class="section">
          <h2>Performance Metrics</h2>
          <PerformanceMonitor modelId={selectedModelId} />
        </div>
        
        <div class="section">
          <div class="section-header">
            <h2>Model Configuration</h2>
            <button 
              class="toggle-config"
              on:click={() => showConfig = !showConfig}
            >
              {showConfig ? 'Hide' : 'Show'} Configuration
            </button>
          </div>
          {#if showConfig}
            <ModelConfigPanel modelId={selectedModelId} />
          {/if}
        </div>
      {:else}
        <div class="empty-state">
          <svg width="120" height="120" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
            <line x1="9" y1="9" x2="15" y2="9"></line>
            <line x1="9" y1="12" x2="15" y2="12"></line>
            <line x1="9" y1="15" x2="12" y2="15"></line>
          </svg>
          <h3>No Model Selected</h3>
          <p>Select a model from the list to view performance metrics and configuration options.</p>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .ai-management-view {
    padding: 20px;
    max-width: 1600px;
    margin: 0 auto;
  }
  
  .header {
    margin-bottom: 30px;
  }
  
  .header h1 {
    margin: 0 0 10px 0;
    font-size: 2em;
  }
  
  .subtitle {
    color: #666;
    margin: 0;
  }
  
  .layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 30px;
  }
  
  .section {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
  }
  
  .section h2 {
    margin: 0 0 20px 0;
    font-size: 1.3em;
  }
  
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }
  
  .section-header h2 {
    margin: 0;
  }
  
  .toggle-config {
    padding: 6px 12px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.9em;
    cursor: pointer;
  }
  
  .toggle-config:hover {
    background: #0056b3;
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 20px;
    text-align: center;
    color: #999;
  }
  
  .empty-state svg {
    margin-bottom: 20px;
    opacity: 0.3;
  }
  
  .empty-state h3 {
    margin: 0 0 10px 0;
    font-size: 1.2em;
    color: #666;
  }
  
  .empty-state p {
    margin: 0;
    max-width: 400px;
  }
  
  @media (max-width: 1200px) {
    .layout {
      grid-template-columns: 1fr;
    }
  }
  
  @media (prefers-color-scheme: dark) {
    .section {
      background: #2a2a2a;
      color: #e0e0e0;
    }
    
    .subtitle {
      color: #999;
    }
    
    .empty-state {
      color: #666;
    }
    
    .empty-state h3 {
      color: #999;
    }
  }
</style>