<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  
  // State
  let audioDevices: string[] = [];
  let whisperModels: string[] = [];
  let languages: [string, string][] = [];
  let selectedDevice: string = '';
  let selectedModel: string = '';
  let selectedLanguage: string = 'auto';
  let isRecording: boolean = false;
  let transcriptionText: string = '';
  let peakLevel: number = 0;
  
  // Voice command state
  let voiceCommandsEnabled: boolean = false;
  let voiceCommandPrefix: string = '';
  let voiceCommandRequirePrefix: boolean = false;
  let lastCommand: any = null;
  let commandHistoryExpanded: boolean = false;
  let commandHistory: any[] = [];
  
  // Setup intervals for polling
  let peakLevelInterval: number | null = null;
  let transcriptionInterval: number | null = null;
  let commandCheckInterval: number | null = null;
  
  // Advanced transcription state
  let translateToEnglish: boolean = false;
  
  // Fetch data on component mount
  onMount(async () => {
    try {
      // Get devices and models from backend
      audioDevices = await invoke('get_audio_devices');
      whisperModels = await invoke('get_whisper_models');
      
      // Get language options
      try {
        languages = await invoke('get_supported_languages');
      } catch (error) {
        console.error('Failed to load language options:', error);
        // Fallback to basic languages
        languages = [
          ['auto', 'Auto-detect'],
          ['en', 'English'],
          ['es', 'Spanish'],
          ['fr', 'French'],
          ['de', 'German']
        ];
      }
      
      // Load saved settings if available
      try {
        const settings = await invoke('get_settings');
        if (settings) {
          selectedDevice = settings.device_name || (audioDevices.length > 0 ? audioDevices[0] : '');
          selectedModel = settings.model_name || (whisperModels.length > 0 ? whisperModels[0] : '');
          
          // Load speech settings if available
          if (settings.speech) {
            selectedLanguage = settings.speech.language || 'auto';
            translateToEnglish = settings.speech.translate_to_english || false;
          }
        } else {
          if (audioDevices.length > 0) selectedDevice = audioDevices[0];
          if (whisperModels.length > 0) selectedModel = whisperModels[0];
        }
      } catch (error) {
        // If settings not available, use defaults
        if (audioDevices.length > 0) selectedDevice = audioDevices[0];
        if (whisperModels.length > 0) selectedModel = whisperModels[0];
      }
      
      // Load voice command settings
      try {
        const voiceSettings = await invoke('get_voice_command_settings');
        if (voiceSettings) {
          voiceCommandsEnabled = voiceSettings.enabled;
          voiceCommandPrefix = voiceSettings.command_prefix || '';
          voiceCommandRequirePrefix = voiceSettings.require_prefix;
        }
      } catch (error) {
        console.error('Failed to load voice command settings:', error);
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
      
      // Setup interval to check for voice commands
      commandCheckInterval = window.setInterval(async () => {
        if (isRecording && voiceCommandsEnabled) {
          try {
            const command = await invoke('plugin:voice_commands:get_last_command');
            if (command && (!lastCommand || lastCommand.trigger_text !== command.trigger_text)) {
              // Add to command history
              commandHistory = [command, ...commandHistory.slice(0, 9)]; // Keep only last 10 commands
              
              // Execute command action
              executeVoiceCommand(command);
              
              // Clear the command so we don't process it again
              await invoke('plugin:voice_commands:clear_last_command');
              
              // Update lastCommand
              lastCommand = command;
            }
          } catch (error) {
            console.error('Failed to check for voice commands:', error);
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
    
    if (commandCheckInterval !== null) {
      clearInterval(commandCheckInterval);
    }
    
    // Stop recording if active
    if (isRecording) {
      stopRecording();
    }
    
    // Stop voice commands if active
    if (voiceCommandsEnabled) {
      toggleVoiceCommands(false);
    }
  });
  
  // Execute a voice command
  async function executeVoiceCommand(command: any) {
    console.log('Executing voice command:', command);
    
    // Handle different command types
    switch (command.command_type) {
      case 'delete':
        // For now just show a notification
        showCommandNotification(`Deleted last words`);
        break;
        
      case 'undo':
        showCommandNotification(`Undo action`);
        break;
        
      case 'redo':
        showCommandNotification(`Redo action`);
        break;
        
      case 'capitalize':
        showCommandNotification(`Capitalized text`);
        break;
        
      case 'lowercase':
        showCommandNotification(`Lowercased text`);
        break;
        
      case 'newline':
        showCommandNotification(`Added new line`);
        // Could insert a newline character in the transcription
        transcriptionText += '\n';
        break;
        
      case 'newparagraph':
        showCommandNotification(`Added new paragraph`);
        // Could insert a new paragraph break in the transcription
        transcriptionText += '\n\n';
        break;
        
      case 'period':
        showCommandNotification(`Added period`);
        // Could add a period to the transcription
        transcriptionText += '.';
        break;
        
      case 'comma':
        showCommandNotification(`Added comma`);
        // Could add a comma to the transcription
        transcriptionText += ',';
        break;
        
      case 'questionmark':
        showCommandNotification(`Added question mark`);
        // Could add a question mark to the transcription
        transcriptionText += '?';
        break;
        
      case 'exclamationmark':
        showCommandNotification(`Added exclamation mark`);
        // Could add an exclamation mark to the transcription
        transcriptionText += '!';
        break;
        
      case 'pause':
        showCommandNotification(`Paused recording`);
        await stopRecording();
        break;
        
      case 'resume':
        showCommandNotification(`Resumed recording`);
        await startRecording();
        break;
        
      case 'stop':
        showCommandNotification(`Stopped recording`);
        await stopRecording();
        break;
        
      default:
        showCommandNotification(`Unknown command: ${command.command_type}`);
    }
  }
  
  function showCommandNotification(message: string) {
    // For now, just log to console
    console.log('COMMAND:', message);
    // In a real app, you'd show a toast notification or UI indication
  }
  
  // Toggle voice commands
  async function toggleVoiceCommands(enable: boolean = !voiceCommandsEnabled) {
    try {
      await invoke('toggle_voice_commands', { enabled: enable });
      voiceCommandsEnabled = enable;
      
      // Save settings
      await invoke('save_voice_command_settings', {
        enabled: voiceCommandsEnabled,
        command_prefix: voiceCommandPrefix || null,
        require_prefix: voiceCommandRequirePrefix,
        sensitivity: 0.8 // Default value
      });
      
      console.log(`Voice commands ${voiceCommandsEnabled ? 'enabled' : 'disabled'}`);
    } catch (error) {
      console.error('Failed to toggle voice commands:', error);
    }
  }
  
  // Start recording and transcription
  async function startRecording() {
    try {
      // Start audio recording
      await invoke('plugin:audio:start_recording', { deviceName: selectedDevice });
      
      // Start transcription with language settings
      await invoke('plugin:transcribe:start_transcription', {
        options: {
          language: selectedLanguage, 
          translate_to_english: translateToEnglish
        }
      });
      
      // Start voice commands if enabled
      if (voiceCommandsEnabled) {
        await invoke('plugin:voice_commands:start_voice_commands');
      }
      
      isRecording = true;
      console.log(`Started recording and transcription with device: ${selectedDevice}, language: ${selectedLanguage}`);
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
      
      // Stop voice commands
      if (voiceCommandsEnabled) {
        await invoke('plugin:voice_commands:stop_voice_commands');
      }
      
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
  
  // Toggle command history panel
  function toggleCommandHistory() {
    commandHistoryExpanded = !commandHistoryExpanded;
  }
  
  // Get language display name
  function getLanguageDisplayName(code: string): string {
    const lang = languages.find(l => l[0] === code);
    return lang ? lang[1] : code;
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
  
  <div class="transcription-controls">
    <div class="language-selector">
      <label for="language-select">Language:</label>
      <select id="language-select" bind:value={selectedLanguage} disabled={isRecording}>
        {#each languages as [code, name]}
          <option value={code}>{name}</option>
        {/each}
      </select>
    </div>
    
    <div class="translate-toggle">
      <label class="toggle-switch-small">
        <input 
          type="checkbox" 
          bind:checked={translateToEnglish} 
          disabled={isRecording || selectedLanguage === 'en'}
        >
        <span class="toggle-slider-small"></span>
      </label>
      <span class="toggle-label">Translate to English</span>
    </div>
  </div>
  
  <div class="voice-commands">
    <div class="voice-commands-header">
      <h2>Voice Commands</h2>
      <label class="toggle-switch">
        <input type="checkbox" bind:checked={voiceCommandsEnabled} on:change={() => toggleVoiceCommands(voiceCommandsEnabled)}>
        <span class="toggle-slider"></span>
      </label>
    </div>
    
    {#if voiceCommandsEnabled}
      <div class="voice-command-settings">
        <div class="setting-row">
          <label for="command-prefix">Command Prefix (optional)</label>
          <input id="command-prefix" type="text" bind:value={voiceCommandPrefix} placeholder="e.g., Hey" />
        </div>
        
        <div class="setting-row">
          <label for="require-prefix">Require Prefix</label>
          <input id="require-prefix" type="checkbox" bind:checked={voiceCommandRequirePrefix} />
        </div>
      </div>
      
      <div class="command-history-header" on:click={toggleCommandHistory}>
        <h3>Command History</h3>
        <span class="expand-icon">{commandHistoryExpanded ? '▼' : '▶'}</span>
      </div>
      
      {#if commandHistoryExpanded}
        <div class="command-history">
          {#if commandHistory.length === 0}
            <p class="no-commands">No commands detected yet</p>
          {:else}
            <ul>
              {#each commandHistory as command}
                <li>
                  <span class="command-type">{command.command_type}</span>
                  <span class="command-trigger">"{command.trigger_text}"</span>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    {/if}
  </div>
  
  <div class="transcription-area">
    <div class="transcription-header">
      <div class="transcription-title">
        <h2>Transcription</h2>
        <span class="transcription-info">
          {selectedLanguage === 'auto' 
            ? 'Auto-detecting language' 
            : `Language: ${getLanguageDisplayName(selectedLanguage)}`}
          {translateToEnglish && selectedLanguage !== 'en' ? ' (Translating to English)' : ''}
        </span>
      </div>
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
    justify-content: space-between;
    margin-bottom: 1.5rem;
  }
  
  .select-container {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 30%;
  }
  
  label {
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
    color: #7f8c8d;
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
    border-radius: 20px;
    border: none;
    background-color: #3498db;
    color: white;
    font-weight: bold;
    cursor: pointer;
    transition: background-color 0.3s;
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
    height: 8px;
    background-color: #ecf0f1;
    border-radius: 4px;
    margin-bottom: 2rem;
    overflow: hidden;
  }
  
  .level-indicator {
    height: 100%;
    background-color: #3498db;
    transition: width 0.1s ease-out;
  }
  
  .transcription-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    background-color: #f8f9fa;
    border-radius: 8px;
    padding: 0.5rem 1rem;
  }
  
  .language-selector {
    display: flex;
    align-items: center;
  }
  
  .language-selector label {
    margin-right: 0.5rem;
    font-size: 0.9rem;
    color: #7f8c8d;
  }
  
  .language-selector select {
    padding: 0.3rem 0.5rem;
    border-radius: 4px;
    border: 1px solid #ddd;
    background-color: white;
    font-size: 0.9rem;
  }
  
  .translate-toggle {
    display: flex;
    align-items: center;
  }
  
  .toggle-switch-small {
    position: relative;
    display: inline-block;
    width: 40px;
    height: 20px;
    margin-right: 0.5rem;
  }
  
  .toggle-switch-small input {
    opacity: 0;
    width: 0;
    height: 0;
  }
  
  .toggle-slider-small {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: #ccc;
    transition: .4s;
    border-radius: 20px;
  }
  
  .toggle-slider-small:before {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: .4s;
    border-radius: 50%;
  }
  
  input:checked + .toggle-slider-small {
    background-color: #2ecc71;
  }
  
  input:focus + .toggle-slider-small {
    box-shadow: 0 0 1px #2ecc71;
  }
  
  input:checked + .toggle-slider-small:before {
    transform: translateX(20px);
  }
  
  .toggle-label {
    font-size: 0.9rem;
    color: #7f8c8d;
  }
  
  .transcription-title {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }
  
  .transcription-info {
    font-size: 0.8rem;
    color: #7f8c8d;
    margin-top: 0.2rem;
  }
  
  .voice-commands {
    margin-bottom: 2rem;
    background-color: #f8f9fa;
    border-radius: 8px;
    padding: 1rem;
    text-align: left;
  }
  
  .voice-commands-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .voice-commands-header h2 {
    margin: 0;
    font-size: 1.5rem;
    color: #2c3e50;
  }
  
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 50px;
    height: 24px;
  }
  
  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }
  
  .toggle-slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: #ccc;
    transition: .4s;
    border-radius: 24px;
  }
  
  .toggle-slider:before {
    position: absolute;
    content: "";
    height: 16px;
    width: 16px;
    left: 4px;
    bottom: 4px;
    background-color: white;
    transition: .4s;
    border-radius: 50%;
  }
  
  input:checked + .toggle-slider {
    background-color: #2ecc71;
  }
  
  input:focus + .toggle-slider {
    box-shadow: 0 0 1px #2ecc71;
  }
  
  input:checked + .toggle-slider:before {
    transform: translateX(26px);
  }
  
  .voice-command-settings {
    margin-top: 1rem;
    padding: 0.5rem 0;
    border-top: 1px solid #ddd;
    border-bottom: 1px solid #ddd;
  }
  
  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin: 0.5rem 0;
  }
  
  .setting-row input[type="text"] {
    flex: 1;
    max-width: 200px;
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid #ddd;
  }
  
  .command-history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 1rem;
    cursor: pointer;
  }
  
  .command-history-header h3 {
    margin: 0.5rem 0;
    font-size: 1.1rem;
    color: #34495e;
  }
  
  .expand-icon {
    font-size: 0.9rem;
    color: #7f8c8d;
  }
  
  .command-history {
    background-color: #fff;
    border: 1px solid #e0e0e0;
    border-radius: 4px;
    padding: 0.5rem;
    margin-top: 0.5rem;
    max-height: 150px;
    overflow-y: auto;
  }
  
  .command-history ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  
  .command-history li {
    margin: 0.3rem 0;
    font-size: 0.9rem;
  }
  
  .command-type {
    font-weight: bold;
    color: #3498db;
  }
  
  .command-trigger {
    margin-left: 0.5rem;
    color: #7f8c8d;
    font-style: italic;
  }
  
  .no-commands {
    text-align: center;
    color: #95a5a6;
    font-style: italic;
    font-size: 0.9rem;
  }
  
  .transcription-area {
    background-color: #fff;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    padding: 1rem;
    text-align: left;
  }
  
  .transcription-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .transcription-header h2 {
    margin: 0;
    font-size: 1.5rem;
    color: #2c3e50;
  }
  
  .clear-button {
    padding: 0.3rem 0.8rem;
    border-radius: 4px;
    border: none;
    background-color: #e74c3c;
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
  }
  
  .clear-button:hover {
    background-color: #c0392b;
  }
  
  .transcription-text {
    min-height: 200px;
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.5;
    color: #34495e;
  }
  
  .placeholder {
    color: #95a5a6;
    font-style: italic;
  }
  
  footer {
    margin-top: 2rem;
    font-size: 0.8rem;
    color: #95a5a6;
  }
</style> 
