<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  // Types
  interface CustomModel {
    id: string;
    name: string;
    original_filename: string;
    description: string;
    model_type: string;
    import_date: string;
    tags: string[];
    is_active: boolean;
    usage_count: number;
    validation_result: {
      is_valid: boolean;
      model_info?: {
        model_size_bytes: number;
        estimated_parameters: number;
      };
    };
  }
  
  interface ModelUpdateRequest {
    name?: string;
    description?: string;
    tags?: string[];
    capabilities?: string[];
    supported_languages?: string[];
  }
  
  // State
  let customModels: CustomModel[] = [];
  let selectedModel: CustomModel | null = null;
  let editingModel: CustomModel | null = null;
  let isLoading = false;
  let error = '';
  let successMessage = '';
  let searchQuery = '';
  let filterByType = 'all';
  let sortBy: 'name' | 'date' | 'usage' = 'date';
  
  // Edit form state
  let editForm = {
    name: '',
    description: '',
    tags: [] as string[],
    newTag: '',
    capabilities: [] as string[],
    newCapability: '',
    languages: [] as string[],
    newLanguage: ''
  };
  
  // Load models on mount
  onMount(async () => {
    await loadCustomModels();
  });
  
  // Load custom models
  async function loadCustomModels() {
    isLoading = true;
    error = '';
    
    try {
      customModels = await invoke('list_custom_models');
      sortModels();
    } catch (e) {
      error = `Failed to load custom models: ${e}`;
    } finally {
      isLoading = false;
    }
  }
  
  // Filter models
  $: filteredModels = customModels.filter(model => {
    const matchesSearch = !searchQuery || 
      model.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      model.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      model.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()));
    
    const matchesType = filterByType === 'all' || model.model_type === filterByType;
    
    return matchesSearch && matchesType;
  });
  
  // Sort models
  function sortModels() {
    customModels = [...customModels].sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.name.localeCompare(b.name);
        case 'date':
          return new Date(b.import_date).getTime() - new Date(a.import_date).getTime();
        case 'usage':
          return b.usage_count - a.usage_count;
        default:
          return 0;
      }
    });
  }
  
  // Select model
  function selectModel(model: CustomModel) {
    selectedModel = model;
    editingModel = null;
  }
  
  // Start editing
  function startEdit(model: CustomModel) {
    editingModel = model;
    editForm = {
      name: model.name,
      description: model.description,
      tags: [...model.tags],
      newTag: '',
      capabilities: [],
      newCapability: '',
      languages: [],
      newLanguage: ''
    };
  }
  
  // Cancel editing
  function cancelEdit() {
    editingModel = null;
    editForm = {
      name: '',
      description: '',
      tags: [],
      newTag: '',
      capabilities: [],
      newCapability: '',
      languages: [],
      newLanguage: ''
    };
  }
  
  // Save edits
  async function saveEdits() {
    if (!editingModel) return;
    
    isLoading = true;
    error = '';
    successMessage = '';
    
    try {
      const updates: ModelUpdateRequest = {
        name: editForm.name,
        description: editForm.description,
        tags: editForm.tags,
      };
      
      if (editForm.capabilities.length > 0) {
        updates.capabilities = editForm.capabilities;
      }
      
      if (editForm.languages.length > 0) {
        updates.supported_languages = editForm.languages;
      }
      
      await invoke('update_custom_model', {
        modelId: editingModel.id,
        updates
      });
      
      successMessage = 'Model updated successfully';
      await loadCustomModels();
      cancelEdit();
    } catch (e) {
      error = `Failed to update model: ${e}`;
    } finally {
      isLoading = false;
    }
  }
  
  // Delete model
  async function deleteModel(model: CustomModel) {
    if (!confirm(`Are you sure you want to delete "${model.name}"?`)) return;
    
    isLoading = true;
    error = '';
    
    try {
      await invoke('delete_custom_model', { modelId: model.id });
      successMessage = 'Model deleted successfully';
      await loadCustomModels();
      if (selectedModel?.id === model.id) {
        selectedModel = null;
      }
    } catch (e) {
      error = `Failed to delete model: ${e}`;
    } finally {
      isLoading = false;
    }
  }
  
  // Export model
  async function exportModel(model: CustomModel) {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const savePath = await open({
        directory: false,
        save: true,
        filters: [{
          name: 'ONNX Model',
          extensions: ['onnx']
        }],
        defaultPath: `${model.name}.onnx`
      });
      
      if (savePath && typeof savePath === 'string') {
        await invoke('export_custom_model', {
          modelId: model.id,
          exportPath: savePath
        });
        successMessage = 'Model exported successfully';
      }
    } catch (e) {
      error = `Failed to export model: ${e}`;
    }
  }
  
  // Load model for use
  async function loadModelForUse(model: CustomModel) {
    try {
      await invoke('load_ai_model', { modelId: model.id });
      successMessage = `Model "${model.name}" loaded successfully`;
      dispatch('modelLoaded', { modelId: model.id });
    } catch (e) {
      error = `Failed to load model: ${e}`;
    }
  }
  
  // Add tag
  function addTag() {
    if (editForm.newTag.trim() && !editForm.tags.includes(editForm.newTag.trim())) {
      editForm.tags = [...editForm.tags, editForm.newTag.trim()];
      editForm.newTag = '';
    }
  }
  
  // Remove tag
  function removeTag(tag: string) {
    editForm.tags = editForm.tags.filter(t => t !== tag);
  }
  
  // Add capability
  function addCapability() {
    if (editForm.newCapability.trim() && !editForm.capabilities.includes(editForm.newCapability.trim())) {
      editForm.capabilities = [...editForm.capabilities, editForm.newCapability.trim()];
      editForm.newCapability = '';
    }
  }
  
  // Remove capability
  function removeCapability(cap: string) {
    editForm.capabilities = editForm.capabilities.filter(c => c !== cap);
  }
  
  // Add language
  function addLanguage() {
    if (editForm.newLanguage.trim() && !editForm.languages.includes(editForm.newLanguage.trim())) {
      editForm.languages = [...editForm.languages, editForm.newLanguage.trim()];
      editForm.newLanguage = '';
    }
  }
  
  // Remove language
  function removeLanguage(lang: string) {
    editForm.languages = editForm.languages.filter(l => l !== lang);
  }
  
  // Format file size
  function formatSize(bytes: number): string {
    const gb = bytes / 1e9;
    return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / 1e6).toFixed(0)} MB`;
  }
  
  // Format date
  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString();
  }
  
  // Get model type color
  function getTypeColor(type: string): string {
    switch (type) {
      case 'TextGeneration': return '#007bff';
      case 'TextClassification': return '#28a745';
      case 'TokenClassification': return '#17a2b8';
      case 'Seq2Seq': return '#ffc107';
      default: return '#6c757d';
    }
  }
</script>

<div class="custom-model-manager">
  <div class="header">
    <h3>Custom Model Management</h3>
    <button class="refresh-button" on:click={loadCustomModels} disabled={isLoading}>
      🔄 Refresh
    </button>
  </div>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  {#if successMessage}
    <div class="success">{successMessage}</div>
  {/if}
  
  <div class="controls">
    <div class="search-box">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Search models..."
      />
    </div>
    
    <div class="filters">
      <select bind:value={filterByType}>
        <option value="all">All Types</option>
        <option value="TextGeneration">Text Generation</option>
        <option value="TextClassification">Text Classification</option>
        <option value="TokenClassification">Token Classification</option>
        <option value="Seq2Seq">Seq2Seq</option>
        <option value="Unknown">Unknown</option>
      </select>
      
      <select bind:value={sortBy} on:change={sortModels}>
        <option value="date">Sort by Date</option>
        <option value="name">Sort by Name</option>
        <option value="usage">Sort by Usage</option>
      </select>
    </div>
  </div>
  
  <div class="model-browser">
    <div class="model-list">
      {#if isLoading}
        <div class="loading">Loading models...</div>
      {:else if filteredModels.length === 0}
        <div class="empty">No custom models found</div>
      {:else}
        {#each filteredModels as model}
          <div 
            class="model-item" 
            class:selected={selectedModel?.id === model.id}
            on:click={() => selectModel(model)}
          >
            <div class="model-header">
              <h4>{model.name}</h4>
              <span 
                class="model-type" 
                style="background-color: {getTypeColor(model.model_type)}"
              >
                {model.model_type}
              </span>
            </div>
            <div class="model-meta">
              <span>📅 {formatDate(model.import_date)}</span>
              <span>📊 Used {model.usage_count} times</span>
            </div>
            {#if model.tags.length > 0}
              <div class="tags">
                {#each model.tags as tag}
                  <span class="tag">{tag}</span>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
    
    {#if selectedModel && !editingModel}
      <div class="model-details">
        <div class="details-header">
          <h3>{selectedModel.name}</h3>
          <div class="actions">
            <button on:click={() => startEdit(selectedModel)} title="Edit">
              ✏️
            </button>
            <button on:click={() => exportModel(selectedModel)} title="Export">
              💾
            </button>
            <button on:click={() => loadModelForUse(selectedModel)} title="Load Model">
              ▶️
            </button>
            <button on:click={() => deleteModel(selectedModel)} title="Delete" class="delete">
              🗑️
            </button>
          </div>
        </div>
        
        <div class="detail-section">
          <h4>Description</h4>
          <p>{selectedModel.description || 'No description provided'}</p>
        </div>
        
        <div class="detail-section">
          <h4>Model Information</h4>
          <div class="info-grid">
            <div class="info-item">
              <span class="label">Original File:</span>
              <span>{selectedModel.original_filename}</span>
            </div>
            <div class="info-item">
              <span class="label">Type:</span>
              <span>{selectedModel.model_type}</span>
            </div>
            <div class="info-item">
              <span class="label">Size:</span>
              <span>{formatSize(selectedModel.validation_result.model_info?.model_size_bytes || 0)}</span>
            </div>
            <div class="info-item">
              <span class="label">Parameters:</span>
              <span>~{((selectedModel.validation_result.model_info?.estimated_parameters || 0) / 1e6).toFixed(1)}M</span>
            </div>
            <div class="info-item">
              <span class="label">Valid:</span>
              <span class:valid={selectedModel.validation_result.is_valid}>
                {selectedModel.validation_result.is_valid ? '✓ Yes' : '✗ No'}
              </span>
            </div>
            <div class="info-item">
              <span class="label">Usage Count:</span>
              <span>{selectedModel.usage_count}</span>
            </div>
          </div>
        </div>
        
        {#if selectedModel.tags.length > 0}
          <div class="detail-section">
            <h4>Tags</h4>
            <div class="tags">
              {#each selectedModel.tags as tag}
                <span class="tag">{tag}</span>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
    
    {#if editingModel}
      <div class="model-edit">
        <h3>Edit Model</h3>
        
        <div class="form-group">
          <label for="edit-name">Name</label>
          <input
            id="edit-name"
            type="text"
            bind:value={editForm.name}
            placeholder="Model name"
          />
        </div>
        
        <div class="form-group">
          <label for="edit-description">Description</label>
          <textarea
            id="edit-description"
            bind:value={editForm.description}
            placeholder="Model description"
            rows="3"
          />
        </div>
        
        <div class="form-group">
          <label>Tags</label>
          <div class="tag-input">
            <input
              type="text"
              bind:value={editForm.newTag}
              placeholder="Add tag..."
              on:keydown={(e) => e.key === 'Enter' && addTag()}
            />
            <button on:click={addTag}>Add</button>
          </div>
          <div class="tags">
            {#each editForm.tags as tag}
              <span class="tag">
                {tag}
                <button on:click={() => removeTag(tag)}>×</button>
              </span>
            {/each}
          </div>
        </div>
        
        <div class="form-group">
          <label>Capabilities (Advanced)</label>
          <div class="tag-input">
            <input
              type="text"
              bind:value={editForm.newCapability}
              placeholder="Add capability..."
              on:keydown={(e) => e.key === 'Enter' && addCapability()}
            />
            <button on:click={addCapability}>Add</button>
          </div>
          <div class="tags">
            {#each editForm.capabilities as cap}
              <span class="tag">
                {cap}
                <button on:click={() => removeCapability(cap)}>×</button>
              </span>
            {/each}
          </div>
        </div>
        
        <div class="form-group">
          <label>Supported Languages (Advanced)</label>
          <div class="tag-input">
            <input
              type="text"
              bind:value={editForm.newLanguage}
              placeholder="Add language code (e.g., en, es)..."
              on:keydown={(e) => e.key === 'Enter' && addLanguage()}
            />
            <button on:click={addLanguage}>Add</button>
          </div>
          <div class="tags">
            {#each editForm.languages as lang}
              <span class="tag">
                {lang}
                <button on:click={() => removeLanguage(lang)}>×</button>
              </span>
            {/each}
          </div>
        </div>
        
        <div class="edit-actions">
          <button on:click={cancelEdit} class="cancel">Cancel</button>
          <button on:click={saveEdits} disabled={isLoading} class="save">
            {isLoading ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .custom-model-manager {
    padding: 20px;
    max-width: 1200px;
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
  
  .refresh-button {
    padding: 8px 16px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .refresh-button:hover {
    background: #0056b3;
  }
  
  .refresh-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .error {
    background: #f8d7da;
    color: #721c24;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .success {
    background: #d4edda;
    color: #155724;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .controls {
    display: flex;
    gap: 20px;
    margin-bottom: 20px;
  }
  
  .search-box {
    flex: 1;
  }
  
  .search-box input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 1em;
  }
  
  .filters {
    display: flex;
    gap: 10px;
  }
  
  .filters select {
    padding: 8px 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 1em;
  }
  
  .model-browser {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 20px;
    height: 600px;
  }
  
  .model-list {
    background: #f8f9fa;
    border-radius: 8px;
    padding: 20px;
    overflow-y: auto;
  }
  
  .loading,
  .empty {
    text-align: center;
    color: #666;
    padding: 40px;
  }
  
  .model-item {
    background: white;
    padding: 15px;
    border-radius: 4px;
    margin-bottom: 10px;
    cursor: pointer;
    border: 2px solid transparent;
    transition: all 0.2s;
  }
  
  .model-item:hover {
    border-color: #007bff;
  }
  
  .model-item.selected {
    border-color: #007bff;
    background: #e7f1ff;
  }
  
  .model-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 5px;
  }
  
  .model-header h4 {
    margin: 0;
    font-size: 1.1em;
  }
  
  .model-type {
    color: white;
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 0.8em;
  }
  
  .model-meta {
    display: flex;
    gap: 15px;
    font-size: 0.85em;
    color: #666;
    margin-bottom: 5px;
  }
  
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-top: 5px;
  }
  
  .tag {
    background: #e9ecef;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 0.8em;
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .tag button {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1.2em;
    padding: 0;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .model-details,
  .model-edit {
    background: white;
    border: 1px solid #dee2e6;
    border-radius: 8px;
    padding: 30px;
    overflow-y: auto;
  }
  
  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }
  
  .details-header h3 {
    margin: 0;
  }
  
  .actions {
    display: flex;
    gap: 10px;
  }
  
  .actions button {
    padding: 8px 12px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1.2em;
  }
  
  .actions button:hover {
    background: #0056b3;
  }
  
  .actions button.delete {
    background: #dc3545;
  }
  
  .actions button.delete:hover {
    background: #c82333;
  }
  
  .detail-section {
    margin-bottom: 30px;
  }
  
  .detail-section h4 {
    margin: 0 0 10px 0;
    color: #333;
  }
  
  .detail-section p {
    margin: 0;
    color: #666;
  }
  
  .info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 15px;
  }
  
  .info-item {
    display: flex;
    justify-content: space-between;
  }
  
  .info-item .label {
    font-weight: 500;
    color: #666;
  }
  
  .info-item .valid {
    color: #28a745;
  }
  
  .form-group {
    margin-bottom: 20px;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: 500;
  }
  
  .form-group input,
  .form-group textarea {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 1em;
  }
  
  .tag-input {
    display: flex;
    gap: 10px;
    margin-bottom: 10px;
  }
  
  .tag-input input {
    flex: 1;
  }
  
  .tag-input button {
    padding: 8px 16px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .edit-actions {
    display: flex;
    gap: 10px;
    margin-top: 30px;
  }
  
  .edit-actions button {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1em;
  }
  
  .edit-actions .cancel {
    background: #6c757d;
    color: white;
  }
  
  .edit-actions .save {
    background: #28a745;
    color: white;
  }
  
  .edit-actions button:hover {
    opacity: 0.9;
  }
  
  .edit-actions button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  @media (max-width: 768px) {
    .model-browser {
      grid-template-columns: 1fr;
      height: auto;
    }
    
    .controls {
      flex-direction: column;
    }
    
    .filters {
      width: 100%;
    }
  }
  
  @media (prefers-color-scheme: dark) {
    .model-list {
      background: #2a2a2a;
    }
    
    .model-item {
      background: #1a1a1a;
    }
    
    .model-item.selected {
      background: #1a3a5a;
    }
    
    .model-details,
    .model-edit {
      background: #2a2a2a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .detail-section h4 {
      color: #e0e0e0;
    }
    
    .detail-section p {
      color: #999;
    }
    
    .info-item .label {
      color: #999;
    }
    
    .tag {
      background: #3a3a3a;
      color: #e0e0e0;
    }
    
    .search-box input,
    .filters select,
    .form-group input,
    .form-group textarea {
      background: #1a1a1a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
  }
</style>