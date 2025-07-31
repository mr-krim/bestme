<script lang="ts">
  // Using Electron API instead of Tauri
  import { onMount } from 'svelte';
  
  // Text injection settings
  let enabled = false;
  let injectionMode: 'type' | 'paste' | 'direct' = 'type';
  let typeDelay = 10;
  let restoreClipboard = true;
  let permissions = {
    available: false,
    accessibility_granted: null as boolean | null,
    has_required_privileges: false,
    message: '',
    required_actions: [] as string[]
  };
  
  // Active window info
  let activeWindow = {
    title: '',
    app_name: '',
    app_path: null as string | null,
    pid: null as number | null,
    is_elevated: false,
    is_focused: false
  };
  
  // Load current settings
  onMount(async () => {
    try {
      permissions = await window.electronAPI.invoke('check_injection_permissions');
      if (permissions.has_required_privileges) {
        activeWindow = await window.electronAPI.invoke('get_active_window');
      }
    } catch (error) {
      console.error('Failed to load text injection status:', error);
    }
  });
  
  // Enable/disable text injection
  async function toggleInjection() {
    try {
      await window.electronAPI.invoke('enable_text_injection', { enabled: !enabled });
      enabled = !enabled;
    } catch (error) {
      console.error('Failed to toggle text injection:', error);
    }
  }
  
  // Request permissions
  async function requestPermissions() {
    try {
      await window.electronAPI.invoke('request_injection_permissions');
      // Re-check permissions
      permissions = await window.electronAPI.invoke('check_injection_permissions');
    } catch (error) {
      console.error('Failed to request permissions:', error);
    }
  }
  
  // Update injection mode
  async function updateMode() {
    try {
      await window.electronAPI.invoke('set_injection_mode', { 
        mode: {
          [injectionMode]: injectionMode === 'type' 
            ? { delay_ms: typeDelay, simulate_typing: true }
            : injectionMode === 'paste'
            ? { restore_clipboard: restoreClipboard, use_clipboard: true }
            : {}
        }
      });
    } catch (error) {
      console.error('Failed to update injection mode:', error);
    }
  }
  
  // Test injection
  async function testInjection() {
    const testText = "Hello from BestMe! This is a test of text injection.";
    try {
      await window.electronAPI.invoke('inject_text', { text: testText });
    } catch (error) {
      console.error('Failed to inject text:', error);
    }
  }
  
  // Refresh active window
  async function refreshActiveWindow() {
    try {
      activeWindow = await window.electronAPI.invoke('get_active_window');
    } catch (error) {
      console.error('Failed to get active window:', error);
    }
  }
</script>

