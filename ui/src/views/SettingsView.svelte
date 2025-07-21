<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  // Settings state
  let audioDevices: [string, string][] = []; // Expecting [id, name] tuples
  let whisperModels: string[] = [];
  let selectedDevice: string | null = null;
  let selectedModel = 'small';
  let isAutoTranscribe = true;
  let offlineMode = true;
  
  // Language and advanced options
  let languages: [string, string][] = []; // Expecting [code, name] tuples
  let selectedLanguage = 'auto';
  let autoPunctuate = true;
  let translateToEnglish = false;
  
  // Tab selection
  let activeTab = 'general';

  // Save status state
  let isSaving = false;
  let saveMessage = '';
  let saveMessageType: 'success' | 'error' | '' = '';
  let saveMessageTimeout: number | undefined = undefined;

  // Initial settings state for comparison
  let initialSelectedDevice: string | null = null;
  let initialSelectedModel: string | null = null;
  let initialIsAutoTranscribe: boolean | null = null;
  let initialOfflineMode: boolean | null = null;
  let initialSelectedLanguage: string | null = null;
  let initialAutoPunctuate: boolean | null = null;
  let initialTranslateToEnglish: boolean | null = null;
  
  onMount(async () => {
    try {
      // Fetch initial data from backend
      const [devices, models, langs, settingsData] = await Promise.all([
        invoke<[string, string][]>('get_audio_devices'),
        invoke<string[]>('get_whisper_models'),
        invoke<[string, string][]>('get_supported_languages'),
        invoke<any>('get_settings') // Fetch settings as any type for now
      ]);

      audioDevices = devices;
      whisperModels = models;
      // Add "Auto-detect" option manually if needed by backend/UI logic
      languages = [['auto', 'Auto-detect'], ...langs]; 

      // Define default values before potentially overwriting with saved settings
      const defaultDevice = devices.length > 0 ? devices[0][0] : null;
      const defaultModel = 'small';
      const defaultAutoTranscribe = true;
      const defaultOfflineMode = true;
      const defaultLanguage = 'auto';
      const defaultAutoPunctuate = true;
      const defaultTranslateToEnglish = false;

      // Apply loaded settings or fall back to defaults
      if (settingsData) {
        const savedDeviceName = settingsData.audio?.input_device;
        selectedDevice = savedDeviceName || defaultDevice;
        selectedModel = settingsData.audio?.speech?.model_size || defaultModel;
        isAutoTranscribe = settingsData.general?.auto_transcribe ?? defaultAutoTranscribe;
        offlineMode = settingsData.general?.offline_mode ?? defaultOfflineMode;
        selectedLanguage = settingsData.audio?.speech?.language || defaultLanguage;
        autoPunctuate = settingsData.audio?.speech?.auto_punctuate ?? defaultAutoPunctuate;
        translateToEnglish = settingsData.audio?.speech?.translate_to_english ?? defaultTranslateToEnglish;
      } else {
        // Fallback to defaults if settings couldn't be loaded
        selectedDevice = defaultDevice;
        selectedModel = defaultModel;
        isAutoTranscribe = defaultAutoTranscribe;
        offlineMode = defaultOfflineMode;
        selectedLanguage = defaultLanguage;
        autoPunctuate = defaultAutoPunctuate;
        translateToEnglish = defaultTranslateToEnglish;
      }

      // Store initial values *after* applying loaded settings or defaults
      initialSelectedDevice = selectedDevice;
      initialSelectedModel = selectedModel;
      initialIsAutoTranscribe = isAutoTranscribe;
      initialOfflineMode = offlineMode;
      initialSelectedLanguage = selectedLanguage;
      initialAutoPunctuate = autoPunctuate;
      initialTranslateToEnglish = translateToEnglish;

    } catch (error) {
      console.error("Failed to load settings data:", error);
      // Keep default values or show an error message to the user
      // For now, we fallback to defaults/empty lists
      audioDevices = audioDevices.length > 0 ? audioDevices : [];
      whisperModels = whisperModels.length > 0 ? whisperModels : ['small']; // Default model
      languages = languages.length > 0 ? languages : [['auto', 'Auto-detect']];
      // Set current values to defaults on error
      selectedDevice = null;
      selectedModel = 'small';
      isAutoTranscribe = true;
      offlineMode = true;
      selectedLanguage = 'auto';
      autoPunctuate = true;
      translateToEnglish = false;
      // Also reset initial values to null/defaults to prevent false unsaved state
      initialSelectedDevice = null;
      initialSelectedModel = 'small';
      initialIsAutoTranscribe = true;
      initialOfflineMode = true;
      initialSelectedLanguage = 'auto';
      initialAutoPunctuate = true;
      initialTranslateToEnglish = false;
    }
  });

  // Reactive check for unsaved changes
  $: hasUnsavedChanges = 
    selectedDevice !== initialSelectedDevice ||
    selectedModel !== initialSelectedModel ||
    isAutoTranscribe !== initialIsAutoTranscribe ||
    offlineMode !== initialOfflineMode ||
    selectedLanguage !== initialSelectedLanguage ||
    autoPunctuate !== initialAutoPunctuate ||
    translateToEnglish !== initialTranslateToEnglish;

  // Save settings
  async function saveSettings() {
    isSaving = true;
    saveMessage = ''; // Clear previous message
    saveMessageType = '';
    if (saveMessageTimeout) clearTimeout(saveMessageTimeout); // Clear previous timeout

    try {
      const speechSettings = {
        language: selectedLanguage,
        auto_punctuate: autoPunctuate,
        translate_to_english: translateToEnglish,
        // Add other speech settings from UI if they exist (context_formatting, etc.)
        // segment_duration: ..., 
        // buffer_size: ... 
      };

      await invoke('save_all_settings', {
        deviceName: selectedDevice, // Send the ID
        modelName: selectedModel,
        autoTranscribe: isAutoTranscribe,
        offlineMode: offlineMode,
        speechSettings: speechSettings
      });

      saveMessage = 'Settings saved successfully!';
      saveMessageType = 'success';
      // Update initial state after successful save
      initialSelectedDevice = selectedDevice;
      initialSelectedModel = selectedModel;
      initialIsAutoTranscribe = isAutoTranscribe;
      initialOfflineMode = offlineMode;
      initialSelectedLanguage = selectedLanguage;
      initialAutoPunctuate = autoPunctuate;
      initialTranslateToEnglish = translateToEnglish;

    } catch (error) {
      console.error("Failed to save settings:", error);
      saveMessage = `Error saving settings: ${error}`;
      saveMessageType = 'error';
    } finally {
      isSaving = false;
      // Clear the message after 3 seconds
      saveMessageTimeout = window.setTimeout(() => {
        saveMessage = '';
        saveMessageType = '';
      }, 3000);
    }
  }
