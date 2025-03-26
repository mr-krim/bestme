<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import { listen } from '@tauri-apps/api/event';
  
  // Settings state
  let audioDevices: string[] = [];
  let whisperModels: string[] = [];
  let modelInfo: any[] = [];
  let selectedDevice: string = '';
  let selectedModel: string = 'small';
  let isAutoTranscribe: boolean = true;
  let offlineMode: boolean = true;
  let isSaving: boolean = false;
  let saveMessage: string = '';

  // Download state
  let downloadingModel: string | null = null;
  let downloadProgress: number = 0;
  let downloadError: string = '';
  let modelExists: {[key: string]: boolean} = {};
  let isCheckingModels: boolean = true;
  
  // Clean up listeners
  let unlistenFns: Array<() => void> = [];
  
  onMount(async () => {
    try {
      // Get devices and models
      audioDevices = await invoke('get_audio_devices');
      whisperModels = await invoke('get_whisper_models');
      modelInfo = await invoke('get_model_download_info');
      
      // Load saved settings
      const settings = await invoke('get_settings');
      if (settings) {
        selectedDevice = settings.device_name || (audioDevices.length > 0 ? audioDevices[0] : '');
        selectedModel = settings.model_name || 'small';
        isAutoTranscribe = settings.auto_transcribe !== undefined ? settings.auto_transcribe : true;
        offlineMode = settings.offline_mode !== undefined ? settings.offline_mode : true;
      } else if (audioDevices.length > 0) {
        selectedDevice = audioDevices[0];
      }
      
      // Check if models exist and download progress
      await checkDownloadStatus();
      
      // Listen for download progress events
      const unlisten1 = await listen('model-download-progress', (event: any) => {
        const { model, progress } = event.payload;
        if (model) {
          downloadingModel = model;
          downloadProgress = Math.floor(progress * 100);
        }
      });
      
      // Listen for download complete events
      const unlisten2 = await listen('model-download-complete', async (event: any) => {
        const { model } = event.payload;
        downloadingModel = null;
        downloadProgress = 0;
        
        // Update model status
        await checkDownloadStatus();
      });
      
      // Store unlisteners for cleanup
      unlistenFns.push(unlisten1, unlisten2);
    } catch (error) {
      console.error('Failed to load settings data:', error);
    }
  });
  
  onDestroy(() => {
    // Clean up all listeners
    unlistenFns.forEach(fn => fn());
  });
  
  // Check download status of all models
  async function checkDownloadStatus() {
    isCheckingModels = true;
    try {
      // Check progress of any ongoing downloads
      const progress: any = await invoke('plugin:transcribe:get_download_progress');
      if (progress) {
        const [model, percentage] = progress;
        downloadingModel = model;
        downloadProgress = Math.floor(percentage * 100);
      } else {
        downloadingModel = null;
        downloadProgress = 0;
      }
      
      // This would be implemented to check if model files exist
      // In a full implementation, we would add a command to check each model file
      // For now, we'll simulate this with a placeholder function that returns after a delay
      await new Promise(resolve => setTimeout(resolve, 500));
      
      // Simulated check (in a real implementation, we'd call backend to check files)
      modelExists = {
        tiny: false,
        base: false,
        small: false,
        medium: false,
        large: false
      };
      
      isCheckingModels = false;
    } catch (error) {
      console.error('Failed to check model status:', error);
      isCheckingModels = false;
    }
  }
  
  // Download a model
  async function downloadModel(model: string) {
    if (downloadingModel) {
      return; // Already downloading
    }
    
    downloadError = '';
    try {
      await invoke('plugin:transcribe:download_model_command', { modelSize: model });
      // The actual progress updates will come through the event listener
    } catch (error) {
      console.error(`Failed to download model ${model}:`, error);
      downloadError = `Failed to download: ${error}`;
      downloadingModel = null;
      downloadProgress = 0;
    }
  }
  
  // Save settings
  async function saveSettings() {
    try {
      isSaving = true;
      saveMessage = '';
      
      // Save settings to backend
      await invoke('save_settings', {
        deviceName: selectedDevice,
        modelName: selectedModel,
        autoTranscribe: isAutoTranscribe,
        offlineMode: offlineMode
      });
      
      saveMessage = 'Settings saved successfully!';
      
      // Clear message after 3 seconds
      setTimeout(() => {
        saveMessage = '';
      }, 3000);
    } catch (error) {
      console.error('Failed to save settings:', error);
      saveMessage = `Error: ${error}`;
    } finally {
      isSaving = false;
    }
  }
  
  // Get model info by name
  function getModelInfo(name: string) {
    return modelInfo.find(m => m.name === name) || { size_mb: '?', description: '?' };
  }
</script>

