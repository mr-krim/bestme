<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  
  interface DownloadInfo {
    model_id: string;
    model_name: string;
    total_bytes: number;
    downloaded_bytes: number;
    progress_percent: number;
    download_speed: number; // bytes per second
    eta_seconds: number;
    status: 'pending' | 'downloading' | 'verifying' | 'extracting' | 'completed' | 'failed';
    error?: string;
  }
  
  // State
  let downloads: Map<string, DownloadInfo> = new Map();
  let completedDownloads: DownloadInfo[] = [];
  let unlisteners: (() => void)[] = [];
  
  // Lifecycle
  onMount(async () => {
    // Listen for download progress
    const unlistenProgress = await listen('model-download-progress', (event: any) => {
      const info = event.payload as DownloadInfo;
      downloads.set(info.model_id, info);
      downloads = downloads;
    });
    
    // Listen for download completion
    const unlistenComplete = await listen('model-download-complete', (event: any) => {
      const { model_id, success, error } = event.payload;
      const download = downloads.get(model_id);
      if (download) {
        download.status = success ? 'completed' : 'failed';
        if (error) download.error = error;
        completedDownloads = [...completedDownloads, download];
        downloads.delete(model_id);
        downloads = downloads;
      }
    });
    
    unlisteners = [unlistenProgress, unlistenComplete];
    
    // Load any existing downloads
    await loadActiveDownloads();
  });
  
  onDestroy(() => {
    unlisteners.forEach(fn => fn());
  });
  
  // Load active downloads
  async function loadActiveDownloads() {
    try {
      const activeDownloads = await invoke<DownloadInfo[]>('get_active_downloads');
      activeDownloads.forEach(download => {
        downloads.set(download.model_id, download);
      });
      downloads = downloads;
    } catch (e) {
      // No active downloads or error
    }
  }
  
  // Cancel download
  async function cancelDownload(modelId: string) {
    try {
      await invoke('cancel_model_download', { modelId });
      downloads.delete(modelId);
      downloads = downloads;
    } catch (e) {
      console.error('Failed to cancel download:', e);
    }
  }
  
  // Retry download
  async function retryDownload(modelId: string) {
    try {
      await invoke('download_model', { modelId });
      // Remove from completed list
      completedDownloads = completedDownloads.filter(d => d.model_id !== modelId);
    } catch (e) {
      console.error('Failed to retry download:', e);
    }
  }
  
  // Clear completed downloads
  function clearCompleted() {
    completedDownloads = [];
  }
  
  // Format file size
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  }
  
  // Format time
  function formatTime(seconds: number): string {
    if (seconds === Infinity || isNaN(seconds)) return 'Unknown';
    
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    } else {
      return `${secs}s`;
    }
  }
  
  // Get status color
  function getStatusColor(status: string): string {
    switch (status) {
      case 'downloading': return '#007bff';
      case 'verifying': return '#17a2b8';
      case 'extracting': return '#ffc107';
      case 'completed': return '#28a745';
      case 'failed': return '#dc3545';
      default: return '#6c757d';
    }
  }
  
  // Get status icon
  function getStatusIcon(status: string): string {
    switch (status) {
      case 'downloading': return '⬇️';
      case 'verifying': return '🔍';
      case 'extracting': return '📦';
      case 'completed': return '✅';
      case 'failed': return '❌';
      default: return '⏳';
    }
  }
</script>

