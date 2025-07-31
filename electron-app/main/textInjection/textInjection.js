// Text injection using clipboard - simplified implementation without robotjs
const { clipboard } = require('electron');

class TextInjection {
  constructor() {
    this.isEnabled = true;
    this.lastInjectionTime = 0;
    this.injectionDelay = 100; // Minimum delay between injections (ms)
  }
  
  async initialize() {
    console.log('TextInjection initialized (using clipboard)');
  }
  
  async injectText(text) {
    if (!this.isEnabled) {
      console.log('Text injection is disabled');
      return false;
    }
    
    if (!text || text.length === 0) {
      console.log('No text to inject');
      return false;
    }
    
    // Rate limiting
    const now = Date.now();
    if (now - this.lastInjectionTime < this.injectionDelay) {
      console.log('Text injection rate limited');
      return false;
    }
    
    try {
      console.log(`Injecting text: "${text.substring(0, 50)}${text.length > 50 ? '...' : ''}"`);
      
      // Small delay to ensure the target application is ready
      await this.sleep(50);
      
      // Type the text
      robot.typeString(text);
      
      this.lastInjectionTime = now;
      
      console.log('Text injected successfully');
      return true;
    } catch (error) {
      console.error('Failed to inject text:', error);
      throw error;
    }
  }
  
  async injectTextWithFormatting(text, options = {}) {
    if (!this.isEnabled) {
      return false;
    }
    
    try {
      const {
        addNewline = false,
        addParagraph = false,
        replaceSelection = false,
        appendMode = false
      } = options;
      
      // If replacing selection, select all first
      if (replaceSelection) {
        robot.keyTap('a', ['control']); // Ctrl+A
        await this.sleep(10);
      }
      
      // If append mode, move cursor to end
      if (appendMode) {
        robot.keyTap('end', ['control']); // Ctrl+End
        await this.sleep(10);
      }
      
      // Add paragraph break if requested
      if (addParagraph) {
        robot.keyTap('enter');
        robot.keyTap('enter');
        await this.sleep(10);
      }
      
      // Type the text
      robot.typeString(text);
      
      // Add newline if requested
      if (addNewline) {
        robot.keyTap('enter');
      }
      
      console.log('Formatted text injected successfully');
      return true;
    } catch (error) {
      console.error('Failed to inject formatted text:', error);
      throw error;
    }
  }
  
  async injectCommand(command) {
    if (!this.isEnabled) {
      return false;
    }
    
    try {
      console.log(`Executing text injection command: ${command}`);
      
      switch (command.toLowerCase()) {
        case 'new_paragraph':
        case 'new paragraph':
          robot.keyTap('enter');
          robot.keyTap('enter');
          break;
          
        case 'new_line':
        case 'new line':
          robot.keyTap('enter');
          break;
          
        case 'backspace':
          robot.keyTap('backspace');
          break;
          
        case 'delete':
          robot.keyTap('delete');
          break;
          
        case 'select_all':
        case 'select all':
          robot.keyTap('a', ['control']);
          break;
          
        case 'copy':
          robot.keyTap('c', ['control']);
          break;
          
        case 'paste':
          robot.keyTap('v', ['control']);
          break;
          
        case 'undo':
          robot.keyTap('z', ['control']);
          break;
          
        case 'redo':
          robot.keyTap('y', ['control']);
          break;
          
        default:
          console.warn(`Unknown command: ${command}`);
          return false;
      }
      
      return true;
    } catch (error) {
      console.error('Failed to execute text injection command:', error);
      throw error;
    }
  }
  
  async simulateKeyPress(key, modifiers = []) {
    if (!this.isEnabled) {
      return false;
    }
    
    try {
      console.log(`Simulating key press: ${key} with modifiers: ${modifiers.join(', ')}`);
      
      robot.keyTap(key, modifiers);
      return true;
    } catch (error) {
      console.error('Failed to simulate key press:', error);
      throw error;
    }
  }
  
  async pasteTextFromClipboard() {
    if (!this.isEnabled) {
      return false;
    }
    
    try {
      robot.keyTap('v', ['control']);
      console.log('Pasted text from clipboard');
      return true;
    } catch (error) {
      console.error('Failed to paste from clipboard:', error);
      throw error;
    }
  }
  
  async copyTextToClipboard() {
    if (!this.isEnabled) {
      return false;
    }
    
    try {
      robot.keyTap('c', ['control']);
      console.log('Copied text to clipboard');
      return true;
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
      throw error;
    }
  }
  
  enable() {
    this.isEnabled = true;
    console.log('Text injection enabled');
  }
  
  disable() {
    this.isEnabled = false;
    console.log('Text injection disabled');
  }
  
  isTextInjectionEnabled() {
    return this.isEnabled;
  }
  
  setInjectionDelay(delayMs) {
    this.injectionDelay = Math.max(10, delayMs); // Minimum 10ms delay
    console.log(`Text injection delay set to: ${this.injectionDelay}ms`);
  }
  
  // Get current cursor position (if supported)
  getCursorPosition() {
    try {
      const pos = robot.getMousePos();
      return { x: pos.x, y: pos.y };
    } catch (error) {
      console.error('Failed to get cursor position:', error);
      return null;
    }
  }
  
  // Get screen information
  getScreenInfo() {
    try {
      const size = robot.getScreenSize();
      return {
        width: size.width,
        height: size.height
      };
    } catch (error) {
      console.error('Failed to get screen info:', error);
      return null;
    }
  }
  
  // Check if a specific application is active (basic implementation)
  async checkActiveApplication() {
    // This is a simplified implementation
    // In a real application, you might want to use platform-specific APIs
    // to detect the active window/application
    try {
      const mousePos = robot.getMousePos();
      return {
        isActive: true,
        mousePosition: mousePos
      };
    } catch (error) {
      console.error('Failed to check active application:', error);
      return { isActive: false };
    }
  }
  
  // Utility function for delays
  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  // Test text injection functionality
  async testTextInjection() {
    try {
      console.log('Testing text injection...');
      
      // Wait a moment for user to focus target application
      await this.sleep(2000);
      
      // Test basic text injection
      await this.injectText('This is a test message from BestMe.');
      
      console.log('Text injection test completed');
      return true;
    } catch (error) {
      console.error('Text injection test failed:', error);
      return false;
    }
  }
}

module.exports = TextInjection;