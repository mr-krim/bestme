# BestMe UI Vision and Implementation Plan

This document outlines the comprehensive UI design and implementation strategy for BestMe across Windows and macOS platforms.

## Overall Layout Structure

### Three-Panel Layout
```
+----------------+-------------------+-------------------------------+
| Main Functions | Chat/History List |                               |
| (Navigation)   | (Context)         |       Main Content Area       |
|                |                   |    (Transcription + Chat)     |
|                |                   |                               |
|                |                   |                               |
|                |                   |                               |
|                |                   |                               |
+----------------+-------------------+-------------------------------+
|               Status Bar / Control Center / Metrics                |
+-------------------------------------------------------------------|
```

## Key Components Design

### Left Panel: Core Navigation
- **Transcription Mode** - Primary speech-to-text functionality
- **Chat Mode** - AI-assisted conversation interface
- **Voice Commands** - Configure and view available commands
- **Settings** - Less frequently accessed than quick settings
- **Saved Transcripts** - History and saved content
- **Devices** - Audio device management
- **Help & Documentation**

### Middle Panel: Context-Aware Content
- In **Transcription Mode**: Shows recent transcription sessions
- In **Chat Mode**: Shows conversation history with different AI assistants
- In **Voice Commands**: Shows command categories and recently used commands
- Collapsible to maximize main content area when needed

### Main Panel: Active Content
- **Transcription View**: Real-time transcription with confidence indicators
- **Chat Interface**: AI conversation with message bubbles
- **Split View Option**: Transcribe and chat simultaneously
- **Visual Audio Indicators**: Waveform/spectrum visualization during recording

### Top Bar: Quick Actions
- **Microphone Toggle**: Start/stop recording with visual indicator
- **AI Assistant Selector**: Switch between different AI models
- **Language Selector**: Change transcription language
- **Quick Settings**: Most common adjustments (model size, auto-punctuate)
- **Theme Toggle**: Light/dark mode switch

### Bottom Bar: System Information
- **App Version**: Current version with update notification
- **UI Mode Selector**: Toggle between User/Power User/Advanced views
- **System Resource Usage**: CPU/RAM visualization
- **Transcription Status**: Words processed, accuracy metrics
- **Connection Status**: Online/offline indicator for AI services

## Implementation Approach

### Hybrid Strategy
To implement this while maintaining our cross-platform roadmap:

1. **Windows Implementation**: 
   - Extend the current `TranscriptionWindow` with a more complex layout
   - Use the Windows API's docking window capabilities
   - Implement custom drawing for the UI components
   - Add resource monitoring via Windows Management Instrumentation (WMI)

```rust
// Example structure for enhanced window.rs with multi-panel support
pub struct MultiPanelWindow {
    hwnd: HWND,
    left_panel: Panel,
    middle_panel: Panel,
    main_panel: Panel,
    top_bar: Panel,
    bottom_bar: Panel,
    // other fields
}
```

2. **macOS Implementation**:
   - Use Cocoa's NSWindow with NSStackView for layout
   - Leverage NSOutlineView for the navigation panel
   - Implement NSTableView for the middle panel
   - Use CoreAnimation for smooth transitions and effects

```rust
// Example for macOS using Cocoa bindings
#[cfg(target_os = "macos")]
pub struct MultiPanelWindow {
    window: id, // NSWindow
    left_panel: id, // NSView
    middle_panel: id, // NSView
    main_panel: id, // NSView
    // other fields
}
```

3. **Shared Logic Layer**:
   - Create platform-agnostic state management
   - Implement consistent event handling
   - Define common UI component behaviors

```rust
pub struct AppState {
    active_panel: PanelType,
    recording_status: RecordingStatus,
    chat_history: Vec<ChatMessage>,
    transcription_history: Vec<TranscriptionSession>,
    // other shared state
}
```

## Transition to Tauri (Future-Proof Approach)

For long-term cross-platform consistency, we'll:

