<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { toast } from '@zerodevx/svelte-toast';
  
  // Vocabulary types
  interface VocabularyEntry {
    term: string;
    boost: number;
    category?: string;
    variants: string[];
    context?: string;
  }
  
  interface VocabularyCategory {
    name: string;
    description: string;
    enabled: boolean;
  }
  
  // State
  let entries: VocabularyEntry[] = [];
  let categories: VocabularyCategory[] = [];
  let searchQuery = '';
  let selectedCategory = 'all';
  let showAddDialog = false;
  let editingEntry: VocabularyEntry | null = null;
  
  // Form state
  let newEntry: VocabularyEntry = {
    term: '',
    boost: 1.5,
    category: 'custom',
    variants: [],
    context: ''
  };
  let variantInput = '';
  
  // Filtered entries based on search and category
  $: filteredEntries = entries.filter(entry => {
    const matchesSearch = !searchQuery || 
      entry.term.toLowerCase().includes(searchQuery.toLowerCase()) ||
      entry.variants.some(v => v.toLowerCase().includes(searchQuery.toLowerCase()));
    
    const matchesCategory = selectedCategory === 'all' || 
      entry.category === selectedCategory;
    
    return matchesSearch && matchesCategory;
  });
  
  onMount(async () => {
    await loadVocabulary();
  });
  
  async function loadVocabulary() {
    try {
      const [entriesData, categoriesData] = await Promise.all([
        invoke<VocabularyEntry[]>('vocabulary_get_all'),
        invoke<VocabularyCategory[]>('vocabulary_get_categories')
      ]);
      
      entries = entriesData;
      categories = categoriesData;
    } catch (error) {
      console.error('Failed to load vocabulary:', error);
      toast.push(`Error loading vocabulary: ${error}`);
    }
  }
  
  async function addEntry() {
    if (!newEntry.term) {
      toast.push('Please enter a term');
      return;
    }
    
    try {
      await invoke('vocabulary_add_entry', { entry: newEntry });
      toast.push(`Added "${newEntry.term}" to vocabulary`);
      
      // Reset form
      newEntry = {
        term: '',
        boost: 1.5,
        category: 'custom',
        variants: [],
        context: ''
      };
      variantInput = '';
      showAddDialog = false;
      
      // Reload
      await loadVocabulary();
    } catch (error) {
      console.error('Failed to add entry:', error);
      toast.push(`Error adding entry: ${error}`);
    }
  }
  
  async function updateEntry() {
    if (!editingEntry) return;
    
    try {
      await invoke('vocabulary_update_entry', { entry: editingEntry });
      toast.push(`Updated "${editingEntry.term}"`);
      
      editingEntry = null;
      await loadVocabulary();
    } catch (error) {
      console.error('Failed to update entry:', error);
      toast.push(`Error updating entry: ${error}`);
    }
  }
  
  async function deleteEntry(term: string) {
    if (!confirm(`Delete "${term}" from vocabulary?`)) return;
    
    try {
      await invoke('vocabulary_remove_entry', { term });
      toast.push(`Removed "${term}" from vocabulary`);
      await loadVocabulary();
    } catch (error) {
      console.error('Failed to delete entry:', error);
      toast.push(`Error deleting entry: ${error}`);
    }
  }
  
  async function toggleCategory(category: string, enabled: boolean) {
    try {
      await invoke('vocabulary_set_category_enabled', { category, enabled });
      toast.push(`${enabled ? 'Enabled' : 'Disabled'} category "${category}"`);
      await loadVocabulary();
    } catch (error) {
      console.error('Failed to toggle category:', error);
      toast.push(`Error toggling category: ${error}`);
    }
  }
  
  async function importVocabulary() {
    try {
      const imported = await invoke<number>('vocabulary_import_csv');
      toast.push(`Imported ${imported} vocabulary entries`);
      await loadVocabulary();
    } catch (error) {
      console.error('Failed to import vocabulary:', error);
      toast.push(`Error importing vocabulary: ${error}`);
    }
  }
  
  async function exportVocabulary() {
    try {
      const exported = await invoke<number>('vocabulary_export_csv');
      toast.push(`Exported ${exported} vocabulary entries`);
    } catch (error) {
      console.error('Failed to export vocabulary:', error);
      toast.push(`Error exporting vocabulary: ${error}`);
    }
  }
  
  function addVariant() {
    if (variantInput && !newEntry.variants.includes(variantInput)) {
      newEntry.variants = [...newEntry.variants, variantInput];
      variantInput = '';
    }
  }
  
  function removeVariant(variant: string) {
    newEntry.variants = newEntry.variants.filter(v => v !== variant);
  }
  
  function startEdit(entry: VocabularyEntry) {
    editingEntry = { ...entry, variants: [...entry.variants] };
  }
  
  function cancelEdit() {
    editingEntry = null;
  }
</script>

