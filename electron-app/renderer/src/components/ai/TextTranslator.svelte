<script lang="ts">
  // Using Electron API instead of Tauri
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let text: string = '';
  export let targetLanguage: string = 'es';
  export let sourceLanguage: string = 'auto';
  
  let translation = '';
  let isLoading = false;
  let error = '';
  let autoDetectSource = true;
  
  // Common languages
  const languages = [
    { code: 'auto', name: 'Auto-detect' },
    { code: 'en', name: 'English' },
    { code: 'es', name: 'Spanish' },
    { code: 'fr', name: 'French' },
    { code: 'de', name: 'German' },
    { code: 'it', name: 'Italian' },
    { code: 'pt', name: 'Portuguese' },
    { code: 'ru', name: 'Russian' },
    { code: 'ja', name: 'Japanese' },
    { code: 'ko', name: 'Korean' },
    { code: 'zh', name: 'Chinese' },
    { code: 'ar', name: 'Arabic' },
    { code: 'hi', name: 'Hindi' },
    { code: 'nl', name: 'Dutch' },
    { code: 'pl', name: 'Polish' },
    { code: 'tr', name: 'Turkish' },
    { code: 'vi', name: 'Vietnamese' },
    { code: 'th', name: 'Thai' },
    { code: 'he', name: 'Hebrew' },
    { code: 'sv', name: 'Swedish' }
  ];
  
  // Filter out auto-detect for target language
  $: targetLanguages = languages.filter(lang => lang.code !== 'auto');
  
  async function translate() {
    if (!text.trim()) {
      error = 'Please provide text to translate';
      return;
    }
    
    if (sourceLanguage === targetLanguage && sourceLanguage !== 'auto') {
      error = 'Source and target languages cannot be the same';
      return;
    }
    
    isLoading = true;
    error = '';
    translation = '';
    
    try {
      const result = await window.electronAPI.invoke('translate_text', {
        text: text.trim(),
        targetLanguage,
        sourceLanguage: sourceLanguage === 'auto' ? null : sourceLanguage
      });
      
      translation = result;
      dispatch('translated', { 
        original: text, 
        translation, 
        sourceLanguage: sourceLanguage === 'auto' ? 'detected' : sourceLanguage,
        targetLanguage 
      });
    } catch (e) {
      error = `Translation failed: ${e}`;
      console.error('Translation error:', e);
      
      if (e.includes('not initialized')) {
        error = 'Please initialize a cloud AI provider first (OpenRouter, OpenAI, or Requesty)';
      }
    } finally {
      isLoading = false;
    }
  }
  
  function swapLanguages() {
    if (sourceLanguage !== 'auto' && translation) {
      // Swap text and translation
      const temp = text;
      text = translation;
      translation = temp;
      
      // Swap languages
      const tempLang = sourceLanguage;
      sourceLanguage = targetLanguage;
      targetLanguage = tempLang;
    }
  }
  
  function copyTranslation() {
    navigator.clipboard.writeText(translation);
    dispatch('copied', { text: translation });
  }
  
  function clear() {
    text = '';
    translation = '';
    error = '';
  }
  
  // Character count
  $: charCount = text.length;
  $: translationCharCount = translation.length;
</script>