1. Build the Tauri frontend in parallel:
   ```
   ui/
   ├── src/
   │   ├── components/
   │   │   ├── LeftPanel.svelte
   │   │   ├── MiddlePanel.svelte
   │   │   ├── MainPanel.svelte
   │   │   ├── TopBar.svelte
   │   │   └── BottomBar.svelte
   │   ├── views/
   │   │   ├── TranscriptionView.svelte
   │   │   └── ChatView.svelte
   │   └── App.svelte
   ```

2. Design responsive components that adapt to three UI modes:
   - **User Mode**: Simplified interface with fewer options
   - **Power User**: More controls and customization options
   - **Advanced**: Full feature set with technical settings

3. Create platform-specific styling using CSS variables:
   ```css
   :root {
     /* Windows-specific styles */
     --primary-font: 'Segoe UI', sans-serif;
     --window-border-radius: 4px;
   }
   
   @media (platform: macos) {
     :root {
       /* macOS-specific styles */
       --primary-font: -apple-system, BlinkMacSystemFont, sans-serif;
       --window-border-radius: 10px;
     }
   }
   ```

## Visual Enhancements

1. **Transition Effects**: Add subtle animations for panel resizing and view changes
2. **Speech Visualization**: Real-time waveform display while recording
3. **Color-coded Confidence**: Highlight transcribed words based on confidence level
4. **Adaptive Theme**: Match system dark/light mode preferences
5. **Variable Density Display**: Adjust information density based on UI mode

## Implementation Phases

### Phase 1: Windows Prototype (Current Priority)
1. Enhance the existing `TranscriptionWindow` with multi-panel layout
2. Implement panel creation and management
3. Create the basic navigation structure
4. Add transcription view with improved visuals
5. Implement system resource monitoring

```rust
// Example implementation steps for Windows
impl MultiPanelWindow {
    pub fn new(config_manager: Arc<Mutex<ConfigManager>>, device_manager: Arc<DeviceManager>) -> Result<Self> {
        // Initialize window with multi-panel layout
        // Create child windows for each panel
        // Set up event handling
    }
    
    fn create_panels(&mut self) -> Result<()> {
        // Create and position each panel
        self.left_panel = self.create_panel(PanelType::Navigation)?;
        self.middle_panel = self.create_panel(PanelType::Context)?;
        self.main_panel = self.create_panel(PanelType::Content)?;
        // Create top and bottom bars
    }
}
```

### Phase 2: Core Functionality
1. Implement transcription visualization
2. Add AI chat integration
3. Develop voice command interface
4. Create settings panel
5. Build history and saved content management

### Phase 3: macOS Implementation
1. Create Cocoa-based UI implementation
2. Apply macOS-specific design patterns and interactions
3. Ensure feature parity with Windows version
4. Test on various macOS versions

### Phase 4: Tauri Integration
1. Develop web-based UI components
2. Create responsive layouts for all screen sizes
3. Implement platform-specific styling
4. Bridge Rust backend functionality to frontend
5. Transition to hybrid native/web approach

## Performance Considerations

1. **Efficient Rendering**: Use hardware acceleration where available
2. **Memory Management**: Optimize resource usage for transcription and AI models
3. **Lazy Loading**: Load UI components on demand
4. **Background Processing**: Handle intensive tasks in separate threads
5. **Resource Monitoring**: Track and optimize system resource usage

## Accessibility Features

1. **Keyboard Navigation**: Full keyboard control for all functions
2. **Screen Reader Support**: Proper labeling for assistive technologies
3. **High Contrast Mode**: Support for system accessibility settings
4. **Font Scaling**: Respect system font size settings
5. **Focus Indicators**: Clear visual cues for keyboard focus

## Next Steps

1. Create a prototype of the multi-panel Windows UI
2. Implement navigation and panel switching
3. Add basic transcription visualization
4. Integrate system monitoring
5. Begin parallel development of Tauri components 
