<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import VADVisualization from '../components/VADVisualization.svelte';
  import { AITools } from '../components/ai';
  
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
  
  // Voice command state
  let voiceCommandEnabled = false;
  let aiVoiceCommandsEnabled = false;
  let lastCommand: { command: string; success: boolean; result?: string; error?: string } | null = null;
  let commandFeedbackTimeout: ReturnType<typeof setTimeout> | undefined;
  
  // AI Tools state
  let showAITools = false;
  
  // Event dispatcher
  const dispatch = createEventDispatcher();
  
  // Intervals - Use ReturnType<typeof setInterval> for Node.js/browser typing
  let transcriptionInterval: ReturnType<typeof setInterval> | undefined;
  let peakLevelInterval: ReturnType<typeof setInterval> | undefined;
  
  // Event listeners
  let voiceCommandListener: (() => void) | undefined;
  
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
      
      // Listen for voice command events
      voiceCommandListener = await listen('voice-command:executed', (event) => {
        const payload = event.payload as { command: string; success: boolean; result?: string; error?: string };
        lastCommand = payload;
        
        // Clear previous feedback timeout
        if (commandFeedbackTimeout) {
          clearTimeout(commandFeedbackTimeout);
        }
        
        // Hide feedback after 5 seconds
        commandFeedbackTimeout = setTimeout(() => {
          lastCommand = null;
        }, 5000);
      });
      
      // Check if voice commands are enabled
      try {
        // This assumes we have a command to check voice command status
        // We'll need to implement this in the backend if not present
        voiceCommandEnabled = await invoke<boolean>('transcribe.is_voice_commands_enabled');
      } catch (error) {
        console.warn('Could not check voice command status:', error);
      }
      
      // Check if AI voice commands are enabled
      try {
        const aiSettings = await invoke<any>('get_ai_voice_settings');
        aiVoiceCommandsEnabled = aiSettings?.enabled || false;
      } catch (error) {
        console.warn('Could not check AI voice command status:', error);
      }
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
    
    if (voiceCommandListener) {
      voiceCommandListener();
    }
    
    if (commandFeedbackTimeout) {
      clearTimeout(commandFeedbackTimeout);
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
  
  // Toggle voice commands
  async function toggleVoiceCommands() {
    try {
      await invoke('transcribe.set_voice_commands_enabled', { enabled: !voiceCommandEnabled });
      voiceCommandEnabled = !voiceCommandEnabled;
    } catch (error) {
      console.error('Failed to toggle voice commands:', error);
    }
  }
  
  // AI Tools handlers
  function toggleAITools() {
    showAITools = !showAITools;
  }
  
  function handleAIProcessed(event) {
    console.log('AI processed:', event.detail);
    // Could show a notification or update UI
  }
  
  function handleAICopied(event) {
    console.log('Copied to clipboard:', event.detail);
    // Could show a toast notification
  }
  
  // Copy transcription to clipboard
  async function copyTranscription() {
    if (transcriptionText) {
      await navigator.clipboard.writeText(transcriptionText);
      // Could show a notification
    }
  }
  
  // Clear transcription
  function clearTranscription() {
    transcriptionText = '';
  }
  
  // Save transcription (placeholder - implement based on your needs)
  async function saveTranscription() {
    if (transcriptionText) {
      // Implement save functionality
      console.log('Save functionality to be implemented');
    }
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
      
      <label class="checkbox">
        <input type="checkbox" bind:checked={voiceCommandEnabled} on:change={toggleVoiceCommands} />
        <span>Voice Commands</span>
      </label>
    </div>
  </div>
  
  <!-- Audio Visualization -->
  <div class="audio-visualization">
    <!-- VAD Visualization -->
    <VADVisualization />
    
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
  
  <!-- Voice Command Feedback -->
  {#if voiceCommandEnabled && lastCommand}
    <div class="voice-command-feedback {lastCommand.success ? 'success' : 'error'}">
      <div class="command-icon">
        {lastCommand.success ? '✓' : '✗'}
      </div>
      <div class="command-details">
        <div class="command-name">Command: {lastCommand.command}</div>
        {#if lastCommand.result}
          <div class="command-result">{lastCommand.result}</div>
        {/if}
        {#if lastCommand.error}
          <div class="command-error">{lastCommand.error}</div>
        {/if}
      </div>
    </div>
  {/if}
  
  <!-- AI Voice Command Status -->
  {#if voiceCommandEnabled && aiVoiceCommandsEnabled}
    <div class="ai-status-indicator">
      <span class="ai-icon">🤖</span>
      <span>AI Voice Commands Active</span>
    </div>
  {/if}
  
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
    <button class="action-button" disabled={!transcriptionText} on:click={copyTranscription}>
      Copy
    </button>
    <button class="action-button" disabled={!transcriptionText} on:click={saveTranscription}>
      Save
    </button>
    <button class="action-button" disabled={!transcriptionText} on:click={clearTranscription}>
      Clear
    </button>
    <button class="action-button ai-tools" disabled={!transcriptionText} on:click={toggleAITools}>
      🤖 AI Tools
    </button>
  </div>
  
  <!-- AI Tools Panel -->
  {#if showAITools}
    <div class="ai-tools-panel">
      <AITools
        initialText={transcriptionText}
        on:processed={handleAIProcessed}
        on:copied={handleAICopied}
      />
    </div>
  {/if}
</div>

<style>
  .transcription-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px;
    box-sizing: border-box;
    overflow: hidden;
  }
  
  .control-panel {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background-color: var(--panel-bg, #f5f5f5);
    border-radius: 4px;
    margin-bottom: 12px;
    flex-shrink: 0;
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
    margin-bottom: 12px;
    flex-shrink: 0;
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
    margin-top: 12px;
    flex-shrink: 0;
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
  
  .action-button.ai-tools {
    background-color: var(--ai-button-bg, #673ab7);
    color: white;
  }
  
  .action-button.ai-tools:hover:not(:disabled) {
    background-color: var(--ai-button-hover, #5e35b1);
  }
  
  /* AI Tools Panel */
  .ai-tools-panel {
    margin-top: 16px;
    padding: 20px;
    background-color: var(--panel-bg, #f5f5f5);
    border-radius: 8px;
    border: 1px solid var(--border-color, #e0e0e0);
    animation: slideDown 0.3s ease-out;
  }
  
  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  /* Voice Command Feedback */
  .voice-command-feedback {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 4px;
    animation: slideIn 0.3s ease-out;
    margin-bottom: 12px;
    flex-shrink: 0;
  }
  
  .voice-command-feedback.success {
    background-color: var(--success-bg, #e8f5e9);
    border: 1px solid var(--success-border, #4caf50);
  }
  
  .voice-command-feedback.error {
    background-color: var(--error-bg, #ffebee);
    border: 1px solid var(--error-border, #f44336);
  }
  
  /* AI Status Indicator */
  .ai-status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background-color: var(--ai-bg, #e3f2fd);
    border: 1px solid var(--ai-border, #2196f3);
    border-radius: 4px;
    margin-bottom: 12px;
    font-size: 0.9em;
    flex-shrink: 0;
    color: var(--ai-text, #1565c0);
  }
  
  .ai-icon {
    font-size: 1.2em;
  }
  
  .command-icon {
    font-size: 20px;
    font-weight: bold;
  }
  
  .voice-command-feedback.success .command-icon {
    color: var(--success-color, #4caf50);
  }
  
  .voice-command-feedback.error .command-icon {
    color: var(--error-color, #f44336);
  }
  
  .command-details {
    flex: 1;
  }
  
  .command-name {
    font-weight: 500;
    margin-bottom: 4px;
  }
  
  .command-result {
    font-size: 0.9rem;
    color: var(--text-secondary, #666);
  }
  
  .command-error {
    font-size: 0.9rem;
    color: var(--error-color, #f44336);
  }
  
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
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
      --success-bg: #1b5e20;
      --success-border: #2e7d32;
      --success-color: #4caf50;
      --error-bg: #3e2723;
      --error-border: #c62828;
      --error-color: #f44336;
    }
    
    .setting select {
      background-color: #333;
      color: #eee;
    }
  }
</style> 
