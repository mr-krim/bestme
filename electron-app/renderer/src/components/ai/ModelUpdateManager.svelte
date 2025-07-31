<script lang="ts">
  // Using Electron API instead of Tauri
  import { onMount, onDestroy } from 'svelte';
  // Using Electron API events instead of Tauri
  
  // Types
  interface ModelUpdate {
    model_id: string;
    current_version: string;
    new_version: string;
    release_date: string;
    size_bytes: number;
    changelog: string;
    improvements: string[];
    download_url: string;
    sha256: string;
    is_critical: boolean;
    compatibility_notes?: string;
  }
  
  interface UpdateCheckResult {
    last_check: string;
    available_updates: ModelUpdate[];
    failed_checks: [string, string][];
  }
  
  interface UpdateCheckerConfig {
    auto_check_enabled: boolean;
    check_interval_seconds: number;
    auto_download: boolean;
    wifi_only: boolean;
    max_concurrent_downloads: number;
  }
  
  // State
  let updateResult: UpdateCheckResult | null = null;
  let isChecking = false;
  let isDownloading: { [key: string]: boolean } = {};
  let downloadProgress: { [key: string]: number } = {};
  let error = '';
  let showSettings = false;
  let updateHistory: ModelUpdate[] = [];
  let lastCheckTime: Date | null = null;
  let unlistenDownloadProgress: (() => void) | null = null;
  let autoCheckInterval: NodeJS.Timeout | null = null;
  
  let config: UpdateCheckerConfig = {
    auto_check_enabled: true,
    check_interval_seconds: 86400, // 24 hours
    auto_download: false,
    wifi_only: true,
    max_concurrent_downloads: 2,
  };
  
  // Lifecycle
  onMount(async () => {
    await loadConfig();
    await checkForUpdates(false);
    await loadUpdateHistory();
    
    // Listen for download progress
    unlistenDownloadProgress = await window.electronAPI.listen('model-download-progress', (event: any) => {
      const { model_id, progress } = event.payload;
      if (isDownloading[model_id]) {
        downloadProgress[model_id] = progress;
        downloadProgress = downloadProgress;
      }
    });
    
    // Set up auto-check interval
    if (config.auto_check_enabled) {
      autoCheckInterval = setInterval(() => {
        checkForUpdates(false);
      }, config.check_interval_seconds * 1000);
    }
  });
  
  onDestroy(() => {
    if (unlistenDownloadProgress) {
      unlistenDownloadProgress();
    }
    if (autoCheckInterval) {
      clearInterval(autoCheckInterval);
    }
  });
  
  // Load configuration
  async function loadConfig() {
    // In a real app, this would load from persistent storage
    // For now, we use defaults
  }
  
  // Check for updates
  async function checkForUpdates(force: boolean) {
    isChecking = true;
    error = '';
    
    try {
      updateResult = await window.electronAPI.invoke('check_model_updates', { force });
      lastCheckTime = new Date(updateResult.last_check);
      
      // Show notification if updates are available
      if (updateResult.available_updates.length > 0) {
        showUpdateNotification(updateResult.available_updates.length);
      }
    } catch (e) {
      error = `Failed to check for updates: ${e}`;
    } finally {
      isChecking = false;
    }
  }
  
  // Download update
  async function downloadUpdate(update: ModelUpdate) {
    isDownloading[update.model_id] = true;
    downloadProgress[update.model_id] = 0;
    error = '';
    
    try {
      await window.electronAPI.invoke('download_model_update', { update });
      
      // Remove from available updates
      if (updateResult) {
        updateResult.available_updates = updateResult.available_updates
          .filter(u => u.model_id !== update.model_id);
      }
      
      // Refresh update history
      await loadUpdateHistory();
    } catch (e) {
      error = `Failed to download update: ${e}`;
    } finally {
      delete isDownloading[update.model_id];
      delete downloadProgress[update.model_id];
      isDownloading = isDownloading;
      downloadProgress = downloadProgress;
    }
  }
  
  // Load update history
  async function loadUpdateHistory() {
    try {
      updateHistory = await window.electronAPI.invoke('get_update_history');
    } catch (e) {
      console.error('Failed to load update history:', e);
    }
  }
  
  // Update configuration
  async function updateConfig() {
    try {
      await window.electronAPI.invoke('configure_update_checker', { config });
      
      // Reset auto-check interval
      if (autoCheckInterval) {
        clearInterval(autoCheckInterval);
      }
      
      if (config.auto_check_enabled) {
        autoCheckInterval = setInterval(() => {
          checkForUpdates(false);
        }, config.check_interval_seconds * 1000);
      }
      
      error = '';
    } catch (e) {
      error = `Failed to update configuration: ${e}`;
    }
  }
  
  // Clear cache
  async function clearCache() {
    try {
      await window.electronAPI.invoke('clear_update_cache');
      updateResult = null;
      error = '';
    } catch (e) {
      error = `Failed to clear cache: ${e}`;
    }
  }
  
  // Show update notification
  function showUpdateNotification(count: number) {
    // In a real app, this would show a system notification
    console.log(`${count} model update(s) available`);
  }
  
  // Format file size
  function formatSize(bytes: number): string {
    const gb = bytes / 1e9;
    return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / 1e6).toFixed(0)} MB`;
  }
  
  // Format date
  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString();
  }
  
  // Format time ago
  function formatTimeAgo(date: Date): string {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    
    if (hours < 1) {
      const minutes = Math.floor(diff / (1000 * 60));
      return `${minutes} minutes ago`;
    } else if (hours < 24) {
      return `${hours} hours ago`;
    } else {
      const days = Math.floor(hours / 24);
      return `${days} days ago`;
    }
  }
  
  // Get update badge color
  function getUpdateBadgeColor(update: ModelUpdate): string {
    if (update.is_critical) return '#dc3545';
    const versionDiff = update.new_version.split('.')[0] !== update.current_version.split('.')[0];
    return versionDiff ? '#ffc107' : '#28a745';
  }
</script>

<div class="model-update-manager">
  <div class="header">
    <h3>Model Updates</h3>
    <div class="header-actions">
      {#if lastCheckTime}
        <span class="last-check">
          Last check: {formatTimeAgo(lastCheckTime)}
        </span>
      {/if}
      <button 
        class="check-button"
        on:click={() => checkForUpdates(true)}
        disabled={isChecking}
      >
        {isChecking ? 'Checking...' : 'Check Now'}
      </button>
    </div>
  </div>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  {#if updateResult && updateResult.available_updates.length > 0}
    <div class="updates-section">
      <h4>Available Updates ({updateResult.available_updates.length})</h4>
      
      {#each updateResult.available_updates as update}
        <div class="update-card" class:critical={update.is_critical}>
          <div class="update-header">
            <div class="update-title">
              <h5>{update.model_id}</h5>
              <div class="version-info">
                <span class="current-version">{update.current_version}</span>
                <span class="arrow">→</span>
                <span class="new-version">{update.new_version}</span>
                <span 
                  class="update-badge"
                  style="background-color: {getUpdateBadgeColor(update)}"
                >
                  {update.is_critical ? 'Critical' : 'Update'}
                </span>
              </div>
            </div>
            <div class="update-meta">
              <span class="release-date">{formatDate(update.release_date)}</span>
              <span class="size">{formatSize(update.size_bytes)}</span>
            </div>
          </div>
          
          {#if update.improvements.length > 0}
            <div class="improvements">
              <h6>Improvements:</h6>
              <ul>
                {#each update.improvements as improvement}
                  <li>{improvement}</li>
                {/each}
              </ul>
            </div>
          {/if}
          
          <div class="changelog">
            <h6>Changelog:</h6>
            <pre>{update.changelog}</pre>
          </div>
          
          {#if update.compatibility_notes}
            <div class="compatibility-warning">
              <strong>⚠️ Compatibility Notes:</strong>
              <p>{update.compatibility_notes}</p>
            </div>
          {/if}
          
          <div class="update-actions">
            {#if isDownloading[update.model_id]}
              <div class="download-progress">
                <progress 
                  value={downloadProgress[update.model_id] || 0} 
                  max="100"
                />
                <span>{downloadProgress[update.model_id] || 0}%</span>
              </div>
            {:else}
              <button 
                class="download-button"
                on:click={() => downloadUpdate(update)}
              >
                Download Update
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {:else if updateResult}
    <div class="no-updates">
      <p>All models are up to date!</p>
    </div>
  {/if}
  
  {#if updateResult && updateResult.failed_checks.length > 0}
    <div class="failed-checks">
      <h4>Failed Checks</h4>
      {#each updateResult.failed_checks as [modelId, error]}
        <div class="failed-item">
          <span class="model-id">{modelId}</span>
          <span class="error-msg">{error}</span>
        </div>
      {/each}
    </div>
  {/if}
  
  <div class="settings-section">
    <button 
      class="toggle-settings"
      on:click={() => showSettings = !showSettings}
    >
      {showSettings ? '▼' : '▶'} Update Settings
    </button>
    
    {#if showSettings}
      <div class="settings-panel">
        <div class="setting-item">
          <label>
            <input 
              type="checkbox" 
              bind:checked={config.auto_check_enabled}
            />
            Enable automatic update checks
          </label>
        </div>
        
        {#if config.auto_check_enabled}
          <div class="setting-item">
            <label for="check-interval">Check interval (hours)</label>
            <input 
              id="check-interval"
              type="number"
              value={config.check_interval_seconds / 3600}
              on:change={(e) => config.check_interval_seconds = e.target.value * 3600}
              min="1"
              max="168"
            />
          </div>
        {/if}
        
        <div class="setting-item">
          <label>
            <input 
              type="checkbox" 
              bind:checked={config.auto_download}
            />
            Automatically download updates
          </label>
        </div>
        
        {#if config.auto_download}
          <div class="setting-item">
            <label>
              <input 
                type="checkbox" 
                bind:checked={config.wifi_only}
              />
              Only download on WiFi
            </label>
          </div>
        {/if}
        
        <div class="setting-actions">
          <button on:click={updateConfig}>Save Settings</button>
          <button on:click={clearCache} class="secondary">Clear Cache</button>
        </div>
      </div>
    {/if}
  </div>
  
  {#if updateHistory.length > 0}
    <div class="history-section">
      <h4>Update History</h4>
      <div class="history-list">
        {#each updateHistory.slice(0, 5) as update}
          <div class="history-item">
            <span class="model">{update.model_id}</span>
            <span class="version">{update.new_version}</span>
            <span class="date">{formatDate(update.release_date)}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .model-update-manager {
    padding: 20px;
    max-width: 900px;
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
  
  .header-actions {
    display: flex;
    align-items: center;
    gap: 15px;
  }
  
  .last-check {
    font-size: 0.9em;
    color: #666;
  }
  
  .check-button {
    padding: 8px 16px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .check-button:hover:not(:disabled) {
    background: #0056b3;
  }
  
  .check-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .error {
    background: #f8d7da;
    color: #721c24;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .updates-section h4 {
    margin: 0 0 20px 0;
  }
  
  .update-card {
    background: #f8f9fa;
    border: 2px solid #dee2e6;
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 20px;
  }
  
  .update-card.critical {
    border-color: #dc3545;
    background: #fff5f5;
  }
  
  .update-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 15px;
  }
  
  .update-title h5 {
    margin: 0 0 8px 0;
    font-size: 1.1em;
  }
  
  .version-info {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.9em;
  }
  
  .current-version {
    color: #666;
  }
  
  .arrow {
    color: #999;
  }
  
  .new-version {
    font-weight: 600;
    color: #28a745;
  }
  
  .update-badge {
    padding: 2px 8px;
    border-radius: 12px;
    color: white;
    font-size: 0.8em;
    font-weight: 500;
  }
  
  .update-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
    font-size: 0.9em;
    color: #666;
  }
  
  .improvements,
  .changelog {
    margin-bottom: 15px;
  }
  
  .improvements h6,
  .changelog h6 {
    margin: 0 0 8px 0;
    font-size: 0.95em;
    color: #333;
  }
  
  .improvements ul {
    margin: 0;
    padding-left: 20px;
  }
  
  .improvements li {
    font-size: 0.9em;
    color: #666;
    margin-bottom: 4px;
  }
  
  .changelog pre {
    margin: 0;
    padding: 10px;
    background: white;
    border-radius: 4px;
    font-size: 0.85em;
    white-space: pre-wrap;
  }
  
  .compatibility-warning {
    background: #fff3cd;
    border: 1px solid #ffeaa7;
    border-radius: 4px;
    padding: 12px;
    margin-bottom: 15px;
    font-size: 0.9em;
  }
  
  .compatibility-warning p {
    margin: 5px 0 0 0;
  }
  
  .update-actions {
    display: flex;
    justify-content: flex-end;
  }
  
  .download-button {
    padding: 8px 20px;
    background: #28a745;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .download-button:hover {
    background: #218838;
  }
  
  .download-progress {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 200px;
  }
  
  .download-progress progress {
    flex: 1;
    height: 20px;
  }
  
  .no-updates {
    text-align: center;
    padding: 60px 20px;
    color: #666;
  }
  
  .failed-checks {
    margin-top: 30px;
  }
  
  .failed-checks h4 {
    margin: 0 0 15px 0;
    color: #dc3545;
  }
  
  .failed-item {
    display: flex;
    justify-content: space-between;
    padding: 8px;
    background: #f8d7da;
    border-radius: 4px;
    margin-bottom: 8px;
    font-size: 0.9em;
  }
  
  .model-id {
    font-weight: 500;
  }
  
  .error-msg {
    color: #721c24;
  }
  
  .settings-section {
    margin-top: 30px;
  }
  
  .toggle-settings {
    background: none;
    border: none;
    color: #007bff;
    cursor: pointer;
    padding: 5px 0;
    font-size: 1em;
  }
  
  .settings-panel {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-top: 15px;
  }
  
  .setting-item {
    margin-bottom: 15px;
  }
  
  .setting-item label {
    display: block;
    margin-bottom: 5px;
  }
  
  .setting-item input[type="number"] {
    width: 80px;
    padding: 4px 8px;
    border: 1px solid #ced4da;
    border-radius: 4px;
  }
  
  .setting-actions {
    display: flex;
    gap: 10px;
    margin-top: 20px;
  }
  
  .setting-actions button {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .setting-actions button:first-child {
    background: #007bff;
    color: white;
  }
  
  .setting-actions button.secondary {
    background: #6c757d;
    color: white;
  }
  
  .history-section {
    margin-top: 30px;
  }
  
  .history-section h4 {
    margin: 0 0 15px 0;
  }
  
  .history-list {
    background: #f8f9fa;
    padding: 15px;
    border-radius: 8px;
  }
  
  .history-item {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr;
    gap: 15px;
    padding: 8px 0;
    border-bottom: 1px solid #e9ecef;
    font-size: 0.9em;
  }
  
  .history-item:last-child {
    border-bottom: none;
  }
  
  @media (prefers-color-scheme: dark) {
    .last-check {
      color: #999;
    }
    
    .update-card {
      background: #2a2a2a;
      border-color: #4a4a4a;
    }
    
    .update-card.critical {
      background: #3a2020;
      border-color: #dc3545;
    }
    
    .version-info .current-version {
      color: #999;
    }
    
    .update-meta {
      color: #999;
    }
    
    .improvements h6,
    .changelog h6 {
      color: #e0e0e0;
    }
    
    .improvements li {
      color: #999;
    }
    
    .changelog pre {
      background: #1a1a1a;
      color: #e0e0e0;
    }
    
    .compatibility-warning {
      background: #3a3a20;
      border-color: #5a5a30;
      color: #e0e0e0;
    }
    
    .failed-item {
      background: #3a2020;
    }
    
    .error-msg {
      color: #ff9999;
    }
    
    .settings-panel,
    .history-list {
      background: #2a2a2a;
    }
    
    .setting-item input[type="number"] {
      background: #1a1a1a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .history-item {
      border-bottom-color: #4a4a4a;
    }
  }
</style>