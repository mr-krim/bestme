<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  
  // Types
  interface AICommand {
    id: string;
    trigger: string;
    action: string;
    description: string;
    examples: string[];
    enabled: boolean;
  }
  
  interface CommandHistory {
    timestamp: string;
    command: string;
    interpreted: string;
    executed: boolean;
    confidence: number;
  }
  
  // State
  let aiEnabled = false;
  let naturalLanguage = true;
  let contextAware = true;
  let confidenceThreshold = 0.7;
  let showAdvanced = false;
  let isProcessing = false;
  let lastCommand: CommandHistory | null = null;
  
  // AI Commands
  let aiCommands: AICommand[] = [
    {
      id: 'smart_delete',
      trigger: 'delete',
      action: 'Smart deletion with context',
      description: 'Intelligently deletes text based on natural language',
      examples: [
        'delete the last sentence',
        'remove everything after the comma',
        'erase the previous paragraph'
      ],
      enabled: true
    },
    {
      id: 'smart_format',
      trigger: 'format',
      action: 'Intelligent text formatting',
      description: 'Formats text based on context and intent',
      examples: [
        'make this a bullet list',
        'format as a quote',
        'convert to title case'
      ],
      enabled: true
    },
    {
      id: 'smart_edit',
      trigger: 'change',
      action: 'AI-powered text editing',
      description: 'Makes complex edits using AI understanding',
      examples: [
        'change all instances of John to Jane',
        'make this more formal',
        'fix the grammar'
      ],
      enabled: true
    },
    {
      id: 'smart_insert',
      trigger: 'add',
      action: 'Context-aware insertion',
      description: 'Inserts text intelligently based on context',
      examples: [
        'add a comma after each item',
        'insert a greeting at the beginning',
        'add quotation marks around the title'
      ],
      enabled: true
    },
    {
      id: 'smart_navigate',
      trigger: 'go to',
      action: 'Intelligent navigation',
      description: 'Navigate through text using natural language',
      examples: [
        'go to the next section',
        'jump to the conclusion',
        'find the email address'
      ],
      enabled: true
    },
    {
      id: 'smart_action',
      trigger: 'action',
      action: 'Complex AI actions',
      description: 'Performs complex actions using AI',
      examples: [
        'summarize this paragraph',
        'translate to Spanish',
        'explain this in simple terms'
      ],
      enabled: true
    }
  ];
  
  // Command history
  let commandHistory: CommandHistory[] = [];
  let commandListener: any = null;
  
  onMount(async () => {
    // Load settings
    await loadSettings();
    
    // Listen for voice commands
    commandListener = await listen('voice-command:detected', (event: any) => {
      if (aiEnabled) {
        processAICommand(event.payload);
      }
    });
  });
  
  onDestroy(() => {
    if (commandListener) {
      commandListener();
    }
  });
  
  // Load AI voice command settings
  async function loadSettings() {
    try {
      const settings = await invoke('get_ai_voice_settings');
      if (settings) {
        aiEnabled = settings.enabled || false;
        naturalLanguage = settings.natural_language || true;
        contextAware = settings.context_aware || true;
        confidenceThreshold = settings.confidence_threshold || 0.7;
      }
    } catch (error) {
      console.error('Failed to load AI voice settings:', error);
    }
  }
  
  // Save settings
  async function saveSettings() {
    try {
      await invoke('save_ai_voice_settings', {
        settings: {
          enabled: aiEnabled,
          natural_language: naturalLanguage,
          context_aware: contextAware,
          confidence_threshold: confidenceThreshold,
          enabled_commands: aiCommands.filter(cmd => cmd.enabled).map(cmd => cmd.id)
        }
      });
    } catch (error) {
      console.error('Failed to save AI voice settings:', error);
    }
  }
  
  // Process AI command
  async function processAICommand(command: any) {
    isProcessing = true;
    
    try {
      // Use AI to interpret the command
      const result = await invoke('process_ai_voice_command', {
        text: command.text,
        context: command.context
      });
      
      // Add to history
      const historyEntry: CommandHistory = {
        timestamp: new Date().toISOString(),
        command: command.text,
        interpreted: result.interpretation,
        executed: result.success,
        confidence: result.confidence
      };
      
      commandHistory = [historyEntry, ...commandHistory].slice(0, 50);
      lastCommand = historyEntry;
      
      // Show feedback
      if (result.success) {
        showSuccessFeedback(result.interpretation);
      } else {
        showErrorFeedback(result.error || 'Command not understood');
      }
    } catch (error) {
      console.error('Failed to process AI command:', error);
      showErrorFeedback('AI processing failed');
    } finally {
      isProcessing = false;
    }
  }
  
  // Toggle AI
  async function toggleAI() {
    aiEnabled = !aiEnabled;
    await saveSettings();
  }
  
  // Toggle command
  function toggleCommand(commandId: string) {
    const command = aiCommands.find(cmd => cmd.id === commandId);
    if (command) {
      command.enabled = !command.enabled;
      saveSettings();
    }
  }
  
  // Clear history
  function clearHistory() {
    commandHistory = [];
    lastCommand = null;
  }
  
  // Show feedback
  function showSuccessFeedback(message: string) {
    // In a real app, this would show a toast or notification
    console.log('Success:', message);
  }
  
  function showErrorFeedback(message: string) {
    // In a real app, this would show a toast or notification
    console.error('Error:', message);
  }
  
  // Test command
  async function testCommand(example: string) {
    await processAICommand({ text: example, context: '' });
  }
