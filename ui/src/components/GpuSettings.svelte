<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { toast } from '@zerodevx/svelte-toast';
  
  // GPU information
  let gpuInfo = {
    available: false,
    backend: null as string | null,
    device_name: null as string | null,
    memory_mb: null as number | null,
    driver_version: null as string | null
  };
  
  let gpuEnabled = false;
  let availableBackends: string[] = [];
  let loading = true;
  
  // Load GPU information
  onMount(async () => {
    try {
      // Check if GPU is enabled at compile time
      gpuEnabled = await invoke('is_gpu_enabled');
      
      if (gpuEnabled) {
        // Get GPU info
        gpuInfo = await invoke('get_gpu_info');
        
        // Get available backends
        availableBackends = await invoke('get_available_gpu_backends');
      }
    } catch (error) {
      console.error('Failed to load GPU information:', error);
      toast.push('Failed to load GPU information', {
        theme: {
          '--toastBackground': '#F56565',
          '--toastColor': 'white',
          '--toastBarBackground': '#C53030'
        }
      });
    } finally {
      loading = false;
    }
  });
  
  // Format memory size
  function formatMemory(mb: number | null): string {
    if (!mb) return 'Unknown';
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(1)} GB`;
    }
    return `${mb} MB`;
  }
  
  // Get GPU status icon
  function getStatusIcon(available: boolean): string {
    return available ? '✅' : '❌';
  }
  
  // Get backend icon
  function getBackendIcon(backend: string | null): string {
    if (!backend) return '💻';
    switch (backend) {
      case 'CUDA': return '🟢'; // NVIDIA
      case 'Metal': return '🍎'; // Apple
      case 'HIP/ROCm': return '🔴'; // AMD
      case 'Vulkan': return '🔷'; // Cross-platform
      default: return '💻';
    }
  }
</script>

<div class="gpu-settings">
  <h3>GPU Acceleration</h3>
  
  {#if loading}
    <div class="loading">
      <span class="spinner"></span>
      Loading GPU information...
    </div>
  {:else if !gpuEnabled}
    <div class="warning-box">
      <p>⚠️ GPU acceleration is not enabled in this build.</p>
      <p class="info-text">
        To enable GPU acceleration, rebuild the application with GPU features:
      </p>
      <ul class="build-commands">
        <li><code>cargo build --release --features gpu-cuda</code> for NVIDIA GPUs</li>
        <li><code>cargo build --release --features gpu-metal</code> for Apple Silicon</li>
        <li><code>cargo build --release --features gpu-hipblas</code> for AMD GPUs</li>
      </ul>
    </div>
  {:else}
    <div class="gpu-info-grid">
      <div class="info-item">
        <span class="label">Status:</span>
        <span class="value">
          {getStatusIcon(gpuInfo.available)}
          {gpuInfo.available ? 'Available' : 'Not Available'}
        </span>
      </div>
      
      {#if gpuInfo.available}
        <div class="info-item">
          <span class="label">Backend:</span>
          <span class="value">
            {getBackendIcon(gpuInfo.backend)}
            {gpuInfo.backend || 'Unknown'}
          </span>
        </div>
        
        {#if gpuInfo.device_name}
          <div class="info-item full-width">
            <span class="label">Device:</span>
            <span class="value">{gpuInfo.device_name}</span>
          </div>
        {/if}
        
        {#if gpuInfo.memory_mb}
          <div class="info-item">
            <span class="label">Memory:</span>
            <span class="value">{formatMemory(gpuInfo.memory_mb)}</span>
          </div>
        {/if}
        
        {#if gpuInfo.driver_version}
          <div class="info-item">
            <span class="label">Driver:</span>
            <span class="value">{gpuInfo.driver_version}</span>
          </div>
        {/if}
      {:else}
        <div class="info-item full-width">
          <p class="no-gpu-text">
            No compatible GPU detected. The application will use CPU for transcription.
          </p>
        </div>
      {/if}
    </div>
    
    {#if availableBackends.length > 0}
      <div class="backends-section">
        <h4>Compiled GPU Backends:</h4>
        <div class="backend-list">
          {#each availableBackends as backend}
            <span class="backend-chip">
              {getBackendIcon(backend)} {backend}
            </span>
          {/each}
        </div>
      </div>
    {/if}
    
    <div class="performance-note">
      <p>
        <strong>Performance Impact:</strong>
        {#if gpuInfo.available}
          GPU acceleration can provide 5-15x faster transcription for medium and large models.
        {:else}
          Without GPU acceleration, transcription speed depends on CPU performance. 
          Consider using smaller models (tiny, base) for better real-time performance.
        {/if}
      </p>
    </div>
  {/if}
</div>

<style>
  .gpu-settings {
    padding: 1rem;
  }
  
  h3 {
    margin-bottom: 1rem;
    font-size: 1.2rem;
    font-weight: 600;
  }
  
  h4 {
    margin-top: 1.5rem;
    margin-bottom: 0.5rem;
    font-size: 1rem;
    font-weight: 500;
  }
  
  .loading {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    color: var(--text-secondary);
  }
  
  .spinner {
    display: inline-block;
    width: 1rem;
    height: 1rem;
    border: 2px solid var(--border-color);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .warning-box {
    background-color: var(--warning-bg, #FEF3C7);
    border: 1px solid var(--warning-border, #F59E0B);
    border-radius: 0.5rem;
    padding: 1rem;
    margin-bottom: 1rem;
  }
  
  .warning-box p {
    margin: 0.5rem 0;
  }
  
  .info-text {
    font-size: 0.9rem;
    color: var(--text-secondary);
  }
  
  .build-commands {
    list-style: none;
    padding: 0;
    margin: 0.5rem 0;
  }
  
  .build-commands li {
    margin: 0.25rem 0;
    font-size: 0.85rem;
  }
  
  .build-commands code {
    background-color: var(--code-bg, #F3F4F6);
    padding: 0.2rem 0.4rem;
    border-radius: 0.25rem;
    font-family: monospace;
    font-size: 0.8rem;
  }
  
  .gpu-info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
    margin-bottom: 1rem;
  }
  
  .info-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .info-item.full-width {
    grid-column: span 2;
  }
  
  .label {
    font-size: 0.9rem;
    color: var(--text-secondary);
    font-weight: 500;
  }
  
  .value {
    font-size: 1rem;
    font-weight: 500;
  }
  
  .no-gpu-text {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }
  
  .backends-section {
    margin-top: 1.5rem;
  }
  
  .backend-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  
  .backend-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.75rem;
    background-color: var(--chip-bg, #E5E7EB);
    border-radius: 1rem;
    font-size: 0.9rem;
    font-weight: 500;
  }
  
  .performance-note {
    margin-top: 1.5rem;
    padding: 1rem;
    background-color: var(--note-bg, #F3F4F6);
    border-radius: 0.5rem;
    font-size: 0.9rem;
  }
  
  .performance-note p {
    margin: 0;
  }
  
  /* Dark mode support */
  :global(.dark) .warning-box {
    background-color: #7C2D12;
    border-color: #DC2626;
  }
  
  :global(.dark) .build-commands code {
    background-color: #374151;
  }
  
  :global(.dark) .backend-chip {
    background-color: #374151;
  }
  
  :global(.dark) .performance-note {
    background-color: #1F2937;
  }
</style>