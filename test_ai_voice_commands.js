// Test script for AI voice command integration
// Run this in the browser console when the app is running

async function testAIVoiceCommands() {
  const { invoke } = window.__TAURI__.core;
  
  console.log('Testing AI Voice Commands...');
  
  // 1. Get current AI voice settings
  try {
    const settings = await invoke('get_ai_voice_settings');
    console.log('Current AI Voice Settings:', settings);
  } catch (error) {
    console.error('Failed to get AI voice settings:', error);
  }
  
  // 2. Enable AI voice commands
  try {
    await invoke('save_ai_voice_settings', {
      settings: {
        enabled: true,
        natural_language: true,
        context_aware: true,
        confidence_threshold: 0.7
      }
    });
    console.log('AI voice commands enabled');
  } catch (error) {
    console.error('Failed to enable AI voice commands:', error);
  }
  
  // 3. Test processing a voice command
  try {
    const result = await invoke('process_ai_voice_command', {
      text: 'delete the last word',
      context: 'This is a test sentence'
    });
    console.log('AI command processing result:', result);
  } catch (error) {
    console.error('Failed to process AI voice command:', error);
  }
  
  // 4. Test another command
  try {
    const result2 = await invoke('process_ai_voice_command', {
      text: 'make this uppercase',
      context: 'hello world'
    });
    console.log('AI command processing result 2:', result2);
  } catch (error) {
    console.error('Failed to process AI voice command 2:', error);
  }
}

// Run the test
testAIVoiceCommands();