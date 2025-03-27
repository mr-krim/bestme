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
  
  // Voice command variables
  let commandFeedback = null;
  let commandFeedbackTimeout = null;
  
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
        // Use the new config API
        const voiceConfig = await invoke('plugin:voice_commands:get_voice_command_config');
        if (voiceConfig) {
          voiceCommandsEnabled = voiceConfig.enabled;
          voiceCommandPrefix = voiceConfig.prefix || 'computer';
          voiceCommandRequirePrefix = voiceConfig.require_prefix;
        }
      } catch (error) {
        console.error('Failed to load voice command config:', error);
        // Try fallback to old API
        try {
          const voiceSettings = await invoke('get_voice_command_settings');
          if (voiceSettings) {
            voiceCommandsEnabled = voiceSettings.enabled;
            voiceCommandPrefix = voiceSettings.command_prefix || 'computer';
            voiceCommandRequirePrefix = voiceSettings.require_prefix;
          }
        } catch (fallbackError) {
          console.error('Failed to load voice command settings:', fallbackError);
        }
      }
      
      // Try to get command history
      try {
        commandHistory = await invoke('plugin:voice_commands:get_command_history');
      } catch (error) {
        console.error('Failed to load command history:', error);
        commandHistory = [];
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
      
      // Setup interval to check for voice commands with improved command handling
      commandCheckInterval = window.setInterval(async () => {
        if (isRecording && voiceCommandsEnabled) {
          try {
            const command = await invoke('plugin:voice_commands:get_last_command');
            if (command && (!lastCommand || lastCommand.trigger_text !== command.trigger_text)) {
              // Get the full command history
              commandHistory = await invoke('plugin:voice_commands:get_command_history');
              
              // Execute command action with visual feedback
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
  
  // Execute a voice command with improved feedback
  async function executeVoiceCommand(command: any) {
    console.log('Executing voice command:', command);
    
    // Display the command feedback
    showCommandFeedback(command);
    
    // Handle different command types
    switch (command.command_type) {
      case 'delete':
        // Delete the last few words from the transcription
        const words = transcriptionText.trim().split(/\s+/);
        if (words.length > 0) {
          // Remove the last 1-3 words based on command parameters
          const wordsToRemove = command.parameters ? 
            parseInt(command.parameters) : 
            Math.min(3, Math.max(1, Math.floor(words.length * 0.1)));
          
          transcriptionText = words.slice(0, -wordsToRemove).join(' ');
        }
        break;
        
      case 'undo':
        // Undo functionality could be implemented with a history stack
        showCommandFeedback({ command_type: 'undo', message: 'Undo action' });
        break;
        
      case 'redo':
        showCommandFeedback({ command_type: 'redo', message: 'Redo action' });
        break;
        
      case 'capitalize':
        // Capitalize the last word
        const lastSpaceIndex = transcriptionText.lastIndexOf(' ');
        if (lastSpaceIndex >= 0) {
          const lastWord = transcriptionText.substring(lastSpaceIndex + 1);
          const capitalizedWord = lastWord.charAt(0).toUpperCase() + lastWord.slice(1);
          transcriptionText = transcriptionText.substring(0, lastSpaceIndex + 1) + capitalizedWord;
        }
        break;
        
      case 'lowercase':
        // Lowercase the last word
        const lastSpaceIdx = transcriptionText.lastIndexOf(' ');
        if (lastSpaceIdx >= 0) {
          const lastWord = transcriptionText.substring(lastSpaceIdx + 1);
          const lowercaseWord = lastWord.toLowerCase();
          transcriptionText = transcriptionText.substring(0, lastSpaceIdx + 1) + lowercaseWord;
        }
        break;
        
      case 'newline':
        transcriptionText += '\n';
        break;
        
      case 'newparagraph':
        transcriptionText += '\n\n';
        break;
        
      case 'period':
        // Add period and ensure proper spacing
        transcriptionText = transcriptionText.trimRight() + '. ';
        break;
        
      case 'comma':
        // Add comma and ensure proper spacing
        transcriptionText = transcriptionText.trimRight() + ', ';
        break;
        
      case 'questionmark':
        // Add question mark and ensure proper spacing
        transcriptionText = transcriptionText.trimRight() + '? ';
        break;
        
      case 'exclamationmark':
        // Add exclamation mark and ensure proper spacing
        transcriptionText = transcriptionText.trimRight() + '! ';
        break;
        
      case 'pause':
        await stopRecording();
        break;
        
      case 'resume':
        await startRecording();
        break;
        
      case 'stop':
        await stopRecording();
        break;
        
      default:
        showCommandFeedback({ 
          command_type: 'unknown', 
          message: `Unknown command: ${command.command_type}` 
        });
    }
  }
  
  // Show improved command feedback with animation
  function showCommandFeedback(command: any) {
    // Create feedback message
    let message = '';
    switch(command.command_type) {
      case 'delete': message = 'Deleted text'; break;
      case 'undo': message = 'Undo action'; break;
      case 'redo': message = 'Redo action'; break;
      case 'capitalize': message = 'Capitalized text'; break;
      case 'lowercase': message = 'Lowercased text'; break;
      case 'newline': message = 'New line added'; break;
      case 'newparagraph': message = 'New paragraph added'; break;
      case 'period': message = 'Period added'; break;
      case 'comma': message = 'Comma added'; break;
      case 'questionmark': message = 'Question mark added'; break;
      case 'exclamationmark': message = 'Exclamation mark added'; break;
      case 'pause': message = 'Recording paused'; break;
      case 'resume': message = 'Recording resumed'; break;
      case 'stop': message = 'Recording stopped'; break;
      default: message = command.message || `Command: ${command.command_type}`;
    }
    
    // Set command feedback
    commandFeedback = {
      type: command.command_type,
      message: message,
      show: true
    };
    
    // Clear any existing timeout
    if (commandFeedbackTimeout) {
      clearTimeout(commandFeedbackTimeout);
    }
    
    // Set timeout to hide feedback after 3 seconds
    commandFeedbackTimeout = setTimeout(() => {
      commandFeedback = null;
    }, 3000);
  }
  
  // Toggle voice commands
  async function toggleVoiceCommands(enable: boolean = !voiceCommandsEnabled) {
    try {
      // Update voice command state
      voiceCommandsEnabled = enable;
      
      // Try new API first
      try {
        // Get current config
        let config;
        try {
          config = await invoke('plugin:voice_commands:get_voice_command_config');
        } catch {
          config = {
            prefix: voiceCommandPrefix || 'computer',
            require_prefix: voiceCommandRequirePrefix,
            confidence: 0.7
          };
        }
        
        // Update config
        const updatedConfig = {
          ...config,
          enabled: voiceCommandsEnabled
        };
        
        // Save config
        await invoke('plugin:voice_commands:set_voice_command_config', { config: updatedConfig });
      } catch (configError) {
        console.error('Failed to set voice command config, falling back to old API:', configError);
        
        // Fall back to old API
        await invoke('toggle_voice_commands', { enabled: enable });
        
        // Save settings
        await invoke('save_voice_command_settings', {
          enabled: voiceCommandsEnabled,
          command_prefix: voiceCommandPrefix || null,
          require_prefix: voiceCommandRequirePrefix,
          sensitivity: 0.8 // Default value
        });
      }
      
      // If recording, apply changes immediately
      if (isRecording) {
        if (voiceCommandsEnabled) {
          await invoke('plugin:voice_commands:start_voice_commands');
        } else {
          await invoke('plugin:voice_commands:stop_voice_commands');
        }
      }
      
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
      if (isRecording) {
        await invoke('plugin:transcribe:clear_transcription');
      }
      transcriptionText = '';
    } catch (error) {
      console.error('Failed to clear transcription:', error);
    }
  }
  
  // Toggle command history panel
  function toggleCommandHistory() {
    commandHistoryExpanded = !commandHistoryExpanded;
    if (commandHistoryExpanded) {
      refreshCommandHistory();
    }
  }
  
  // Get language display name
  function getLanguageDisplayName(code: string): string {
    const lang = languages.find(l => l[0] === code);
    return lang ? lang[1] : code;
  }
  
  // Format timestamp for displaying in the command history
  function formatTimestamp(timestamp: number): string {
    if (!timestamp) return 'Unknown';
    
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }
  
  // Refresh command history
  async function refreshCommandHistory() {
    try {
      commandHistory = await invoke('plugin:voice_commands:get_command_history');
    } catch (error) {
      console.error('Failed to refresh command history:', error);
      commandHistory = [];
    }
  }
  
  // Clear command history
  async function clearCommandHistory() {
    try {
      await invoke('plugin:voice_commands:clear_command_history');
      commandHistory = [];
    } catch (error) {
      console.error('Failed to clear command history:', error);
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
      
      <div class="voice-command-indicator">
        <div class="indicator-badge">
          Voice Commands Active
        </div>
        {#if lastCommand}
          <div class="last-command">
            Last command: <span class="command-type-{lastCommand.command_type}">{lastCommand.command_type}</span>
          </div>
        {/if}
      </div>
    {/if}
  </div>
  
  <div class="transcription-container">
    <!-- Transcription area -->
    <div class="transcription-area">
      <div class="transcription-header">
        <div class="language-indicator">
          <span class="indicator-label">Language:</span>
          <span class="indicator-value">{getLanguageDisplayName(selectedLanguage)}</span>
          {#if translateToEnglish && selectedLanguage !== 'en'}
            <span class="translation-indicator">(Translating to English)</span>
          {/if}
        </div>
        <div class="actions">
          <button class="action-button" on:click={() => transcriptionText = ''} disabled={isRecording}>
            Clear
          </button>
          <button class="action-button" on:click={() => {
            commandHistoryExpanded = !commandHistoryExpanded;
            if (commandHistoryExpanded) {
              refreshCommandHistory();
            }
          }}>
            {commandHistoryExpanded ? 'Hide' : 'Show'} History
          </button>
        </div>
      </div>
      <textarea
        bind:value={transcriptionText}
        placeholder="Transcription will appear here..."
        readonly={isRecording}
      ></textarea>
    </div>
    
    <!-- Command History Panel -->
    {#if commandHistoryExpanded}
      <div class="command-history-panel">
        <div class="command-history-header">
          <h3>Voice Command History</h3>
          <div class="command-history-actions">
            <button class="action-button" on:click={refreshCommandHistory}>
              Refresh
            </button>
            <button class="action-button" on:click={clearCommandHistory}>
              Clear History
            </button>
          </div>
        </div>
        <div class="command-history-list">
          {#if commandHistory.length === 0}
            <div class="command-history-empty">No commands detected yet</div>
          {:else}
            {#each commandHistory as command}
              <div class="command-history-item">
                <div class="command-type {command.command_type}">
                  {command.command_type}
                </div>
                <div class="command-trigger">
                  "{command.trigger_text}"
                </div>
                <div class="command-time">
                  {formatTimestamp(command.timestamp)}
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}
  </div>
  
  {#if commandFeedback && commandFeedback.show}
    <div class="command-feedback command-feedback-{commandFeedback.type}">
      <div class="command-feedback-icon">
        {#if commandFeedback.type === 'delete'}
          🗑️
        {:else if commandFeedback.type === 'undo'}
          ↩️
        {:else if commandFeedback.type === 'redo'}
          ↪️
        {:else if commandFeedback.type === 'capitalize' || commandFeedback.type === 'lowercase'}
          🔤
        {:else if commandFeedback.type === 'newline' || commandFeedback.type === 'newparagraph'}
          ↵
        {:else if ['period', 'comma', 'questionmark', 'exclamationmark'].includes(commandFeedback.type)}
          ✏️
        {:else if ['pause', 'resume', 'stop'].includes(commandFeedback.type)}
          ⏯️
        {:else}
          🎤
        {/if}
      </div>
      <div class="command-feedback-message">{commandFeedback.message}</div>
    </div>
  {/if}
  
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
    align-items: center;
    cursor: pointer;
    padding: 5px 0;
    margin-top: 15px;
  }
  
  .command-history-header h3 {
    flex-grow: 1;
    margin: 0;
    font-size: 1rem;
  }
  
  .clear-history-button {
    background-color: transparent;
    border: none;
    color: #6c757d;
    font-size: 0.8rem;
    cursor: pointer;
    margin-right: 10px;
  }
  
  .clear-history-button:hover {
    color: #dc3545;
  }
  
  .expand-icon {
    color: #6c757d;
    font-size: 0.8rem;
  }
  
  .command-history {
    max-height: 200px;
    overflow-y: auto;
    background-color: #f8f9fa;
    border-radius: 6px;
    margin-top: 10px;
    padding: 10px;
    border: 1px solid #dee2e6;
  }
  
  .command-item {
    display: flex;
    align-items: center;
    padding: 5px 0;
    border-bottom: 1px solid #eee;
  }
  
  .command-type {
    padding: 3px 6px;
    border-radius: 4px;
    margin-right: 8px;
    font-size: 0.8rem;
    font-weight: bold;
    background-color: #e9ecef;
  }
  
  .command-type-delete { background-color: #f8d7da; color: #721c24; }
  .command-type-undo, .command-type-redo { background-color: #d1ecf1; color: #0c5460; }
  .command-type-capitalize, .command-type-lowercase { background-color: #fff3cd; color: #856404; }
  .command-type-newline, .command-type-newparagraph { background-color: #d4edda; color: #155724; }
  .command-type-period, .command-type-comma, 
  .command-type-questionmark, .command-type-exclamationmark { background-color: #e2e3e5; color: #383d41; }
  .command-type-pause, .command-type-resume, .command-type-stop { background-color: #cce5ff; color: #004085; }
  
  .command-trigger {
    flex-grow: 1;
    font-style: italic;
    color: #6c757d;
    font-size: 0.9rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .command-time {
    font-size: 0.75rem;
    color: #adb5bd;
    margin-left: 8px;
  }
  
  .no-commands {
    text-align: center;
    color: #95a5a6;
    font-style: italic;
    font-size: 0.9rem;
  }
  
  .transcription-area {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    padding: 15px;
    background-color: white;
    border-radius: 5px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
    margin-bottom: 10px;
  }
  
  .transcription-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
    padding-bottom: 10px;
    border-bottom: 1px solid #eee;
  }
  
  textarea {
    flex-grow: 1;
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    resize: none;
    font-size: 16px;
    line-height: 1.5;
    font-family: inherit;
  }
  
  textarea:focus {
    outline: none;
    border-color: #3498db;
  }
  
  textarea:read-only {
    background-color: #f9f9f9;
    cursor: default;
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
  
  .command-feedback {
    position: fixed;
    top: 20px;
    right: 20px;
    background-color: #343a40;
    color: white;
    padding: 10px 15px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    box-shadow: 0 4px 12px rgba(0,0,0,0.1);
    z-index: 1000;
    animation: fadeInOut 3s ease-in-out;
  }
  
  .command-feedback-icon {
    font-size: 1.5rem;
    margin-right: 12px;
  }
  
  .command-feedback-message {
    font-size: 0.9rem;
  }
  
  @keyframes fadeInOut {
    0% { opacity: 0; transform: translateY(-20px); }
    10% { opacity: 1; transform: translateY(0); }
    80% { opacity: 1; transform: translateY(0); }
    100% { opacity: 0; transform: translateY(-20px); }
  }
  
  /* Command History Panel */
  .transcription-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }
  
  .command-history-panel {
    background-color: #f5f5f5;
    border-top: 1px solid #ddd;
    padding: 10px;
    height: 200px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
  
  .command-history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }
  
  .command-history-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  
  .command-history-actions {
    display: flex;
    gap: 8px;
  }
  
  .command-history-list {
    flex-grow: 1;
    overflow-y: auto;
  }
  
  .command-history-empty {
    color: #888;
    font-style: italic;
    text-align: center;
    padding: 20px;
  }
  
  .command-history-item {
    display: flex;
    align-items: center;
    padding: 8px;
    border-bottom: 1px solid #eee;
    font-size: 13px;
  }
  
  .command-history-item:last-child {
    border-bottom: none;
  }
  
  .command-type {
    background-color: #3498db;
    color: white;
    padding: 2px 6px;
    border-radius: 4px;
    margin-right: 10px;
    font-size: 12px;
    min-width: 80px;
    text-align: center;
  }
  
  .command-type.delete { background-color: #e74c3c; }
  .command-type.pause { background-color: #f39c12; }
  .command-type.resume { background-color: #2ecc71; }
  .command-type.stop { background-color: #e74c3c; }
  .command-type.newline { background-color: #9b59b6; }
  .command-type.newparagraph { background-color: #9b59b6; }
  
  .command-trigger {
    flex-grow: 1;
    font-style: italic;
    color: #555;
  }
  
  .command-time {
    color: #888;
    font-size: 12px;
  }
  
  .language-indicator {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .indicator-label {
    font-weight: 600;
    font-size: 12px;
  }
  
  .indicator-value {
    font-weight: normal;
    background-color: #f0f0f0;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
  }
  
  .translation-indicator {
    color: #3498db;
    font-size: 12px;
  }
  
  .actions {
    display: flex;
    gap: 8px;
  }
  
  .action-button {
    background-color: #f0f0f0;
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 12px;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .action-button:hover:not(:disabled) {
    background-color: #e0e0e0;
  }
  
  .action-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .voice-command-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background-color: #f8f9fa;
    border-radius: 8px;
  }
  
  .indicator-badge {
    font-size: 1rem;
    font-weight: bold;
    color: #2c3e50;
  }
  
  .last-command {
    font-size: 0.8rem;
    color: #7f8c8d;
    margin-top: 0.5rem;
  }
</style> 
