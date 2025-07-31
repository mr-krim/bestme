<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  // Using Electron API instead of Tauri
  
  // Import shared types 
  import type { Conversation, ChatMessage } from '../types'; 

  // Define local types based on imported ones or primitives
  type ChatModel = string;
  type SendMessageResponse = ChatMessage; 

  // Props
  export let chatMessages: ChatMessage[] = []; 
  export let isProcessing = false;
  
  // Local state with types
  let messageInput = '';
  let aiModels: ChatModel[] = [];
  let selectedModel: ChatModel | null = null; 
  let conversations: Conversation[] = [];
  let activeConversationId: string | null = null; 
  
  onMount(async () => {
    try {
      aiModels = await invoke<ChatModel[]>('chat.get_available_models');
      
      if (aiModels.length > 0) {
        selectedModel = aiModels[0];
      }
      
      conversations = await invoke<Conversation[]>('chat.get_conversations');
      
      if (conversations.length === 0) {
        if (!selectedModel) {
          console.error("Cannot create conversation: No AI model selected or available.");
          return; // Prevent invoke call if no model
        }
        const newConversation = await invoke<Conversation>('chat.create_conversation', {
          title: 'New Conversation',
          model: selectedModel
        });
        
        conversations = [newConversation];
        activeConversationId = newConversation.id; // Access id after typing
      } else {
        activeConversationId = conversations[0].id;
        
        if (activeConversationId) { // Ensure we have an ID before fetching
          chatMessages = await invoke<ChatMessage[]>('chat.get_messages', {
            conversationId: activeConversationId
          });
        }
      }
    } catch (error) {
      console.error('Error initializing chat view:', error);
    }
  });
  
  async function sendMessage() {
    if (!messageInput.trim() || isProcessing || !activeConversationId || !selectedModel) return;
    
    const userMessage: ChatMessage = {
      id: generateId(),
      text: messageInput,
      sender: 'user',
      timestamp: Date.now() // Use number timestamp
    };
    
    chatMessages = [...chatMessages, userMessage];
    
    const inputText = messageInput;
    messageInput = '';
    isProcessing = true;
    
    try {
      const response = await invoke<SendMessageResponse>('chat.send_message', {
        conversationId: activeConversationId,
        message: inputText,
        model: selectedModel
      });
      
      if (response && response.text) { 
        chatMessages = [...chatMessages, {
          id: response.id || generateId(),
          text: response.text,
          sender: 'ai', 
          timestamp: response.timestamp || Date.now() // Use number timestamp
        }];
      } else {
        throw new Error("Invalid AI response received");
      }
    } catch (error) {
      console.error('Error sending message:', error);
      
      chatMessages = [...chatMessages, {
        id: generateId(),
        text: `Sorry, there was an error processing your message. (${error instanceof Error ? error.message : error})`,
        sender: 'system',
        timestamp: Date.now() // Use number timestamp
      }];
    } finally {
      isProcessing = false;
    }
  }
  
  // Type event
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  }
  
  async function createNewConversation() {
    if (!selectedModel) {
      console.error("Cannot create conversation: No AI model selected.");
      // Maybe add a user notification (toast?)
      return;
    }
    try {
      const newConversation = await invoke<Conversation>('chat.create_conversation', {
        title: 'New Conversation',
        model: selectedModel
      });
      
      conversations = [newConversation, ...conversations];
      activeConversationId = newConversation.id; // Access id after typing
      chatMessages = [];
    } catch (error) {
      console.error('Error creating new conversation:', error);
    }
  }
  
  // Type parameter
  async function switchConversation(conversationId: string) {
    if (conversationId === activeConversationId) return;
    
    try {
      activeConversationId = conversationId;
      
      chatMessages = await invoke<ChatMessage[]>('chat.get_messages', {
        conversationId
      });
    } catch (error) {
      console.error('Error switching conversation:', error);
      chatMessages = [];
    }
  }
  
  // Type event
  function updateModel(event: Event) {
    const target = event.target as HTMLSelectElement;
    selectedModel = target.value;
  }
  
  function generateId(): string { // Add return type
    return `temp-${Date.now()}-${Math.floor(Math.random() * 1000)}`;
  }
  
  // Scroll to bottom when messages change
  $: if (chatMessages.length > 0) {
    setTimeout(() => {
      const chatContainer = document.querySelector('.messages-container');
      if (chatContainer) {
        chatContainer.scrollTop = chatContainer.scrollHeight;
      }
    }, 0);
  }
</script>