<div class="vocabulary-view">
  <div class="header">
    <h2>Vocabulary Management</h2>
    <p class="subtitle">Boost recognition accuracy with custom terms</p>
  </div>
  
  <div class="controls">
    <div class="search-bar">
      <input
        type="text"
        placeholder="Search vocabulary..."
        bind:value={searchQuery}
        class="search-input"
      />
    </div>
    
    <div class="filter-controls">
      <select bind:value={selectedCategory} class="category-filter">
        <option value="all">All Categories</option>
        {#each categories as category}
          <option value={category.name}>{category.description}</option>
        {/each}
      </select>
      
      <button class="btn btn-primary" on:click={() => showAddDialog = true}>
        Add Term
      </button>
      
      <button class="btn btn-secondary" on:click={importVocabulary}>
        Import CSV
      </button>
      
      <button class="btn btn-secondary" on:click={exportVocabulary}>
        Export CSV
      </button>
    </div>
  </div>
  
  <!-- Categories Section -->
  <div class="categories-section">
    <h3>Categories</h3>
    <div class="categories-grid">
      {#each categories as category}
        <div class="category-card" class:disabled={!category.enabled}>
          <div class="category-info">
            <h4>{category.description}</h4>
            <span class="category-name">{category.name}</span>
          </div>
          <label class="toggle">
            <input
              type="checkbox"
              checked={category.enabled}
              on:change={(e) => toggleCategory(category.name, e.currentTarget.checked)}
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      {/each}
    </div>
  </div>
  
  <!-- Vocabulary Entries -->
  <div class="entries-section">
    <h3>Vocabulary Entries ({filteredEntries.length})</h3>
    <div class="entries-list">
      {#each filteredEntries as entry}
        {#if editingEntry && editingEntry.term === entry.term}
          <!-- Edit mode -->
          <div class="entry-card editing">
            <div class="entry-form">
              <input
                type="text"
                bind:value={editingEntry.term}
                placeholder="Term"
                class="input-field"
              />
              
              <input
                type="number"
                bind:value={editingEntry.boost}
                min="0.5"
                max="5"
                step="0.1"
                class="boost-input"
              />
              
              <select bind:value={editingEntry.category} class="category-select">
                {#each categories as category}
                  <option value={category.name}>{category.description}</option>
                {/each}
              </select>
              
              <textarea
                bind:value={editingEntry.context}
                placeholder="Context hint (optional)"
                class="context-input"
              />
              
              <div class="form-actions">
                <button class="btn btn-primary" on:click={updateEntry}>Save</button>
                <button class="btn btn-secondary" on:click={cancelEdit}>Cancel</button>
              </div>
            </div>
          </div>
        {:else}
          <!-- Display mode -->
          <div class="entry-card">
            <div class="entry-header">
              <h4 class="term">{entry.term}</h4>
              <div class="boost-badge">×{entry.boost}</div>
            </div>
            
            {#if entry.variants.length > 0}
              <div class="variants">
                Variants: {entry.variants.join(', ')}
              </div>
            {/if}
            
            {#if entry.context}
              <div class="context">
                {entry.context}
              </div>
            {/if}
            
            <div class="entry-footer">
              <span class="category-tag">{entry.category}</span>
              <div class="entry-actions">
                <button class="btn-icon" on:click={() => startEdit(entry)}>
                  ✏️
                </button>
                <button class="btn-icon delete" on:click={() => deleteEntry(entry.term)}>
                  🗑️
                </button>
              </div>
            </div>
          </div>
        {/if}
      {/each}
      
      {#if filteredEntries.length === 0}
        <div class="empty-state">
          <p>No vocabulary entries found</p>
          {#if searchQuery}
            <p class="hint">Try a different search term</p>
          {:else}
            <p class="hint">Add some terms to boost recognition accuracy</p>
          {/if}
        </div>
      {/if}
    </div>
  </div>
  
  <!-- Add Dialog -->
  {#if showAddDialog}
    <div class="dialog-overlay" on:click={() => showAddDialog = false}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Add Vocabulary Term</h3>
        
        <div class="form-group">
          <label>Term *</label>
          <input
            type="text"
            bind:value={newEntry.term}
            placeholder="e.g., API, GraphQL, Kubernetes"
            class="input-field"
          />
        </div>
        
        <div class="form-group">
          <label>Boost Factor</label>
          <input
            type="number"
            bind:value={newEntry.boost}
            min="0.5"
            max="5"
            step="0.1"
            class="boost-input"
          />
          <span class="hint">Higher values increase recognition likelihood</span>
        </div>
        
        <div class="form-group">
          <label>Category</label>
          <select bind:value={newEntry.category} class="category-select">
            {#each categories as category}
              <option value={category.name}>{category.description}</option>
            {/each}
          </select>
        </div>
        
        <div class="form-group">
          <label>Variants</label>
          <div class="variant-input-group">
            <input
              type="text"
              bind:value={variantInput}
              placeholder="Add variant spelling"
              on:keydown={(e) => e.key === 'Enter' && addVariant()}
              class="input-field"
            />
            <button class="btn btn-secondary" on:click={addVariant}>Add</button>
          </div>
          
          <div class="variants-list">
            {#each newEntry.variants as variant}
              <span class="variant-chip">
                {variant}
                <button class="remove" on:click={() => removeVariant(variant)}>×</button>
              </span>
            {/each}
          </div>
        </div>
        
        <div class="form-group">
          <label>Context Hint</label>
          <textarea
            bind:value={newEntry.context}
            placeholder="Optional context to help recognition"
            class="context-input"
          />
        </div>
        
        <div class="dialog-actions">
          <button class="btn btn-primary" on:click={addEntry}>Add Term</button>
          <button class="btn btn-secondary" on:click={() => showAddDialog = false}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .vocabulary-view {
    padding: 20px;
    height: 100%;
    overflow-y: auto;
  }
  
  .header {
    margin-bottom: 24px;
  }
  
  .header h2 {
    margin: 0 0 8px 0;
    color: var(--text-primary);
  }
  
  .subtitle {
    color: var(--text-secondary);
    margin: 0;
  }
  
  .controls {
    display: flex;
    gap: 16px;
    margin-bottom: 24px;
    align-items: center;
  }
  
  .search-bar {
    flex: 1;
  }
  
  .search-input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
  }
  
  .filter-controls {
    display: flex;
    gap: 8px;
  }
  
  .category-filter {
    padding: 8px 12px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--dropdown-bg);
    color: var(--text-primary);
  }
  
  .btn {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
  }
  
  .btn-primary {
    background: var(--primary-color);
    color: white;
  }
  
  .btn-primary:hover {
    background: var(--primary-color-hover);
  }
  
  .btn-secondary {
    background: var(--button-bg);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
  }
  
  .btn-secondary:hover {
    background: var(--button-hover-bg);
  }
  
  .categories-section {
    margin-bottom: 32px;
  }
  
  .categories-section h3 {
    margin: 0 0 16px 0;
    color: var(--text-primary);
  }
  
  .categories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 16px;
  }
  
  .category-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    background: var(--panel-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    transition: all 0.2s;
  }
  
  .category-card.disabled {
    opacity: 0.6;
  }
  
  .category-info h4 {
    margin: 0 0 4px 0;
    font-size: 14px;
    font-weight: 500;
  }
  
  .category-name {
    font-size: 12px;
    color: var(--text-secondary);
  }
  
  .toggle {
    position: relative;
    display: inline-block;
    width: 48px;
    height: 24px;
  }
  
  .toggle input {
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
  
  .toggle input:checked + .toggle-slider {
    background-color: var(--primary-color);
  }
  
  .toggle input:checked + .toggle-slider:before {
    transform: translateX(24px);
  }
  
  .entries-section h3 {
    margin: 0 0 16px 0;
    color: var(--text-primary);
  }
  
  .entries-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
  }
  
  .entry-card {
    padding: 16px;
    background: var(--panel-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    transition: all 0.2s;
  }
  
  .entry-card:hover {
    border-color: var(--primary-color);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }
  
  .entry-card.editing {
    border-color: var(--primary-color);
  }
  
  .entry-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  
  .term {
    margin: 0;
    font-size: 16px;
    font-weight: 500;
    color: var(--text-primary);
  }
  
  .boost-badge {
    padding: 4px 8px;
    background: var(--primary-color);
    color: white;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 500;
  }
  
  .variants {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 8px;
  }
  
  .context {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 12px;
    font-style: italic;
  }
  
  .entry-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .category-tag {
    padding: 4px 8px;
    background: var(--button-bg);
    border-radius: 4px;
    font-size: 12px;
    color: var(--text-secondary);
  }
  
  .entry-actions {
    display: flex;
    gap: 8px;
  }
  
  .btn-icon {
    padding: 4px 8px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    opacity: 0.7;
    transition: all 0.2s;
  }
  
  .btn-icon:hover {
    opacity: 1;
    transform: scale(1.1);
  }
  
  .btn-icon.delete:hover {
    color: #f44336;
  }
  
  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    padding: 48px;
    color: var(--text-secondary);
  }
  
  .hint {
    font-size: 14px;
    color: var(--text-tertiary);
  }
  
  /* Dialog styles */
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  
  .dialog {
    background: var(--content-bg);
    border-radius: 8px;
    padding: 24px;
    max-width: 500px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
  }
  
  .dialog h3 {
    margin: 0 0 20px 0;
    color: var(--text-primary);
  }
  
  .form-group {
    margin-bottom: 16px;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 4px;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
  }
  
  .input-field,
  .boost-input,
  .category-select,
  .context-input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
  }
  
  .context-input {
    min-height: 80px;
    resize: vertical;
  }
  
  .variant-input-group {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
  }
  
  .variant-input-group .input-field {
    flex: 1;
  }
  
  .variants-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  
  .variant-chip {
    padding: 4px 12px;
    background: var(--button-bg);
    border-radius: 16px;
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  
  .variant-chip .remove {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    opacity: 0.7;
  }
  
  .variant-chip .remove:hover {
    opacity: 1;
  }
  
  .dialog-actions,
  .form-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 20px;
  }
  
  .entry-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  
  /* Dark mode adjustments */
  @media (prefers-color-scheme: dark) {
    .category-card:hover,
    .entry-card:hover {
      box-shadow: 0 2px 8px rgba(255, 255, 255, 0.1);
    }
  }
</style>