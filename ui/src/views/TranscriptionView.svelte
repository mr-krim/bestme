<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  // Define expected types from backend
  type AudioDevice = string; // Assuming backend sends device names as strings for now
  type WhisperModel = string; // Assuming backend sends model names as strings
  type LanguageTuple = [string, string]; // [code, name]
  
  // Props
  export let transcriptionText = '';
  export let isRecording = false;
  export let peakLevel = 0;
  export let selectedDevice: AudioDevice | null = null; // Allow null initially
  export let selectedModel: WhisperModel | null = null; // Allow null initially
  export let selectedLanguage = 'auto';
  export let translateToEnglish = false;
  export let autoPunctuate = true;
  
  // Local state with types
  let devices: AudioDevice[] = [];
  let models: WhisperModel[] = [];
  let languages: LanguageTuple[] = [];
  let wordCount = 0;
  
  // Event dispatcher
  const dispatch = createEventDispatcher();
  
  // Intervals - Use ReturnType<typeof setInterval> for Node.js/browser typing
  let transcriptionInterval: ReturnType<typeof setInterval> | undefined;
  let peakLevelInterval: ReturnType<typeof setInterval> | undefined;
  
  // Computed properties
  $: {
    if (transcriptionText) {
      wordCount = transcriptionText.split(/\s+/).filter(Boolean).length;
    } else {
      wordCount = 0;
    }
    
    dispatch('updateStats', { wordCount });
  }
  
  onMount(async () => {
    try {
      // Add types to invoke calls
      devices = await invoke<AudioDevice[]>('audio.get_audio_devices');
      models = await invoke<WhisperModel[]>('transcribe.get_whisper_models');
      languages = await invoke<LanguageTuple[]>('transcribe.get_supported_languages');
      
      // Set defaults if not provided
      if (!selectedDevice && devices.length > 0) {
        selectedDevice = devices[0];
      }
      
      if (!selectedModel && models.length > 0) {
        selectedModel = models[0];
      }
      
      transcriptionInterval = setInterval(async () => {
        if (isRecording) {
          try {
            // Assume transcription is a string
            const newText = await invoke<string>('transcribe.get_transcription');
            if (newText && newText !== transcriptionText) {
              transcriptionText = newText;
            }
          } catch (error) {
            console.error('Failed to get transcription:', error);
          }
        }
      }, 300);
      
      peakLevelInterval = setInterval(async () => {
        if (isRecording) {
          try {
            // Assume peak level is a number
            peakLevel = await invoke<number>('audio.get_peak_level');
          } catch (error) {
            console.error('Failed to get peak level:', error);
          }
        }
      }, 100);
    } catch (error) {
      console.error('Error initializing transcription view:', error);
    }
  });
  
  onDestroy(() => {
    if (transcriptionInterval) {
      clearInterval(transcriptionInterval);
    }
    
    if (peakLevelInterval) {
      clearInterval(peakLevelInterval);
    }
    
    if (isRecording) {
      stopRecording();
    }
  });
  
  // Toggle recording
  async function toggleRecording() {
    if (isRecording) {
      await stopRecording();
    } else {
      await startRecording();
    }
  }
  
  // Start recording
  async function startRecording() {
    try {
      await invoke('audio.start_recording', {
        deviceName: selectedDevice,
        model: selectedModel,
        language: selectedLanguage,
        translateToEnglish,
        autoPunctuate
      });
      
      isRecording = true;
    } catch (error) {
      console.error('Failed to start recording:', error);
    }
  }
  
  // Stop recording
  async function stopRecording() {
    try {
      await invoke('audio.stop_recording');
      isRecording = false;
      peakLevel = 0;
    } catch (error) {
      console.error('Failed to stop recording:', error);
    }
  }
  
  // Type the event parameter
  function updateDevice(event: Event) {
    const target = event.target as HTMLSelectElement;
    selectedDevice = target.value;
  }
  
  function updateModel(event: Event) {
    const target = event.target as HTMLSelectElement;
    selectedModel = target.value;
  }
  
  function updateLanguage(event: Event) {
    const target = event.target as HTMLSelectElement;
    selectedLanguage = target.value;
  }
</script>

