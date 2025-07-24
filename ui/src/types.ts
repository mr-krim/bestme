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

// AI Model types
export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  architecture: string;
  parameters: string;
  size_bytes: number;
  capabilities: string[];
  performance_class: 'fast' | 'balanced' | 'quality';
  hardware_requirements: {
    min_ram_gb: number;
    recommended_vram_gb?: number;
    supports_gpu: boolean;
  };
}

export interface ModelPerformance {
  model_id: string;
  avg_tokens_per_second: number;
  avg_latency_ms: number;
  memory_usage_mb: number;
  last_updated: Date;
}

export interface DownloadInfo {
  model_id: string;
  model_name: string;
  total_bytes: number;
  downloaded_bytes: number;
  progress_percent: number;
  download_speed: number;
  eta_seconds: number;
  status: 'pending' | 'downloading' | 'verifying' | 'extracting' | 'completed' | 'failed';
  error?: string;
}

export interface ModelConfig {
  model_id: string;
  batch_size: number;
  max_tokens: number;
  temperature: number;
  top_p: number;
  use_gpu: boolean;
  gpu_device_id?: number;
  num_threads: number;
  memory_limit_mb?: number;
  quantization?: 'none' | 'int8' | 'int4' | 'fp16';
  enable_caching: boolean;
  cache_size: number;
  context_window: number;
  streaming_enabled: boolean;
  fallback_enabled: boolean;
  timeout_ms: number;
}

export interface GpuDevice {
  id: number;
  name: string;
  memory_mb: number;
  compute_capability?: string;
} 
