<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { slide } from 'svelte/transition';
  import { listen } from '@tauri-apps/api/event';
  
  // Import svelte-toast
  // import notifications, { ToastContainer } from 'svelte-toast'; // Old library
  import { SvelteToast, toast } from '@zerodevx/svelte-toast'; // New library @zerodevx/svelte-toast
  
  // Import components
  import LeftPanel from './components/LeftPanel.svelte';
  import MiddlePanel from './components/MiddlePanel.svelte';
  import MainPanel from './components/MainPanel.svelte';
  import BottomBar from './components/BottomBar.svelte';
  import TopBar from './components/TopBar.svelte';
  
  // Import theme functions
  import { applyTheme, getSystemTheme } from './theme';
  
  // Import shared types
  import type {
    AppConfig,
    PanelItem,
    SessionItem,
    CommandItem,
    SavedTranscriptListItem,
    SavedTranscriptContent,
    ChatSessionListItem,
    ChatSessionContent,
    ChatMessage
  } from './types'; // Import from types.ts
  
  // State
  // Persist and restore active panel state
  let activePanel: string;
  try {
    const storedPanel = localStorage.getItem('activePanel');
    activePanel = storedPanel || 'transcription';
  } catch (e) {
    console.error("Failed to read activePanel from localStorage:", e);
    activePanel = 'transcription'; // Default value on error
  }
  // Save active panel to localStorage on change
  $: {
    try {
      localStorage.setItem('activePanel', activePanel);
    } catch (e) {
      console.error("Failed to save activePanel to localStorage:", e);
    }
  }
  
  let isRecording = false;
  let transcriptionText = '';
  let peakLevel = 0;
  let chatMessages: any[] = [];
  let wordCount = 0;
  let accuracy = 95;
  let cpuUsage = 0.0; // Initialize as number
  let memoryUsage = 0.0; // Initialize as number
  let isOnline = true; // Default to true, update from backend
  let appVersion = '0.1.0';
  let isProcessingChat = false;
  
  // Middle panel state
  let middlePanelItems: PanelItem[] = [];
  // Persist and restore selected item state
  let selectedItemId: string | null;
  try {
    const storedItem = localStorage.getItem('selectedItemId');
    selectedItemId = storedItem ?? null; // Use nullish coalescing
  } catch (e) {
    console.error("Failed to read selectedItemId from localStorage:", e);
    selectedItemId = null; // Default value on error
  }
  // Save or clear selectedItemId in localStorage on change
  $: {
    try {
      if (selectedItemId !== null) {
        localStorage.setItem('selectedItemId', selectedItemId);
      } else {
        localStorage.removeItem('selectedItemId');
      }
    } catch (e) {
      console.error("Failed to update selectedItemId in localStorage:", e);
    }
  }
  
  // Derive selected item object from ID and items list
  $: selectedItem = middlePanelItems.find(item => item.id === selectedItemId) || null;
  
  // --- Fetch Middle Panel Items --- START
  let previousActivePanel = activePanel; // Store the initial active panel

  $: {
    // If the activePanel has genuinely changed from its previous state,
    // then it's appropriate to clear the selected item ID.
    if (activePanel !== previousActivePanel) {
      selectedItemId = null;
      previousActivePanel = activePanel; // Update previousActivePanel for the next change
    }

    const panelToLoad = activePanel; // Use a new const for clarity in the async function
    const loadMiddlePanelItems = async (currentPanel: string) => {
      console.log(`Fetching items for middle panel: ${currentPanel}`);
      middlePanelItems = []; // Clear items while loading
      // selectedItemId = null; // This line was moved to the outer reactive block
      try {
        let command: string | null = null;
        // Determine the correct backend command based on the active panel
        switch (currentPanel) {
          case 'transcription':
            command = 'get_recent_transcription_list'; // Placeholder
            break;
          case 'saved-transcripts':
            command = 'list_saved_transcripts'; // Corrected command name
            break;
          case 'chat':
            command = 'get_chat_session_list'; // Placeholder
            break;
          case 'voice-commands':
            command = 'get_voice_command_list'; // Placeholder
            break;
          // Add cases for other panels that have middle panel lists
          default:
            // Panels like 'settings', 'devices', 'help' might not have middle panel items
            console.log(`No items to fetch for panel: ${currentPanel}`);
            middlePanelItems = []; // Ensure it's empty for panels without lists
            return; // Exit early
        }

        if (command) {
            console.log(`Invoking command: ${command}`);
            const items = await invoke<PanelItem[]>(command);
            // Check if panel is still the same after await
            if (activePanel === currentPanel) {
                middlePanelItems = items;
                console.log(`Loaded ${items.length} items for panel: ${currentPanel}`);
            } else {
               console.log(`Panel changed while fetching items for ${currentPanel}. Aborting update.`);
            }
        }
      } catch (error) {
          console.error(`Failed to load items for panel ${currentPanel}:`, error);
          // notifications.error('Load Error', `Failed to load list for ${currentPanel}: ${error}`); // Old
          toast.push(`Error loading list for ${currentPanel}: ${error}`); // New
          // Ensure items remain empty on error, even if panel didn't change
          if (activePanel === currentPanel) {
              middlePanelItems = []; 
          }
      }
    };
    loadMiddlePanelItems(panelToLoad); // Trigger load when activePanel changes
  }
  // --- Fetch Middle Panel Items --- END
  
  // TopBar state
  let aiModels: string[] = []; // Whisper models
  let languages: [string, string][] = [];
  let selectedWhisperModel: string;
  let selectedLanguage: string;
  let currentTheme: 'light' | 'dark' | 'system';

  // AI Chat State
  let isFetchingTranscript = false;
  let requestyApiKey: string | null;
  let selectedChatModel: string;

  // Add back the missing state variable
  let selectedTranscriptContent: string | null = null;

  // State for transcript fetching
  let currentlyFetchingTranscriptId: string | null = null; // Track the ID being fetched

  // Load initial settings from localStorage or defaults
  try {
    selectedWhisperModel = localStorage.getItem('selectedWhisperModel') || 'small'; // Renamed key
    selectedLanguage = localStorage.getItem('selectedLanguage') || 'auto';
    currentTheme = (localStorage.getItem('theme') as 'light' | 'dark' | 'system') || 'system';
    requestyApiKey = localStorage.getItem('requestyApiKey') || null; // Load API key
    selectedChatModel = localStorage.getItem('selectedChatModel') || 'gemini-1.5-pro-latest'; // Load chat model
  } catch (e) {
    console.error("Failed to load settings from localStorage:", e);
    selectedWhisperModel = 'small'; // Renamed
    selectedLanguage = 'auto';
    currentTheme = 'system';
    requestyApiKey = null;
    selectedChatModel = 'gemini-1.5-pro-latest';
  }

  // Apply initial theme
  applyTheme(currentTheme);

  // State for the messages of the currently selected chat session
  let currentChatMessages: ChatMessage[] = [];
  
  // --- Fetch full transcript content --- START
  async function loadTranscriptContent(id: string | null) {
    // Prevent fetch if panel isn't right, no ID, or already fetching this specific ID
    if (activePanel !== 'saved-transcripts' || !id || currentlyFetchingTranscriptId === id) {
        // If no id is provided (or null), ensure content is cleared
        if (!id && selectedTranscriptContent !== null) {
             selectedTranscriptContent = null;
        }
        // If panel changed away from transcripts, ensure fetching state is reset
        if (activePanel !== 'saved-transcripts' && isFetchingTranscript) {
             isFetchingTranscript = false;
             currentlyFetchingTranscriptId = null;
        }
        return; // Exit if no fetch needed
    }

    // Start fetching
    isFetchingTranscript = true;
    currentlyFetchingTranscriptId = id; // Mark this ID as being fetched
    selectedTranscriptContent = null; // Clear previous content immediately
    console.log(`Fetching transcript content for ${id}...`);

    try {
        const fullItem = await invoke<SavedTranscriptContent>('get_saved_transcript', { id });
        // Check if still relevant *after* await (panel and selected ID haven't changed)
        if (activePanel === 'saved-transcripts' && selectedItemId === id) {
            selectedTranscriptContent = fullItem.content;
        } else {
             console.log(`Fetch for ${id} aborted or became irrelevant (panel/selection changed during fetch).`);
             // Ensure content is cleared if fetch became irrelevant while running
             if (selectedTranscriptContent !== null) {
                  selectedTranscriptContent = null;
             }
        }
    } catch (error) {
        console.error(`Failed to load content for ${id}:`, error);
        // notifications.error('Load Error', `Failed to load transcript content: ${error}`); // Old
        toast.push(`Error loading transcript content for ${id}: ${error}`); // New
        // Show error only if still relevant
        if (activePanel === 'saved-transcripts' && selectedItemId === id) {
            selectedTranscriptContent = "Error: Could not load content.";
        }
    } finally {
        // Reset fetching state only if we finished fetching the ID we were tasked with
        if (currentlyFetchingTranscriptId === id) {
             isFetchingTranscript = false;
             currentlyFetchingTranscriptId = null;
        }
    }
  }

  // Reactive trigger: Call loadTranscriptContent whenever activePanel or selectedItemId changes.
  // Pass null if not on the correct panel to handle clearing.
  $: loadTranscriptContent(activePanel === 'saved-transcripts' ? selectedItemId : null);
  // --- Fetch full transcript content --- END
  
  // Fetch full chat session content when a chat item is selected
  let currentChatFetchId: string | null = null; // Prevent race conditions
  $: (async () => {
    if (activePanel === 'chat' && selectedItemId && selectedItemId !== currentChatFetchId) {
      const fetchId = selectedItemId;
      currentChatFetchId = fetchId;
      currentChatMessages = []; // Clear previous messages while loading
      console.log(`Fetching chat session for ${fetchId}...`);
      try {
        const sessionContent = await invoke<ChatSessionContent>('get_chat_session', { id: fetchId });
        // Check if the selection hasn't changed while fetching
        if (selectedItemId === fetchId) { 
          currentChatMessages = sessionContent.messages;
        }
    } catch (error) {
        console.error(`Failed to load chat session for ${fetchId}:`, error);
        // notifications.error('Load Error', `Failed to load chat session: ${error}`); // Old
        toast.push(`Error loading chat session for ${fetchId}: ${error}`); // New
        // Optionally show error to user
        if (selectedItemId === fetchId) {
          // Maybe display an error message in the chat?
          currentChatMessages = [{
            id: `system-error-${Date.now()}`,
            sender: 'system', 
            text: `Error: Could not load chat content. ${error}`,
            timestamp: Date.now()
          }];
        }
      } finally {
        // Reset fetch guard if this was the latest fetch for this ID
        if (currentChatFetchId === fetchId) {
          currentChatFetchId = null;
        }
      }
    } else if (activePanel !== 'chat') {
      // Clear messages if navigating away from chat
      currentChatMessages = []; 
      currentChatFetchId = null;
    }
  })();
  
  // Handle panel navigation
  function handleNavigate(event: CustomEvent<{ panel: string }>) {
    activePanel = event.detail.panel;
    selectedItemId = null;
  }
  
  // Handle item selection
  function handleSelect(event: CustomEvent<{ id: string }>) {
    selectedItemId = event.detail.id;
    // Here you would load data based on the selected item - Now handled reactively by passing selectedItem
  }
  
  // Handle microphone toggle
  async function handleMicToggle() {
    console.log(`Mic toggle requested. Currently recording: ${isRecording}`);
    if (isRecording) {
      // Request to stop recording
      try {
        await invoke('stop_transcription');
        console.log('Stop transcription requested successfully.');
        // isRecording state will be updated by the 'transcribe:stopped' event listener
    } catch (error) {
        console.error("Failed to invoke stop_transcription:", error);
        // TODO: Show error feedback to user
      }
    } else {
      // Request to start recording
      // Optional: Clear previous transcription first?
      // transcriptionText = ''; 
      try {
        await invoke('start_transcription', { 
            // Pass necessary options if the command expects them
            // e.g., options: { language: selectedLanguage, model_size: selectedWhisperModel }
            // Currently, the backend command doesn't seem to take options here,
            // it likely uses the config. Let's assume no options needed for now.
        });
        console.log('Start transcription requested successfully.');
        // isRecording state will be updated by the 'transcribe:started' event listener
    } catch (error) {
        console.error("Failed to invoke start_transcription:", error);
        // TODO: Show error feedback to user
      }
    }
  }
  
  // Handle theme toggle
  function handleThemeToggle() {
    const themes: ('light' | 'dark' | 'system')[] = ['light', 'dark', 'system'];
    const currentIndex = themes.indexOf(currentTheme);
    currentTheme = themes[(currentIndex + 1) % themes.length];
    applyTheme(currentTheme);
  }
  
  // Handle navigation request from TopBar settings button
  function handleNavigateSettings() {
    activePanel = 'settings';
    selectedItemId = null; // Clear selection when changing panel context
  }
  
  // Handle request to save transcript from MainPanel
  async function handleSaveTranscriptRequest(event: CustomEvent<{ title: string; content: string }>) {
    const { title, content } = event.detail;
    if (!title || !content) {
      console.error("Save request missing title or content");
      // notifications.error('Save Error', "Cannot save transcript without title or content."); // Old
      toast.push("Save Error: Cannot save transcript without title or content."); // New
      return;
    }

    try {
      await invoke('save_transcript', { title, content });
      console.log("Transcript saved successfully:", title);
      // notifications.success('Saved', `Transcript "${title}" saved successfully!`); // Old
      toast.push(`Saved: Transcript "${title}" saved successfully!`); // New
      // Optionally, clear current transcriptionText or navigate away?
      // Refresh the saved list if currently viewing it?
      if (activePanel === 'saved-transcripts') {
        // Trigger reload - simplest way is to briefly switch panel and back, or re-call invoke
        // For now, manually trigger reactive update by re-assigning activePanel
        const current = activePanel;
        activePanel = ''; // Force temporary change
        await new Promise(resolve => setTimeout(resolve, 0)); // Allow UI cycle
        activePanel = current; 
      }
    } catch (error) {
      console.error("Failed to save transcript:", error);
      // notifications.error('Save Error', `Failed to save transcript: ${error}`); // Old
      toast.push(`Save Error: Failed to save transcript: ${error}`); // New
    }
  }
  
  // Handle request to delete transcript from MainPanel
  async function handleDeleteTranscriptRequest(event: CustomEvent<{ id: string }>) {
    const { id } = event.detail;
    console.log(`Requesting delete for ${id}`);

    try {
      await invoke('delete_saved_transcript', { id });
      console.log("Transcript deleted successfully:", id);

      // Clear selection
      selectedItemId = null;
      selectedTranscriptContent = null;

      // Refresh the saved list
      // For now, manually trigger reactive update by re-assigning activePanel
      const current = activePanel;
      if (current === 'saved-transcripts') { // Only refresh if viewing the list
        activePanel = ''; // Force temporary change
        await new Promise(resolve => setTimeout(resolve, 0)); // Allow UI cycle
        activePanel = current;
      } 
      // notifications.success("Transcript deleted."); // Old
      toast.push("Transcript deleted."); // New
    } catch (error) {
      console.error(`Failed to delete transcript ${id}:`, error);
      // notifications.error('Delete Error', `Failed to delete transcript: ${error}`); // Old
      toast.push(`Delete Error: Failed to delete transcript ${id}: ${error}`); // New
    }
  }
  
  // Handle request to delete chat session from MainPanel
  async function handleDeleteChatSession(event: CustomEvent<{ id: string }>) {
    const { id } = event.detail;
    console.log(`Requesting delete for chat session ${id}`);

    try {
      await invoke('delete_chat_session', { id });
      // notifications.success("Chat session deleted."); // Old
      toast.push("Chat session deleted."); // New

      // Clear selection if the deleted item was selected
      if (selectedItemId === id) {
        selectedItemId = null;
        currentChatMessages = [];
      }

      // Refresh the chat list
      const current = activePanel;
      if (current === 'chat') { // Only refresh if viewing the list
        activePanel = ''; 
        await new Promise(resolve => setTimeout(resolve, 0)); 
        activePanel = current;
      } 
    } catch (error) {
      console.error(`Failed to delete chat session ${id}:`, error);
      // notifications.error('Delete Error', `Failed to delete chat session: ${error}`); // Old
      toast.push(`Delete Error: Failed to delete chat session ${id}: ${error}`); // New
    }
  }
  
  // Handle request to send chat message from MainPanel
  async function handleSendChatMessage(event: CustomEvent<{ sessionId: string | null; message: string }>) {
    const { sessionId, message } = event.detail;
    const userMessage: ChatMessage = {
      id: `temp-${Date.now()}`,
      sender: 'user',
      text: message,
      timestamp: Date.now() // Use timestamp field with number type
    };
    currentChatMessages = [...currentChatMessages, userMessage];
    isProcessingChat = true;
    // const loadingToastId = notifications.loading('Waiting for AI...'); // Old
    const loadingToastId = toast.push('Waiting for AI...'); // New, returns ID
    try {
      const [newSessionId, aiMessage] = await invoke<[string, ChatMessage]>('send_chat_message', { 
        sessionId,
        userMessage: message 
      });
      // Ensure aiMessage has a number timestamp if needed, otherwise backend needs alignment
      currentChatMessages = [...currentChatMessages, aiMessage]; 
      if (!sessionId && newSessionId) {
        selectedItemId = newSessionId; // Select the new session
        // Trigger list refresh
        const current = activePanel;
        if (current === 'chat') {
          activePanel = ''; 
          await new Promise(resolve => setTimeout(resolve, 0)); 
          activePanel = current;
        }
      } else {
         // Refresh messages for current session (might not be needed if AI message added above)
         // Re-assigning triggers reactivity if needed
         currentChatMessages = [...currentChatMessages]; 
      }
      // notifications.remove(loadingToastId); // Old
      if (loadingToastId) toast.pop(loadingToastId); // New
    } catch (error) {
      // notifications.remove(loadingToastId); // Old
      if (loadingToastId) toast.pop(loadingToastId); // New
      console.error("Failed to send chat message:", error);
      // notifications.error('Send Error', `Failed to get AI response: ${error}`); // Old
      toast.push(`Send Error: Failed to get AI response: ${error}`); // New
      const errorMessage: ChatMessage = {
          id: `error-${Date.now()}`,
          sender: 'system',
          text: `Error sending message: ${error}`,
          timestamp: Date.now() // Use timestamp field with number type
      };
      currentChatMessages = [...currentChatMessages, errorMessage];
    } finally {
      isProcessingChat = false;
    }
  }
  
  // Update stats from transcription
  function updateStats(event: CustomEvent<{ wordCount?: number }>) {
    if (event.detail.wordCount !== undefined) {
      wordCount = event.detail.wordCount;
    }
  }
  
  // Setup system monitoring
  let monitoringInterval: ReturnType<typeof setInterval>;
  let eventListeners: (() => void)[] = []; // Explicitly type the array
  
  // Function to check backend status
  async function checkBackendStatus() {
    try {
      // Fetch real system metrics from backend
      const [cpu, mem, onlineStatus] = await Promise.all([
        invoke<number>('system.get_cpu_usage'),
        invoke<number>('system.get_memory_usage'),
        invoke<boolean>('system.get_online_status')
      ]);

      // Round to 1 decimal place for cleaner display
      cpuUsage = parseFloat(cpu.toFixed(1));
      memoryUsage = parseFloat(mem.toFixed(1));
      isOnline = onlineStatus;

        } catch (error) {
      console.error("Failed to fetch system metrics or online status:", error);
      // Keep previous values or set defaults on error
      cpuUsage = cpuUsage || 0;
      memoryUsage = memoryUsage || 0;
      isOnline = false; // Assume offline if status check fails
    }
  }
  
  onMount(async () => {
    // Initial check for system status
    checkBackendStatus();

    // Fetch models and languages for selectors
    try {
      const [models, langs] = await Promise.all([
        invoke<string[]>('get_whisper_models'),
        invoke<[string, string][]>('get_supported_languages')
      ]);
      aiModels = models;
      languages = [['auto', 'Auto-detect'], ...langs];
      // Validation moved after loading full settings
    } catch (error) {
      console.error("Failed to load TopBar data in App:", error);
      // notifications.error('Load Error', `Failed to load models/languages: ${error}`); // Old
      toast.push(`Load Error: Failed to load models/languages: ${error}`); // New
      // Provide minimal defaults if fetch fails but selections were loaded from localStorage
      if (!aiModels.includes(selectedWhisperModel)) aiModels = [selectedWhisperModel];
      if (!languages.some(l => l[0] === selectedLanguage)) languages = [[selectedLanguage, selectedLanguage]];
    }
    
    // Load full settings from backend (overwrites localStorage values if successful)
    try {
      console.log("Fetching full application settings...");
      const loadedConfig = await invoke<AppConfig>('get_settings');
      console.log("Loaded config:", loadedConfig);

      // Update state from loaded config
      // Use nullish coalescing (??) to keep localStorage/initial defaults if backend value is null/undefined
      selectedWhisperModel = loadedConfig.audio.speech.model_size ?? selectedWhisperModel;
      selectedLanguage = loadedConfig.audio.speech.language ?? selectedLanguage;
      requestyApiKey = loadedConfig.ai.requesty_api_key ?? requestyApiKey;
      selectedChatModel = loadedConfig.ai.chat_model ?? selectedChatModel;

      // Update detailed Whisper settings state
      // whisperAutoPunctuate = loadedConfig.audio.speech.auto_punctuate ?? whisperAutoPunctuate;
      // whisperTranslateToEnglish = loadedConfig.audio.speech.translate_to_english ?? whisperTranslateToEnglish;
      // whisperContextFormatting = loadedConfig.audio.speech.context_formatting ?? whisperContextFormatting;
      // whisperSegmentDuration = loadedConfig.audio.speech.segment_duration ?? whisperSegmentDuration;
      // whisperBufferSize = loadedConfig.audio.speech.buffer_size ?? whisperBufferSize;

      // Update other settings if needed (e.g., theme)
      // currentTheme = loadedConfig.general.theme ?? currentTheme; // Example

      // Ensure loaded selections are valid against fetched lists
      if (!aiModels.includes(selectedWhisperModel) && aiModels.length > 0) {
          console.warn(`Loaded model '${selectedWhisperModel}' not in available list, defaulting to ${aiModels[0]}`);
          selectedWhisperModel = aiModels[0];
      }
      if (!languages.some(l => l[0] === selectedLanguage) && languages.length > 0) {
          console.warn(`Loaded language '${selectedLanguage}' not in available list, defaulting to ${languages[0][0]}`);
          selectedLanguage = languages[0][0];
      }

    } catch (error) {
      console.error("Failed to load settings from backend:", error);
      // notifications.error('Load Error', `Failed to load settings: ${error}. Using defaults/localStorage fallback.`); // Old
      toast.push(`Load Error: Failed to load settings: ${error}. Using defaults/localStorage fallback.`); // New
      // Fallback to localStorage loading is implicitly handled as state was already initialized from it
    }
    
    // Setup event listeners for transcription
    try {
      const updateListener = await listen<string>('transcription:update', (event) => {
        console.log('Transcription update:', event.payload);
        transcriptionText = event.payload; // Update the transcription text state
      });

      const clearListener = await listen('transcription:clear', () => {
        console.log('Transcription clear event');
        transcriptionText = ''; // Clear the text
      });

      const errorListener = await listen<{ error: string }>('transcribe:error', (event) => {
        console.error('Transcription error:', event.payload.error);
        // notifications.error('Transcription Error', `Transcription error: ${event.payload.error}`); // Old
        toast.push(`Transcription Error: ${event.payload.error}`); // New
      });

      // Optional: Listen for started/stopped if needed to sync isRecording more reliably
      const startedListener = await listen('transcribe:started', () => {
          console.log('Transcription started event');
          isRecording = true;
      });
      const stoppedListener = await listen('transcribe:stopped', () => {
          console.log('Transcription stopped event');
          isRecording = false;
      });

      eventListeners = [
          updateListener,
          clearListener,
          errorListener,
          startedListener, 
          stoppedListener
      ];

    } catch (error) {
        console.error("Failed to set up transcription event listeners:", error);
        // notifications.error('Listener Setup Error', `Listener setup failed: ${error}`); // Old
        toast.push(`Listener Setup Error: Failed to set up transcription event listeners: ${error}`); // New
    }

    // Setup polling interval for system status
    monitoringInterval = setInterval(checkBackendStatus, 5000);
    
    // Check initial recording state
    try {
      const recording = await invoke<boolean>('is_transcribing');
      isRecording = recording;
      console.log('Initial recording state:', recording);
    } catch (error) {
      console.error('Failed to get initial recording state:', error);
    }
  });
  
  onDestroy(() => {
    if (monitoringInterval) {
      clearInterval(monitoringInterval);
    }
    // Cleanup event listeners
    console.log('Cleaning up event listeners...');
    eventListeners.forEach(unlisten => unlisten());
  });
