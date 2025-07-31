const { app, BrowserWindow, ipcMain, Menu, Tray, nativeImage } = require('electron');
const path = require('path');
const isDev = process.env.NODE_ENV === 'development';

// Import our main modules
const AudioManager = require('./audio/audioManager');
const TranscriptionManager = require('./transcription/transcriptionManager');
const DatabaseManager = require('./database/databaseManager');
const ConfigManager = require('./config/configManager');
const TrayManager = require('./tray/trayManager');
const TextInjection = require('./textInjection/textInjection');

class BestMeApp {
  constructor() {
    this.mainWindow = null;
    this.trayManager = null;
    this.isQuitting = false;
    
    // Initialize managers
    this.configManager = new ConfigManager();
    this.databaseManager = new DatabaseManager();
    this.audioManager = new AudioManager();
    this.transcriptionManager = new TranscriptionManager();
    this.textInjection = new TextInjection();
    
    this.setupApp();
  }
  
  setupApp() {
    // Set up app event handlers
    app.whenReady().then(() => this.onReady());
    app.on('window-all-closed', () => this.onWindowAllClosed());
    app.on('activate', () => this.onActivate());
    app.on('before-quit', () => { this.isQuitting = true; });
    
    // Set up IPC handlers
    this.setupIpcHandlers();
  }
  
  async onReady() {
    console.log('BestMe Electron app starting...');
    
    // Initialize managers
    await this.initializeManagers();
    
    // Create main window
    this.createMainWindow();
    
    // Create system tray
    await this.createTray();
    
    // Set up menu
    this.createMenu();
    
    console.log('BestMe app ready!');
  }
  
  async initializeManagers() {
    try {
      await this.configManager.initialize();
      await this.databaseManager.initialize();
      await this.audioManager.initialize(this.configManager);
      await this.transcriptionManager.initialize(this.configManager);
      
      console.log('All managers initialized successfully');
    } catch (error) {
      console.error('Failed to initialize managers:', error);
      throw error;
    }
  }
  
  createMainWindow() {
    // Create the browser window
    this.mainWindow = new BrowserWindow({
      width: 960,
      height: 720,
      minWidth: 800,
      minHeight: 600,
      show: false, // Don't show until ready
      icon: this.getAppIcon(),
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
        preload: path.join(__dirname, '../preload/preload.js'),
        webSecurity: !isDev
      }
    });
    
    // Load the app
    this.mainWindow.loadFile(path.join(__dirname, '../renderer/dist/index.html'));
    
    // Always open DevTools for debugging
    this.mainWindow.webContents.openDevTools();
    
    // Handle window events
    this.mainWindow.once('ready-to-show', () => {
      this.mainWindow.show();
      console.log('Main window shown');
    });
    
    this.mainWindow.on('close', (event) => {
      if (!this.isQuitting) {
        event.preventDefault();
        this.mainWindow.hide();
        
        // Show tray notification on first minimize
        if (this.trayManager) {
          this.trayManager.showMinimizeNotification();
        }
      }
    });
    
