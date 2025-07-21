// src/types.ts

// Base type for items shown in MiddlePanel
export type PanelItemBase = { id: string; title: string };

// Specific item types shown in MiddlePanel
export type SessionItem = PanelItemBase & { date: string }; // Used for transcripts/chats
export type CommandItem = PanelItemBase; // Used for voice commands
export type PanelItem = SessionItem | CommandItem;

// Type matching backend Chat structs
export type ChatMessage = {
  id: string; // Can be temporary client-side ID or backend ID
  text: string;
  sender: 'user' | 'ai' | 'system';
  timestamp: number; // Use number (milliseconds since epoch)
};

export type Conversation = {
  id: string;
  title: string;
  model: string; 
  // Add other fields if needed (e.g., created_timestamp_ms)
};

// Type matching backend SavedTranscriptListItem
export type SavedTranscriptListItem = {
  id: string;
  title: string;
  date: string; // ISO 8601 string
};

// Type matching backend SavedTranscriptFileContent
export type SavedTranscriptContent = {
  title: string;
  timestamp_ms: number; // i64 becomes number
  content: string;
};

// Type matching the backend response for ChatSessionListItem
export type ChatSessionListItem = {
  id: string;
  title: string;
  date: string; // ISO 8601 string
};

// Type matching backend ChatSessionContent (used in App.svelte)
export type ChatSessionContent = {
  id: string;
  title: string;
  created_timestamp_ms: number;
  messages: ChatMessage[]; // Use the shared ChatMessage type with number timestamp
};

// Type for the full App Config (used in App.svelte)
export type AppConfig = {
  general: {
    // theme?: 'light' | 'dark' | 'system';
    // auto_transcribe?: boolean;
    // offline_mode?: boolean;
  };
  audio: {
    input_device: string | null;
    speech: { 
      model_size: string; 
      language: string;
      auto_punctuate: boolean;
      translate_to_english: boolean;
      context_formatting: boolean;
      segment_duration: number; 
      buffer_size: number; 
    };
    voice_commands: {
      // ... voice command settings
    };
  };
  ai: {
    requesty_api_key: string | null;
    chat_model: string;
  };
}; 
