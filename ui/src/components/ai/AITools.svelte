<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import TextSummarizer from './TextSummarizer.svelte';
  import TextTranslator from './TextTranslator.svelte';
  
  const dispatch = createEventDispatcher();
  
  export let initialText: string = '';
  export let activeTab: 'summarize' | 'translate' = 'summarize';
  
  let sharedText = initialText;
  
  // Handle events from child components
  function handleSummarized(event) {
    dispatch('processed', {
      type: 'summarized',
      ...event.detail
    });
  }
  
  function handleTranslated(event) {
    dispatch('processed', {
      type: 'translated',
      ...event.detail
    });
  }
  
  function handleCopied(event) {
    dispatch('copied', event.detail);
  }
  
  // Update shared text when initial text changes
  $: sharedText = initialText;
</script>

<div class="ai-tools">
  <div class="tabs">
    <button
      class="tab"
      class:active={activeTab === 'summarize'}
      on:click={() => activeTab = 'summarize'}
    >
      📝 Summarize
    </button>
    <button
      class="tab"
      class:active={activeTab === 'translate'}
      on:click={() => activeTab = 'translate'}
    >
      🌐 Translate
    </button>
  </div>
  
  <div class="tab-content">
    {#if activeTab === 'summarize'}
      <TextSummarizer 
        bind:text={sharedText}
        on:summarized={handleSummarized}
        on:copied={handleCopied}
      />
    {:else if activeTab === 'translate'}
      <TextTranslator
        bind:text={sharedText}
        on:translated={handleTranslated}
        on:copied={handleCopied}
      />
    {/if}
  </div>
</div>

<style>
  .ai-tools {
    max-width: 1200px;
    margin: 0 auto;
  }
  
  .tabs {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
    border-bottom: 2px solid #e9ecef;
    padding: 0 20px;
  }
  
  .tab {
    padding: 12px 24px;
    background: none;
    border: none;
    border-bottom: 3px solid transparent;
    color: #666;
    font-size: 1.1em;
    cursor: pointer;
    transition: all 0.3s;
  }
  
  .tab:hover {
    color: #333;
  }
  
  .tab.active {
    color: #007bff;
    border-bottom-color: #007bff;
  }
  
  .tab-content {
    animation: fadeIn 0.3s ease-in;
  }
  
  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  @media (prefers-color-scheme: dark) {
    .tabs {
      border-bottom-color: #3a3a3a;
    }
    
    .tab {
      color: #999;
    }
    
    .tab:hover {
      color: #e0e0e0;
    }
    
    .tab.active {
      color: #4dabf7;
      border-bottom-color: #4dabf7;
    }
  }
</style>