import { render, fireEvent, screen, waitFor, within } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Import shared types for mock data
import type { AppConfig, SavedTranscriptContent, ChatSessionContent, ChatMessage } from './types';

// Mock the Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockImplementation(async (command: string, args?: any) => { // Added args parameter
    console.log(`[Mock Invoke] Called command: ${command}`, args ? `with args: ${JSON.stringify(args)}` : ''); // Log calls

    // Provide mock responses for commands called by App.svelte ON MOUNT or directly
    if (command === 'system.get_cpu_usage') return 5.0;
    if (command === 'system.get_memory_usage') return 10.0;
    if (command === 'system.get_online_status') return true;
    if (command === 'get_whisper_models') return ['tiny', 'base', 'small']; // Mock data
    if (command === 'get_supported_languages') return [['auto', 'Auto-detect'], ['en', 'English'], ['es', 'Spanish']]; // Mock data
    if (command === 'get_settings') { // Mock settings response
        const mockConfig: AppConfig = {
            audio: {
                input_device: null,
                speech: {
                    model_size: 'small',
                    language: 'en',
                    auto_punctuate: true,
                    translate_to_english: false,
                    context_formatting: true,
                    segment_duration: 5,
                    buffer_size: 10
                }
            },
            ai: { requesty_api_key: null, chat_model: 'gemini-1.5-pro-latest' },
            general: { theme: 'system' }
        };
      return mockConfig;
    }
    // *** Mocks for ChatView.svelte internal calls ***
    if (command === 'chat.get_available_models') {
      console.log('[Mock Invoke] Mocking chat.get_available_models');
      return ['mock-gpt-3.5', 'mock-claude'];
    }
    if (command === 'chat.get_conversations') {
      console.log('[Mock Invoke] Mocking chat.get_conversations');
      // Return empty array to potentially trigger create_conversation
      // Or return a mock conversation:
      // return [{ id: 'conv1', title: 'Mock Conversation', model: 'mock-gpt-3.5', created_at: new Date().toISOString(), last_updated_at: new Date().toISOString() }];
      return [];
    }
    if (command === 'chat.create_conversation') {
      console.log('[Mock Invoke] Mocking chat.create_conversation with args:', args);
      return { 
        id: args?.id || 'new-conv-' + Date.now(), 
        title: args?.title || 'New Mock Conversation', 
        model: args?.model || 'mock-gpt-3.5', 
        created_at: new Date().toISOString(), 
        last_updated_at: new Date().toISOString() 
      };
    }
    if (command === 'chat.get_messages') {
      console.log('[Mock Invoke] Mocking chat.get_messages for conversationId:', args?.conversationId);
      return []; // Return empty messages for simplicity
    }
    // *** ADDED mocks for commands called reactively ***
    if (command === 'get_saved_transcript') {
      console.log(`[Mock Invoke] Mocking get_saved_transcript for id: ${args?.id}`);
      const mockTranscript: SavedTranscriptContent = {
          id: args?.id ?? 'unknown',
          title: `Transcript ${args?.id ?? 'Unknown'}`,
          content: `This is the mock content for transcript ${args?.id ?? 'unknown'}. Timestamp: ${Date.now()}`
      };
      return mockTranscript;
    }
    if (command === 'get_chat_session') {
       console.log(`[Mock Invoke] Mocking get_chat_session for id: ${args?.id}`);
       const mockMessages: ChatMessage[] = args?.id === 'chat1'
         ? [{ id: 'msg1', sender: 'user', text: 'Hello from chat1', timestamp: Date.now() }]
         : [];
       const mockSession: ChatSessionContent = {
           id: args?.id ?? 'unknown-chat',
           title: `Chat ${args?.id ?? 'Unknown'}`,
           messages: mockMessages,
           created_at: new Date().toISOString(),
           last_updated_at: new Date().toISOString()
       };
       return mockSession;
    }
    // *** ADDED mocks for middle panel list commands ***
    if (command === 'get_recent_transcription_list') {
        console.log(`[Mock Invoke] Mocking ${command}`);
        // Return mock SessionItem[]
        return [
            { id: 'recent1', title: 'Recent Session 1', date: new Date(Date.now() - 1000 * 60 * 5).toISOString() },
            { id: 'recent2', title: 'Another Recent Session', date: new Date(Date.now() - 1000 * 60 * 60).toISOString() },
        ];
    }
    if (command === 'get_saved_transcript_list') {
        console.log(`[Mock Invoke] Mocking ${command}`);
        // Return mock SavedTranscriptListItem[] (similar structure to SessionItem for now)
        return [
            { id: 'saved1', title: 'My Important Meeting', date: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString() },
            { id: 'saved2', title: 'Lecture Notes', date: new Date(Date.now() - 1000 * 60 * 60 * 48).toISOString() },
        ];
    }
    if (command === 'get_chat_session_list') {
        console.log(`[Mock Invoke] Mocking ${command}`);
        // Return mock ChatSessionListItem[]
        return [
            { id: 'chat1', title: 'Quick Question', last_updated_at: new Date(Date.now() - 1000 * 60 * 2).toISOString() },
            { id: 'chat2', title: 'Project Planning', last_updated_at: new Date(Date.now() - 1000 * 60 * 30).toISOString() },
        ];
    }
    // Mock responses for commands triggered by user actions (buttons, etc.)
    if (command === 'start_transcription') return undefined;
    if (command === 'stop_transcription') return undefined;
    if (command === 'save_transcript') return undefined; // Assume success
    if (command === 'delete_saved_transcript') return undefined; // Assume success
    if (command === 'delete_chat_session') return undefined; // Assume success
    if (command === 'send_chat_message') {
        console.log(`[Mock Invoke] Mocking send_chat_message for session: ${args?.sessionId}, message: ${args?.userMessage}`);
        const aiMessage: ChatMessage = {
            id: `ai-${Date.now()}`,
            sender: 'ai',
            text: `Mock AI response to: "${args?.userMessage}"`,
            timestamp: Date.now()
        };
        const newSessionId = args?.sessionId === null ? `new-session-${Date.now()}` : args?.sessionId;
        return [newSessionId, aiMessage];
    }
    if (command === 'save_all_settings') return undefined; // Assume success

    // Default for unhandled commands
    console.warn(`[Mock Invoke] Unhandled command: ${command}`);
    return undefined;
  }),
  // Mock listen to prevent errors and return a dummy unlisten function
  listen: vi.fn().mockResolvedValue(() => {
    // console.log('[Mock Listen] Registered listener');
    return () => {
      // console.log('[Mock Listen] Unregistered listener');
    };
  })
}));

