<script lang="ts">
  // Using Electron API instead of Tauri
  import { onMount } from 'svelte';
  
  // Types
  interface TextCharacteristics {
    length: number;
    word_count: number;
    avg_word_length: number;
    sentence_count: number;
    avg_sentence_length: number;
    complexity_score: number;
    language: string;
    domain: string;
    contains_code: boolean;
    contains_math: boolean;
    contains_urls: boolean;
    reading_level: number;
    vocabulary_diversity: number;
  }
  
  interface SelectionResult {
    selected_model: {
      id: string;
      name: string;
      performance_class: string;
    };
    capability_score: {
      overall: number;
      scores: Record<string, number>;
      reasons: string[];
    };
    performance_score?: number;
    combined_score: number;
    alternatives: Array<[{ id: string; name: string }, number]>;
    selection_reason: string;
    cached: boolean;
  }
  
  interface SelectorConfig {
    use_performance_metrics: boolean;
    min_score_threshold: number;
    cache_ttl_seconds: number;
    max_cache_entries: number;
    capability_weight: number;
    performance_weight: number;
    prefer_loaded_models: boolean;
  }
  
  // Props
  export let onModelSelected: (modelId: string) => void = () => {};
  export let sampleText: string = '';
  
  // State
  let autoSelectEnabled = true;
  let textToAnalyze = '';
  let characteristics: TextCharacteristics | null = null;
  let selectionResult: SelectionResult | null = null;
  let isAnalyzing = false;
  let isSelecting = false;
  let error = '';
  let showConfig = false;
  let selectionHistory: Array<{ model_id: string; reason: string; score: number }> = [];
  
  let config: SelectorConfig = {
    use_performance_metrics: true,
    min_score_threshold: 0.6,
    cache_ttl_seconds: 3600,
    max_cache_entries: 1000,
    capability_weight: 0.6,
    performance_weight: 0.4,
    prefer_loaded_models: true,
  };
  
  // Load selection history on mount
  onMount(async () => {
    await loadSelectionHistory();
  });
  
  // Analyze text characteristics
  async function analyzeText() {
    if (!textToAnalyze.trim()) {
      error = 'Please enter some text to analyze';
      return;
    }
    
    isAnalyzing = true;
    error = '';
    
    try {
      characteristics = await window.electronAPI.invoke('analyze_text_characteristics', {
        text: textToAnalyze
      });
    } catch (e) {
      error = `Analysis failed: ${e}`;
      characteristics = null;
    } finally {
      isAnalyzing = false;
    }
  }
  
  // Auto-select model
  async function autoSelectModel() {
    if (!textToAnalyze.trim()) {
      error = 'Please enter some text for model selection';
      return;
    }
    
    isSelecting = true;
    error = '';
    
    try {
      selectionResult = await window.electronAPI.invoke('auto_select_model', {
        text: textToAnalyze
      });
      
      if (selectionResult) {
        onModelSelected(selectionResult.selected_model.id);
      }
      
      // Refresh history
      await loadSelectionHistory();
    } catch (e) {
      error = `Selection failed: ${e}`;
      selectionResult = null;
    } finally {
      isSelecting = false;
    }
  }
  
  // Update configuration
  async function updateConfig() {
    try {
      await window.electronAPI.invoke('update_model_selector_config', { config });
      error = '';
    } catch (e) {
      error = `Failed to update configuration: ${e}`;
    }
  }
  
  // Load selection history
  async function loadSelectionHistory() {
    try {
      selectionHistory = await window.electronAPI.invoke('get_model_selection_history');
    } catch (e) {
      console.error('Failed to load selection history:', e);
    }
  }
  
  // Clear cache
  async function clearCache() {
    try {
      await window.electronAPI.invoke('clear_model_selection_cache');
      error = '';
    } catch (e) {
      error = `Failed to clear cache: ${e}`;
    }
  }
  
  // Use sample text
  function useSampleText() {
    textToAnalyze = sampleText;
  }
  
  // Format domain name
  function formatDomain(domain: string): string {
    return domain.charAt(0).toUpperCase() + domain.slice(1).replace(/_/g, ' ');
  }
  
  // Get complexity label
  function getComplexityLabel(score: number): string {
    if (score > 0.8) return 'Very Complex';
    if (score > 0.6) return 'Complex';
    if (score > 0.4) return 'Moderate';
    if (score > 0.2) return 'Simple';
    return 'Very Simple';
  }
  
  // Get score color
  function getScoreColor(score: number): string {
    if (score > 0.9) return '#28a745';
    if (score > 0.7) return '#17a2b8';
    if (score > 0.5) return '#ffc107';
    return '#dc3545';
  }
</script>