</script>

<div class="ai-voice-commands">
  <div class="header">
    <h3>AI Voice Commands</h3>
    <label class="toggle">
      <input type="checkbox" bind:checked={aiEnabled} on:change={toggleAI} />
      <span>Enable AI Commands</span>
    </label>
  </div>
  
  {#if aiEnabled}
    <div class="settings">
      <div class="setting-group">
        <label class="checkbox">
          <input type="checkbox" bind:checked={naturalLanguage} on:change={saveSettings} />
          <span>Natural Language Understanding</span>
        </label>
        <p class="description">Understand commands in natural language, not just keywords</p>
      </div>
      
      <div class="setting-group">
        <label class="checkbox">
          <input type="checkbox" bind:checked={contextAware} on:change={saveSettings} />
          <span>Context Awareness</span>
        </label>
        <p class="description">Consider surrounding text when interpreting commands</p>
      </div>
      
      <div class="setting-group">
        <label for="confidence">Confidence Threshold</label>
        <div class="slider-group">
          <input
            id="confidence"
            type="range"
            min="0.5"
            max="0.95"
            step="0.05"
            bind:value={confidenceThreshold}
            on:change={saveSettings}
          />
          <span class="value">{(confidenceThreshold * 100).toFixed(0)}%</span>
        </div>
        <p class="description">Minimum confidence required to execute commands</p>
      </div>
    </div>
    
    <div class="commands-section">
      <h4>Available AI Commands</h4>
      <div class="commands-list">
        {#each aiCommands as command}
          <div class="command-card" class:disabled={!command.enabled}>
            <div class="command-header">
              <div class="command-info">
                <h5>{command.action}</h5>
                <p class="trigger">Trigger: "{command.trigger}"</p>
              </div>
              <label class="toggle">
                <input
                  type="checkbox"
                  checked={command.enabled}
                  on:change={() => toggleCommand(command.id)}
                />
              </label>
            </div>
            <p class="description">{command.description}</p>
            <div class="examples">
              <span class="examples-label">Examples:</span>
              <div class="example-list">
                {#each command.examples as example}
                  <button
                    class="example"
                    on:click={() => testCommand(example)}
                    disabled={!command.enabled || isProcessing}
                  >
                    "{example}"
                  </button>
                {/each}
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
    
    {#if commandHistory.length > 0}
      <div class="history-section">
        <div class="history-header">
          <h4>Command History</h4>
          <button class="clear-button" on:click={clearHistory}>Clear</button>
        </div>
        <div class="history-list">
          {#each commandHistory.slice(0, 10) as entry}
            <div class="history-entry" class:success={entry.executed}>
              <div class="entry-header">
                <span class="command">"{entry.command}"</span>
                <span class="confidence">{(entry.confidence * 100).toFixed(0)}%</span>
              </div>
              <div class="interpreted">→ {entry.interpreted}</div>
              <div class="timestamp">{new Date(entry.timestamp).toLocaleTimeString()}</div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
    
    {#if isProcessing}
      <div class="processing">
        <div class="spinner"></div>
        <span>Processing command...</span>
      </div>
    {/if}
  {:else}
    <div class="disabled-message">
      <p>Enable AI Voice Commands to use natural language for controlling your transcription.</p>
      <p>Say things like "delete the last sentence" or "make this a bullet list".</p>
    </div>
  {/if}
</div>

<style>
  .ai-voice-commands {
    padding: 20px;
    max-width: 800px;
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
  
  .toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }
  
  .toggle input[type="checkbox"] {
    width: 20px;
    height: 20px;
    cursor: pointer;
  }
  
  .settings {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
  }
  
  .setting-group {
    margin-bottom: 20px;
  }
  
  .setting-group:last-child {
    margin-bottom: 0;
  }
  
  .checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    margin-bottom: 5px;
  }
  
  .description {
    margin: 0;
    font-size: 0.85em;
    color: #666;
    margin-left: 28px;
  }
  
  .slider-group {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 8px;
  }
  
  .slider-group input[type="range"] {
    flex: 1;
  }
  
  .value {
    min-width: 45px;
    text-align: right;
    font-weight: 500;
  }
  
  .commands-section {
    margin-bottom: 30px;
  }
  
  .commands-section h4 {
    margin: 0 0 15px 0;
  }
  
  .commands-list {
    display: grid;
    gap: 15px;
  }
  
  .command-card {
    background: white;
    border: 1px solid #dee2e6;
    border-radius: 8px;
    padding: 15px;
    transition: opacity 0.3s;
  }
  
  .command-card.disabled {
    opacity: 0.6;
  }
  
  .command-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 10px;
  }
  
  .command-info h5 {
    margin: 0 0 5px 0;
  }
  
  .trigger {
    margin: 0;
    font-size: 0.9em;
    color: #666;
    font-style: italic;
  }
  
  .examples {
    margin-top: 10px;
    font-size: 0.9em;
  }
  
  .examples-label {
    color: #666;
    margin-right: 10px;
  }
  
  .example-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 5px;
  }
  
  .example {
    padding: 4px 12px;
    background: #e9ecef;
    border: 1px solid #ced4da;
    border-radius: 20px;
    font-size: 0.85em;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .example:hover:not(:disabled) {
    background: #007bff;
    color: white;
    border-color: #007bff;
  }
  
  .example:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
  
  .history-section {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
  }
  
  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 15px;
  }
  
  .history-header h4 {
    margin: 0;
  }
  
  .clear-button {
    padding: 6px 12px;
    background: #6c757d;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9em;
  }
  
  .clear-button:hover {
    background: #5a6268;
  }
  
  .history-list {
    display: grid;
    gap: 10px;
  }
  
  .history-entry {
    background: white;
    padding: 12px;
    border-radius: 4px;
    border-left: 3px solid #dc3545;
  }
  
  .history-entry.success {
    border-left-color: #28a745;
  }
  
  .entry-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 5px;
  }
  
  .command {
    font-weight: 500;
  }
  
  .confidence {
    font-size: 0.85em;
    color: #666;
  }
  
  .interpreted {
    font-size: 0.9em;
    color: #495057;
    margin-bottom: 3px;
  }
  
  .timestamp {
    font-size: 0.8em;
    color: #666;
  }
  
  .processing {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 20px;
    background: rgba(0, 123, 255, 0.1);
    border-radius: 8px;
    margin-top: 20px;
  }
  
  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid #f3f3f3;
    border-top: 2px solid #007bff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .disabled-message {
    text-align: center;
    padding: 40px;
    color: #666;
  }
  
  .disabled-message p {
    margin: 10px 0;
  }
  
  @media (prefers-color-scheme: dark) {
    .ai-voice-commands {
      color: #e0e0e0;
    }
    
    .settings,
    .history-section {
      background: #2a2a2a;
    }
    
    .command-card {
      background: #1a1a1a;
      border-color: #4a4a4a;
    }
    
    .description,
    .trigger,
    .examples-label,
    .confidence,
    .timestamp {
      color: #999;
    }
    
    .example {
      background: #3a3a3a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .example:hover:not(:disabled) {
      background: #4dabf7;
      border-color: #4dabf7;
    }
    
    .history-entry {
      background: #1a1a1a;
    }
    
    .interpreted {
      color: #b0b0b0;
    }
  }
</style>