// Voice Commands plugin initialization script
window.__TAURI_PLUGIN_VOICE_COMMANDS__ = {
  // Initialize the plugin
  init() {
    console.log("Initializing Voice Commands plugin");
    
    // Create event emitter for command events
    const listeners = {
      "command-detected": [],
      "command-error": [],
      "started": [],
      "stopped": [],
      "status-change": []
    };
    
    // Listen for events from the Rust side
    window.__TAURI__.event.listen("voice-command:detected", (event) => {
      const command = event.payload;
      listeners["command-detected"].forEach(cb => cb(command));
    });
    
    window.__TAURI__.event.listen("voice-command:error", (event) => {
      const error = event.payload;
      listeners["command-error"].forEach(cb => cb(error));
    });
    
    window.__TAURI__.event.listen("voice-command:started", () => {
      listeners["started"].forEach(cb => cb());
      listeners["status-change"].forEach(cb => cb(true));
    });
    
    window.__TAURI__.event.listen("voice-command:stopped", () => {
      listeners["stopped"].forEach(cb => cb());
      listeners["status-change"].forEach(cb => cb(false));
    });
    
    // Export API
    return {
      // Start voice command processing
      start() {
        return window.__TAURI__.event.emit("voice-command:start");
      },
      
      // Stop voice command processing
      stop() {
        return window.__TAURI__.event.emit("voice-command:stop");
      },
      
      // Update the current text for voice commands to process
      updateText(text) {
        return window.__TAURI__.event.emit("voice-command:update-text", text);
      },
      
      // Get the current status
      async getStatus() {
        return window.__TAURI__.invoke("plugin:voice_commands:get_voice_commands_status");
      },
      
      // Get the current text
      async getText() {
        return window.__TAURI__.invoke("plugin:voice_commands:get_voice_commands_text");
      },
      
      // Get command history
      async getHistory() {
        return window.__TAURI__.invoke("plugin:voice_commands:get_voice_commands_history");
      },
      
      // Event subscription
      onCommandDetected(callback) {
        listeners["command-detected"].push(callback);
        return () => {
          const index = listeners["command-detected"].indexOf(callback);
          if (index !== -1) listeners["command-detected"].splice(index, 1);
        };
      },
      
      onError(callback) {
        listeners["command-error"].push(callback);
        return () => {
          const index = listeners["command-error"].indexOf(callback);
          if (index !== -1) listeners["command-error"].splice(index, 1);
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
      
      onStatusChange(callback) {
        listeners["status-change"].push(callback);
        return () => {
          const index = listeners["status-change"].indexOf(callback);
          if (index !== -1) listeners["status-change"].splice(index, 1);
        };
      }
    };
  }
}; 