<div class="text-injection-settings">
  <h3>Text Injection Settings</h3>
  
  <!-- Permission Status -->
  <div class="permission-status">
    <h4>Permission Status</h4>
    <div class="status-info">
      <p class:available={permissions.available} class:unavailable={!permissions.available}>
        {permissions.message}
      </p>
      
      {#if permissions.required_actions.length > 0}
        <div class="required-actions">
          <p>Required actions:</p>
          <ul>
            {#each permissions.required_actions as action}
              <li>{action}</li>
            {/each}
          </ul>
          {#if permissions.accessibility_granted === false}
            <button on:click={requestPermissions}>Request Permissions</button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
  
  <!-- Main Controls -->
  {#if permissions.has_required_privileges}
    <div class="main-controls">
      <label class="toggle">
        <input type="checkbox" checked={enabled} on:change={toggleInjection}>
        <span>Enable Text Injection</span>
      </label>
      
      {#if enabled}
        <!-- Injection Mode -->
        <div class="mode-selection">
          <h4>Injection Mode</h4>
          <label>
            <input type="radio" bind:group={injectionMode} value="type" on:change={updateMode}>
            <span>Type (simulate keystrokes)</span>
          </label>
          <label>
            <input type="radio" bind:group={injectionMode} value="paste" on:change={updateMode}>
            <span>Paste (use clipboard)</span>
          </label>
          <label>
            <input type="radio" bind:group={injectionMode} value="direct" on:change={updateMode}>
            <span>Direct (fastest)</span>
          </label>
        </div>
        
        <!-- Mode-specific settings -->
        {#if injectionMode === 'type'}
          <div class="mode-settings">
            <label>
              <span>Typing delay (ms):</span>
              <input type="number" bind:value={typeDelay} min="0" max="1000" on:change={updateMode}>
            </label>
          </div>
        {:else if injectionMode === 'paste'}
          <div class="mode-settings">
            <label>
              <input type="checkbox" bind:checked={restoreClipboard} on:change={updateMode}>
              <span>Restore clipboard after injection</span>
            </label>
          </div>
        {/if}
        
        <!-- Test Button -->
        <div class="test-section">
          <button on:click={testInjection} class="test-button">
            Test Text Injection
          </button>
          <p class="test-hint">Click to inject test text into the active window</p>
        </div>
      {/if}
    </div>
  {/if}
  
  <!-- Active Window Info -->
  <div class="active-window">
    <h4>Active Window <button on:click={refreshActiveWindow} class="refresh">↻</button></h4>
    {#if activeWindow.app_name}
      <div class="window-info">
        <p><strong>Application:</strong> {activeWindow.app_name}</p>
        <p><strong>Window Title:</strong> {activeWindow.title || 'N/A'}</p>
        {#if activeWindow.pid}
          <p><strong>Process ID:</strong> {activeWindow.pid}</p>
        {/if}
        {#if activeWindow.is_elevated}
          <p class="elevated">⚠️ Running with elevated privileges</p>
        {/if}
      </div>
    {:else}
      <p class="no-window">No active window detected</p>
    {/if}
  </div>
</div>

<style>
  .text-injection-settings {
    padding: 1rem;
    background: var(--secondary-bg, #f5f5f5);
    border-radius: 8px;
    margin: 1rem 0;
  }
  
  h3 {
    margin: 0 0 1rem 0;
    color: var(--primary-color, #4a7dff);
  }
  
  h4 {
    margin: 1rem 0 0.5rem 0;
    color: var(--text-color, #333);
    font-size: 1.1rem;
  }
  
  .permission-status {
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: var(--main-bg, #fff);
    border-radius: 4px;
  }
  
  .status-info p {
    margin: 0.5rem 0;
  }
  
  .available {
    color: var(--success-color, #28a745);
  }
  
  .unavailable {
    color: var(--error-color, #dc3545);
  }
  
  .required-actions {
    margin-top: 1rem;
  }
  
  .required-actions ul {
    margin: 0.5rem 0 1rem 1.5rem;
    font-size: 0.9rem;
  }
  
  .main-controls {
    margin: 1rem 0;
  }
  
  .toggle {
    display: flex;
    align-items: center;
    margin-bottom: 1rem;
    cursor: pointer;
  }
  
  .toggle input[type="checkbox"] {
    margin-right: 0.5rem;
  }
  
  .mode-selection {
    margin: 1rem 0;
    padding: 1rem;
    background: var(--main-bg, #fff);
    border-radius: 4px;
  }
  
  .mode-selection label {
    display: block;
    margin: 0.5rem 0;
    cursor: pointer;
  }
  
  .mode-selection input[type="radio"] {
    margin-right: 0.5rem;
  }
  
  .mode-settings {
    margin: 1rem 0;
    padding: 0.5rem 1rem;
    background: var(--main-bg, #fff);
    border-radius: 4px;
  }
  
  .mode-settings label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .mode-settings input[type="number"] {
    width: 80px;
    padding: 0.25rem;
    border: 1px solid var(--border-color, #ccc);
    border-radius: 4px;
    background: var(--secondary-bg, #f5f5f5);
    color: var(--text-color, #333);
  }
  
  .test-section {
    margin: 1.5rem 0;
    text-align: center;
  }
  
  .test-button {
    padding: 0.75rem 1.5rem;
    background: var(--primary-color, #4a7dff);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .test-button:hover {
    background: var(--primary-hover, #3a6eee);
  }
  
  .test-hint {
    margin: 0.5rem 0 0 0;
    font-size: 0.9rem;
    color: var(--text-secondary, #666);
  }
  
  .active-window {
    margin-top: 1.5rem;
    padding: 1rem;
    background: var(--main-bg, #fff);
    border-radius: 4px;
  }
  
  .active-window h4 {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0 0 0.5rem 0;
  }
  
  .refresh {
    padding: 0.25rem 0.5rem;
    background: transparent;
    border: 1px solid var(--border-color, #ccc);
    border-radius: 4px;
    cursor: pointer;
    font-size: 1.2rem;
    transition: background 0.2s;
  }
  
  .refresh:hover {
    background: var(--secondary-bg, #f5f5f5);
  }
  
  .window-info p {
    margin: 0.25rem 0;
    font-size: 0.9rem;
  }
  
  .window-info strong {
    color: var(--text-secondary, #666);
  }
  
  .elevated {
    color: var(--warning-color, #ff9800);
    font-weight: bold;
  }
  
  .no-window {
    color: var(--text-secondary, #666);
    font-style: italic;
  }
  
  button {
    cursor: pointer;
  }
  
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>