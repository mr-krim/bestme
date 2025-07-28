<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';
  
  // Types
  interface ValidationResult {
    is_valid: boolean;
    model_info?: ModelInfo;
    issues: ValidationIssue[];
    warnings: string[];
    compatibility: CompatibilityInfo;
  }
  
  interface ModelInfo {
    input_names: string[];
    output_names: string[];
    input_shapes: Record<string, number[]>;
    output_shapes: Record<string, number[]>;
    opset_version: number;
    model_size_bytes: number;
    estimated_parameters: number;
  }
  
  interface ValidationIssue {
    severity: 'Error' | 'Warning' | 'Info';
    category: string;
    message: string;
    details?: string;
  }
  
  interface CompatibilityInfo {
    supported_architectures: string[];
    minimum_onnx_version: string;
    requires_gpu: boolean;
    estimated_memory_mb: number;
    supports_dynamic_shapes: boolean;
    supports_quantization: boolean;
  }
  
  interface ImportOptions {
    name: string;
    description: string;
    model_type: 'TextGeneration' | 'TextClassification' | 'TokenClassification' | 'Seq2Seq' | 'Unknown';
    tags: string[];
    validate_thoroughly: boolean;
    auto_generate_metadata: boolean;
  }
  
  interface ImportResult {
    success: boolean;
    model_id?: string;
    validation_result: ValidationResult;
    error?: string;
  }
  
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
    validation_result: ValidationResult;
  }
  
  // State
  let step: 'select' | 'validate' | 'configure' | 'import' | 'complete' = 'select';
  let selectedFile: string = '';
  let validationResult: ValidationResult | null = null;
  let isValidating = false;
  let isImporting = false;
  let error = '';
  let customModels: CustomModel[] = [];
  
  let importOptions: ImportOptions = {
    name: '',
    description: '',
    model_type: 'Unknown',
    tags: [],
    validate_thoroughly: true,
    auto_generate_metadata: true,
  };
  
  let newTag = '';
  let importResult: ImportResult | null = null;
  
  // Load custom models on mount
  onMount(async () => {
    await loadCustomModels();
  });
  
  // Select file
  async function selectFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'ONNX Model',
          extensions: ['onnx']
        }],
        title: 'Select ONNX Model'
      });
      
      if (selected && typeof selected === 'string') {
        selectedFile = selected;
        // Extract filename for default name
        const filename = selected.split(/[\\/]/).pop() || '';
        importOptions.name = filename.replace('.onnx', '');
        
        step = 'validate';
        await validateModel();
      }
    } catch (e) {
      error = `Failed to select file: ${e}`;
    }
  }
  
  // Validate model
  async function validateModel() {
    if (!selectedFile) return;
    
    isValidating = true;
    error = '';
    
    try {
      validationResult = await invoke('validate_onnx_model', {
        path: selectedFile
      });
      
      if (validationResult.is_valid) {
        // Auto-detect model type based on structure
        detectModelType();
      }
    } catch (e) {
      error = `Validation failed: ${e}`;
    } finally {
      isValidating = false;
    }
  }
  
  // Detect model type
  function detectModelType() {
    if (!validationResult?.model_info) return;
    
    const info = validationResult.model_info;
    
    // Check input/output patterns
    if (info.input_names.some(n => n.includes('decoder')) && 
        info.input_names.some(n => n.includes('encoder'))) {
      importOptions.model_type = 'Seq2Seq';
    } else if (info.output_names.some(n => n.includes('logits')) &&
               info.output_shapes[info.output_names[0]]?.length >= 3) {
      importOptions.model_type = 'TextGeneration';
    } else if (info.output_names.length === 1 &&
               info.output_shapes[info.output_names[0]]?.length === 2) {
      importOptions.model_type = 'TextClassification';
    } else if (info.output_shapes[Object.keys(info.output_shapes)[0]]?.length >= 3) {
      importOptions.model_type = 'TokenClassification';
    }
  }
  
  // Add tag
  function addTag() {
    if (newTag.trim() && !importOptions.tags.includes(newTag.trim())) {
      importOptions.tags = [...importOptions.tags, newTag.trim()];
      newTag = '';
    }
  }
  
  // Remove tag
  function removeTag(tag: string) {
    importOptions.tags = importOptions.tags.filter(t => t !== tag);
  }
  
  // Import model
  async function importModel() {
    if (!selectedFile || !importOptions.name) return;
    
    isImporting = true;
    error = '';
    
    try {
      importResult = await invoke('import_custom_model', {
        path: selectedFile,
        options: importOptions
      });
      
      if (importResult.success) {
        step = 'complete';
        await loadCustomModels();
      } else {
        error = importResult.error || 'Import failed';
      }
    } catch (e) {
      error = `Import failed: ${e}`;
    } finally {
      isImporting = false;
    }
  }
  
  // Load custom models
  async function loadCustomModels() {
    try {
      customModels = await invoke('list_custom_models');
    } catch (e) {
      console.error('Failed to load custom models:', e);
    }
  }
  
  // Delete custom model
  async function deleteModel(modelId: string) {
    if (!confirm('Are you sure you want to delete this model?')) return;
    
    try {
      await invoke('delete_custom_model', { modelId });
      await loadCustomModels();
    } catch (e) {
      error = `Failed to delete model: ${e}`;
    }
  }
  
  // Export custom model
  async function exportModel(modelId: string) {
    try {
      const savePath = await open({
        directory: false,
        save: true,
        filters: [{
          name: 'ONNX Model',
          extensions: ['onnx']
        }],
        defaultPath: `${modelId}.onnx`
      });
      
      if (savePath && typeof savePath === 'string') {
        await invoke('export_custom_model', {
          modelId,
          exportPath: savePath
        });
      }
    } catch (e) {
      error = `Failed to export model: ${e}`;
    }
  }
  
  // Reset import
  function resetImport() {
    step = 'select';
    selectedFile = '';
    validationResult = null;
    importResult = null;
    importOptions = {
      name: '',
      description: '',
      model_type: 'Unknown',
      tags: [],
      validate_thoroughly: true,
      auto_generate_metadata: true,
    };
    error = '';
  }
  
  // Format file size
  function formatSize(bytes: number): string {
    const gb = bytes / 1e9;
    return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / 1e6).toFixed(0)} MB`;
  }
  
  // Get severity color
  function getSeverityColor(severity: string): string {
    switch (severity) {
      case 'Error': return '#dc3545';
      case 'Warning': return '#ffc107';
      case 'Info': return '#17a2b8';
      default: return '#6c757d';
    }
  }
</script>

<div class="custom-model-importer">
  <div class="header">
    <h3>Import Custom Model</h3>
    <div class="steps">
      <div class="step" class:active={step === 'select'} class:completed={step !== 'select'}>
        1. Select
      </div>
      <div class="step" class:active={step === 'validate'} class:completed={['configure', 'import', 'complete'].includes(step)}>
        2. Validate
      </div>
      <div class="step" class:active={step === 'configure'} class:completed={['import', 'complete'].includes(step)}>
        3. Configure
      </div>
      <div class="step" class:active={step === 'import'} class:completed={step === 'complete'}>
        4. Import
      </div>
    </div>
  </div>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  {#if step === 'select'}
    <div class="step-content">
      <div class="file-selector">
        <p>Select an ONNX model file to import:</p>
        <button class="select-button" on:click={selectFile}>
          Choose ONNX File
        </button>
        <div class="requirements">
          <h4>Requirements:</h4>
          <ul>
            <li>File must be in ONNX format (.onnx)</li>
            <li>Model should be compatible with ONNX Runtime</li>
            <li>Recommended size: under 5GB</li>
          </ul>
        </div>
      </div>
    </div>
  {/if}
  
  {#if step === 'validate' && validationResult}
    <div class="step-content">
      <div class="validation-results">
        <div class="validation-header">
          <h4>Validation Results</h4>
          {#if validationResult.is_valid}
            <span class="valid-badge">✓ Valid</span>
          {:else}
            <span class="invalid-badge">✗ Invalid</span>
          {/if}
        </div>
        
        {#if validationResult.model_info}
          <div class="model-info">
            <h5>Model Information</h5>
            <div class="info-grid">
              <div class="info-item">
                <span class="label">Size:</span>
                <span>{formatSize(validationResult.model_info.model_size_bytes)}</span>
              </div>
              <div class="info-item">
                <span class="label">Parameters:</span>
                <span>~{(validationResult.model_info.estimated_parameters / 1e6).toFixed(1)}M</span>
              </div>
              <div class="info-item">
                <span class="label">Inputs:</span>
                <span>{validationResult.model_info.input_names.join(', ')}</span>
              </div>
              <div class="info-item">
                <span class="label">Outputs:</span>
                <span>{validationResult.model_info.output_names.join(', ')}</span>
              </div>
            </div>
          </div>
        {/if}
        
        {#if validationResult.issues.length > 0}
          <div class="issues">
            <h5>Issues</h5>
            {#each validationResult.issues as issue}
              <div class="issue" style="border-left-color: {getSeverityColor(issue.severity)}">
                <div class="issue-header">
                  <span class="severity" style="color: {getSeverityColor(issue.severity)}">
                    {issue.severity}
                  </span>
                  <span class="category">{issue.category}</span>
                </div>
                <p>{issue.message}</p>
                {#if issue.details}
                  <p class="details">{issue.details}</p>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
        
        {#if validationResult.warnings.length > 0}
          <div class="warnings">
            <h5>Warnings</h5>
            <ul>
              {#each validationResult.warnings as warning}
                <li>{warning}</li>
              {/each}
            </ul>
          </div>
        {/if}
        
        {#if validationResult.is_valid}
          <button class="next-button" on:click={() => step = 'configure'}>
            Continue to Configuration
          </button>
        {:else}
          <button class="back-button" on:click={resetImport}>
            Select Different Model
          </button>
        {/if}
      </div>
    </div>
  {/if}
  
  {#if step === 'configure'}
    <div class="step-content">
      <div class="configuration">
        <h4>Model Configuration</h4>
        
        <div class="form-group">
          <label for="model-name">Model Name *</label>
          <input 
            id="model-name"
            type="text"
            bind:value={importOptions.name}
            placeholder="Enter a descriptive name"
            required
          />
        </div>
        
        <div class="form-group">
          <label for="description">Description</label>
          <textarea 
            id="description"
            bind:value={importOptions.description}
            placeholder="Describe what this model does..."
            rows="3"
          />
        </div>
        
        <div class="form-group">
          <label for="model-type">Model Type</label>
          <select id="model-type" bind:value={importOptions.model_type}>
            <option value="TextGeneration">Text Generation</option>
            <option value="TextClassification">Text Classification</option>
            <option value="TokenClassification">Token Classification</option>
            <option value="Seq2Seq">Sequence to Sequence</option>
            <option value="Unknown">Unknown/Other</option>
          </select>
        </div>
        
        <div class="form-group">
          <label>Tags</label>
          <div class="tag-input">
            <input 
              type="text"
              bind:value={newTag}
              placeholder="Add tags..."
              on:keydown={(e) => e.key === 'Enter' && addTag()}
            />
            <button on:click={addTag}>Add</button>
          </div>
          <div class="tags">
            {#each importOptions.tags as tag}
              <span class="tag">
                {tag}
                <button on:click={() => removeTag(tag)}>×</button>
              </span>
            {/each}
          </div>
        </div>
        
        <div class="form-group">
          <label>
            <input 
              type="checkbox"
              bind:checked={importOptions.validate_thoroughly}
            />
            Perform thorough validation
          </label>
        </div>
        
        <div class="form-group">
          <label>
            <input 
              type="checkbox"
              bind:checked={importOptions.auto_generate_metadata}
            />
            Auto-generate metadata
          </label>
        </div>
        
        <div class="actions">
          <button class="back-button" on:click={() => step = 'validate'}>
            Back
          </button>
          <button 
            class="import-button"
            on:click={importModel}
            disabled={!importOptions.name || isImporting}
          >
            {isImporting ? 'Importing...' : 'Import Model'}
          </button>
        </div>
      </div>
    </div>
  {/if}
  
  {#if step === 'complete' && importResult}
    <div class="step-content">
      <div class="import-complete">
        <div class="success-icon">✓</div>
        <h4>Model Imported Successfully!</h4>
        <p>Your custom model has been imported and is ready to use.</p>
        <div class="model-id">Model ID: {importResult.model_id}</div>
        <button class="new-import-button" on:click={resetImport}>
          Import Another Model
        </button>
      </div>
    </div>
  {/if}
  
  {#if customModels.length > 0}
    <div class="custom-models-section">
      <h4>Your Custom Models</h4>
      <div class="models-grid">
        {#each customModels as model}
          <div class="custom-model-card">
            <div class="model-header">
              <h5>{model.name}</h5>
              <span class="model-type">{model.model_type}</span>
            </div>
            {#if model.description}
              <p class="description">{model.description}</p>
            {/if}
            <div class="model-meta">
              <span>Imported: {new Date(model.import_date).toLocaleDateString()}</span>
              <span>Used: {model.usage_count} times</span>
            </div>
            {#if model.tags.length > 0}
              <div class="tags">
                {#each model.tags as tag}
                  <span class="tag">{tag}</span>
                {/each}
              </div>
            {/if}
            <div class="model-actions">
              <button on:click={() => exportModel(model.id)}>Export</button>
              <button on:click={() => deleteModel(model.id)} class="delete">Delete</button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .custom-model-importer {
    padding: 20px;
    max-width: 1000px;
  }
  
  .header {
    margin-bottom: 30px;
  }
  
  .header h3 {
    margin: 0 0 20px 0;
  }
  
  .steps {
    display: flex;
    justify-content: space-between;
    position: relative;
  }
  
  .steps::before {
    content: '';
    position: absolute;
    top: 15px;
    left: 0;
    right: 0;
    height: 2px;
    background: #dee2e6;
    z-index: 0;
  }
  
  .step {
    background: white;
    padding: 10px 20px;
    border-radius: 20px;
    border: 2px solid #dee2e6;
    position: relative;
    z-index: 1;
    color: #6c757d;
  }
  
  .step.active {
    border-color: #007bff;
    color: #007bff;
    font-weight: 500;
  }
  
  .step.completed {
    background: #28a745;
    border-color: #28a745;
    color: white;
  }
  
  .error {
    background: #f8d7da;
    color: #721c24;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .step-content {
    background: #f8f9fa;
    padding: 30px;
    border-radius: 8px;
    margin-bottom: 30px;
  }
  
  .file-selector {
    text-align: center;
  }
  
  .select-button {
    padding: 15px 30px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 1.1em;
    cursor: pointer;
    margin: 20px 0;
  }
  
  .select-button:hover {
    background: #0056b3;
  }
  
  .requirements {
    text-align: left;
    max-width: 400px;
    margin: 30px auto 0;
  }
  
  .requirements h4 {
    margin: 0 0 10px 0;
  }
  
  .requirements ul {
    margin: 0;
    padding-left: 20px;
  }
  
  .validation-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }
  
  .validation-header h4 {
    margin: 0;
  }
  
  .valid-badge {
    color: #28a745;
    font-weight: bold;
    font-size: 1.2em;
  }
  
  .invalid-badge {
    color: #dc3545;
    font-weight: bold;
    font-size: 1.2em;
  }
  
  .model-info,
  .issues,
  .warnings {
    margin-bottom: 20px;
  }
  
  .model-info h5,
  .issues h5,
  .warnings h5 {
    margin: 0 0 10px 0;
  }
  
  .info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }
  
  .info-item {
    display: flex;
    justify-content: space-between;
    padding: 5px 0;
  }
  
  .info-item .label {
    font-weight: 500;
    color: #666;
  }
  
  .issue {
    background: white;
    padding: 15px;
    border-radius: 4px;
    border-left: 4px solid;
    margin-bottom: 10px;
  }
  
  .issue-header {
    display: flex;
    gap: 10px;
    margin-bottom: 5px;
  }
  
  .severity {
    font-weight: bold;
  }
  
  .category {
    color: #666;
    font-size: 0.9em;
  }
  
  .issue p {
    margin: 5px 0;
  }
  
  .details {
    font-size: 0.9em;
    color: #666;
  }
  
  .warnings ul {
    margin: 0;
    padding-left: 20px;
  }
  
  .warnings li {
    color: #856404;
  }
  
  .next-button,
  .back-button,
  .import-button,
  .new-import-button {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    font-size: 1em;
    cursor: pointer;
    margin-top: 20px;
  }
  
  .next-button,
  .import-button {
    background: #007bff;
    color: white;
  }
  
  .next-button:hover,
  .import-button:hover:not(:disabled) {
    background: #0056b3;
  }
  
  .back-button {
    background: #6c757d;
    color: white;
    margin-right: 10px;
  }
  
  .back-button:hover {
    background: #5a6268;
  }
  
  .import-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .configuration h4 {
    margin: 0 0 20px 0;
  }
  
  .form-group {
    margin-bottom: 20px;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: 500;
  }
  
  .form-group input[type="text"],
  .form-group textarea,
  .form-group select {
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
  
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  
  .tag {
    background: #e9ecef;
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 0.9em;
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
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 30px;
  }
  
  .import-complete {
    text-align: center;
    padding: 40px;
  }
  
  .success-icon {
    font-size: 4em;
    color: #28a745;
    margin-bottom: 20px;
  }
  
  .import-complete h4 {
    margin: 0 0 10px 0;
  }
  
  .model-id {
    background: #e9ecef;
    padding: 10px 20px;
    border-radius: 4px;
    font-family: monospace;
    margin: 20px 0;
  }
  
  .new-import-button {
    background: #28a745;
    color: white;
  }
  
  .new-import-button:hover {
    background: #218838;
  }
  
  .custom-models-section {
    margin-top: 40px;
  }
  
  .custom-models-section h4 {
    margin: 0 0 20px 0;
  }
  
  .models-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 20px;
  }
  
  .custom-model-card {
    background: white;
    border: 1px solid #dee2e6;
    border-radius: 8px;
    padding: 20px;
  }
  
  .model-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }
  
  .model-header h5 {
    margin: 0;
  }
  
  .model-type {
    background: #007bff;
    color: white;
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 0.8em;
  }
  
  .description {
    color: #666;
    font-size: 0.9em;
    margin: 10px 0;
  }
  
  .model-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.85em;
    color: #666;
    margin: 10px 0;
  }
  
  .model-actions {
    display: flex;
    gap: 10px;
    margin-top: 15px;
  }
  
  .model-actions button {
    flex: 1;
    padding: 6px 12px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9em;
  }
  
  .model-actions button:first-child {
    background: #17a2b8;
    color: white;
  }
  
  .model-actions button.delete {
    background: #dc3545;
    color: white;
  }
  
  @media (prefers-color-scheme: dark) {
    .step {
      background: #2a2a2a;
      border-color: #4a4a4a;
    }
    
    .step.completed {
      background: #28a745;
    }
    
    .step-content {
      background: #2a2a2a;
    }
    
    .issue {
      background: #1a1a1a;
    }
    
    .form-group input,
    .form-group textarea,
    .form-group select {
      background: #1a1a1a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .tag {
      background: #3a3a3a;
      color: #e0e0e0;
    }
    
    .model-id {
      background: #3a3a3a;
      color: #e0e0e0;
    }
    
    .custom-model-card {
      background: #2a2a2a;
      border-color: #4a4a4a;
      color: #e0e0e0;
    }
    
    .description,
    .model-meta {
      color: #999;
    }
  }
</style>