<main>
  <h1>BestMe Settings</h1>
  
  <div class="settings-container">
    <section>
      <h2>Audio Settings</h2>
      
      <div class="setting-item">
        <label for="device-select">Recording Device</label>
        <select id="device-select" bind:value={selectedDevice}>
          {#each audioDevices as device}
            <option value={device}>{device}</option>
          {/each}
        </select>
      </div>
      
      <div class="setting-item">
        <label>
          <input type="checkbox" bind:checked={isAutoTranscribe} />
          Start transcription automatically
        </label>
      </div>
    </section>
    
    <section>
      <h2>Transcription Settings</h2>
      
      <div class="setting-item">
        <label for="model-select">Whisper Model</label>
        <select id="model-select" bind:value={selectedModel}>
          {#each whisperModels as model}
            <option value={model}>{model}</option>
          {/each}
        </select>
        <div class="help-text">
          Larger models are more accurate but use more resources.
          <br />
          Recommended: small (good balance of speed and accuracy)
        </div>
      </div>
      
      <div class="setting-item">
        <label>
          <input type="checkbox" bind:checked={offlineMode} />
          Offline Mode (no data sent to cloud)
        </label>
      </div>

      <h3>Model Downloads</h3>
      <div class="model-downloads">
        {#if isCheckingModels}
          <div class="loading">Checking model status...</div>
        {:else}
          {#each whisperModels as model}
            <div class="model-item">
              <div class="model-info">
                <strong>{model}</strong> - {getModelInfo(model).description}
              </div>
              <div class="model-actions">
                {#if downloadingModel === model}
                  <div class="download-progress">
                    <progress value={downloadProgress} max="100"></progress>
                    <span>{downloadProgress}%</span>
                  </div>
                {:else if modelExists[model]}
                  <span class="model-status downloaded">✓ Downloaded</span>
                {:else}
                  <button 
                    class="download-button" 
                    on:click={() => downloadModel(model)}
                    disabled={downloadingModel !== null}
                  >
                    Download
                  </button>
                {/if}
              </div>
            </div>
          {/each}

          {#if downloadError}
            <div class="error-message">{downloadError}</div>
          {/if}
        {/if}
      </div>
      
      <div class="help-text model-help">
        Models are downloaded to your local machine and saved for future use.
        <br>
        Download only the models you need to save disk space.
      </div>
    </section>
    
    <div class="buttons">
      {#if saveMessage}
        <div class="save-message" class:error={saveMessage.startsWith('Error')}>
          {saveMessage}
        </div>
      {/if}
      <button class="save-button" on:click={saveSettings} disabled={isSaving}>
        {isSaving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  </div>
</main>

<style>
  main {
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  h1 {
    font-size: 2rem;
    margin-bottom: 2rem;
    color: #2c3e50;
    text-align: center;
  }
  
  h2 {
    font-size: 1.2rem;
    color: #2c3e50;
    margin-bottom: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #ecf0f1;
  }
  
  h3 {
    font-size: 1.1rem;
    color: #2c3e50;
    margin: 1.5rem 0 1rem;
  }
  
  .settings-container {
    background-color: white;
    border-radius: 8px;
    padding: 2rem;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
  }
  
  section {
    margin-bottom: 2rem;
  }
  
  .setting-item {
    margin-bottom: 1.5rem;
  }
  
  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: #2c3e50;
  }
  
  select {
    width: 100%;
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid #ddd;
    background-color: white;
    margin-bottom: 0.5rem;
  }
  
  input[type="checkbox"] {
    margin-right: 0.5rem;
    transform: scale(1.2);
  }
  
  .help-text {
    font-size: 0.8rem;
    color: #7f8c8d;
    margin-top: 0.25rem;
  }
  
  .model-help {
    margin-top: 1rem;
    padding-top: 0.5rem;
  }
  
  .buttons {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    margin-top: 2rem;
    gap: 1rem;
  }
  
  .save-message {
    font-size: 0.9rem;
    color: #27ae60;
    flex: 1;
    text-align: right;
  }
  
  .save-message.error {
    color: #e74c3c;
  }
  
  .save-button {
    padding: 0.5rem 1.5rem;
    border-radius: 4px;
    border: none;
    background-color: #2ecc71;
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
    min-width: 120px;
  }
  
  .save-button:hover:not(:disabled) {
    background-color: #27ae60;
  }
  
  .save-button:disabled {
    background-color: #95a5a6;
    cursor: not-allowed;
    opacity: 0.7;
  }
  
  .model-downloads {
    background-color: #f8f9fa;
    border-radius: 4px;
    padding: 0.5rem;
    margin-bottom: 1rem;
  }
  
  .model-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    border-bottom: 1px solid #eee;
  }
  
  .model-item:last-child {
    border-bottom: none;
  }
  
  .model-info {
    flex: 1;
  }
  
  .model-actions {
    display: flex;
    align-items: center;
    min-width: 120px;
    justify-content: flex-end;
  }
  
  .download-button {
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    border: none;
    background-color: #3498db;
    color: white;
    font-size: 0.8rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .download-button:hover:not(:disabled) {
    background-color: #2980b9;
  }
  
  .download-button:disabled {
    background-color: #95a5a6;
    cursor: not-allowed;
    opacity: 0.7;
  }
  
  .download-progress {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
  }
  
  .download-progress progress {
    flex: 1;
    height: 10px;
  }
  
  .download-progress span {
    font-size: 0.8rem;
    min-width: 32px;
  }
  
  .model-status {
    font-size: 0.8rem;
  }
  
  .model-status.downloaded {
    color: #27ae60;
  }
  
  .error-message {
    color: #e74c3c;
    margin-top: 0.5rem;
    font-size: 0.8rem;
    padding: 0.5rem;
    background-color: #f8d7da;
    border-radius: 4px;
  }
  
  .loading {
    padding: 1rem;
    text-align: center;
    font-style: italic;
    color: #7f8c8d;
  }
</style> 