    this.mainWindow.on('closed', () => {
      this.mainWindow = null;
    });
  }
  
  async createTray() {
    try {
      this.trayManager = new TrayManager(this.mainWindow, this.getAppIcon());
      await this.trayManager.initialize();
      console.log('System tray created');
    } catch (error) {
      console.error('Failed to create system tray:', error);
    }
  }
  
  createMenu() {
    const template = [
      {
        label: 'File',
        submenu: [
          {
            label: 'New Transcription',
            accelerator: 'CmdOrCtrl+N',
            click: () => this.startNewTranscription()
          },
          {
            label: 'Save Transcription',
            accelerator: 'CmdOrCtrl+S',
            click: () => this.saveCurrentTranscription()
          },
          { type: 'separator' },
          {
            label: 'Exit',
            accelerator: process.platform === 'darwin' ? 'Cmd+Q' : 'Ctrl+Q',
            click: () => {
              this.isQuitting = true;
              app.quit();
            }
          }
        ]
      },
      {
        label: 'Recording',
        submenu: [
          {
            label: 'Start Recording',
            accelerator: 'F1',
            click: () => this.startRecording()
          },
          {
            label: 'Stop Recording',
            accelerator: 'F2',
            click: () => this.stopRecording()
          },
          { type: 'separator' },
          {
            label: 'Toggle Voice Commands',
            type: 'checkbox',
            click: (menuItem) => this.toggleVoiceCommands(menuItem.checked)
          }
        ]
      },
      {
        label: 'View',
        submenu: [
          { role: 'reload' },
          { role: 'forceReload' },
          { role: 'toggleDevTools' },
          { type: 'separator' },
          { role: 'resetZoom' },
          { role: 'zoomIn' },
          { role: 'zoomOut' },
          { type: 'separator' },
          { role: 'togglefullscreen' }
        ]
      },
      {
        label: 'Window',
        submenu: [
          { role: 'minimize' },
          { role: 'close' }
        ]
      },
      {
        label: 'Help',
        submenu: [
          {
            label: 'About BestMe',
            click: () => this.showAbout()
          },
          {
            label: 'Check for Updates',
            click: () => this.checkForUpdates()
          }
        ]
      }
    ];
    
    // macOS specific menu adjustments
    if (process.platform === 'darwin') {
      template.unshift({
        label: app.getName(),
        submenu: [
          { role: 'about' },
          { type: 'separator' },
          { role: 'services' },
          { type: 'separator' },
          { role: 'hide' },
          { role: 'hideOthers' },
          { role: 'unhide' },
          { type: 'separator' },
          { role: 'quit' }
        ]
      });
      
      // Window menu adjustments
      template[4].submenu = [
        { role: 'close' },
        { role: 'minimize' },
        { role: 'zoom' },
        { type: 'separator' },
        { role: 'front' }
      ];
    }
    
    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
  }
  
  setupIpcHandlers() {
    // Audio commands
    ipcMain.handle('start-recording', async () => {
      return await this.audioManager.startRecording();
    });
    
    ipcMain.handle('stop-recording', async () => {
      return await this.audioManager.stopRecording();
    });
    
    ipcMain.handle('is-recording', () => {
      return this.audioManager.isRecording();
    });
    
    ipcMain.handle('get-audio-devices', async () => {
      return await this.audioManager.getDevices();
    });
    
    ipcMain.handle('set-audio-device', async (event, deviceId) => {
      return await this.audioManager.setDevice(deviceId);
    });
    
    ipcMain.handle('get-peak-level', () => {
      return this.audioManager.getPeakLevel();
    });
    
    // Transcription commands
    ipcMain.handle('start-transcription', async (event, options) => {
      return await this.transcriptionManager.startTranscription(options);
    });
    
    ipcMain.handle('stop-transcription', async () => {
      return await this.transcriptionManager.stopTranscription();
    });
    
    ipcMain.handle('get-transcription', () => {
      return this.transcriptionManager.getCurrentTranscription();
    });
    
    ipcMain.handle('is-transcribing', () => {
      return this.transcriptionManager.isTranscribing();
    });
    
    ipcMain.handle('clear-transcription', () => {
      return this.transcriptionManager.clearTranscription();
    });
    
    ipcMain.handle('get-whisper-models', () => {
      return this.transcriptionManager.getAvailableModels();
    });
    
    ipcMain.handle('get-supported-languages', () => {
      return this.transcriptionManager.getSupportedLanguages();
    });
    
    // Database/Storage commands
    ipcMain.handle('save-transcript', async (event, title, content) => {
      return await this.databaseManager.saveTranscript(title, content);
    });
    
    ipcMain.handle('list-saved-transcripts', async () => {
      return await this.databaseManager.getSavedTranscripts();
    });
    
    ipcMain.handle('get-saved-transcript', async (event, id) => {
      return await this.databaseManager.getTranscript(id);
    });
    
    ipcMain.handle('delete-saved-transcript', async (event, id) => {
      return await this.databaseManager.deleteTranscript(id);
    });
    
    // Chat commands
    ipcMain.handle('get-chat-session-list', async () => {
      return await this.databaseManager.getChatSessions();
    });
    
    ipcMain.handle('get-chat-session', async (event, id) => {
      return await this.databaseManager.getChatSession(id);
    });
    
    ipcMain.handle('send-chat-message', async (event, sessionId, message) => {
      // This would integrate with AI service
      return await this.handleChatMessage(sessionId, message);
    });
    
    ipcMain.handle('delete-chat-session', async (event, id) => {
      return await this.databaseManager.deleteChatSession(id);
    });
    
    // Voice commands
    ipcMain.handle('get-voice-command-list', () => {
      return this.getVoiceCommands();
    });
    
    // Settings
    ipcMain.handle('get-settings', () => {
      return this.configManager.getSettings();
    });
    
    ipcMain.handle('update-settings', async (event, settings) => {
      return await this.configManager.updateSettings(settings);
    });
    
    // System commands
    ipcMain.handle('system.get-cpu-usage', () => {
      return this.getSystemCpuUsage();
    });
    
    ipcMain.handle('system.get-memory-usage', () => {
      return this.getSystemMemoryUsage();
    });
    
    ipcMain.handle('system.get-online-status', () => {
      return navigator.onLine;
    });
    
    // Text injection
    ipcMain.handle('inject-text', async (event, text) => {
      return await this.textInjection.injectText(text);
    });
  }
  
  // Helper methods
  getAppIcon() {
    const iconPath = path.join(__dirname, '../assets', 
      process.platform === 'win32' ? 'icon.ico' :
      process.platform === 'darwin' ? 'icon.icns' : 'icon.png'
    );
    return nativeImage.createFromPath(iconPath);
  }
  
  onWindowAllClosed() {
    // On macOS, keep app running even when all windows are closed
    if (process.platform !== 'darwin') {
      app.quit();
    }
  }
  
  onActivate() {
    // On macOS, re-create window when dock icon is clicked
    if (BrowserWindow.getAllWindows().length === 0) {
      this.createMainWindow();
    } else if (this.mainWindow) {
      this.mainWindow.show();
    }
  }
  
  // Menu action handlers
  async startNewTranscription() {
    if (this.mainWindow) {
      this.mainWindow.webContents.send('menu-action', 'new-transcription');
    }
  }
  
  async saveCurrentTranscription() {
    if (this.mainWindow) {
      this.mainWindow.webContents.send('menu-action', 'save-transcription');
    }
  }
  
  async startRecording() {
    await this.audioManager.startRecording();
    if (this.mainWindow) {
      this.mainWindow.webContents.send('recording-state-changed', true);
    }
  }
  
  async stopRecording() {
    await this.audioManager.stopRecording();
    if (this.mainWindow) {
      this.mainWindow.webContents.send('recording-state-changed', false);
    }
  }
  
  async toggleVoiceCommands(enabled) {
    // Implementation for voice commands toggle
    console.log('Voice commands:', enabled ? 'enabled' : 'disabled');
  }
  
  showAbout() {
    // Show about dialog
    const { dialog } = require('electron');
    dialog.showMessageBox(this.mainWindow, {
      type: 'info',
      title: 'About BestMe',
      message: 'BestMe v0.1.0',
      detail: 'Modern Speech-to-Text Application\\nBuilt with Electron and Whisper AI'
    });
  }
  
  checkForUpdates() {
    console.log('Checking for updates...');
    // Implementation for update checking
  }
  
  getVoiceCommands() {
    return [
      {
        id: 'start_recording',
        name: 'Start Recording',
        command: 'start recording',
        description: 'Begins audio recording and transcription'
      },
      {
        id: 'stop_recording',
        name: 'Stop Recording', 
        command: 'stop recording',
        description: 'Stops audio recording and transcription'
      },
      {
        id: 'new_paragraph',
        name: 'New Paragraph',
        command: 'new paragraph',
        description: 'Inserts a new paragraph break'
      }
    ];
  }
  
  async handleChatMessage(sessionId, message) {
    // Placeholder for AI chat integration
    const response = {
      id: `msg-${Date.now()}`,
      sender: 'assistant',
      text: `Echo: ${message}`,
      timestamp: Date.now()
    };
    
    return [sessionId || `session-${Date.now()}`, response];
  }
  
  getSystemCpuUsage() {
    // Placeholder implementation
    return Math.random() * 100;
  }
  
  getSystemMemoryUsage() {
    const used = process.memoryUsage();
    return (used.heapUsed / used.heapTotal) * 100;
  }
}

// Create and start the app
new BestMeApp();