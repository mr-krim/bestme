const { spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');
const { EventEmitter } = require('events');

class TranscriptionManager extends EventEmitter {
  constructor() {
    super();
    this.isTranscribingActive = false;
    this.currentTranscription = '';
    this.configManager = null;
    this.whisperProcess = null;
    this.tempAudioFile = null;
    this.audioManager = null;
  }
  
  async initialize(configManager) {
    this.configManager = configManager;
    console.log('TranscriptionManager initialized');
    
    // Ensure models directory exists
    const modelsDir = this.configManager.getModelsDir();
    try {
      await fs.mkdir(modelsDir, { recursive: true });
      console.log(`Models directory ready: ${modelsDir}`);
    } catch (error) {
      console.error('Failed to create models directory:', error);
    }
  }
  
  setAudioManager(audioManager) {
    this.audioManager = audioManager;
    
    // Listen for audio data from AudioManager
    this.audioManager.on('audioData', (audioData) => {
      if (this.isTranscribingActive) {
        this.processAudioData(audioData);
      }
    });
  }
  
  async startTranscription(options = {}) {
    if (this.isTranscribingActive) {
      console.log('Transcription already active');
      return true;
    }
    
    try {
      console.log('Starting transcription...');
      
      this.isTranscribingActive = true;
      this.currentTranscription = '';
      
      // Get transcription settings
      const modelSize = options.model || this.configManager.getModelSize();
      const language = options.language || this.configManager.getLanguage();
      
      console.log(`Using model: ${modelSize}, language: ${language}`);
      
      // Emit started event
      this.emit('transcriptionStarted');
      
      return true;
    } catch (error) {
      console.error('Failed to start transcription:', error);
      this.isTranscribingActive = false;
      throw error;
    }
  }
  
  async stopTranscription() {
    if (!this.isTranscribingActive) {
      console.log('No active transcription to stop');
      return true;
    }
    
    try {
      console.log('Stopping transcription...');
      
      if (this.whisperProcess) {
        this.whisperProcess.kill();
        this.whisperProcess = null;
      }
      
      // Clean up temp files
      if (this.tempAudioFile) {
        try {
          await fs.unlink(this.tempAudioFile);
        } catch (error) {
          console.warn('Failed to clean up temp audio file:', error);
        }
        this.tempAudioFile = null;
      }
      
      this.isTranscribingActive = false;
      
      // Emit stopped event
      this.emit('transcriptionStopped');
      
      return true;
    } catch (error) {
      console.error('Failed to stop transcription:', error);
      throw error;
    }
  }
  
  isTranscribing() {
    return this.isTranscribingActive;
  }
  
  getCurrentTranscription() {
    return this.currentTranscription;
  }
  
  clearTranscription() {
    this.currentTranscription = '';
    this.emit('transcriptionCleared');
  }
  
  async processAudioData(audioData) {
    if (!this.isTranscribingActive) {
      return;
    }
    
    // For this implementation, we'll use a simple placeholder
    // In a real implementation, you would:
    // 1. Buffer audio data
    // 2. Save to temporary WAV file when enough data is collected
    // 3. Run Whisper on the audio file
    // 4. Parse the output and emit transcription updates
    
    // Placeholder: simulate transcription with random text
    if (Math.random() < 0.1) { // 10% chance to add text
      const words = ['hello', 'world', 'this', 'is', 'a', 'test', 'transcription'];
      const word = words[Math.floor(Math.random() * words.length)];
      
      this.currentTranscription += word + ' ';
      this.emit('transcriptionUpdate', {
        text: this.currentTranscription,
        partial: true
      });
    }
  }
  
  async transcribeAudioFile(filePath, options = {}) {
    try {
      const modelSize = options.model || this.configManager.getModelSize();
      const language = options.language || this.configManager.getLanguage();
      
      // Check if Whisper is available
      const whisperCommand = await this.findWhisperCommand();
      if (!whisperCommand) {
        throw new Error('Whisper not found. Please install OpenAI Whisper.');
      }
      
      // Prepare Whisper arguments
      const args = [
        filePath,
        '--model', modelSize,
        '--output_format', 'json'
      ];
      
      if (language !== 'auto') {
        args.push('--language', language);
      }
      
      // Run Whisper
      return new Promise((resolve, reject) => {
        const whisperProcess = spawn(whisperCommand, args);
        let output = '';
        let error = '';
        
        whisperProcess.stdout.on('data', (data) => {
          output += data.toString();
        });
        
        whisperProcess.stderr.on('data', (data) => {
          error += data.toString();
        });
        
        whisperProcess.on('close', (code) => {
          if (code === 0) {
            try {
              const result = JSON.parse(output);
              resolve(result.text || '');
            } catch (parseError) {
              // Fallback to plain text output
              resolve(output.trim());
            }
          } else {
            reject(new Error(`Whisper failed with code ${code}: ${error}`));
          }
        });
        
        whisperProcess.on('error', (err) => {
          reject(err);
        });
      });
    } catch (error) {
      console.error('Failed to transcribe audio file:', error);
      throw error;
    }
  }
  
  async findWhisperCommand() {
    const commands = ['whisper', 'python -m whisper', 'python3 -m whisper'];
    
    for (const cmd of commands) {
      try {
        const testProcess = spawn(cmd.split(' ')[0], ['--help'], { stdio: 'pipe' });
        await new Promise((resolve, reject) => {
          testProcess.on('close', (code) => {
            if (code === 0) resolve();
            else reject();
          });
          testProcess.on('error', reject);
        });
        return cmd;
      } catch (error) {
        continue;
      }
    }
    
    return null;
  }
  
  getAvailableModels() {
    return [
      { id: 'tiny', name: 'Tiny (~39 MB)', size: '39 MB' },
      { id: 'base', name: 'Base (~74 MB)', size: '74 MB' },
      { id: 'small', name: 'Small (~244 MB)', size: '244 MB' },
      { id: 'medium', name: 'Medium (~769 MB)', size: '769 MB' },
      { id: 'large', name: 'Large (~1550 MB)', size: '1550 MB' }
    ];
  }
  
  getSupportedLanguages() {
    return [
      { code: 'auto', name: 'Auto-detect' },
      { code: 'en', name: 'English' },
      { code: 'es', name: 'Spanish' },
      { code: 'fr', name: 'French' },
      { code: 'de', name: 'German' },
      { code: 'it', name: 'Italian' },
      { code: 'pt', name: 'Portuguese' },
      { code: 'ru', name: 'Russian' },
      { code: 'ja', name: 'Japanese' },
      { code: 'ko', name: 'Korean' },
      { code: 'zh', name: 'Chinese' }
    ];
  }
  
  // Get current transcription statistics
  getTranscriptionStats() {
    const wordCount = this.currentTranscription.trim().split(/\s+/).filter(word => word.length > 0).length;
    
    return {
      isTranscribing: this.isTranscribingActive,
      currentText: this.currentTranscription,
      wordCount: wordCount,
      characterCount: this.currentTranscription.length
    };
  }
  
  // Save current transcription to a file
  async saveTranscriptionToFile(filePath) {
    try {
      await fs.writeFile(filePath, this.currentTranscription, 'utf8');
      console.log(`Transcription saved to: ${filePath}`);
      return true;
    } catch (error) {
      console.error('Failed to save transcription:', error);
      throw error;
    }
  }
  
  // Load transcription from a file
  async loadTranscriptionFromFile(filePath) {
    try {
      const content = await fs.readFile(filePath, 'utf8');
      this.currentTranscription = content;
      this.emit('transcriptionUpdate', {
        text: this.currentTranscription,
        partial: false
      });
      console.log(`Transcription loaded from: ${filePath}`);
      return true;
    } catch (error) {
      console.error('Failed to load transcription:', error);
      throw error;
    }
  }
}

module.exports = TranscriptionManager;