<div class="transcription-view">
  <div class="control-panel">
    <button 
      class="record-button {isRecording ? 'active' : ''}" 
      on:click={toggleRecording}
    >
      {isRecording ? 'Stop Recording' : 'Start Recording'}
    </button>
    
    <div class="settings-group">
      <div class="setting">
        <label for="device-select">Device:</label>
        <select id="device-select" value={selectedDevice} on:change={updateDevice}>
          {#each devices as device}
            <option value={device}>{device}</option>
          {/each}
        </select>
      </div>
      
      <div class="setting">
        <label for="model-select">Model:</label>
        <select id="model-select" value={selectedModel} on:change={updateModel}>
          {#each models as model}
            <option value={model}>{model}</option>
          {/each}
        </select>
      </div>
      
      <div class="setting">
        <label for="language-select">Language:</label>
        <select id="language-select" value={selectedLanguage} on:change={updateLanguage}>
          {#each languages as [code, name]}
            <option value={code}>{name}</option>
          {/each}
        </select>
      </div>
    </div>
    
    <div class="checkboxes">
      <label class="checkbox">
        <input type="checkbox" bind:checked={translateToEnglish} />
        <span>Translate to English</span>
      </label>
      
      <label class="checkbox">
        <input type="checkbox" bind:checked={autoPunctuate} />
        <span>Auto Punctuate</span>
      </label>
    </div>
  </div>
  
  <!-- Audio Visualization -->
  <div class="audio-visualization">
    {#if isRecording}
      <div class="waveform">
        {#each Array(20) as _, i}
          <div class="bar" style="height: {Math.max(2, Math.random() * peakLevel * 100)}%"></div>
        {/each}
      </div>
    {:else}
      <div class="placeholder">
        <span>Audio visualization will appear here when recording</span>
      </div>
    {/if}
  </div>
  
  <!-- Transcription Output -->
  <div class="transcription-output">
    {#if transcriptionText}
      <div class="transcription-text">
        {transcriptionText}
      </div>
    {:else}
      <div class="placeholder">
        <span>Your transcription will appear here</span>
      </div>
    {/if}
  </div>
  
  <!-- Quick Actions -->
  <div class="quick-actions">
    <button class="action-button" disabled={!transcriptionText}>
      Copy
    </button>
    <button class="action-button" disabled={!transcriptionText}>
      Save
    </button>
    <button class="action-button" disabled={!transcriptionText}>
      Clear
    </button>
  </div>
</div>

<style>
  .transcription-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 16px;
  }
  
  .control-panel {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background-color: var(--panel-bg, #f5f5f5);
    border-radius: 4px;
  }
  
  .record-button {
    padding: 8px 16px;
    background-color: var(--button-bg, #f0f0f0);
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    font-weight: 500;
  }
  
  .record-button:hover {
    background-color: var(--button-hover-bg, #e5e5e5);
  }
  
  .record-button.active {
    background-color: var(--active-button-bg, #ff5252);
    color: white;
  }
  
  .settings-group {
    display: flex;
    gap: 12px;
  }
  
  .setting {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  
  .setting label {
    font-size: 0.85rem;
    color: var(--text-secondary, #666);
  }
  
  .setting select {
    padding: 6px 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    min-width: 120px;
  }
  
  .checkboxes {
    display: flex;
    gap: 12px;
  }
  
  .checkbox {
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
  }
  
  .audio-visualization {
    height: 80px;
    background-color: var(--visualization-bg, #f9f9f9);
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    overflow: hidden;
    position: relative;
  }
  
  .waveform {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 100%;
    padding: 0 16px;
  }
  
  .bar {
    width: 3px;
    background-color: var(--waveform-color, #2196f3);
    margin-right: 3px;
    border-radius: 1px;
    animation: pulse 0.5s infinite alternate;
  }
  
  @keyframes pulse {
    from { opacity: 0.7; }
    to { opacity: 1; }
  }
  
  .transcription-output {
    flex: 1;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    padding: 16px;
    overflow-y: auto;
    background-color: var(--output-bg, white);
  }
  
  .transcription-text {
    white-space: pre-wrap;
    line-height: 1.6;
  }
  
  .placeholder {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    color: var(--text-tertiary, #999);
    font-style: italic;
  }
  
  .quick-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  
  .action-button {
    padding: 6px 12px;
    background-color: var(--button-bg, #f0f0f0);
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .action-button:hover:not(:disabled) {
    background-color: var(--button-hover-bg, #e5e5e5);
  }
  
  .action-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    :root {
      --panel-bg: #252525;
      --button-bg: #333;
      --button-hover-bg: #444;
      --active-button-bg: #b71c1c;
      --border-color: #444;
      --text-secondary: #aaa;
      --text-tertiary: #888;
      --visualization-bg: #222;
      --waveform-color: #1976d2;
      --output-bg: #1a1a1a;
    }
    
    .setting select {
      background-color: #333;
      color: #eee;
    }
  }
</style> 
