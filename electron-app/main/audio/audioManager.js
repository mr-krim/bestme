const mic = require('mic');
const { EventEmitter } = require('events');

class AudioManager extends EventEmitter {
  constructor() {
    super();
    this.micInstance = null;
    this.isRecordingActive = false;
    this.currentDevice = null;
    this.peakLevel = 0;
    this.audioBuffer = [];
    this.configManager = null;
  }
  
  async initialize(configManager) {
    this.configManager = configManager;
    console.log('AudioManager initialized');
    
    // Get preferred device from settings
    const preferredDevice = this.configManager.getSetting('audio.input_device');
    if (preferredDevice) {
      await this.setDevice(preferredDevice);
    }
  }
  
  async getDevices() {
    try {
      // For simplicity, return default device info
      // In a real implementation, you'd enumerate actual audio devices
      return [
        {
          id: 'default',
          name: 'Default Microphone',
          is_default: true
        }
      ];
    } catch (error) {
      console.error('Failed to get audio devices:', error);
      return [];
    }
  }
  
  async setDevice(deviceId) {
    try {
      this.currentDevice = deviceId;
      console.log(`Audio device set to: ${deviceId}`);
      
      // Update settings
      if (this.configManager) {
        this.configManager.setSetting('audio.input_device', deviceId);
      }
      
      return true;
    } catch (error) {
      console.error('Failed to set audio device:', error);
      throw error;
    }
  }
  
  async startRecording() {
    if (this.isRecordingActive) {
      console.log('Recording already active');
      return true;
    }
    
    try {
      console.log('Starting audio recording...');
      
      // Configure microphone
      const micOptions = {
        rate: 16000,        // Sample rate
        channels: 1,        // Mono
        debug: false,
        exitOnSilence: 6    // Exit after 6 seconds of silence
      };
      
      this.micInstance = mic(micOptions);
      const micInputStream = this.micInstance.getAudioStream();
      
      // Handle audio data
      micInputStream.on('data', (data) => {
        this.audioBuffer.push(data);
        this.updatePeakLevel(data);
        
        // Emit audio data for transcription
        this.emit('audioData', data);
      });
      
      micInputStream.on('error', (error) => {
        console.error('Microphone error:', error);
        this.emit('error', error);
      });
      
      micInputStream.on('startComplete', () => {
        console.log('Microphone started successfully');
        this.isRecordingActive = true;
        this.emit('recordingStarted');
      });
      
      micInputStream.on('stopComplete', () => {
        console.log('Microphone stopped');
        this.isRecordingActive = false;
        this.emit('recordingStopped');
      });
      
      // Start the microphone
      this.micInstance.start();
      
      return true;
    } catch (error) {
      console.error('Failed to start recording:', error);
      throw error;
    }
  }
  
  async stopRecording() {
    if (!this.isRecordingActive) {
      console.log('No active recording to stop');
      return true;
    }
    
    try {
      console.log('Stopping audio recording...');
      
      if (this.micInstance) {
        this.micInstance.stop();
        this.micInstance = null;
      }
      
      this.isRecordingActive = false;
      this.audioBuffer = [];
      this.peakLevel = 0;
      
      return true;
    } catch (error) {
      console.error('Failed to stop recording:', error);
      throw error;
    }
  }
  
  isRecording() {
    return this.isRecordingActive;
  }
  
  getPeakLevel() {
    return this.peakLevel;
  }
  
  updatePeakLevel(audioData) {
    if (!audioData || audioData.length === 0) {
      this.peakLevel = 0;
      return;
    }
    
    // Calculate RMS (Root Mean Square) for peak level
    let sum = 0;
    for (let i = 0; i < audioData.length; i += 2) {
      // Convert 16-bit PCM to float
      const sample = audioData.readInt16LE(i) / 32768.0;
      sum += sample * sample;
    }
    
    const rms = Math.sqrt(sum / (audioData.length / 2));
    this.peakLevel = Math.min(1.0, rms * 10); // Scale and clamp to [0, 1]
  }
  
  getAudioBuffer() {
    return Buffer.concat(this.audioBuffer);
  }
  
  clearAudioBuffer() {
    this.audioBuffer = [];
  }
  
  // Get current recording statistics
  getRecordingStats() {
    const totalSamples = this.audioBuffer.reduce((sum, buf) => sum + buf.length, 0) / 2;
    const durationSeconds = totalSamples / 16000; // 16kHz sample rate
    
    return {
      isRecording: this.isRecordingActive,
      duration: durationSeconds,
      peakLevel: this.peakLevel,
      bufferSize: this.audioBuffer.length,
      device: this.currentDevice
    };
  }
  
  // Test microphone access
  async testMicrophone() {
    try {
      const testMic = mic({ rate: 16000, channels: 1, debug: false });
      const testStream = testMic.getAudioStream();
      
      return new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          testMic.stop();
          reject(new Error('Microphone test timeout'));
        }, 3000);
        
        testStream.on('data', () => {
          clearTimeout(timeout);
          testMic.stop();
          resolve(true);
        });
        
        testStream.on('error', (error) => {
          clearTimeout(timeout);
          reject(error);
        });
        
        testMic.start();
      });
    } catch (error) {
      console.error('Microphone test failed:', error);
      return false;
    }
  }
}

module.exports = AudioManager;