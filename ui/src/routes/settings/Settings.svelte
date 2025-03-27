<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount, onDestroy } from 'svelte';
  import { writable } from 'svelte/store';
  import Tabs from './Tabs.svelte';
  
  // Create local settings store
  const settings = writable({});
  
  // Add missing variables
  let activeTab = 'general';
  let saved = false;
  
  // Voice command settings
  let voiceCommandsEnabled = false;
  let voiceCommandPrefix = "computer";
  let voiceCommandRequirePrefix = true;
  let voiceCommandConfidence = 0.7;
  let availableCommands = [
    { name: 'delete', description: 'Delete the last few words', examples: ['delete that', 'delete last 3 words'] },
    { name: 'undo', description: 'Undo last action', examples: ['undo that', 'undo last change'] },
    { name: 'redo', description: 'Redo last undone action', examples: ['redo that', 'redo last change'] },
    { name: 'capitalize', description: 'Capitalize the last word', examples: ['capitalize that', 'capitalize last word'] },
    { name: 'lowercase', description: 'Lowercase the last word', examples: ['lowercase that', 'lowercase last word'] },
    { name: 'newline', description: 'Add a new line', examples: ['new line', 'line break'] },
    { name: 'newparagraph', description: 'Add a new paragraph', examples: ['new paragraph', 'paragraph break'] },
    { name: 'period', description: 'Add a period', examples: ['period', 'full stop'] },
    { name: 'comma', description: 'Add a comma', examples: ['comma'] },
    { name: 'questionmark', description: 'Add a question mark', examples: ['question mark'] },
    { name: 'exclamationmark', description: 'Add an exclamation mark', examples: ['exclamation mark', 'exclamation point'] },
    { name: 'pause', description: 'Pause recording', examples: ['pause recording', 'pause'] },
    { name: 'resume', description: 'Resume recording', examples: ['resume recording', 'resume'] },
    { name: 'stop', description: 'Stop recording', examples: ['stop recording', 'stop'] }
  ];
  
  // Load settings
  onMount(async () => {
    try {
      // Load all settings from the backend
      const allSettings = await invoke('get_settings');
      
      // Update local store
      settings.set(allSettings);
      
      // Voice command settings
      if (allSettings.voice_commands) {
        voiceCommandsEnabled = allSettings.voice_commands.enabled || false;
        voiceCommandPrefix = allSettings.voice_commands.prefix || "computer";
        voiceCommandRequirePrefix = allSettings.voice_commands.require_prefix !== false; // default to true
        voiceCommandConfidence = allSettings.voice_commands.confidence || 0.7;
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  });
  
  // Save settings
  async function saveSettings() {
    try {
      // Save voice command settings
      const voiceCommandSettings = {
        enabled: voiceCommandsEnabled,
        prefix: voiceCommandPrefix,
        require_prefix: voiceCommandRequirePrefix,
        confidence: voiceCommandConfidence
      };
      
      await invoke('save_all_settings', {
        voiceCommands: voiceCommandSettings
      });
      
      saved = true;
      setTimeout(() => {
        saved = false;
      }, 3000);
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  }
</script>

<div class="settings-container">
  <h1>Settings</h1>
  
  <Tabs>
    <div slot="tabs">
      <button class:active={activeTab === 'general'} on:click={() => activeTab = 'general'}>General</button>
      <button class:active={activeTab === 'transcription'} on:click={() => activeTab = 'transcription'}>Transcription</button>
      <button class:active={activeTab === 'voice-commands'} on:click={() => activeTab = 'voice-commands'}>Voice Commands</button>
      <button class:active={activeTab === 'advanced'} on:click={() => activeTab = 'advanced'}>Advanced</button>
    </div>
    
    <div slot="content">
      <!-- General Tab -->
      {#if activeTab === 'general'}
        <!-- ... existing general settings ... -->
      {/if}
      
      <!-- Transcription Tab -->
      {#if activeTab === 'transcription'}
        <!-- ... existing transcription settings ... -->
      {/if}
      
      <!-- Voice Commands Tab -->
      {#if activeTab === 'voice-commands'}
        <div class="settings-section">
          <h2>Voice Commands</h2>
          
          <div class="setting-row">
            <label class="toggle-setting">
              <span>Enable Voice Commands</span>
              <input type="checkbox" bind:checked={voiceCommandsEnabled}>
              <span class="toggle"></span>
            </label>
          </div>
          
          <div class="setting-row" class:disabled={!voiceCommandsEnabled}>
            <label>
              <span>Command Prefix</span>
              <input type="text" bind:value={voiceCommandPrefix} disabled={!voiceCommandsEnabled}>
            </label>
            <div class="setting-description">
              The word to say before a command (e.g., "computer, delete that")
            </div>
          </div>
          
          <div class="setting-row" class:disabled={!voiceCommandsEnabled}>
            <label class="toggle-setting">
              <span>Require Prefix</span>
              <input type="checkbox" bind:checked={voiceCommandRequirePrefix} disabled={!voiceCommandsEnabled}>
              <span class="toggle"></span>
            </label>
            <div class="setting-description">
              If enabled, commands must start with the prefix. If disabled, any recognized command will be executed.
            </div>
          </div>
          
          <div class="setting-row" class:disabled={!voiceCommandsEnabled}>
            <label>
              <span>Command Confidence Threshold</span>
              <input 
                type="range" 
                min="0.1" 
                max="1.0" 
                step="0.05" 
                bind:value={voiceCommandConfidence} 
                disabled={!voiceCommandsEnabled}
              >
              <span class="confidence-value">{(voiceCommandConfidence * 100).toFixed(0)}%</span>
            </label>
            <div class="setting-description">
              Higher values make command detection more accurate but might miss some commands.
            </div>
          </div>
          
          <div class="command-list" class:disabled={!voiceCommandsEnabled}>
            <h3>Available Commands</h3>
            <div class="command-table">
              <div class="command-header">
                <div class="command-name">Command</div>
                <div class="command-description">Description</div>
                <div class="command-examples">Example Phrases</div>
              </div>
              
              {#each availableCommands as command}
                <div class="command-row">
                  <div class="command-name">{command.name}</div>
                  <div class="command-description">{command.description}</div>
                  <div class="command-examples">
                    {#each command.examples as example, i}
                      "{example}"{i < command.examples.length - 1 ? ', ' : ''}
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {/if}
      
      <!-- Advanced Tab -->
      {#if activeTab === 'advanced'}
        <!-- ... existing advanced settings ... -->
      {/if}
    </div>
  </Tabs>
  
  <div class="settings-actions">
    <button class="primary-button" on:click={saveSettings}>Save Settings</button>
    {#if saved}
      <div class="save-notification">Settings saved!</div>
    {/if}
  </div>
</div>

<style>
  /* ... existing styles ... */
  
  .command-list {
    margin-top: 20px;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    overflow: hidden;
  }
  
  .command-list h3 {
    margin: 0;
    padding: 15px;
    background-color: #f5f5f5;
    border-bottom: 1px solid #e0e0e0;
    font-size: 1rem;
  }
  
  .command-table {
    padding: 10px;
    max-height: 300px;
    overflow-y: auto;
  }
  
  .command-header {
    display: grid;
    grid-template-columns: 1fr 2fr 2fr;
    gap: 10px;
    padding: 10px;
    font-weight: bold;
    border-bottom: 1px solid #e0e0e0;
  }
  
  .command-row {
    display: grid;
    grid-template-columns: 1fr 2fr 2fr;
    gap: 10px;
    padding: 10px;
    border-bottom: 1px solid #f0f0f0;
  }
  
  .command-row:last-child {
    border-bottom: none;
  }
  
  .command-name {
    font-weight: 500;
  }
  
  .command-description {
    color: #555;
  }
  
  .command-examples {
    color: #777;
    font-style: italic;
    font-size: 0.9em;
  }
  
  .confidence-value {
    display: inline-block;
    width: 40px;
    text-align: right;
    margin-left: 10px;
  }
  
  /* Disabled state styling */
  .disabled {
    opacity: 0.6;
    pointer-events: none;
  }
  
  input[type="range"] {
    width: 200px;
    vertical-align: middle;
  }
</style> 
