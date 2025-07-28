<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let text: string = '';
  export let maxLength: number = 100;
  
  let summary = '';
  let isLoading = false;
  let error = '';
  let customLength = maxLength;
  let showAdvanced = false;
  
  // Preset lengths
  const presets = [
    { label: 'Tweet', value: 50 },
    { label: 'Short', value: 100 },
    { label: 'Medium', value: 200 },
    { label: 'Long', value: 500 }
  ];
  
  async function summarize() {
    if (!text.trim()) {
      error = 'Please provide text to summarize';
      return;
    }
    
    isLoading = true;
    error = '';
    summary = '';
    
    try {
      // Check if cloud AI is initialized
      const result = await invoke('summarize_text', {
        text: text.trim(),
        maxLength: customLength
      });
      
      summary = result;
      dispatch('summarized', { original: text, summary, maxLength: customLength });
    } catch (e) {
      error = `Summarization failed: ${e}`;
      console.error('Summarization error:', e);
      
      // Try to provide helpful error message
      if (e.includes('not initialized')) {
        error = 'Please initialize a cloud AI provider first (OpenRouter, OpenAI, or Requesty)';
      }
    } finally {
      isLoading = false;
    }
  }
  
  function setPreset(value: number) {
    customLength = value;
  }
  
  function copyToClipboard() {
    navigator.clipboard.writeText(summary);
    dispatch('copied', { text: summary });
  }
  
  function clear() {
    text = '';
    summary = '';
    error = '';
  }
  
  // Character and word count
  $: charCount = text.length;
  $: wordCount = text.trim() ? text.trim().split(/\s+/).length : 0;
  $: summaryWordCount = summary.trim() ? summary.trim().split(/\s+/).length : 0;
</script>