<div class="chat-view">
  <div class="chat-header">
    <div class="model-selector">
      <label for="ai-model">AI Model:</label>
      <select id="ai-model" value={selectedModel} on:change={updateModel}>
        {#each aiModels as model}
          <option value={model}>{model}</option>
        {/each}
      </select>
    </div>
    
    <button class="new-chat-button" on:click={createNewConversation}>
      New Conversation
    </button>
  </div>
  
  <div class="messages-container">
    {#if chatMessages.length === 0}
      <div class="empty-chat">
        <div class="empty-state">
          <h3>Start a New Conversation</h3>
          <p>Type a message below to start chatting with the AI assistant.</p>
        </div>
      </div>
    {:else}
      {#each chatMessages as message (message.id)}
        <div class="message {message.sender}">
          <div class="message-content">
            {message.text}
          </div>
          <div class="message-timestamp">
            {formatTime(message.timestamp)}
          </div>
        </div>
      {/each}
      
      {#if isProcessing}
        <div class="message ai processing">
          <div class="typing-indicator">
            <span></span>
            <span></span>
            <span></span>
          </div>
        </div>
      {/if}
    {/if}
  </div>
  
  <div class="message-input-container">
    <textarea 
      class="message-input" 
      placeholder="Type a message..." 
      bind:value={messageInput}
      on:keydown={handleKeydown}
    ></textarea>
    <button 
      class="send-button" 
      on:click={sendMessage}
      disabled={!messageInput.trim() || isProcessing}
    >
      Send
    </button>
  </div>
</div>

<style>
  .chat-view {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  
  .chat-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background-color: var(--header-bg, #f5f5f5);
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }
  
  .model-selector {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .model-selector label {
    font-size: 0.9rem;
    color: var(--text-secondary, #666);
  }
  
  .model-selector select {
    padding: 6px 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    background-color: var(--dropdown-bg, white);
  }
  
  .new-chat-button {
    padding: 6px 12px;
    background-color: var(--button-bg, #f0f0f0);
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .new-chat-button:hover {
    background-color: var(--button-hover-bg, #e5e5e5);
  }
  
  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
  }
  
  .empty-chat {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    color: var(--text-tertiary, #999);
  }
  
  .empty-state {
    text-align: center;
    max-width: 400px;
  }
  
  .empty-state h3 {
    margin-bottom: 8px;
    color: var(--text-primary, #333);
  }
  
  .message {
    max-width: 80%;
    margin-bottom: 16px;
    padding: 12px 16px;
    border-radius: 8px;
    position: relative;
  }
  
  .message.user {
    align-self: flex-end;
    background-color: var(--user-message-bg, #e3f2fd);
    color: var(--user-message-color, #0d47a1);
    border-bottom-right-radius: 0;
  }
  
  .message.ai {
    align-self: flex-start;
    background-color: var(--ai-message-bg, #f5f5f5);
    color: var(--ai-message-color, #333);
    border-bottom-left-radius: 0;
  }
  
  .message.system {
    align-self: center;
    background-color: var(--system-message-bg, #fff3e0);
    color: var(--system-message-color, #e65100);
    max-width: 90%;
  }
  
  .message-content {
    white-space: pre-wrap;
    word-break: break-word;
  }
  
  .message-timestamp {
    font-size: 0.7rem;
    color: var(--text-tertiary, #999);
    text-align: right;
    margin-top: 4px;
  }
  
  .message.processing {
    padding: 8px 16px;
  }
  
  .typing-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 24px;
  }
  
  .typing-indicator span {
    height: 8px;
    width: 8px;
    border-radius: 50%;
    background-color: var(--typing-indicator-color, #999);
    margin: 0 2px;
    opacity: 0.7;
    animation: typing 1s infinite ease-in-out;
  }
  
  .typing-indicator span:nth-child(1) {
    animation-delay: 0s;
  }
  
  .typing-indicator span:nth-child(2) {
    animation-delay: 0.15s;
  }
  
  .typing-indicator span:nth-child(3) {
    animation-delay: 0.3s;
  }
  
  @keyframes typing {
    0%, 100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-5px);
    }
  }
  
  .message-input-container {
    display: flex;
    margin: 16px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 8px;
    overflow: hidden;
    background-color: var(--input-container-bg, white);
  }
  
  .message-input {
    flex: 1;
    padding: 12px;
    border: none;
    resize: none;
    min-height: 24px;
    max-height: 120px;
    font-family: inherit;
    background-color: transparent;
  }
  
  .message-input:focus {
    outline: none;
  }
  
  .send-button {
    padding: 0 16px;
    background-color: var(--primary-color, #2196f3);
    color: white;
    border: none;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .send-button:hover:not(:disabled) {
    background-color: var(--primary-color-hover, #1976d2);
  }
  
  .send-button:disabled {
    background-color: var(--disabled-color, #bdbdbd);
    cursor: not-allowed;
  }
  
  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    .chat-view {
      --header-bg: #252525;
      --border-color: #444;
      --button-bg: #333;
      --button-hover-bg: #444;
      --dropdown-bg: #333;
      --text-primary: #eee;
      --text-secondary: #bbb;
      --text-tertiary: #999;
      --user-message-bg: #0d47a1;
      --user-message-color: #fff;
      --ai-message-bg: #333;
      --ai-message-color: #eee;
      --system-message-bg: #5d4037;
      --system-message-color: #ffe0b2;
      --typing-indicator-color: #bbb;
      --primary-color: #1976d2;
      --primary-color-hover: #1565c0;
      --disabled-color: #555;
      --input-container-bg: #333;
    }
    
    .model-selector select {
      color: #eee;
    }
    
    .message-input {
      color: #eee;
    }
  }
</style>

<script context="module" lang="ts">
  // No separate import needed here if type is defined/imported in main script scope
  
  // Reference ChatMessage timestamp type directly (assuming it's available from main script scope)
  function formatTime(timestamp: ChatMessage['timestamp']): string { 
    if (!timestamp) return '';
    const date = new Date(timestamp);
    if (isNaN(date.getTime())) return ''; 
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
</script> 