<div class="auto-model-selector">
  <div class="header">
    <h3>Automatic Model Selection</h3>
    <label class="toggle">
      <input type="checkbox" bind:checked={autoSelectEnabled} />
      <span>Enable Auto-Selection</span>
    </label>
  </div>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  <div class="input-section">
    <label for="text-input">Text to Analyze</label>
    <textarea
      id="text-input"
      bind:value={textToAnalyze}
      placeholder="Enter text here to analyze and select the best model..."
      rows="6"
    />
    {#if sampleText}
      <button class="link-button" on:click={useSampleText}>
        Use sample text
      </button>
    {/if}
  </div>
  
  <div class="actions">
    <button 
      on:click={analyzeText}
      disabled={isAnalyzing || !textToAnalyze.trim()}
      class="secondary"
    >
      {isAnalyzing ? 'Analyzing...' : 'Analyze Text'}
    </button>
    <button 
      on:click={autoSelectModel}
      disabled={isSelecting || !textToAnalyze.trim() || !autoSelectEnabled}
      class="primary"
    >
      {isSelecting ? 'Selecting...' : 'Auto-Select Model'}
    </button>
  </div>
  
  {#if characteristics}
    <div class="characteristics">
      <h4>Text Characteristics</h4>
      <div class="char-grid">
        <div class="char-item">
          <span class="label">Length</span>
          <span class="value">{characteristics.word_count} words</span>
        </div>
        <div class="char-item">
          <span class="label">Complexity</span>
          <span class="value" style="color: {getScoreColor(characteristics.complexity_score)}">
            {getComplexityLabel(characteristics.complexity_score)}
          </span>
        </div>
        <div class="char-item">
          <span class="label">Domain</span>
          <span class="value">{formatDomain(characteristics.domain)}</span>
        </div>
        <div class="char-item">
          <span class="label">Language</span>
          <span class="value">{characteristics.language.toUpperCase()}</span>
        </div>
        <div class="char-item">
          <span class="label">Reading Level</span>
          <span class="value">Grade {characteristics.reading_level.toFixed(1)}</span>
        </div>
        <div class="char-item">
          <span class="label">Special Content</span>
          <span class="value">
            {#if characteristics.contains_code}Code {/if}
            {#if characteristics.contains_math}Math {/if}
            {#if characteristics.contains_urls}URLs {/if}
            {#if !characteristics.contains_code && !characteristics.contains_math && !characteristics.contains_urls}
              None
            {/if}
          </span>
        </div>
      </div>
    </div>
  {/if}
  
  {#if selectionResult}
    <div class="selection-result">
      <h4>Selection Result</h4>
      <div class="selected-model">
        <div class="model-info">
          <h5>{selectionResult.selected_model.name}</h5>
          <p class="reason">{selectionResult.selection_reason}</p>
        </div>
        <div class="scores">
          <div class="score-item">
            <span class="score-label">Overall Score</span>
            <span 
              class="score-value"
              style="color: {getScoreColor(selectionResult.combined_score)}"
            >
              {(selectionResult.combined_score * 100).toFixed(0)}%
            </span>
          </div>
          {#if selectionResult.performance_score !== undefined}
            <div class="score-item">
              <span class="score-label">Performance</span>
              <span class="score-value">
                {(selectionResult.performance_score * 100).toFixed(0)}%
              </span>
            </div>
          {/if}
          {#if selectionResult.cached}
            <div class="cached-badge">Cached</div>
          {/if}
        </div>
      </div>
      
      {#if selectionResult.capability_score.reasons.length > 0}
        <div class="reasons">
          <h5>Selection Details</h5>
          <ul>
            {#each selectionResult.capability_score.reasons as reason}
              <li>{reason}</li>
            {/each}
          </ul>
        </div>
      {/if}
      
      {#if selectionResult.alternatives.length > 0}
        <div class="alternatives">
          <h5>Alternative Models</h5>
          {#each selectionResult.alternatives as [model, score]}
            <div class="alt-model">
              <span>{model.name}</span>
              <span class="alt-score">{(score * 100).toFixed(0)}%</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
  
  <div class="config-section">
    <button 
      class="toggle-config"
      on:click={() => showConfig = !showConfig}
    >
      {showConfig ? '▼' : '▶'} Configuration
    </button>
    
    {#if showConfig}
      <div class="config-panel">
        <div class="config-item">
          <label>
            <input type="checkbox" bind:checked={config.use_performance_metrics} />
            Use Performance Metrics
          </label>
        </div>
        
        <div class="config-item">
          <label>
            <input type="checkbox" bind:checked={config.prefer_loaded_models} />
            Prefer Loaded Models
          </label>
        </div>
        
        <div class="config-item">
          <label for="min-score">Minimum Score Threshold</label>
          <input 
            id="min-score"
            type="range"
            bind:value={config.min_score_threshold}
            min="0"
            max="1"
            step="0.1"
          />
          <span>{config.min_score_threshold}</span>
        </div>
        
        <div class="config-item">
          <label for="cap-weight">Capability Weight</label>
          <input 
            id="cap-weight"
            type="range"
            bind:value={config.capability_weight}
            min="0"
            max="1"
            step="0.1"
          />
          <span>{config.capability_weight}</span>
        </div>
        
        <div class="config-item">
          <label for="perf-weight">Performance Weight</label>
          <input 
            id="perf-weight"
            type="range"
            bind:value={config.performance_weight}
            min="0"
            max="1"
            step="0.1"
          />
          <span>{config.performance_weight}</span>
        </div>
        
        <div class="config-actions">
          <button on:click={updateConfig}>Save Configuration</button>
          <button on:click={clearCache} class="secondary">Clear Cache</button>
        </div>
      </div>
    {/if}
  </div>
  
  {#if selectionHistory.length > 0}
    <div class="history">
      <h4>Recent Selections</h4>
      <div class="history-list">
        {#each selectionHistory.slice(0, 5) as item}
          <div class="history-item">
            <span class="model-id">{item.model_id}</span>
            <span class="score">{(item.score * 100).toFixed(0)}%</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .auto-model-selector {
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
    cursor: pointer;
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
  
  textarea {
    width: 100%;
    padding: 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-family: inherit;
    font-size: 0.95em;
    resize: vertical;
  }
  
  .link-button {
    background: none;
    border: none;
    color: #007bff;
    cursor: pointer;
    padding: 5px 0;
    font-size: 0.9em;
    text-decoration: underline;
  }
  
  .actions {
    display: flex;
    gap: 10px;
    margin-bottom: 30px;
  }
  
  button {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    font-size: 0.95em;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  button.primary {
    background: #007bff;
    color: white;
  }
  
  button.primary:hover:not(:disabled) {
    background: #0056b3;
  }
  
  button.secondary {
    background: #6c757d;
    color: white;
  }
  
  button.secondary:hover:not(:disabled) {
    background: #5a6268;
  }
  
  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .characteristics {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
  }
  
  .characteristics h4 {
    margin: 0 0 15px 0;
  }
  
  .char-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 15px;
  }
  
  .char-item {
    display: flex;
    flex-direction: column;
  }
  
  .char-item .label {
    font-size: 0.85em;
    color: #666;
    margin-bottom: 4px;
  }
  
  .char-item .value {
    font-weight: 500;
  }
  
  .selection-result {
    background: #e7f3ff;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
  }
  
  .selection-result h4 {
    margin: 0 0 15px 0;
  }
  
  .selected-model {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 20px;
  }
  
  .model-info h5 {
    margin: 0 0 8px 0;
    font-size: 1.1em;
  }
  
  .model-info .reason {
    margin: 0;
    color: #666;
  }
  
  .scores {
    display: flex;
    gap: 20px;
    align-items: center;
  }
  
  .score-item {
    text-align: center;
  }
  
  .score-label {
    display: block;
    font-size: 0.85em;
    color: #666;
    margin-bottom: 4px;
  }
  
  .score-value {
    font-size: 1.5em;
    font-weight: bold;
  }
  
  .cached-badge {
    padding: 4px 8px;
    background: #17a2b8;
    color: white;
    border-radius: 12px;
    font-size: 0.8em;
  }
  
  .reasons,
  .alternatives {
    margin-top: 15px;
  }
  
  .reasons h5,
  .alternatives h5 {
    margin: 0 0 10px 0;
    font-size: 0.95em;
  }
  
  .reasons ul {
    margin: 0;
    padding-left: 20px;
  }
  
  .reasons li {
    font-size: 0.9em;
    color: #666;
  }
  
  .alt-model {
    display: flex;
    justify-content: space-between;
    padding: 5px 0;
    font-size: 0.9em;
  }
  
  .alt-score {
    color: #666;
  }
  
  .config-section {
    margin-top: 30px;
  }
  
  .toggle-config {
    background: none;
    border: none;
    color: #007bff;
    cursor: pointer;
    padding: 5px 0;
    font-size: 1em;
  }
  
  .config-panel {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-top: 15px;
  }
  
  .config-item {
    margin-bottom: 15px;
  }
  
  .config-item label {
    display: block;
    margin-bottom: 5px;
  }
  
  .config-item input[type="range"] {
    width: 200px;
    margin-right: 10px;
  }
  
  .config-actions {
    display: flex;
    gap: 10px;
    margin-top: 20px;
  }
  
  .history {
    margin-top: 30px;
  }
  
  .history h4 {
    margin: 0 0 15px 0;
  }
  
  .history-list {
    background: #f8f9fa;
    padding: 15px;
    border-radius: 8px;
  }
  
  .history-item {
    display: flex;
    justify-content: space-between;
    padding: 5px 0;
    font-size: 0.9em;
  }
  
  .model-id {
    font-weight: 500;
  }
  
  @media (prefers-color-scheme: dark) {
    .characteristics {
      background: #2a2a2a;
    }
    
    .selection-result {
      background: #1a3a52;
    }
    
    .char-item .label,
    .score-label,
    .model-info .reason,
    .reasons li,
    .alt-score {
      color: #999;
    }
    
    .config-panel,
    .history-list {
      background: #2a2a2a;
    }
    
    textarea {
      background: #1a1a1a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
  }
</style>