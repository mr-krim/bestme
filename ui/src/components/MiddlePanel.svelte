<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  // Props
  export let activePanel: string;
  export let items: any[] = [];
  export let selectedItemId: string | null = null;
  
  // Event dispatcher
  const dispatch = createEventDispatcher();
  
  // Computed property for panel title
  $: panelTitle = getPanelTitle(activePanel);
  
  // Format date for display
  function formatDate(dateString: string): string {
    if (!dateString) return '';
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return 'Invalid Date';
    
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} minutes ago`;
    
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours} hours ago`;
    
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    
    return date.toLocaleDateString();
  }
  
  // Get panel title based on active panel
  function getPanelTitle(panel: string): string {
    switch (panel) {
      case 'transcription': return 'Recent Transcriptions';
      case 'chat': return 'Chat History';
      case 'voice-commands': return 'Voice Commands';
      case 'settings': return 'Settings';
      case 'saved-transcripts': return 'Saved Transcripts';
      case 'devices': return 'Devices';
      case 'help': return 'Help Topics';
      default: return 'Items';
    }
  }
  
  // Handle item click
  function handleItemClick(item: any): void {
    selectedItemId = item.id;
    dispatch('select', { id: item.id });
  }

  // Handle keyboard interaction for list items
  function handleItemKeyDown(event: KeyboardEvent, item: any): void {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleItemClick(item);
    }
  }
</script>

<div class="middle-panel" data-testid="middle-panel">
  <div class="panel-header">
    <h2>{panelTitle}</h2>
  </div>
  
  <div class="panel-content">
    {#if items.length === 0}
      <div class="empty-state">
        <p>No items to display</p>
      </div>
    {:else}
      <div class="item-list">
        {#each items as item (item.id)}
          <div 
            class="item {selectedItemId === item.id ? 'selected' : ''}" 
            role="button"
            tabindex="0"
            on:click={() => handleItemClick(item)}
            on:keydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                handleItemClick(item);
              }
            }}
          >
            <div class="item-title">{item.title}</div>
            {#if item.date}
              <div class="item-date">{formatDate(item.date)}</div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .middle-panel {
    width: 250px;
    background-color: var(--panel-bg);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
  }
  
  .panel-header {
    padding: 16px;
    border-bottom: 1px solid var(--border-color);
  }
  
  .panel-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }
  
  .panel-content {
    flex: 1;
    overflow-y: auto;
  }
  
  .item-list {
    display: flex;
    flex-direction: column;
  }
  
  .item {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color);
    cursor: pointer;
  }
  
  .item:hover {
    background-color: rgba(0, 0, 0, 0.05);
  }
  
  .item.selected {
    background-color: rgba(0, 0, 0, 0.1);
  }
  
  .item-title {
    font-weight: 500;
    margin-bottom: 4px;
  }
  
  .item-date {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary);
  }
</style>
