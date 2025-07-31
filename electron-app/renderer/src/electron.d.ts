// Electron API type declarations for renderer process

interface ElectronAPI {
  // Audio methods
  startRecording(): Promise<boolean>;
  stopRecording(): Promise<boolean>;
  isRecording(): Promise<boolean>;
  getAudioDevices(): Promise<AudioDevice[]>;
  setAudioDevice(deviceId: string): Promise<boolean>;
  getPeakLevel(): Promise<number>;

  // Transcription methods
  startTranscription(options?: TranscriptionOptions): Promise<boolean>;
  stopTranscription(): Promise<boolean>;
  getTranscription(): Promise<string>;
  isTranscribing(): Promise<boolean>;
  clearTranscription(): Promise<void>;
  getWhisperModels(): Promise<WhisperModel[]>;
  getSupportedLanguages(): Promise<Language[]>;

  // Database/Storage methods
  saveTranscript(title: string, content: string): Promise<SavedTranscript>;
  listSavedTranscripts(): Promise<SavedTranscript[]>;
  getSavedTranscript(id: string): Promise<SavedTranscript>;
  deleteSavedTranscript(id: string): Promise<boolean>;

  // Chat methods
  getChatSessionList(): Promise<ChatSession[]>;
  getChatSession(id: string): Promise<ChatSessionWithMessages>;
  sendChatMessage(sessionId: string, message: string): Promise<[string, ChatMessage]>;
  deleteChatSession(id: string): Promise<boolean>;

  // Voice commands
  getVoiceCommandList(): Promise<VoiceCommand[]>;

  // Settings
  getSettings(): Promise<AppSettings>;
  updateSettings(settings: Partial<AppSettings>): Promise<boolean>;

  // System methods
  getSystemCpuUsage(): Promise<number>;
  getSystemMemoryUsage(): Promise<number>;
  getSystemOnlineStatus(): Promise<boolean>;

  // Text injection
  injectText(text: string): Promise<boolean>;

  // Event system (similar to Tauri's listen/emit)
  on(channel: string, callback: (event: { payload: any }) => void): () => void;
  emit(channel: string, data: any): void;
  window.electronAPI.listen(event: string, callback: (event: { payload: any }) => void): () => void;

  // Invoke method (similar to Tauri's invoke)
  invoke<T = any>(command: string, args?: any): Promise<T>;
}

interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

interface TranscriptionOptions {
  model?: string;
  language?: string;
}

interface WhisperModel {
  id: string;
  name: string;
  size: string;
}

interface Language {
  code: string;
  name: string;
}

interface SavedTranscript {
  id: string;
  name: string;
  subtitle: string;
  created_at: string;
  word_count: number;
  language?: string;
}

interface ChatSession {
  id: string;
  name: string;
  subtitle: string;
  created_at: string;
  updated_at: string;
  message_count: number;
}

interface ChatMessage {
  id: string;
  sender: 'user' | 'assistant' | 'system';
  text: string;
  timestamp: string;
}

interface ChatSessionWithMessages extends ChatSession {
  messages: ChatMessage[];
}

interface VoiceCommand {
  id: string;
  name: string;
  command: string;
  description: string;
}

interface AppSettings {
  version: string;
  general: {
    theme: 'light' | 'dark' | 'system';
    offline_mode: boolean;
    auto_start_recording: boolean;
    minimize_to_tray: boolean;
  };
  audio: {
    input_device: string | null;
    speech: {
      model_size: string;
      language: string;
      translate_to_english: boolean;
      auto_punctuate: boolean;
      context_formatting: boolean;
      model_path: string | null;
    };
    voice_commands: {
      enabled: boolean;
      prefix: string;
      require_prefix: boolean;
    };
  };
  ai: {
    requesty_api_key: string | null;
    chat_model: string;
    providers: {
      openai: { api_key: string | null; enabled: boolean };
      anthropic: { api_key: string | null; enabled: boolean };
      google: { api_key: string | null; enabled: boolean };
    };
  };
  whisper_params: {
    temperature: number;
    vad_enabled: boolean;
    vad_threshold: number;
    beam_size: number;
    no_speech_threshold: number;
    min_speech_duration_ms: number;
    max_silence_duration_ms: number;
    initial_prompt: string | null;
    patience: number;
    best_of: number;
  };
}

// Global declaration
declare global {
  interface Window {
    electronAPI: ElectronAPI;
    __TAURI_IPC__: {
      invoke<T = any>(command: string, args?: any): Promise<T>;
    };
  }
}

export {};