</script>

<div class="app-container">
  <TopBar 
    bind:selectedWhisperModel
    bind:selectedLanguage
    {aiModels}
    {languages}
    {isRecording}
    on:toggleMic={handleMicToggle}
    on:toggleTheme={handleThemeToggle}
    on:navigateSettings={handleNavigateSettings}
  />
  
  <div class="panels-container">
    <LeftPanel 
      bind:activeItem={activePanel} 
      on:navigate={handleNavigate} 
    />
    
    {#key activePanel}
      <div in:slide={{ duration: 300 }} out:slide={{ duration: 300 }}>
        <MiddlePanel
          activePanel={activePanel}
          items={middlePanelItems}
          selectedItemId={selectedItemId}
          on:select={handleSelect}
        />
        </div>
    {/key}
    
    {#key activePanel}
      <div in:slide={{ duration: 300 }} out:slide={{ duration: 300 }}>
        <MainPanel
          activePanel={activePanel}
          selectedItem={selectedItem}
          selectedTranscriptContent={selectedTranscriptContent}
          transcriptionText={transcriptionText}
          currentChatMessages={currentChatMessages}
          on:saveTranscriptRequest={handleSaveTranscriptRequest}
          on:deleteTranscriptRequest={handleDeleteTranscriptRequest}
          on:sendChatMessage={handleSendChatMessage}
          on:deleteChatRequest={handleDeleteChatSession}
          isProcessingChat={isProcessingChat}
        />
      </div>
    {/key}
  </div>
  
  <!-- Bottom Bar -->
  <BottomBar 
    {appVersion}
    {isOnline}
    {cpuUsage}
    {memoryUsage}
    {wordCount}
    {accuracy}
  />
  
  <!-- Global Toast Container -->
  <!-- <ToastContainer notifications={$notifications} /> --> <!-- Old library -->
  <SvelteToast /> <!-- New library @zerodevx/svelte-toast -->
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    height: 100vh;
    overflow: hidden;
  }
  
  :global(#app) {
    height: 100vh;
  }
  
  :global(:root) {
    /* Light mode variables */
    --app-bg: #f9f9f9;
    --content-bg: #fff;
    --panel-bg: #f5f5f5;
    --secondary-bg: #f9f9f9;
    --controls-bg: #f5f5f5;
    --status-bar-bg: #f0f0f0;
    
    --text-primary: #333;
    --text-secondary: #666;
    --text-tertiary: #999;
    
    --border-color: #e0e0e0;
    
    --button-bg: #f0f0f0;
    --button-hover-bg: #e5e5e5;
    --active-button-bg: #ff5252;
    
    --input-bg: #fff;
    --dropdown-bg: #fff;
    
    --hover-bg: #eaeaea;
    --active-bg: #e2e2e2;
    --selected-bg: #e8e8e8;
    
    --primary-color: #2196f3;
    --primary-color-hover: #1976d2;
    
    --waveform-color: #2196f3;
    --inactive-bar: #e0e0e0;
    --active-bar: #4caf50;
    
    --user-message-bg: #e3f2fd;
    --user-message-color: #0d47a1;
    --ai-message-bg: #f5f5f5;
    --ai-message-color: #333;
    --system-message-bg: #fff3e0;
    --system-message-color: #e65100;
    
    --online-color: #4caf50;
    --offline-color: #f44336;
    --progress-bg: #e0e0e0;
    --progress-color: #2196f3;
  }
  
  /* Dark Theme applied via body.dark-theme */
  :global(body.dark-theme :root) {
    /* Dark mode variables */
    --app-bg: #121212;
    --content-bg: #1a1a1a;
    --panel-bg: #252525;
    --secondary-bg: #252525;
    --controls-bg: #252525;
    --status-bar-bg: #252525;
    
    --text-primary: #eee;
    --text-secondary: #bbb;
    --text-tertiary: #888;
    
    --border-color: #333;
    
    --button-bg: #333;
    --button-hover-bg: #444;
    --active-button-bg: #b71c1c;
    
    --input-bg: #333;
    --dropdown-bg: #333;
    
    --hover-bg: #2a2a2a;
    --active-bg: #333;
    --selected-bg: #333;
    
    --primary-color: #1976d2;
    --primary-color-hover: #1565c0;
    
    --waveform-color: #1976d2;
    --inactive-bar: #444;
    --active-bar: #43a047;
    
    --user-message-bg: #0d47a1;
    --user-message-color: #fff;
    --ai-message-bg: #333;
    --ai-message-color: #eee;
    --system-message-bg: #5d4037;
    --system-message-color: #ffe0b2;
    
    --online-color: #43a047;
    --offline-color: #e53935;
    --progress-bg: #444;
    --progress-color: #1976d2;
  }
  
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background-color: var(--app-bg);
    color: var(--text-primary);
    overflow: hidden;
    position: relative;
  }
  
  .panels-container {
    display: flex;
    flex: 1;
    overflow: hidden;
    height: calc(100vh - 48px - 32px);
    width: 100%;
  }
</style> 
