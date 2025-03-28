// Audio plugin initialization script
window.__TAURI_PLUGIN_AUDIO__ = {
  // Initialize the plugin
  init() {
    console.log("Initializing Audio plugin");
    
    // Create event emitter for audio events
    const listeners = {
      "level-change": [],
      "recording-started": [],
      "recording-stopped": [],
      "error": []
    };
    
    // Listen for events from the Rust side
    window.__TAURI__.event.listen("audio:level-change", (event) => {
      const level = event.payload;
      listeners["level-change"].forEach(cb => cb(level));
    });
    
    window.__TAURI__.event.listen("audio:started", () => {
      listeners["recording-started"].forEach(cb => cb());
    });
    
    window.__TAURI__.event.listen("audio:stopped", () => {
      listeners["recording-stopped"].forEach(cb => cb());
    });
    
    window.__TAURI__.event.listen("audio:error", (event) => {
      const error = event.payload;
      listeners["error"].forEach(cb => cb(error));
    });
    
    // Export API
    return {
      // Start audio recording
      async startRecording(deviceName) {
        return window.__TAURI__.invoke("plugin:audio:start_recording", { deviceName });
      },
      
      // Stop audio recording
      async stopRecording() {
        return window.__TAURI__.invoke("plugin:audio:stop_recording");
      },
      
      // Get current audio level
      async getLevel() {
        return window.__TAURI__.invoke("plugin:audio:get_level");
      },
      
      // Check if recording is active
      async isRecording() {
        return window.__TAURI__.invoke("plugin:audio:is_recording");
      },
      
      // Get available audio devices
      async getAudioDevices() {
        return window.__TAURI__.invoke("plugin:audio:get_audio_devices");
      },
      
      // Set audio device
      async setDevice(deviceId) {
        return window.__TAURI__.invoke("plugin:audio:set_device", { deviceId });
      },
      
      // Event subscriptions
      onLevelChange(callback) {
        listeners["level-change"].push(callback);
        return () => {
          const index = listeners["level-change"].indexOf(callback);
          if (index !== -1) listeners["level-change"].splice(index, 1);
        };
      },
      
      onRecordingStarted(callback) {
        listeners["recording-started"].push(callback);
        return () => {
          const index = listeners["recording-started"].indexOf(callback);
          if (index !== -1) listeners["recording-started"].splice(index, 1);
        };
      },
      
      onRecordingStopped(callback) {
        listeners["recording-stopped"].push(callback);
        return () => {
          const index = listeners["recording-stopped"].indexOf(callback);
          if (index !== -1) listeners["recording-stopped"].splice(index, 1);
        };
      },
      
      onError(callback) {
        listeners["error"].push(callback);
        return () => {
          const index = listeners["error"].indexOf(callback);
          if (index !== -1) listeners["error"].splice(index, 1);
        };
      }
    };
  }
}; 
