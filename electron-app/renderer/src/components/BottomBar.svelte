<script lang="ts">
  // Props
  export let appVersion = '0.1.0';
  export let isOnline = true;
  export let cpuUsage = 0;
  export let memoryUsage = 0;
  export let wordCount = 0;
  export let accuracy = 100;
</script>

<div class="status-bar">
  <div class="left-info">
    <div class="version">v{appVersion}</div>
    <div class="connection-status">
      <span class="indicator {isOnline ? 'online' : 'offline'}"></span>
      <span>{isOnline ? 'Online' : 'Offline'}</span>
    </div>
  </div>
  
  <div class="center-info">
    <div class="ui-mode-selector">
      <select>
        <option value="user">User</option>
        <option value="power-user">Power User</option>
        <option value="advanced">Advanced</option>
      </select>
    </div>
  </div>
  
  <div class="right-info">
    <div class="system-resources">
      <div class="resource-item">
        <span class="label">CPU:</span>
        <div class="progress-bar">
          <div class="progress" style="width: {cpuUsage}%"></div>
        </div>
        <span class="value">{cpuUsage}%</span>
      </div>
      
      <div class="resource-item">
        <span class="label">RAM:</span>
        <div class="progress-bar">
          <div class="progress" style="width: {memoryUsage}%"></div>
        </div>
        <span class="value">{memoryUsage}%</span>
      </div>
    </div>
    
    <div class="transcription-stats">
      <div class="stat-item">
        <span class="label">Words:</span>
        <span class="value">{wordCount}</span>
      </div>
      
      <div class="stat-item">
        <span class="label">Accuracy:</span>
        <span class="value">{accuracy}%</span>
      </div>
    </div>
  </div>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    background-color: var(--status-bar-bg, #f0f0f0);
    border-top: 1px solid var(--border-color, #e0e0e0);
    padding: 0 16px;
    font-size: 0.75rem;
  }
  
  .left-info, .center-info, .right-info {
    display: flex;
    align-items: center;
  }
  
  .version {
    margin-right: 16px;
    color: var(--text-secondary, #666);
  }
  
  .connection-status {
    display: flex;
    align-items: center;
  }
  
  .indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-right: 4px;
  }
  
  .indicator.online {
    background-color: var(--online-color, #4caf50);
  }
  
  .indicator.offline {
    background-color: var(--offline-color, #f44336);
  }
  
  .ui-mode-selector select {
    padding: 2px 8px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    font-size: 0.75rem;
    background-color: var(--dropdown-bg, #fff);
  }
  
  .system-resources, .transcription-stats {
    display: flex;
    align-items: center;
  }
  
  .system-resources {
    margin-right: 16px;
  }
  
  .resource-item, .stat-item {
    display: flex;
    align-items: center;
    margin-right: 12px;
  }
  
  .label {
    margin-right: 4px;
    color: var(--text-secondary, #666);
  }
  
  .progress-bar {
    width: 48px;
    height: 6px;
    background-color: var(--progress-bg, #e0e0e0);
    border-radius: 3px;
    overflow: hidden;
    margin-right: 4px;
  }
  
  .progress {
    height: 100%;
    background-color: var(--progress-color, #2196f3);
  }
  
  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    .status-bar {
      --status-bar-bg: #252525;
      --border-color: #333;
      --text-secondary: #aaa;
      --dropdown-bg: #333;
      --progress-bg: #444;
      --progress-color: #1976d2;
      --online-color: #43a047;
      --offline-color: #e53935;
      background-color: var(--status-bar-bg);
      border-top-color: var(--border-color);
    }
    
    .ui-mode-selector select {
      color: #eee;
    }
  }
</style> 
