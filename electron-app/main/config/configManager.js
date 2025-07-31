const Store = require('electron-store').default;
const { app } = require('electron');
const path = require('path');

class ConfigManager {
  constructor() {
    this.store = new Store({
      name: 'bestme-config',
      defaults: {
        version: '0.1.0',
        general: {
          theme: 'system',
          offline_mode: false,
          auto_start_recording: true,
          minimize_to_tray: true
        },
        audio: {
          input_device: null, // Auto-detect
          speech: {
            model_size: 'small',
            language: 'auto',
            translate_to_english: false,
            auto_punctuate: true,
            context_formatting: true,
            model_path: null
          },
          voice_commands: {
            enabled: false,
            prefix: 'computer',
            require_prefix: true
          }
        },
        ai: {
          requesty_api_key: null,
          chat_model: 'gemini-1.5-pro-latest',
          providers: {
            openai: { api_key: null, enabled: false },
            anthropic: { api_key: null, enabled: false },
            google: { api_key: null, enabled: false }
          }
        },
        whisper_params: {
          temperature: 0.0,
          vad_enabled: true,
          vad_threshold: 0.6,
          beam_size: 0,
          no_speech_threshold: 0.6,
          min_speech_duration_ms: 250,
          max_silence_duration_ms: 2000,
          initial_prompt: null,
          patience: 1.0,
          best_of: 1
        }
      }
    });
  }
  
  async initialize() {
    console.log('ConfigManager initialized');
    console.log('Config file location:', this.store.path);
    
    // Ensure config directory exists
    const configDir = path.dirname(this.store.path);
    const fs = require('fs').promises;
    try {
      await fs.mkdir(configDir, { recursive: true });
    } catch (error) {
      console.error('Failed to create config directory:', error);
    }
    
    // Migrate old settings if needed
    await this.migrateSettings();
  }
  
  async migrateSettings() {
    const currentVersion = this.store.get('version', '0.0.0');
    
    if (currentVersion !== '0.1.0') {
      console.log(`Migrating settings from ${currentVersion} to 0.1.0`);
      
      // Perform any necessary migrations here
      // For now, just update the version
      this.store.set('version', '0.1.0');
    }
  }
  
  getSettings() {
    return this.store.store;
  }
  
  getSetting(key, defaultValue = null) {
    return this.store.get(key, defaultValue);
  }
  
  async updateSettings(newSettings) {
    try {
      // Merge with existing settings
      const currentSettings = this.getSettings();
      const mergedSettings = this.deepMerge(currentSettings, newSettings);
      
      // Validate settings before saving
      this.validateSettings(mergedSettings);
      
      // Save to store
      Object.keys(mergedSettings).forEach(key => {
        this.store.set(key, mergedSettings[key]);
      });
      
      console.log('Settings updated successfully');
      return true;
    } catch (error) {
      console.error('Failed to update settings:', error);
      throw error;
    }
  }
  
  setSetting(key, value) {
    this.store.set(key, value);
  }
  
  deleteSetting(key) {
    this.store.delete(key);
  }
  
  resetToDefaults() {
    this.store.clear();
    console.log('Settings reset to defaults');
  }
  
  validateSettings(settings) {
    // Validate critical settings
    if (settings.audio?.speech?.model_size) {
      const validModels = ['tiny', 'base', 'small', 'medium', 'large'];
      if (!validModels.includes(settings.audio.speech.model_size)) {
        throw new Error(`Invalid model size: ${settings.audio.speech.model_size}`);
      }
    }
    
    if (settings.whisper_params?.temperature !== undefined) {
      const temp = settings.whisper_params.temperature;
      if (temp < 0 || temp > 2) {
        throw new Error(`Invalid temperature: ${temp}. Must be between 0 and 2.`);
      }
    }
    
    return true;
  }
  
  deepMerge(target, source) {
    const result = { ...target };
    
    for (const key in source) {
      if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
        result[key] = this.deepMerge(target[key] || {}, source[key]);
      } else {
        result[key] = source[key];
      }
    }
    
    return result;
  }
  
  // Convenience methods for common settings
  getModelSize() {
    return this.getSetting('audio.speech.model_size', 'small');
  }
  
  setModelSize(size) {
    this.setSetting('audio.speech.model_size', size);
  }
  
  getLanguage() {
    return this.getSetting('audio.speech.language', 'auto');
  }
  
  setLanguage(language) {
    this.setSetting('audio.speech.language', language);
  }
  
  isVoiceCommandsEnabled() {
    return this.getSetting('audio.voice_commands.enabled', false);
  }
  
  setVoiceCommandsEnabled(enabled) {
    this.setSetting('audio.voice_commands.enabled', enabled);
  }
  
  getTheme() {
    return this.getSetting('general.theme', 'system');
  }
  
  setTheme(theme) {
    this.setSetting('general.theme', theme);
  }
  
  // Get app data directory
  getAppDataDir() {
    return app.getPath('userData');
  }
  
  // Get models directory
  getModelsDir() {
    const customPath = this.getSetting('audio.speech.model_path');
    if (customPath) {
      return path.resolve(customPath);
    }
    return path.join(this.getAppDataDir(), 'models');
  }
  
  // Export settings to JSON file
  async exportSettings(filePath) {
    const fs = require('fs').promises;
    const settings = this.getSettings();
    
    try {
      await fs.writeFile(filePath, JSON.stringify(settings, null, 2));
      console.log(`Settings exported to: ${filePath}`);
      return true;
    } catch (error) {
      console.error('Failed to export settings:', error);
      throw error;
    }
  }
  
  // Import settings from JSON file
  async importSettings(filePath) {
    const fs = require('fs').promises;
    
    try {
      const data = await fs.readFile(filePath, 'utf8');
      const importedSettings = JSON.parse(data);
      
      // Validate before importing
      this.validateSettings(importedSettings);
      
      await this.updateSettings(importedSettings);
      console.log(`Settings imported from: ${filePath}`);
      return true;
    } catch (error) {
      console.error('Failed to import settings:', error);
      throw error;
    }
  }
}

module.exports = ConfigManager;