<div class="text-summarizer">
  <div class="header">
    <h3>Text Summarizer</h3>
    <button class="toggle-advanced" on:click={() => showAdvanced = !showAdvanced}>
      {showAdvanced ? '⚙️ Simple' : '⚙️ Advanced'}
    </button>
  </div>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  <div class="input-section">
    <label for="input-text">Text to Summarize</label>
    <textarea
      id="input-text"
      bind:value={text}
      placeholder="Paste or type the text you want to summarize..."
      rows="8"
      disabled={isLoading}
    />
    <div class="text-stats">
      <span>{charCount} characters</span>
      <span>{wordCount} words</span>
    </div>
  </div>
  
  <div class="controls">
    <div class="length-control">
      <label for="max-length">Summary Length</label>
      {#if showAdvanced}
        <div class="advanced-controls">
          <input
            id="max-length"
            type="number"
            bind:value={customLength}
            min="10"
            max="1000"
            step="10"
            disabled={isLoading}
          />
          <span class="unit">words</span>
        </div>
      {:else}
        <div class="preset-buttons">
          {#each presets as preset}
            <button
              class="preset"
              class:active={customLength === preset.value}
              on:click={() => setPreset(preset.value)}
              disabled={isLoading}
            >
              {preset.label}
            </button>
          {/each}
        </div>
      {/if}
    </div>
    
    <div class="action-buttons">
      <button 
        class="clear-button" 
        on:click={clear}
        disabled={isLoading || (!text && !summary)}
      >
        Clear
      </button>
      <button 
        class="summarize-button" 
        on:click={summarize}
        disabled={isLoading || !text.trim()}
      >
        {isLoading ? 'Summarizing...' : 'Summarize'}
      </button>
    </div>
  </div>
  
  {#if summary}
    <div class="output-section">
      <div class="output-header">
        <label>Summary</label>
        <button class="copy-button" on:click={copyToClipboard} title="Copy to clipboard">
          📋 Copy
        </button>
      </div>
      <div class="summary-text">
        {summary}
      </div>
      <div class="summary-stats">
        <span>{summaryWordCount} words</span>
        <span class="reduction">
          {Math.round((1 - summaryWordCount / wordCount) * 100)}% reduction
        </span>
      </div>
    </div>
  {/if}
  
  {#if isLoading}
    <div class="loading-overlay">
      <div class="spinner"></div>
      <p>Generating summary...</p>
    </div>
  {/if}
</div>

<style>
  .text-summarizer {
    padding: 20px;
    max-width: 800px;
    margin: 0 auto;
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
  
  .toggle-advanced {
    padding: 6px 12px;
    background: #6c757d;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9em;
  }
  
  .toggle-advanced:hover {
    background: #5a6268;
  }
  
  .error {
    background: #f8d7da;
    color: #721c24;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .input-section {
    margin-bottom: 20px;
  }
  
  .input-section label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
  }
  
  .input-section textarea {
    width: 100%;
    padding: 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 1em;
    resize: vertical;
  }
  
  .text-stats {
    display: flex;
    gap: 20px;
    margin-top: 5px;
    font-size: 0.85em;
    color: #666;
  }
  
  .controls {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: 20px;
    gap: 20px;
  }
  
  .length-control {
    flex: 1;
  }
  
  .length-control label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
  }
  
  .advanced-controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .advanced-controls input {
    width: 100px;
    padding: 8px;
    border: 1px solid #ced4da;
    border-radius: 4px;
  }
  
  .unit {
    color: #666;
  }
  
  .preset-buttons {
    display: flex;
    gap: 10px;
  }
  
  .preset {
    padding: 8px 16px;
    background: #e9ecef;
    border: 1px solid #ced4da;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .preset:hover {
    background: #dee2e6;
  }
  
  .preset.active {
    background: #007bff;
    color: white;
    border-color: #007bff;
  }
  
  .preset:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .action-buttons {
    display: flex;
    gap: 10px;
  }
  
  .clear-button,
  .summarize-button {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1em;
  }
  
  .clear-button {
    background: #6c757d;
    color: white;
  }
  
  .clear-button:hover:not(:disabled) {
    background: #5a6268;
  }
  
  .summarize-button {
    background: #007bff;
    color: white;
  }
  
  .summarize-button:hover:not(:disabled) {
    background: #0056b3;
  }
  
  .clear-button:disabled,
  .summarize-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .output-section {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-top: 30px;
  }
  
  .output-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  
  .output-header label {
    font-weight: 500;
  }
  
  .copy-button {
    padding: 6px 12px;
    background: #17a2b8;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9em;
  }
  
  .copy-button:hover {
    background: #138496;
  }
  
  .summary-text {
    background: white;
    padding: 15px;
    border-radius: 4px;
    line-height: 1.6;
    margin-bottom: 10px;
  }
  
  .summary-stats {
    display: flex;
    justify-content: space-between;
    font-size: 0.85em;
    color: #666;
  }
  
  .reduction {
    color: #28a745;
    font-weight: 500;
  }
  
  .loading-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  
  .spinner {
    width: 50px;
    height: 50px;
    border: 5px solid #f3f3f3;
    border-top: 5px solid #007bff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .loading-overlay p {
    margin-top: 20px;
    color: white;
    font-size: 1.1em;
  }
  
  @media (max-width: 600px) {
    .controls {
      flex-direction: column;
      align-items: stretch;
    }
    
    .preset-buttons {
      flex-wrap: wrap;
    }
  }
  
  @media (prefers-color-scheme: dark) {
    .text-summarizer {
      color: #e0e0e0;
    }
    
    .input-section textarea,
    .advanced-controls input {
      background: #1a1a1a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .text-stats,
    .unit,
    .summary-stats {
      color: #999;
    }
    
    .preset {
      background: #2a2a2a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .preset:hover {
      background: #3a3a3a;
    }
    
    .output-section {
      background: #2a2a2a;
    }
    
    .summary-text {
      background: #1a1a1a;
      color: #e0e0e0;
    }
  }
</style>