<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  // Import shared types
  import type { PanelItem, ChatMessage, SavedTranscriptListItem, SavedTranscriptContent } from '../types'; // Added SavedTranscriptListItem and SavedTranscriptContent

  // Import child components from correct directory
  import TranscriptionView from '../views/TranscriptionView.svelte';
  import ChatView from '../views/ChatView.svelte'; // Import ChatView
  import SettingsView from '../views/SettingsView.svelte';
  import VocabularyView from '../views/VocabularyView.svelte';
  // Comment out missing components
  // import VoiceCommandView from './VoiceCommandView.svelte'; 
  import SavedTranscriptView from '../views/SavedTranscriptView.svelte'; // Corrected path and uncommented

  // Remove local type definitions - use imported ones
  /*
  type PanelItemBase = { id: string; title: string };
  type SessionItem = PanelItemBase & { date: string };
  type CommandItem = PanelItemBase;
  type PanelItem = SessionItem | CommandItem;

  type ChatMessage = {
    sender: string; 
    text: string;
    timestamp_ms: number;
  };
  */

  // Props from App.svelte
  export let activePanel: string;
  export let transcriptionText: string = '';
  export let currentChatMessages: ChatMessage[] = []; // Use imported ChatMessage
  export let isProcessingChat: boolean = false;
  export let selectedItem: PanelItem | null = null; // Added selectedItem prop
  export let selectedTranscriptContent: string | null = null; // Added selectedTranscriptContent prop

  const dispatch = createEventDispatcher();

  // Computed property for type assertion
  $: typedSelectedItem = selectedItem as SavedTranscriptListItem | null;

  // Define event handlers as constants
  const handleSaveTranscript = (e: CustomEvent<{ title: string; content: string }>) => {
    dispatch('saveTranscriptRequest', e.detail);
  };

  const handleSendChat = (e: CustomEvent<{ sessionId: string | null; message: string }>) => {
    dispatch('sendChatMessage', e.detail);
  };

  const handleDeleteChat = (e: CustomEvent<{ id: string }>) => {
    dispatch('deleteChatRequest', e.detail);
  };

  const handleDeleteTranscript = (e: CustomEvent<{ id: string }>) => {
    dispatch('deleteTranscriptRequest', e.detail);
  };

  // Remove chat-specific variables and functions
  // let chatInput: string = '';
  // let messageListElement: HTMLDivElement;
  // function handleChatSend() { ... }
  // function handleDeleteChatClick() { ... }
  // function formatTime() { ... }
  // Scroll logic is now handled within ChatView

</script>

<div class="main-panel-content">
  {#if activePanel === 'transcription'}
    <TranscriptionView 
      bind:transcriptionText 
      on:saveRequest={handleSaveTranscript} 
    />
  {:else if activePanel === 'chat'}
    <!-- Use the ChatView component -->
    <ChatView 
      chatMessages={currentChatMessages}
      isProcessing={isProcessingChat}
      on:sendMessage={handleSendChat}
      on:deleteRequest={handleDeleteChat}
    />
  {:else if activePanel === 'saved-transcripts'}
    <SavedTranscriptView 
      transcriptItem={typedSelectedItem} 
      content={selectedTranscriptContent} 
      on:deleteRequest={handleDeleteTranscript}
    />
  {:else if activePanel === 'settings'}
    <SettingsView />
  {:else if activePanel === 'vocabulary'}
    <VocabularyView />
  <!-- Comment out block for missing VoiceCommandView -->
  <!-- {:else if activePanel === 'voice-commands'}
    <VoiceCommandView />
  -->
  {:else}
    <!-- Default or Empty State -->
    <div class="placeholder-view">
      <h2>Select an item or panel</h2>
      <p>The main content area will appear here.</p>
    </div>
  {/if}
</div>

<style>
  .main-panel-content {
    flex-grow: 1;
    overflow: hidden; /* Prevent scrollbars on the panel itself */
    display: flex;
    flex-direction: column;
    background-color: var(--content-bg);
  }

  /* Removed chat-specific styles - they are now in ChatView.svelte */

  .placeholder-view {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: var(--text-secondary);
  }
  .placeholder-view h2 {
      margin-bottom: 0.5rem;
  }
</style>
