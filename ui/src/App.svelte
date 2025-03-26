<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  
  // State
  let audioDevices: string[] = [];
  let whisperModels: string[] = [];
  let selectedDevice: string = '';
  let selectedModel: string = '';
  let isRecording: boolean = false;
  let transcriptionText: string = '';
  let peakLevel: number = 0;
  
  // Setup intervals for polling
  let peakLevelInterval: number | null = null;
  let transcriptionInterval: number | null = null;
  
  // Fetch data on component mount
  onMount(async () => {
    try {
      // Get devices and models from backend
      audioDevices = await invoke('get_audio_devices');
      whisperModels = await invoke('get_whisper_models');
      
      // Load saved settings if available
      try {
        const settings = await invoke('get_settings');
        if (settings) {
          selectedDevice = settings.device_name || (audioDevices.length > 0 ? audioDevices[0] : '');
          selectedModel = settings.model_name || (whisperModels.length > 0 ? whisperModels[0] : '');
        } else {
          if (audioDevices.length > 0) selectedDevice = audioDevices[0];
          if (whisperModels.length > 0) selectedModel = whisperModels[0];
        }
      } catch (error) {
        // If settings not available, use defaults
        if (audioDevices.length > 0) selectedDevice = audioDevices[0];
        if (whisperModels.length > 0) selectedModel = whisperModels[0];
      }
      
      // Setup interval to poll for peak level
      peakLevelInterval = window.setInterval(async () => {
        if (isRecording) {
          try {
            peakLevel = await invoke('plugin:audio:get_peak_level');
          } catch (error) {
            console.error('Failed to get peak level:', error);
          }
        }
      }, 100);
      
      // Setup interval to poll for transcription
      transcriptionInterval = window.setInterval(async () => {
        if (isRecording) {
          try {
            const newText = await invoke('plugin:transcribe:get_transcription');
            if (newText !== transcriptionText) {
              transcriptionText = newText;
            }
          } catch (error) {
            console.error('Failed to get transcription:', error);
          }
        }
      }, 300);
    } catch (error) {
      console.error('Failed to load initial data:', error);
    }
  });
  
  // Clean up on component destroy
  onDestroy(() => {
    if (peakLevelInterval !== null) {
      clearInterval(peakLevelInterval);
    }
    
    if (transcriptionInterval !== null) {
      clearInterval(transcriptionInterval);
    }
    
    // Stop recording if active
    if (isRecording) {
      stopRecording();
    }
  });
  
  // Start recording and transcription
  async function startRecording() {
    try {
      // Start audio recording
      await invoke('plugin:audio:start_recording', { deviceName: selectedDevice });
      
      // Start transcription
      await invoke('plugin:transcribe:start_transcription');
      
      isRecording = true;
      console.log('Started recording and transcription with device:', selectedDevice);
    } catch (error) {
      console.error('Failed to start recording:', error);
    }
  }
  
  // Stop recording and transcription
  async function stopRecording() {
    try {
      // Stop transcription first
      await invoke('plugin:transcribe:stop_transcription');
      
      // Then stop audio recording
      await invoke('plugin:audio:stop_recording');
      
      isRecording = false;
      console.log('Stopped recording and transcription');
    } catch (error) {
      console.error('Failed to stop recording:', error);
    }
  }
  
  // Handle start/stop recording
  const toggleRecording = async () => {
    if (!isRecording) {
      await startRecording();
    } else {
      await stopRecording();
    }
  };
  
  // Clear transcription text
  async function clearTranscription() {
    try {
      await invoke('plugin:transcribe:clear_transcription');
      transcriptionText = '';
    } catch (error) {
      console.error('Failed to clear transcription:', error);
    }
  }
</script>

<main>
  <h1>BestMe</h1>
  
  <div class="controls">
    <div class="select-container">
      <label for="device-select">Audio Device</label>
      <select id="device-select" bind:value={selectedDevice}>
        {#each audioDevices as device}
          <option value={device}>{device}</option>
        {/each}
      </select>
    </div>
    
    <div class="select-container">
      <label for="model-select">Whisper Model</label>
      <select id="model-select" bind:value={selectedModel}>
        {#each whisperModels as model}
          <option value={model}>{model}</option>
        {/each}
      </select>
    </div>
    
    <button class="record-button" class:recording={isRecording} on:click={toggleRecording}>
      {isRecording ? 'Stop' : 'Start'} Recording
    </button>
  </div>
  
  <div class="level-meter">
    <div class="level-indicator" style="width: {peakLevel * 100}%"></div>
  </div>
  
  <div class="transcription-area">
    <div class="transcription-header">
      <h2>Transcription</h2>
      <button class="clear-button" on:click={clearTranscription}>Clear</button>
    </div>
    <div class="transcription-text">
      {#if transcriptionText}
        {transcriptionText}
      {:else}
        <span class="placeholder">Start recording to see transcription...</span>
      {/if}
    </div>
  </div>
  
  <footer>
    <p>BestMe v0.1.0 - Modern Speech-to-Text Application</p>
  </footer>
</main>

<style>
  main {
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
    text-align: center;
  }
  
  h1 {
    font-size: 2rem;
    margin-bottom: 2rem;
    color: #2c3e50;
  }
  
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    justify-content: center;
    margin-bottom: 2rem;
  }
  
  .select-container {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    min-width: 200px;
  }
  
  label {
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
  }
  
  .record-button {
    padding: 0.5rem 1.5rem;
    border-radius: 4px;
    border: none;
    background-color: #3498db;
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
    align-self: flex-end;
  }
  
  .record-button:hover {
    background-color: #2980b9;
  }
  
  .record-button.recording {
    background-color: #e74c3c;
  }
  
  .record-button.recording:hover {
    background-color: #c0392b;
  }
  
  .level-meter {
    height: 24px;
    background-color: #ecf0f1;
    border-radius: 12px;
    margin-bottom: 2rem;
    overflow: hidden;
  }
  
  .level-indicator {
    height: 100%;
    background-color: #2ecc71;
    transition: width 0.1s ease-out;
  }
  
  .transcription-area {
    text-align: left;
    margin-bottom: 2rem;
  }
  
  .transcription-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  h2 {
    font-size: 1.2rem;
    color: #2c3e50;
    margin: 0;
  }
  
  .clear-button {
    font-size: 0.8rem;
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    border: none;
    background-color: #e0e0e0;
    color: #555;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .clear-button:hover {
    background-color: #d0d0d0;
  }
  
  .transcription-text {
    min-height: 200px;
    padding: 1rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    background-color: white;
    white-space: pre-wrap;
  }
  
  .placeholder {
    color: #95a5a6;
    font-style: italic;
  }
  
  footer {
    margin-top: 2rem;
    color: #7f8c8d;
    font-size: 0.8rem;
  }
</style> 