</script>

<main>
  <h1>BestMe Settings</h1>
  
  <div class="tabs">
    <button 
      class:active={activeTab === 'general'} 
      on:click={() => activeTab = 'general'}
    >
      General
    </button>
    <button 
      class:active={activeTab === 'transcription'} 
      on:click={() => activeTab = 'transcription'}
    >
      Transcription
    </button>
    <button 
      class:active={activeTab === 'advanced'} 
      on:click={() => activeTab = 'advanced'}
    >
      Advanced
    </button>
  </div>
  
  <div class="settings-container">
    <!-- General Settings -->
    {#if activeTab === 'general'}
      <section>
        <h2>Audio Settings</h2>
        
        <div class="setting-item">
          <label for="device-select">Recording Device</label>
          <select id="device-select" bind:value={selectedDevice} required disabled={audioDevices.length === 0}>
            {#if audioDevices.length === 0}
               <option value={null} disabled>No devices found</option>
            {/if}
            {#each audioDevices as [id, name] (id)}
              <option value={id}>{name}</option>
            {/each}
          </select>
        </div>
        
        <div class="setting-item">
          <label>
            <input type="checkbox" bind:checked={isAutoTranscribe} />
            Start transcribing automatically
          </label>
        </div>
        
        <div class="setting-item">
          <label>
            <input type="checkbox" bind:checked={offlineMode} />
            Use offline mode when available
          </label>
        </div>
      </section>
    {/if}
    
    <!-- Transcription Settings -->
    {#if activeTab === 'transcription'}
      <section>
        <h2>Transcription Settings</h2>
        
        <div class="setting-item">
          <label for="model-select">Whisper Model</label>
          <select id="model-select" bind:value={selectedModel} disabled={whisperModels.length === 0}>
            {#each whisperModels as model}
              <option value={model}>{model}</option>
            {/each}
          </select>
          <p class="description">
            Model size affects accuracy and performance.
          </p>
        </div>
        
        <div class="setting-item">
          <label for="language-select">Default Language</label>
          <select id="language-select" bind:value={selectedLanguage}>
            {#each languages as [code, name]}
              <option value={code}>{name}</option>
            {/each}
          </select>
        </div>
        
        <div class="setting-item">
          <label>
            <input type="checkbox" bind:checked={autoPunctuate} />
            Automatically add punctuation
          </label>
        </div>
        
        <div class="setting-item">
          <label>
            <input type="checkbox" bind:checked={translateToEnglish} />
            Translate to English
          </label>
        </div>
      </section>
    {/if}
    
    <!-- Advanced Settings -->
    {#if activeTab === 'advanced'}
      <section>
        <h2>Advanced Settings</h2>
        <p>These settings are for advanced users only.</p>
        
        <div class="placeholder">
          <p>Advanced settings will be available in a future update.</p>
        </div>
      </section>
    {/if}
  </div>
  
  <div class="actions">
    {#if saveMessage}
      <div class="save-status {saveMessageType}">
        {saveMessage}
      </div>
    {/if}
    <button 
      class="save-button" 
      on:click={saveSettings} 
      disabled={isSaving || !hasUnsavedChanges}
      title={!hasUnsavedChanges ? 'No changes to save' : ''}
    >
      {#if isSaving}Saving...{:else}Save Settings{/if}
    </button>
  </div>
</main>

<style>
  main {
    padding: 20px;
    max-width: 800px;
    margin: 0 auto;
  }
  
  h1 {
    margin-bottom: 20px;
    font-size: 24px;
  }
  
  h2 {
    margin-bottom: 16px;
    font-size: 18px;
  }
  
  .tabs {
    display: flex;
    margin-bottom: 20px;
    border-bottom: 1px solid #ccc;
  }
  
  .tabs button {
    background: none;
    border: none;
    padding: 10px 20px;
    cursor: pointer;
    font-size: 16px;
    border-bottom: 2px solid transparent;
  }
  
  .tabs button.active {
    border-bottom-color: #4a7dff;
    color: #4a7dff;
  }
  
  .setting-item {
    margin-bottom: 20px;
  }
  
  .setting-item label {
    display: block;
    margin-bottom: 6px;
    font-weight: 500;
  }
  
  /* Add styling for select elements */
  select {
    width: 100%;
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 16px;
    background-color: var(--input-bg, white); /* Use CSS variable */
    color: var(--input-text, #333); /* Use CSS variable */
  }

  select:disabled {
    background-color: #eee;
    cursor: not-allowed;
  }

  .description {
    font-size: 14px;
    color: #666;
    margin-top: 4px;
  }
  
  .placeholder {
    background-color: #f4f4f4;
    padding: 20px;
    border-radius: 4px;
    text-align: center;
  }
  
  .actions {
    margin-top: 30px;
    display: flex;
    justify-content: flex-end;
    align-items: center; /* Align items vertically */
    gap: 10px; /* Add space between message and button */
  }

  .save-status {
    padding: 5px 10px;
    border-radius: 4px;
    font-size: 14px;
  }

  .save-status.success {
    background-color: var(--success-bg, #d4edda); /* Use CSS variables for theming */
    color: var(--success-text, #155724);
    border: 1px solid var(--success-border, #c3e6cb);
  }

  .save-status.error {
    background-color: var(--error-bg, #f8d7da);
    color: var(--error-text, #721c24);
    border: 1px solid var(--error-border, #f5c6cb);
  }
  
  .save-button {
    background-color: #4a7dff;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 4px;
    font-size: 16px;
    cursor: pointer;
    transition: background-color 0.2s ease, opacity 0.2s ease; /* Add opacity transition */
  }
  
  .save-button:hover:not(:disabled) { /* Don't change hover style when disabled */
    background-color: #3a6eee;
  }

  .save-button:disabled { /* Style for disabled state */
    background-color: #aabfff; /* Lighter blue when disabled due to no changes */
    opacity: 0.7; /* Make it slightly transparent */
    cursor: not-allowed;
  }
  
  /* Support dark mode */
  @media (prefers-color-scheme: dark) {
    main {
      color: #eee;
      /* Define input variables for dark mode */
      --input-bg: #333;
      --input-text: #eee;
    }
    
    .tabs {
      border-bottom-color: #444;
    }

    select {
      border-color: #555;
    }

    select:disabled {
      background-color: #444;
      color: #aaa;
    }

    .description {
      color: #aaa;
    }
    
    .placeholder {
      background-color: #333;
    }

    .save-status.success {
      background-color: #1e4c2a; /* Darker success */
      color: #c3e6cb;
      border-color: #2a683a;
    }

    .save-status.error {
      background-color: #5d2127; /* Darker error */
      color: #f5c6cb;
      border-color: #8c2d36;
    }

    .save-button:disabled {
      background-color: #4a7dff; /* Keep same blue but change opacity */
      opacity: 0.5;
    }
  }
</style> 
