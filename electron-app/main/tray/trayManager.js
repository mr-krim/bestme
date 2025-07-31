const { Tray, Menu, nativeImage, Notification } = require('electron');

class TrayManager {
  constructor(mainWindow, iconPath) {
    this.mainWindow = mainWindow;
    this.iconPath = iconPath;
    this.tray = null;
    this.hasShownMinimizeNotification = false;
  }
  
  async initialize() {
    try {
      console.log('Creating system tray...');
      
      // Create tray icon
      let trayIcon;
      if (typeof this.iconPath === 'string') {
        trayIcon = nativeImage.createFromPath(this.iconPath);
      } else {
        trayIcon = this.iconPath; // Already a nativeImage
      }
      
      // Resize icon for tray (platform specific)
      if (process.platform === 'darwin') {
        trayIcon = trayIcon.resize({ width: 16, height: 16 });
      } else if (process.platform === 'win32') {
        trayIcon = trayIcon.resize({ width: 16, height: 16 });
      }
      
      this.tray = new Tray(trayIcon);
      
      // Set tooltip
      this.tray.setToolTip('BestMe - Speech to Text');
      
      // Create context menu
      this.createContextMenu();
      
      // Handle tray click events
      this.setupTrayEvents();
      
      console.log('System tray created successfully');
    } catch (error) {
      console.error('Failed to create system tray:', error);
      throw error;
    }
  }
  
  createContextMenu() {
    const contextMenu = Menu.buildFromTemplate([
      {
        label: 'Show BestMe',
        click: () => this.showMainWindow()
      },
      {
        label: 'Hide BestMe',
        click: () => this.hideMainWindow()
      },
      { type: 'separator' },
      {
        label: 'Start Recording',
        id: 'start-recording',
        click: () => this.startRecording()
      },
      {
        label: 'Stop Recording',
        id: 'stop-recording',
        click: () => this.stopRecording(),
        enabled: false
      },
      { type: 'separator' },
      {
        label: 'Settings',
        click: () => this.openSettings()
      },
      {
        label: 'About',
        click: () => this.showAbout()
      },
      { type: 'separator' },
      {
        label: 'Quit BestMe',
        click: () => this.quitApplication()
      }
    ]);
    
    this.tray.setContextMenu(contextMenu);
  }
  
  setupTrayEvents() {
    // Double-click to show/hide window
    this.tray.on('double-click', () => {
      this.toggleMainWindow();
    });
    
    // Single click behavior (platform specific)
    if (process.platform === 'win32') {
      this.tray.on('click', () => {
        this.toggleMainWindow();
      });
    }
  }
  
  showMainWindow() {
    if (this.mainWindow) {
      if (this.mainWindow.isMinimized()) {
        this.mainWindow.restore();
      }
      this.mainWindow.show();
      this.mainWindow.focus();
    }
  }
  
  hideMainWindow() {
    if (this.mainWindow) {
      this.mainWindow.hide();
    }
  }
  
  toggleMainWindow() {
    if (this.mainWindow) {
      if (this.mainWindow.isVisible()) {
        this.hideMainWindow();
      } else {
        this.showMainWindow();
      }
    }
  }
  
  updateRecordingState(isRecording) {
    // Update context menu items based on recording state
    const contextMenu = this.tray.getContextMenu();
    if (contextMenu) {
      const startItem = contextMenu.getMenuItemById('start-recording');
      const stopItem = contextMenu.getMenuItemById('stop-recording');
      
      if (startItem && stopItem) {
        startItem.enabled = !isRecording;
        stopItem.enabled = isRecording;
      }
    }
    
    // Update tray icon or tooltip to indicate recording state
    if (isRecording) {
      this.tray.setToolTip('BestMe - Recording...');
      
      // Show notification for recording started
      this.showNotification('Recording Started', 'BestMe is now recording audio');
    } else {
      this.tray.setToolTip('BestMe - Speech to Text');
      
      // Show notification for recording stopped
      this.showNotification('Recording Stopped', 'BestMe has stopped recording');
    }
  }
  
  startRecording() {
    console.log('Tray: Start recording requested');
    
    // Emit event to main window
    if (this.mainWindow) {
      this.mainWindow.webContents.send('tray-action', 'start-recording');
    }
  }
  
  stopRecording() {
    console.log('Tray: Stop recording requested');
    
    // Emit event to main window
    if (this.mainWindow) {
      this.mainWindow.webContents.send('tray-action', 'stop-recording');
    }
  }
  
  openSettings() {
    console.log('Tray: Open settings requested');
    
    // Show main window and navigate to settings
    this.showMainWindow();
    if (this.mainWindow) {
      this.mainWindow.webContents.send('tray-action', 'open-settings');
    }
  }
  
  showAbout() {
    const { dialog } = require('electron');
    
    dialog.showMessageBox(this.mainWindow, {
      type: 'info',
      title: 'About BestMe',
      message: 'BestMe v0.1.0',
      detail: 'Modern Speech-to-Text Application\\nBuilt with Electron and Whisper AI\\n\\nCopyright © 2025 BestMe Team'
    });
  }
  
  quitApplication() {
    console.log('Tray: Quit application requested');
    
    const { app } = require('electron');
    app.quit();
  }
  
  showMinimizeNotification() {
    if (!this.hasShownMinimizeNotification) {
      this.showNotification(
        'BestMe Minimized to Tray',
        'BestMe is still running in the background. Click the tray icon to show the window again.'
      );
      this.hasShownMinimizeNotification = true;
    }
  }
  
  showNotification(title, message, options = {}) {
    try {
      // Check if notifications are supported and permission is granted
      if (Notification.isSupported()) {
        const notification = new Notification({
          title: title,
          body: message,
          icon: this.iconPath,
          silent: options.silent || false,
          ...options
        });
        
        notification.show();
        
        // Handle notification click
        notification.on('click', () => {
          this.showMainWindow();
        });
        
        return notification;
      }
    } catch (error) {
      console.error('Failed to show notification:', error);
    }
    
    return null;
  }
  
  updateTranscriptionStatus(transcriptionData) {
    if (transcriptionData && transcriptionData.text) {
      const wordCount = transcriptionData.text.trim().split(/\\s+/).length;
      this.tray.setToolTip(`BestMe - ${wordCount} words transcribed`);
    }
  }
  
  destroy() {
    if (this.tray) {
      this.tray.destroy();
      this.tray = null;
      console.log('System tray destroyed');
    }
  }
  
  // Display a balloon notification (Windows specific)
  displayBalloon(title, content, icon = 'info') {
    if (this.tray && process.platform === 'win32') {
      this.tray.displayBalloon({
        title: title,
        content: content,
        icon: this.iconPath
      });
    }
  }
  
  // Flash the tray icon to get user attention
  flashTray(flash = true) {
    if (this.tray) {
      // This would require implementing icon switching for visual feedback
      // For now, just log the action
      console.log(`Tray flash: ${flash ? 'started' : 'stopped'}`);
    }
  }
}

module.exports = TrayManager;