<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  // Navigation options
  const navItems = [
    { id: 'transcription', label: 'Transcription', icon: '🎤' },
    { id: 'chat', label: 'Chat', icon: '💬' },
    { id: 'voice-commands', label: 'Voice Commands', icon: '🗣️' },
    { id: 'vocabulary', label: 'Vocabulary', icon: '📚' },
    { id: 'settings', label: 'Settings', icon: '⚙️' },
    { id: 'saved-transcripts', label: 'Saved Transcripts', icon: '📜' },
    { id: 'devices', label: 'Devices', icon: '🎧' },
    { id: 'help', label: 'Help & Documentation', icon: '❓' }
  ];
  
  // Active navigation item
  export let activeItem = 'transcription';
  
  // Event dispatcher for navigation changes
  const dispatch = createEventDispatcher();
  
  // Handle navigation item click
  function handleNavClick(itemId: string) {
    if (activeItem !== itemId) {
      activeItem = itemId;
      dispatch('navigate', { panel: itemId });
    }
  }

  // Handle keyboard interaction for navigation items
  function handleNavKeyDown(event: KeyboardEvent, itemId: string) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault(); // Prevent spacebar from scrolling
      handleNavClick(itemId);
    }
  }
</script>

<div class="left-panel">
  <div class="logo-area">
    <h2>BestMe</h2>
  </div>
  
  <nav class="nav-items">
    {#each navItems as item}
      <div 
        class="nav-item {activeItem === item.id ? 'active' : ''}" 
        role="button" 
        tabindex="0"
        data-testid={`nav-${item.id}`}
        on:click={() => handleNavClick(item.id)}
        on:keydown={(event) => handleNavKeyDown(event, item.id)}
      >
        <span class="icon">{item.icon}</span>
        <span class="label">{item.label}</span>
      </div>
    {/each}
  </nav>
  
  <div class="version-info">
    <span>v0.1.0</span>
  </div>
</div>

<style>
  .left-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--panel-bg);
    border-right: 1px solid var(--border-color);
    width: 200px;
    box-sizing: border-box;
  }
  
  .logo-area {
    padding: 20px 16px;
    border-bottom: 1px solid var(--border-color);
  }
  
  .logo-area h2 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }
  
  .nav-items {
    flex: 1;
    overflow-y: auto;
    padding: 16px 0;
  }
  
  .nav-item {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    cursor: pointer;
    border-radius: 4px;
    margin: 0 8px 4px 8px;
    transition: background-color 0.2s;
  }
  
  .nav-item:hover {
    background-color: rgba(0, 0, 0, 0.05);
  }
  
  .nav-item.active {
    background-color: rgba(0, 0, 0, 0.1);
    font-weight: 500;
  }
  
  .icon {
    margin-right: 12px;
    font-size: 1.2rem;
  }
  
  .version-info {
    padding: 16px;
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-align: center;
    border-top: 1px solid var(--border-color);
  }
  
  /* Dark mode support is handled by the CSS variables */
</style> 
