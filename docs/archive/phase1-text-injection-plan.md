# Phase 1: Text Injection System - Implementation Plan

## Overview
The Text Injection System is the critical feature that enables BestMe to type transcribed text into any application. This requires platform-specific implementations and careful consideration of security and compatibility.

## Architecture Design

### Core Components

1. **Text Injection Plugin** (`src-tauri/src/plugin/text_injection.rs`)
   - Platform abstraction layer
   - Injection mode management
   - Security and permission handling
   - Event coordination with transcription

2. **Platform Implementations**
   - Windows: `src/text_injection/windows.rs`
   - macOS: `src/text_injection/macos.rs`
   - Linux: `src/text_injection/linux.rs`

3. **Context Detection** (`src/text_injection/context.rs`)
   - Active window detection
   - Application identification
   - Focus state monitoring

4. **Injection Modes**
   - **Type Mode**: Simulates keyboard typing character by character
   - **Paste Mode**: Uses clipboard for faster insertion
   - **Direct Mode**: Application-specific APIs (future)

## Platform-Specific Implementation Details

### Windows Implementation
**APIs to use:**
- `SendInput` from Windows API for keyboard simulation
- `GetForegroundWindow` for active window detection
- `SetClipboardData` for paste mode
- UI Automation for advanced scenarios

**Dependencies:**
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_DataExchange",
    "Win32_Foundation"
]}
```

**Key Challenges:**
- UAC and elevated applications
- Anti-cheat software detection
- IME (Input Method Editor) compatibility

### macOS Implementation
**APIs to use:**
- Core Graphics Event Services for keyboard events
- Accessibility API for window information
- NSPasteboard for clipboard operations

**Dependencies:**
```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.23"
cocoa = "0.25"
objc = "0.2"
```

**Key Challenges:**
- Accessibility permissions required
- Sandboxing restrictions
- Secure input mode in some apps

### Linux Implementation
**APIs to use:**
- X11: XTest extension for keyboard simulation
- Wayland: Limited options, may need AT-SPI
- Clipboard: X11 selections or wl-clipboard

**Dependencies:**
```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11 = { version = "2.21", features = ["xtest"] }
wayland-client = { version = "0.31", optional = true }
```

**Key Challenges:**
- X11 vs Wayland compatibility
- Different desktop environments
- Permission models vary by distro

## Implementation Plan

### Day 1: Core Architecture & Windows
1. **Create text injection module structure** (2 hours)
   - Define traits and interfaces
   - Set up platform conditional compilation
   - Create injection mode enum

2. **Implement Windows keyboard simulation** (4 hours)
   - Basic SendInput implementation
   - Character to virtual key mapping
   - Modifier key handling
   - Unicode character support

3. **Test on Windows** (2 hours)
   - Test with various applications
   - Verify special characters
   - Check performance

### Day 2: macOS Implementation
1. **Set up macOS development environment** (1 hour)
   - Configure permissions
   - Set up code signing

2. **Implement Core Graphics keyboard events** (4 hours)
   - CGEventCreateKeyboardEvent
   - Modifier handling
   - Key mapping tables

3. **Handle macOS security** (2 hours)
   - Request accessibility permissions
   - Handle permission denials gracefully
   - Add user guidance

4. **Test on macOS** (1 hour)
   - Various applications
   - Permission scenarios

### Day 3: Linux Implementation
1. **Implement X11 support** (3 hours)
   - XTest extension usage
   - Key code mapping
   - Window manager compatibility

2. **Add Wayland fallback** (2 hours)
   - Detect display server
   - Implement AT-SPI fallback
   - Handle limitations

3. **Test on Linux** (2 hours)
   - Multiple distros
   - X11 and Wayland
   - Different DEs

### Day 4: Context Detection & Modes
1. **Implement context detection** (3 hours)
   - Active window monitoring
   - Application identification
   - Focus change events

2. **Create injection modes** (2 hours)
   - Type mode with configurable speed
   - Paste mode with clipboard management
   - Mode selection logic

3. **Add application profiles** (2 hours)
   - Profile structure
   - Common app presets
   - User customization

### Day 5: Integration & UI
1. **Integrate with transcription** (3 hours)
   - Connect to transcription events
   - Add injection toggle
   - Handle injection state

2. **Create UI controls** (2 hours)
   - Injection mode selector
   - Speed controls
   - Application profiles UI

3. **Testing & polish** (2 hours)
   - End-to-end testing
   - Performance optimization
   - Error handling

## API Design

```rust
/// Text injection service trait
pub trait TextInjector: Send + Sync {
    /// Inject text using the specified mode
    async fn inject_text(&self, text: &str, mode: InjectionMode) -> Result<()>;
    
    /// Get current active window information
    async fn get_active_window(&self) -> Result<WindowInfo>;
    
    /// Check if injection is available
    fn is_available(&self) -> bool;
    
    /// Get required permissions status
    fn check_permissions(&self) -> PermissionStatus;
}

/// Injection modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InjectionMode {
    /// Type character by character
    Type {
        delay_ms: u32,
        use_shift_for_caps: bool,
    },
    /// Use clipboard for faster insertion
    Paste {
        restore_clipboard: bool,
    },
    /// Direct application API (future)
    Direct,
}

/// Window information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub app_name: String,
    pub app_path: Option<String>,
    pub is_elevated: bool,
}

/// Application profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppProfile {
    pub app_name: String,
    pub preferred_mode: InjectionMode,
    pub requires_special_handling: bool,
    pub custom_settings: HashMap<String, Value>,
}
```

## Security Considerations

1. **Permission Management**
   - Clear user consent flow
   - Minimal permission requests
   - Graceful degradation

2. **Input Validation**
   - Sanitize text before injection
   - Prevent injection attacks
   - Rate limiting

3. **Privacy**
   - Don't log injected text
   - Clear clipboard after use
   - Secure storage of profiles

## Testing Strategy

1. **Unit Tests**
   - Key mapping accuracy
   - Mode selection logic
   - Context detection

2. **Integration Tests**
   - Full injection flow
   - Cross-platform compatibility
   - Performance benchmarks

3. **Manual Testing Matrix**
   - Applications: VS Code, Chrome, Terminal, Office
   - Scenarios: Special chars, Unicode, Fast typing
   - Platforms: Win 10/11, macOS 12+, Ubuntu/Fedora

## Success Criteria

- [ ] Text injection works on all three platforms
- [ ] < 50ms latency from transcription to typing
- [ ] 99%+ character accuracy
- [ ] Smooth integration with existing UI
- [ ] Clear permission flow for users
- [ ] No security warnings from OS

## Known Limitations

1. **Windows**: Some games and anti-cheat protected apps
2. **macOS**: Apps with secure input enabled
3. **Linux**: Wayland has limited options
4. **All**: Elevated/admin applications may block input

## Future Enhancements

1. **Smart Formatting**
   - Auto-capitalization
   - Punctuation correction
   - Code formatting detection

2. **Advanced Modes**
   - Burst typing for commands
   - Slow typing for demonstrations
   - Synchronized with speech pace

3. **Application Integration**
   - Native APIs where available
   - Browser extensions
   - IDE plugins