<div class="download-progress-panel">
  <div class="header">
    <h3>Model Downloads</h3>
    {#if completedDownloads.length > 0}
      <button class="clear-button" on:click={clearCompleted}>
        Clear Completed
      </button>
    {/if}
  </div>
  
  {#if downloads.size === 0 && completedDownloads.length === 0}
    <div class="empty-state">
      <p>No active downloads</p>
    </div>
  {/if}
  
  <!-- Active Downloads -->
  {#each Array.from(downloads.values()) as download}
    <div class="download-item active">
      <div class="download-header">
        <div class="model-info">
          <h4>{download.model_name}</h4>
          <span class="model-id">{download.model_id}</span>
        </div>
        <div class="status" style="color: {getStatusColor(download.status)}">
          <span class="status-icon">{getStatusIcon(download.status)}</span>
          <span class="status-text">{download.status}</span>
        </div>
      </div>
      
      <div class="progress-section">
        <div class="progress-bar">
          <div 
            class="progress-fill"
            style="width: {download.progress_percent}%; background-color: {getStatusColor(download.status)}"
          ></div>
        </div>
        <div class="progress-text">{download.progress_percent.toFixed(1)}%</div>
      </div>
      
      <div class="download-stats">
        <div class="stat">
          <span class="stat-label">Downloaded:</span>
          <span class="stat-value">
            {formatBytes(download.downloaded_bytes)} / {formatBytes(download.total_bytes)}
          </span>
        </div>
        <div class="stat">
          <span class="stat-label">Speed:</span>
          <span class="stat-value">{formatBytes(download.download_speed)}/s</span>
        </div>
        <div class="stat">
          <span class="stat-label">ETA:</span>
          <span class="stat-value">{formatTime(download.eta_seconds)}</span>
        </div>
      </div>
      
      <div class="actions">
        <button class="cancel-button" on:click={() => cancelDownload(download.model_id)}>
          Cancel
        </button>
      </div>
    </div>
  {/each}
  
  <!-- Completed Downloads -->
  {#each completedDownloads as download}
    <div class="download-item completed">
      <div class="download-header">
        <div class="model-info">
          <h4>{download.model_name}</h4>
          <span class="model-id">{download.model_id}</span>
        </div>
        <div class="status" style="color: {getStatusColor(download.status)}">
          <span class="status-icon">{getStatusIcon(download.status)}</span>
          <span class="status-text">{download.status}</span>
        </div>
      </div>
      
      {#if download.error}
        <div class="error-message">{download.error}</div>
      {/if}
      
      {#if download.status === 'completed'}
        <div class="completion-info">
          <span>Downloaded {formatBytes(download.total_bytes)}</span>
        </div>
      {/if}
      
      {#if download.status === 'failed'}
        <div class="actions">
          <button class="retry-button" on:click={() => retryDownload(download.model_id)}>
            Retry
          </button>
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .download-progress-panel {
    padding: 20px;
    max-width: 800px;
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
  
  .clear-button {
    padding: 6px 12px;
    background: #6c757d;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.9em;
    cursor: pointer;
  }
  
  .clear-button:hover {
    background: #5a6268;
  }
  
  .empty-state {
    text-align: center;
    padding: 60px 20px;
    color: #666;
  }
  
  .download-item {
    background: #f8f9fa;
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 15px;
    transition: all 0.3s ease;
  }
  
  .download-item.active {
    border: 2px solid #007bff;
  }
  
  .download-item.completed {
    opacity: 0.8;
  }
  
  .download-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 15px;
  }
  
  .model-info h4 {
    margin: 0 0 5px 0;
    font-size: 1.1em;
  }
  
  .model-id {
    font-size: 0.8em;
    color: #666;
  }
  
  .status {
    display: flex;
    align-items: center;
    gap: 5px;
    font-weight: 500;
  }
  
  .status-icon {
    font-size: 1.2em;
  }
  
  .status-text {
    text-transform: capitalize;
  }
  
  .progress-section {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 15px;
  }
  
  .progress-bar {
    flex: 1;
    height: 20px;
    background: #e9ecef;
    border-radius: 10px;
    overflow: hidden;
  }
  
  .progress-fill {
    height: 100%;
    transition: width 0.3s ease;
  }
  
  .progress-text {
    font-weight: 500;
    min-width: 50px;
    text-align: right;
  }
  
  .download-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 15px;
    margin-bottom: 15px;
  }
  
  .stat {
    display: flex;
    justify-content: space-between;
    font-size: 0.9em;
  }
  
  .stat-label {
    color: #666;
  }
  
  .stat-value {
    font-weight: 500;
  }
  
  .error-message {
    background: #f8d7da;
    color: #721c24;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 15px;
    font-size: 0.9em;
  }
  
  .completion-info {
    color: #28a745;
    font-size: 0.9em;
    margin-bottom: 10px;
  }
  
  .actions {
    display: flex;
    gap: 10px;
  }
  
  .cancel-button,
  .retry-button {
    padding: 6px 16px;
    border: none;
    border-radius: 4px;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .cancel-button {
    background: #dc3545;
    color: white;
  }
  
  .cancel-button:hover {
    background: #c82333;
  }
  
  .retry-button {
    background: #007bff;
    color: white;
  }
  
  .retry-button:hover {
    background: #0056b3;
  }
  
  @media (prefers-color-scheme: dark) {
    .download-item {
      background: #2a2a2a;
      color: #e0e0e0;
    }
    
    .download-item.active {
      border-color: #0056b3;
    }
    
    .model-id,
    .stat-label {
      color: #999;
    }
    
    .progress-bar {
      background: #3a3a3a;
    }
    
    .error-message {
      background: #5a1a1a;
      color: #ff9999;
    }
  }
</style>