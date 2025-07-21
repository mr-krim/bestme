<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Props received from App.svelte
  export let isRecording: boolean;
  export let aiModels: string[];
  export let selectedWhisperModel: string; // Renamed
  export let languages: [string, string][];
  export let selectedLanguage: string; // No default, bound from parent
  
  const dispatch = createEventDispatcher();

  // Dispatch events instead of local functions
  function toggleMic() {
    dispatch('toggleMic');
  }

  function toggleTheme() {
    dispatch('toggleTheme');
  }

  function navigateToSettings() {
    dispatch('navigateSettings');
  }
</script>

<div class="top-bar" role="toolbar">
  <div class="left-controls">
    <button class="mic-toggle {isRecording ? 'active' : ''}" on:click={toggleMic} aria-label={isRecording ? 'Stop Recording' : 'Start Recording'}>
      {isRecording ? '⏹️ Stop' : '🎤 Record'}
    </button>
  </div>
  
  <div class="center-controls">
    <!-- Placeholder for potential future controls -->
  </div>
  
  <div class="right-controls">
    <select bind:value={selectedWhisperModel} disabled={aiModels.length === 0} aria-label="Whisper Model">
      {#each aiModels as model}
        <option value={model}>{model}</option>
      {/each}
    </select>
    
    <select bind:value={selectedLanguage} disabled={languages.length === 0} aria-label="Language">
      {#each languages as [code, name] (code)}
        <option value={code}>{name}</option>
      {/each}
    </select>
    
    <button class="quick-settings" title="Quick Settings" on:click={navigateToSettings}>⚙️</button>
    <button class="theme-toggle" title="Toggle Theme" on:click={toggleTheme}>🌓</button>
  </div>
</div>

<style>
  .top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px; /* Adjust height as needed */
    padding: 0 16px;
    background-color: var(--panel-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .left-controls, .center-controls, .right-controls {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  
  .center-controls {
    flex-grow: 1; /* Allows center to take up space */
    justify-content: center;
  }

  .mic-toggle {
    padding: 6px 12px;
    border-radius: 4px;
    border: 1px solid var(--border-color);
    background-color: var(--button-bg);
    cursor: pointer;
  }
  
  .mic-toggle.active {
     background-color: var(--active-button-bg);
     color: white;
  }
  
  select {
    padding: 6px 8px;
    border-radius: 4px;
    border: 1px solid var(--border-color);
    background-color: var(--input-bg);
  }
  
  button {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 4px;
    color: var(--text-secondary);
  }
  
  button:hover {
     color: var(--text-primary);
  }
  
  /* Adjust dark mode variables if needed */
  @media (prefers-color-scheme: dark) {
     select {
       color: var(--text-primary);
     }
  }
</style> 
