const { contextBridge, ipcRenderer } = require('electron');

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('electronAPI', {
  // Audio methods
  startRecording: () => ipcRenderer.invoke('start-recording'),
  stopRecording: () => ipcRenderer.invoke('stop-recording'),
  isRecording: () => ipcRenderer.invoke('is-recording'),
  getAudioDevices: () => ipcRenderer.invoke('get-audio-devices'),
  setAudioDevice: (deviceId) => ipcRenderer.invoke('set-audio-device', deviceId),
  getPeakLevel: () => ipcRenderer.invoke('get-peak-level'),
  
  // Transcription methods
  startTranscription: (options) => ipcRenderer.invoke('start-transcription', options),
  stopTranscription: () => ipcRenderer.invoke('stop-transcription'),
  getTranscription: () => ipcRenderer.invoke('get-transcription'),
  isTranscribing: () => ipcRenderer.invoke('is-transcribing'),
  clearTranscription: () => ipcRenderer.invoke('clear-transcription'),
  getWhisperModels: () => ipcRenderer.invoke('get-whisper-models'),
  getSupportedLanguages: () => ipcRenderer.invoke('get-supported-languages'),
  
  // Database/Storage methods
  saveTranscript: (title, content) => ipcRenderer.invoke('save-transcript', title, content),
  listSavedTranscripts: () => ipcRenderer.invoke('list-saved-transcripts'),
  getSavedTranscript: (id) => ipcRenderer.invoke('get-saved-transcript', id),
  deleteSavedTranscript: (id) => ipcRenderer.invoke('delete-saved-transcript', id),
  
  // Chat methods
  getChatSessionList: () => ipcRenderer.invoke('get-chat-session-list'),
  getChatSession: (id) => ipcRenderer.invoke('get-chat-session', id),
  sendChatMessage: (sessionId, message) => ipcRenderer.invoke('send-chat-message', sessionId, message),
  deleteChatSession: (id) => ipcRenderer.invoke('delete-chat-session', id),
  
  // Voice commands
  getVoiceCommandList: () => ipcRenderer.invoke('get-voice-command-list'),
  
  // Settings
  getSettings: () => ipcRenderer.invoke('get-settings'),
  updateSettings: (settings) => ipcRenderer.invoke('update-settings', settings),
  
  // System methods
  getSystemCpuUsage: () => ipcRenderer.invoke('system.get-cpu-usage'),
  getSystemMemoryUsage: () => ipcRenderer.invoke('system.get-memory-usage'),
  getSystemOnlineStatus: () => ipcRenderer.invoke('system.get-online-status'),
  
  // Text injection
  injectText: (text) => ipcRenderer.invoke('inject-text', text),
  
  // Event listeners (replacing Tauri's listen function)
  on: (channel, callback) => {
    const validChannels = [
      'transcription:update',
      'transcription:clear', 
      'transcribe:started',
      'transcribe:stopped',
      'transcribe:error',
      'recording-state-changed',
      'menu-action'
    ];
    
    if (validChannels.includes(channel)) {
      const subscription = (event, ...args) => callback({ payload: args[0] });
      ipcRenderer.on(channel, subscription);
      
      // Return unsubscribe function
      return () => ipcRenderer.removeListener(channel, subscription);
    }
  },
  
  // Emit events (replacing Tauri's emit function)
  emit: (channel, data) => {
    ipcRenderer.send(channel, data);
  },
  
  // Invoke method (replacing Tauri's invoke function)
  invoke: (command, args) => {
    // Map command names to IPC method calls
    const commandMap = {
      // Audio commands
      'start_recording': () => ipcRenderer.invoke('start-recording'),
      'stop_recording': () => ipcRenderer.invoke('stop-recording'),
      'is_recording': () => ipcRenderer.invoke('is-recording'),
      'get_audio_devices': () => ipcRenderer.invoke('get-audio-devices'),
      'set_audio_device': (deviceId) => ipcRenderer.invoke('set-audio-device', deviceId),
      'get_level': () => ipcRenderer.invoke('get-peak-level'),
      
      // Transcription commands
      'start_transcription': (options) => ipcRenderer.invoke('start-transcription', options),
      'stop_transcription': () => ipcRenderer.invoke('stop-transcription'),
      'get_transcription': () => ipcRenderer.invoke('get-transcription'),
      'is_transcribing': () => ipcRenderer.invoke('is-transcribing'),
      'clear_transcription': () => ipcRenderer.invoke('clear-transcription'),
      'get_whisper_models': () => ipcRenderer.invoke('get-whisper-models'),
      'get_supported_languages': () => ipcRenderer.invoke('get-supported-languages'),
      
      // Storage commands
      'save_transcript': (data) => ipcRenderer.invoke('save-transcript', data.title, data.content),
      'list_saved_transcripts': () => ipcRenderer.invoke('list-saved-transcripts'),
      'get_saved_transcript': (data) => ipcRenderer.invoke('get-saved-transcript', data.id),
      'delete_saved_transcript': (data) => ipcRenderer.invoke('delete-saved-transcript', data.id),
      
      // Chat commands
      'get_chat_session_list': () => ipcRenderer.invoke('get-chat-session-list'),
      'get_chat_session': (data) => ipcRenderer.invoke('get-chat-session', data.id),
      'send_chat_message': (data) => ipcRenderer.invoke('send-chat-message', data.userMessage, data.sessionId),
      'delete_chat_session': (data) => ipcRenderer.invoke('delete-chat-session', data.id),
      
      // Voice commands
      'get_voice_command_list': () => ipcRenderer.invoke('get-voice-command-list'),
      
      // Settings
      'get_settings': () => ipcRenderer.invoke('get-settings'),
      'update_settings': (settings) => ipcRenderer.invoke('update-settings', settings),
      
      // System commands
      'system.get_cpu_usage': () => ipcRenderer.invoke('system.get-cpu-usage'),
      'system.get_memory_usage': () => ipcRenderer.invoke('system.get-memory-usage'),
      'system.get_online_status': () => ipcRenderer.invoke('system.get-online-status'),
    };
    
    const handler = commandMap[command];
    if (handler) {
      return handler(args);
    } else {
      console.warn(`Unknown command: ${command}`);
      return Promise.reject(new Error(`Unknown command: ${command}`));
    }
  },
  
  // Listen method (similar to Tauri's listen)
  listen: (event, callback) => {
    const validEvents = [
      'transcription:update',
      'transcription:clear',
      'transcribe:started', 
      'transcribe:stopped',
      'transcribe:error',
      'recording-state-changed',
      'menu-action'
    ];
    
    if (validEvents.includes(event)) {
      const subscription = (ipcEvent, ...args) => {
        callback({ payload: args[0] || args });
      };
      
      ipcRenderer.on(event, subscription);
      
      // Return unsubscribe function
      return () => ipcRenderer.removeListener(event, subscription);
    } else {
      console.warn(`Invalid event: ${event}`);
      return () => {}; // Return empty unsubscribe function
    }
  }
});

// Also expose a version of the API that matches Tauri's API more closely
contextBridge.exposeInMainWorld('__TAURI_IPC__', {
  invoke: (command, args) => {
    return window.electronAPI.invoke(command, args);
  }
});

console.log('Preload script loaded - Electron API exposed');