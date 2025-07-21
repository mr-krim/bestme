<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy, tick } from 'svelte';
  import { slide } from 'svelte/transition';

  // Type matching backend Chat structs
  type ChatMessage = {
    sender: string; // "user" or "ai" or "system"
    text: string;
    timestamp_ms: number;
  };

  type ChatSession = {
    id: string;
    title: string;
    date?: string; // Make optional as it might not always be needed here
  } | null;

  // Props
  export let messages: ChatMessage[] = [];
  export let chatSession: ChatSession = null;
  export let isProcessing: boolean = false; // Prop to indicate AI processing

  const dispatch = createEventDispatcher();

  let chatInput: string = '';
  let messageListElement: HTMLDivElement;

  // Scroll to bottom when messages change
  $: if (messageListElement && messages.length > 0) {
    scrollToBottom();
  }
  
  async function scrollToBottom() {
      await tick(); // Wait for DOM update
      if (messageListElement) {
          messageListElement.scrollTop = messageListElement.scrollHeight;
      }
  }

  onMount(() => {
      scrollToBottom(); // Scroll on initial load
  });

  function handleChatSend() {
    if (!chatInput.trim()) return;
    dispatch('sendMessage', { sessionId: chatSession?.id, message: chatInput.trim() });
    chatInput = ''; // Clear input after sending
  }

  function handleDeleteChatClick() {
    if (!chatSession) return;
    // Optionally add a confirmation dialog here
    if (confirm(`Are you sure you want to delete the chat "${chatSession.title}"?`)) {
        dispatch('deleteRequest', { id: chatSession.id });
    }
  }

  // Helper function to format timestamp
  function formatTime(timestampMs: number): string {
    const date = new Date(timestampMs);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
</script>

<div class="chat-view">
  <div class="chat-header">
    <h2>Chat {chatSession ? `- ${chatSession.title}` : '(New Chat)'}</h2>
    {#if chatSession} <!-- Show delete button only if a chat is selected -->
      <button 
        class="delete-button chat-delete-button" 
        on:click={handleDeleteChatClick} 
        title="Delete this chat session"
      >
        🗑️ Delete Chat
      </button>
    {/if}
  </div>

  <div class="chat-area" bind:this={messageListElement}>
    <div class="message-list">
      {#if messages.length === 0}
        <p class="empty-chat-message">Send a message to start the chat.</p>
      {:else}
        {#each messages as message (message.timestamp_ms + message.sender + message.text)} 
          <div class="message {message.sender}" transition:slide={{ duration: 300 }}>
            <p>{message.text}</p>
            <span class="timestamp">{formatTime(message.timestamp_ms)}</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <div class="message-input-area">
    {#if isProcessing}
      <div class="processing-indicator">
        <span>AI is thinking...</span> 
        <!-- Add a spinner icon here later if desired -->
      </div>
    {/if}
    <div class="message-input">
      <textarea 
        placeholder="Type your message..." 
        bind:value={chatInput}
        on:keydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleChatSend(); } }}
        disabled={isProcessing}
      ></textarea>
      <button on:click={handleChatSend} disabled={!chatInput.trim() || isProcessing}>Send</button>
    </div>
  </div>
</div>

<style>
  .chat-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--content-bg);
  }

  .chat-header {
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .chat-header h2 {
    margin: 0;
    font-size: 1.1rem;
    color: var(--text-primary);
  }

  .delete-button {
     background: none;
     border: 1px solid var(--danger-color, #f44336);
     color: var(--danger-color, #f44336);
     padding: 4px 8px;
     border-radius: 4px;
     cursor: pointer;
     font-size: 0.8rem;
     margin-left: 1rem; /* Add some space */
     transition: background-color 0.2s, color 0.2s;
   }

   .delete-button:hover {
     background-color: var(--danger-color-hover-bg, #ffebee); /* Light red background on hover */
     color: var(--danger-color-hover, #c62828); /* Darker red text on hover */
   }

   :global(body.dark-theme) .delete-button {
     border-color: var(--danger-color-dark, #e57373);
     color: var(--danger-color-dark, #e57373);
   }

   :global(body.dark-theme) .delete-button:hover {
     background-color: var(--danger-color-hover-bg-dark, rgba(229, 115, 115, 0.1)); /* More subtle background */
     color: var(--danger-color-hover-dark, #ef9a9a); /* Lighter red text */
   }

  .chat-area {
    flex-grow: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
  }

  .message-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    width: 100%;
  }

  .message {
    padding: 0.5rem 0.75rem;
    border-radius: 8px;
    max-width: 75%;
    word-wrap: break-word;
    position: relative; /* For timestamp positioning */
  }

  .message.user {
    background-color: var(--user-message-bg);
    color: var(--user-message-color);
    align-self: flex-end;
    border-bottom-right-radius: 2px; /* Slight visual cue */
  }

  .message.ai {
    background-color: var(--ai-message-bg);
    color: var(--ai-message-color);
    align-self: flex-start;
    border-bottom-left-radius: 2px;
  }
  
  .message.system {
      background-color: var(--system-message-bg);
      color: var(--system-message-color);
      align-self: center;
      font-style: italic;
      font-size: 0.9em;
      max-width: 90%;
      text-align: center;
  }

  .message p {
    margin: 0 0 0.25rem 0; /* Space for timestamp */
    white-space: pre-wrap; /* Preserve line breaks */
  }

  .timestamp {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    position: absolute;
    bottom: 4px;
    right: 8px;
    opacity: 0.7;
  }
  
  .message.ai .timestamp {
      left: 8px;
      right: auto;
  }
  
  .message.system .timestamp {
      display: none; /* Hide timestamp for system messages */
  }

  .empty-chat-message {
    text-align: center;
    color: var(--text-tertiary);
    margin-top: 2rem;
  }

  .message-input-area {
    padding: 0.5rem 1rem;
    border-top: 1px solid var(--border-color);
    background-color: var(--panel-bg);
  }

  .processing-indicator {
    text-align: center;
    font-style: italic;
    color: var(--text-secondary);
    font-size: 0.9em;
    padding-bottom: 0.5rem;
    /* TODO: Add spinner styles */
  }

  .message-input {
    display: flex;
    gap: 0.5rem;
  }

  .message-input textarea {
    flex-grow: 1;
    padding: 8px 12px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    resize: none; /* Prevent manual resizing */
    min-height: 40px; /* Minimum height */
    max-height: 120px; /* Maximum height before scrolling */
    line-height: 1.4;
    background-color: var(--input-bg);
    color: var(--text-primary);
  }
  
  .message-input textarea:disabled {
      background-color: var(--inactive-bar);
      cursor: not-allowed;
  }

  .message-input button {
    padding: 8px 15px;
    border: none;
    border-radius: 4px;
    background-color: var(--primary-color);
    color: white;
    cursor: pointer;
    align-self: flex-end; /* Align button to bottom of textarea */
    height: 40px; /* Match min-height */
  }

  .message-input button:disabled {
    background-color: var(--inactive-bar);
    cursor: not-allowed;
  }

  .message-input button:hover:not(:disabled) {
    background-color: var(--primary-color-hover);
  }
</style> 
