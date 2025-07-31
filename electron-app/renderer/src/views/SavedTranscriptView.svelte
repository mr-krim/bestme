<script lang="ts">
  import type { SavedTranscriptListItem } from '../types';
  import { createEventDispatcher } from 'svelte';

  export let transcriptItem: SavedTranscriptListItem | null = null;
  export let content: string | null = null;

  const dispatch = createEventDispatcher();

  function handleDelete() {
    if (transcriptItem) {
      dispatch('deleteRequest', { id: transcriptItem.id });
    }
  }

  // Helper to format date, you might want to move this to a utility file later
  function formatDate(dateString: string | undefined): string {
    if (!dateString) return 'N/A';
    try {
      return new Date(dateString).toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch (e) {
      return 'Invalid Date';
    }
  }
</script>

<div class="saved-transcript-view">
  {#if transcriptItem}
    <div class="header">
      <h1>{transcriptItem.title}</h1>
      <p class="date">Saved: {formatDate(transcriptItem.date)}</p>
    </div>
    
    <div class="actions">
      <button class="delete-button" on:click={handleDelete} title="Delete this transcript">
        🗑️ Delete
      </button>
      <!-- Add other actions like Edit, Export, etc. later -->
    </div>

    <div class="content-area">
      {#if content === null}
        <p>Loading content...</p>
      {:else if content === "Error: Could not load content."}
        <p class="error-message">{content}</p>
      {:else if content}
        <textarea readonly value={content}></textarea>
      {:else}
        <p>No content available for this transcript.</p>
      {/if}
    </div>
  {:else}
    <div class="empty-state">
      <p>Select a transcript from the list to view its details.</p>
    </div>
  {/if}
</div>

<style>
  .saved-transcript-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 1rem;
    box-sizing: border-box;
    overflow-y: auto; /* Allow scrolling for long content */
  }

  .header {
    margin-bottom: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .header h1 {
    font-size: 1.8rem;
    margin: 0 0 0.25rem 0;
    color: var(--text-primary);
  }

  .date {
    font-size: 0.9rem;
    color: var(--text-secondary);
    margin: 0;
  }

  .actions {
    margin-bottom: 1rem;
    display: flex;
    gap: 0.5rem;
  }

  .actions button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
    background-color: var(--button-bg);
    color: var(--text-primary);
  }

  .actions button:hover {
    background-color: var(--button-hover-bg);
  }
  
  .delete-button {
    background-color: var(--error-color, #f44336); /* Fallback error color */
    color: white;
  }
  .delete-button:hover {
    background-color: var(--error-color-hover, #d32f2f); /* Fallback hover */
  }


  .content-area {
    flex-grow: 1;
    display: flex; /* For textarea to fill height */
    flex-direction: column;
  }

  .content-area textarea {
    width: 100%;
    flex-grow: 1; /* Make textarea fill available space */
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background-color: var(--input-bg);
    color: var(--text-primary);
    font-family: inherit;
    font-size: 1rem;
    resize: none; /* Prevent manual resizing if desired */
    box-sizing: border-box;
  }
  
  .empty-state, .loading-state {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    text-align: center;
    color: var(--text-secondary);
  }

  .error-message {
    color: var(--error-color, #f44336);
    font-weight: bold;
  }
</style> 
