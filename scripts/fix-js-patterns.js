#!/usr/bin/env node

/**
 * This script fixes the remaining JavaScript API patterns that weren't caught by the previous script
 */

const fs = require('fs');
const path = require('path');

console.log('Fixing remaining JavaScript patterns...');

// Fix App.svelte
let appFile = path.join(process.cwd(), 'ui/src/App.svelte');
if (fs.existsSync(appFile)) {
  let content = fs.readFileSync(appFile, 'utf8');
  
  // Fix voice_commands:update_config pattern
  content = content.replace(
    /await invoke\(['"]plugin:voice_commands:update_config['"]/g,
    "await invoke.voice_commands.update_config"
  );
  
  // Fix transcribe:start_transcription pattern
  content = content.replace(
    /await invoke\(['"]plugin:transcribe:start_transcription['"]/g,
    "await invoke.transcribe.start_transcription"
  );
  
  fs.writeFileSync(appFile, content);
  console.log('Fixed patterns in App.svelte');
}

// Fix Settings.svelte
let settingsFile = path.join(process.cwd(), 'ui/src/Settings.svelte');
if (fs.existsSync(settingsFile)) {
  let content = fs.readFileSync(settingsFile, 'utf8');
  
  // Fix config:save_all_settings pattern
  content = content.replace(
    /await invoke\(['"]plugin:config:save_all_settings['"]/g,
    "await invoke.config.save_all_settings"
  );
  
  fs.writeFileSync(settingsFile, content);
  console.log('Fixed patterns in Settings.svelte');
}

// Fix routes/settings/Settings.svelte
let routeSettingsFile = path.join(process.cwd(), 'ui/src/routes/settings/Settings.svelte');
if (fs.existsSync(routeSettingsFile)) {
  let content = fs.readFileSync(routeSettingsFile, 'utf8');
  
  // Fix get_settings pattern
  content = content.replace(
    /await invoke\(['"]get_settings['"]/g,
    "await invoke.config.get_settings"
  );
  
  // Fix save_all_settings pattern
  content = content.replace(
    /await invoke\(['"]save_all_settings['"]/g,
    "await invoke.config.save_all_settings"
  );
  
  fs.writeFileSync(routeSettingsFile, content);
  console.log('Fixed patterns in routes/settings/Settings.svelte');
}

console.log('All patterns fixed successfully!'); 