import App from './App.svelte';

// Mock localStorage
let store: Record<string, string> = {};
const localStorageMock = {
  getItem: vi.fn((key: string) => store[key] || null),
  setItem: vi.fn((key: string, value: string) => { store[key] = value.toString(); }),
  removeItem: vi.fn((key: string) => { delete store[key]; }),
  clear: vi.fn(() => { store = {}; }),
};

// Assign mock to window.localStorage provided by jsdom environment
Object.defineProperty(window, 'localStorage', { value: localStorageMock });


describe('App Component', () => {
  // Clear mocks and localStorage before each test
  beforeEach(() => {
    vi.clearAllMocks(); // Clears call history, does not reset implementation
    localStorage.clear(); // Uses our mock's clear method
    store = {}; // Reset the underlying store for localStorage mock
     // Reset mock implementation calls (optional, if needed between tests)
     // vi.mocked(invoke).mockClear(); // If using invoke from '@tauri-apps/api' directly
     // vi.mocked(listen).mockClear(); // If using listen from '@tauri-apps/api' directly
  });

  it('renders default panel (Transcription) on initial load', async () => {
    render(App);
    // Check for an element unique to the Transcription view in MainPanel
    // Wait for potential async operations triggered by onMount (like fetching settings)
    await waitFor(() => {
        expect(screen.getByText('Your transcription will appear here')).toBeTruthy();
    });
    // Check MiddlePanel title after potential updates
    await waitFor(() => {
        expect(screen.getByText('Recent Transcriptions')).toBeTruthy();
    });
  });

  it('switches panel to Chat when Chat button in LeftPanel is clicked', async () => {
    render(App);
    await waitFor(() => { // Wait for initial render potentially involving async calls
        expect(screen.getByText('Recent Transcriptions')).toBeTruthy();
    });

    // Find the Chat button (assuming it's the div with text 'Chat' in LeftPanel)
    const chatButton = screen.getByText('Chat');
    await fireEvent.click(chatButton);

    // Wait for potential async updates and transitions
    await waitFor(() => {
        // Check for content unique to the Chat view in MainPanel
        expect(screen.getByPlaceholderText('Type your message...')).toBeTruthy();
        // Check MiddlePanel title changes
        expect(screen.getByText('Chat History')).toBeTruthy();
    });

    // Check if localStorage was updated
    expect(localStorage.setItem).toHaveBeenCalledWith('activePanel', 'chat');
  });

  it('switches panel to Settings using Enter key on LeftPanel item', async () => {
    render(App);
     await waitFor(() => { // Wait for initial render
        expect(screen.getByText('Recent Transcriptions')).toBeTruthy();
    });


    const settingsButton = screen.getByText('Settings');
    await fireEvent.keyDown(settingsButton, { key: 'Enter', code: 'Enter' });

    await waitFor(() => {
        // Check for content unique to Settings view
        expect(screen.getByText('Configure application settings')).toBeTruthy();
    });

    expect(localStorage.setItem).toHaveBeenCalledWith('activePanel', 'settings');
  });

  it('loads initial panel from localStorage if available', async () => {
    // Pre-populate mock localStorage
    localStorage.setItem('activePanel', 'saved-transcripts');
    render(App);

    // Wait for async actions triggered by loading from local storage
    await waitFor(() => {
         // Check if the Saved Transcripts MiddlePanel title is active
         expect(screen.getByRole('heading', { name: /Saved Transcripts/i, level: 2 })).toBeTruthy();
    });
  });

  it('resets selectedItemId when switching panels', async () => {
    // Start on Saved Transcripts and select an item
    localStorage.setItem('activePanel', 'saved-transcripts');
    localStorage.setItem('selectedItemId', 'saved1'); // Use the ID we are mocking
    render(App);

     // Wait for initial rendering and potential fetch triggered by saved1
    await waitFor(() => {
        expect(screen.getByRole('heading', { name: /Saved Transcripts/i, level: 2 })).toBeTruthy();
        // Check if the mock content was potentially rendered in MainPanel (depends on MainPanel implementation)
        // expect(screen.getByText(/This is the mock content for transcript saved1/)).toBeTruthy();
    });

    // Switch to Chat panel using data-testid from LeftPanel
    const chatButton = screen.getByTestId('nav-chat'); // Ensure LeftPanel has data-testid="nav-chat"
    await fireEvent.click(chatButton);

    await waitFor(() => {
        expect(screen.getByText('Chat History')).toBeTruthy();
    });

    // Check localStorage: activePanel updated, selectedItemId removed
    expect(localStorage.setItem).toHaveBeenCalledWith('activePanel', 'chat');
    expect(localStorage.removeItem).toHaveBeenCalledWith('selectedItemId');
  });

}); 
