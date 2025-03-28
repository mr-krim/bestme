// Transcription plugin initialization script
window.__TAURI_PLUGIN_TRANSCRIBE__ = {
  // Initialize the plugin
  init() {
    console.log("Initializing Transcription plugin");
    
    // Create event emitter for transcription events
    const listeners = {
      "update": [],
      "clear": [],
      "started": [],
      "stopped": [],
      "error": [],
      "download-progress": [],
      "download-complete": []
    };
    
    // Listen for events from the Rust side
    window.__TAURI__.event.listen("transcription:update", (event) => {
      const text = event.payload;
      listeners["update"].forEach(cb => cb(text));
    });
    
    window.__TAURI__.event.listen("transcription:clear", () => {
      listeners["clear"].forEach(cb => cb());
    });
    
    window.__TAURI__.event.listen("transcribe:started", () => {
      listeners["started"].forEach(cb => cb());
    });
    
    window.__TAURI__.event.listen("transcribe:stopped", () => {
      listeners["stopped"].forEach(cb => cb());
    });
    
    window.__TAURI__.event.listen("transcribe:error", (event) => {
      const error = event.payload;
      listeners["error"].forEach(cb => cb(error));
    });
    
    window.__TAURI__.event.listen("transcribe:download-progress", (event) => {
      const progress = event.payload;
      listeners["download-progress"].forEach(cb => cb(progress));
    });
    
    window.__TAURI__.event.listen("transcribe:download-complete", (event) => {
      const model = event.payload;
      listeners["download-complete"].forEach(cb => cb(model));
    });
    
    // Export API
    return {
      // Start transcription
      async startTranscription(options) {
        return window.__TAURI__.invoke("plugin:transcribe:start_transcription", { options });
      },
      
      // Stop transcription
      async stopTranscription() {
        return window.__TAURI__.invoke("plugin:transcribe:stop_transcription");
      },
      
      // Get current transcription
      async getTranscription() {
        return window.__TAURI__.invoke("plugin:transcribe:get_transcription");
      },
      
      // Check if transcription is active
      async isTranscribing() {
        return window.__TAURI__.invoke("plugin:transcribe:is_transcribing");
      },
      
      // Clear current transcription
      async clearTranscription() {
        return window.__TAURI__.invoke("plugin:transcribe:clear_transcription");
      },
      
      // Get download progress of the model
      async getDownloadProgress() {
        return window.__TAURI__.invoke("plugin:transcribe:get_download_progress");
      },
      
      // Download a model
      async downloadModel(modelSize) {
        return window.__TAURI__.invoke("plugin:transcribe:download_model_command", { modelSize });
      },
      
      // Check if a model is downloaded
      async isModelDownloaded(modelSize) {
        return window.__TAURI__.invoke("plugin:transcribe:is_model_downloaded", { modelSize });
      },
      
      // Event subscriptions
      onUpdate(callback) {
        listeners["update"].push(callback);
        return () => {
          const index = listeners["update"].indexOf(callback);
          if (index !== -1) listeners["update"].splice(index, 1);
        };
      },
      
      onClear(callback) {
        listeners["clear"].push(callback);
        return () => {
          const index = listeners["clear"].indexOf(callback);
          if (index !== -1) listeners["clear"].splice(index, 1);
        };
      },
      
      onStarted(callback) {
        listeners["started"].push(callback);
        return () => {
          const index = listeners["started"].indexOf(callback);
          if (index !== -1) listeners["started"].splice(index, 1);
        };
      },
      
      onStopped(callback) {
        listeners["stopped"].push(callback);
        return () => {
          const index = listeners["stopped"].indexOf(callback);
          if (index !== -1) listeners["stopped"].splice(index, 1);
        };
      },
      
      onError(callback) {
        listeners["error"].push(callback);
        return () => {
          const index = listeners["error"].indexOf(callback);
          if (index !== -1) listeners["error"].splice(index, 1);
        };
      },
      
      onDownloadProgress(callback) {
        listeners["download-progress"].push(callback);
        return () => {
          const index = listeners["download-progress"].indexOf(callback);
          if (index !== -1) listeners["download-progress"].splice(index, 1);
        };
      },
      
      onDownloadComplete(callback) {
        listeners["download-complete"].push(callback);
        return () => {
          const index = listeners["download-complete"].indexOf(callback);
          if (index !== -1) listeners["download-complete"].splice(index, 1);
        };
      }
    };
  }
}; 