<div class="text-translator">
  <div class="header">
    <h3>Text Translator</h3>
  </div>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  <div class="language-selector">
    <div class="language-group">
      <label for="source-lang">From</label>
      <select 
        id="source-lang"
        bind:value={sourceLanguage}
        disabled={isLoading}
      >
        {#each languages as lang}
          <option value={lang.code}>{lang.name}</option>
        {/each}
      </select>
    </div>
    
    <button 
      class="swap-button"
      on:click={swapLanguages}
      disabled={isLoading || sourceLanguage === 'auto' || !translation}
      title="Swap languages"
    >
      ⇄
    </button>
    
    <div class="language-group">
      <label for="target-lang">To</label>
      <select 
        id="target-lang"
        bind:value={targetLanguage}
        disabled={isLoading}
      >
        {#each targetLanguages as lang}
          <option value={lang.code}>{lang.name}</option>
        {/each}
      </select>
    </div>
  </div>
  
  <div class="translation-container">
    <div class="input-section">
      <div class="section-header">
        <label for="input-text">Original Text</label>
        <span class="char-count">{charCount} characters</span>
      </div>
      <textarea
        id="input-text"
        bind:value={text}
        placeholder="Enter text to translate..."
        rows="6"
        disabled={isLoading}
      />
    </div>
    
    <div class="middle-section">
      <button 
        class="translate-button" 
        on:click={translate}
        disabled={isLoading || !text.trim()}
      >
        {#if isLoading}
          <span class="spinner-small"></span>
          Translating...
        {:else}
          → Translate →
        {/if}
      </button>
    </div>
    
    <div class="output-section">
      <div class="section-header">
        <label>Translation</label>
        {#if translation}
          <div class="output-actions">
            <span class="char-count">{translationCharCount} characters</span>
            <button class="copy-button" on:click={copyTranslation} title="Copy translation">
              📋
            </button>
          </div>
        {/if}
      </div>
      <textarea
        readonly
        value={translation}
        placeholder="Translation will appear here..."
        rows="6"
        class="translation-output"
      />
    </div>
  </div>
  
  <div class="actions">
    <button 
      class="clear-button" 
      on:click={clear}
      disabled={isLoading || (!text && !translation)}
    >
      Clear All
    </button>
  </div>
</div>

<style>
  .text-translator {
    padding: 20px;
    max-width: 1000px;
    margin: 0 auto;
  }
  
  .header {
    margin-bottom: 20px;
  }
  
  .header h3 {
    margin: 0;
  }
  
  .error {
    background: #f8d7da;
    color: #721c24;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .language-selector {
    display: flex;
    gap: 15px;
    align-items: flex-end;
    margin-bottom: 20px;
  }
  
  .language-group {
    flex: 1;
  }
  
  .language-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: 500;
    font-size: 0.9em;
  }
  
  .language-group select {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 1em;
  }
  
  .swap-button {
    padding: 8px 16px;
    background: #6c757d;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1.2em;
    height: 38px;
  }
  
  .swap-button:hover:not(:disabled) {
    background: #5a6268;
  }
  
  .swap-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .translation-container {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    gap: 20px;
    margin-bottom: 20px;
  }
  
  .input-section,
  .output-section {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
  }
  
  .middle-section {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 10px;
  }
  
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }
  
  .section-header label {
    font-weight: 500;
  }
  
  .char-count {
    font-size: 0.85em;
    color: #666;
  }
  
  .output-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .copy-button {
    padding: 4px 8px;
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
  
  textarea {
    width: 100%;
    padding: 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 1em;
    resize: vertical;
    font-family: inherit;
  }
  
  .translation-output {
    background: white;
  }
  
  .translate-button {
    padding: 12px 24px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1em;
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .translate-button:hover:not(:disabled) {
    background: #0056b3;
  }
  
  .translate-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .spinner-small {
    width: 16px;
    height: 16px;
    border: 2px solid #f3f3f3;
    border-top: 2px solid #fff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .actions {
    display: flex;
    justify-content: flex-end;
  }
  
  .clear-button {
    padding: 10px 20px;
    background: #6c757d;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .clear-button:hover:not(:disabled) {
    background: #5a6268;
  }
  
  .clear-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  @media (max-width: 768px) {
    .translation-container {
      grid-template-columns: 1fr;
    }
    
    .middle-section {
      padding: 10px 0;
    }
    
    .translate-button {
      width: 100%;
      justify-content: center;
    }
  }
  
  @media (prefers-color-scheme: dark) {
    .text-translator {
      color: #e0e0e0;
    }
    
    select,
    textarea {
      background: #1a1a1a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .input-section,
    .output-section {
      background: #2a2a2a;
    }
    
    .translation-output {
      background: #1a1a1a;
    }
    
    .char-count {
      color: #999;
    }
  }
</style>