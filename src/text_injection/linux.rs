//! Linux implementation of text injection using X11/Wayland

use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::sync::Arc;
use parking_lot::Mutex;

use super::{TextInjector, InjectionMode, PermissionStatus, WindowInfo};
use super::context::ContextDetector;
use super::keycodes::{VirtualKey, GenericKeycodeMapper, KeycodeMapper};

#[cfg(target_os = "linux")]
use {
    x11::xlib::*,
    x11::xtest::*,
    x11::keysym,
    std::{ptr, time::Duration},
};

/// Linux text injector using X11 or Wayland
pub struct LinuxInjector {
    context_detector: ContextDetector,
    keycode_mapper: GenericKeycodeMapper,
    injection_active: Arc<Mutex<bool>>,
    display_server: DisplayServer,
    #[cfg(target_os = "linux")]
    x11_display: Option<*mut Display>,
}

#[derive(Debug, Clone, Copy)]
enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

// Mark LinuxInjector as Send + Sync since we handle thread safety internally
unsafe impl Send for LinuxInjector {}
unsafe impl Sync for LinuxInjector {}

impl LinuxInjector {
    /// Create a new Linux injector
    pub fn new() -> Result<Self> {
        let display_server = Self::detect_display_server();
        
        #[cfg(target_os = "linux")]
        let x11_display = if matches!(display_server, DisplayServer::X11) {
            unsafe {
                let display = XOpenDisplay(ptr::null());
                if display.is_null() {
                    warn!("Failed to open X11 display");
                    None
                } else {
                    Some(display)
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            context_detector: ContextDetector::new(),
            keycode_mapper: GenericKeycodeMapper::new(),
            injection_active: Arc::new(Mutex::new(false)),
            display_server,
            #[cfg(target_os = "linux")]
            x11_display,
        })
    }
    
    /// Detect whether we're running on X11 or Wayland
    fn detect_display_server() -> DisplayServer {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            DisplayServer::Wayland
        } else if std::env::var("DISPLAY").is_ok() {
            DisplayServer::X11
        } else {
            DisplayServer::Unknown
        }
    }
    
    /// Convert VirtualKey to X11 keysym
    #[cfg(target_os = "linux")]
    fn virtual_key_to_keysym(key: VirtualKey) -> Option<u32> {
        match key {
            // Letters
            VirtualKey::A => Some(keysym::XK_a as u32),
            VirtualKey::B => Some(keysym::XK_b as u32),
            VirtualKey::C => Some(keysym::XK_c as u32),
            VirtualKey::D => Some(keysym::XK_d as u32),
            VirtualKey::E => Some(keysym::XK_e as u32),
            VirtualKey::F => Some(keysym::XK_f as u32),
            VirtualKey::G => Some(keysym::XK_g as u32),
            VirtualKey::H => Some(keysym::XK_h as u32),
            VirtualKey::I => Some(keysym::XK_i as u32),
            VirtualKey::J => Some(keysym::XK_j as u32),
            VirtualKey::K => Some(keysym::XK_k as u32),
            VirtualKey::L => Some(keysym::XK_l as u32),
            VirtualKey::M => Some(keysym::XK_m as u32),
            VirtualKey::N => Some(keysym::XK_n as u32),
            VirtualKey::O => Some(keysym::XK_o as u32),
            VirtualKey::P => Some(keysym::XK_p as u32),
            VirtualKey::Q => Some(keysym::XK_q as u32),
            VirtualKey::R => Some(keysym::XK_r as u32),
            VirtualKey::S => Some(keysym::XK_s as u32),
            VirtualKey::T => Some(keysym::XK_t as u32),
            VirtualKey::U => Some(keysym::XK_u as u32),
            VirtualKey::V => Some(keysym::XK_v as u32),
            VirtualKey::W => Some(keysym::XK_w as u32),
            VirtualKey::X => Some(keysym::XK_x as u32),
            VirtualKey::Y => Some(keysym::XK_y as u32),
            VirtualKey::Z => Some(keysym::XK_z as u32),
            
            // Numbers
            VirtualKey::Num0 => Some(keysym::XK_0 as u32),
            VirtualKey::Num1 => Some(keysym::XK_1 as u32),
            VirtualKey::Num2 => Some(keysym::XK_2 as u32),
            VirtualKey::Num3 => Some(keysym::XK_3 as u32),
            VirtualKey::Num4 => Some(keysym::XK_4 as u32),
            VirtualKey::Num5 => Some(keysym::XK_5 as u32),
            VirtualKey::Num6 => Some(keysym::XK_6 as u32),
            VirtualKey::Num7 => Some(keysym::XK_7 as u32),
            VirtualKey::Num8 => Some(keysym::XK_8 as u32),
            VirtualKey::Num9 => Some(keysym::XK_9 as u32),
            
            // Special keys
            VirtualKey::Space => Some(keysym::XK_space as u32),
            VirtualKey::Enter => Some(keysym::XK_Return as u32),
            VirtualKey::Tab => Some(keysym::XK_Tab as u32),
            VirtualKey::Escape => Some(keysym::XK_Escape as u32),
            VirtualKey::Backspace => Some(keysym::XK_BackSpace as u32),
            VirtualKey::Delete => Some(keysym::XK_Delete as u32),
            VirtualKey::Shift => Some(keysym::XK_Shift_L as u32),
            VirtualKey::Control => Some(keysym::XK_Control_L as u32),
            VirtualKey::Alt => Some(keysym::XK_Alt_L as u32),
            VirtualKey::Meta => Some(keysym::XK_Super_L as u32),
            
            // Punctuation
            VirtualKey::Period => Some(keysym::XK_period as u32),
            VirtualKey::Comma => Some(keysym::XK_comma as u32),
            VirtualKey::Semicolon => Some(keysym::XK_semicolon as u32),
            VirtualKey::Quote => Some(keysym::XK_apostrophe as u32),
            VirtualKey::Slash => Some(keysym::XK_slash as u32),
            VirtualKey::Backslash => Some(keysym::XK_backslash as u32),
            VirtualKey::LeftBracket => Some(keysym::XK_bracketleft as u32),
            VirtualKey::RightBracket => Some(keysym::XK_bracketright as u32),
            VirtualKey::Minus => Some(keysym::XK_minus as u32),
            VirtualKey::Equals => Some(keysym::XK_equal as u32),
            VirtualKey::Grave => Some(keysym::XK_grave as u32),
            
            // Function keys
            VirtualKey::F1 => Some(keysym::XK_F1 as u32),
            VirtualKey::F2 => Some(keysym::XK_F2 as u32),
            VirtualKey::F3 => Some(keysym::XK_F3 as u32),
            VirtualKey::F4 => Some(keysym::XK_F4 as u32),
            VirtualKey::F5 => Some(keysym::XK_F5 as u32),
            VirtualKey::F6 => Some(keysym::XK_F6 as u32),
            VirtualKey::F7 => Some(keysym::XK_F7 as u32),
            VirtualKey::F8 => Some(keysym::XK_F8 as u32),
            VirtualKey::F9 => Some(keysym::XK_F9 as u32),
            VirtualKey::F10 => Some(keysym::XK_F10 as u32),
            VirtualKey::F11 => Some(keysym::XK_F11 as u32),
            VirtualKey::F12 => Some(keysym::XK_F12 as u32),
            
            // Navigation
            VirtualKey::Home => Some(keysym::XK_Home as u32),
            VirtualKey::End => Some(keysym::XK_End as u32),
            VirtualKey::PageUp => Some(keysym::XK_Page_Up as u32),
            VirtualKey::PageDown => Some(keysym::XK_Page_Down as u32),
            VirtualKey::Left => Some(keysym::XK_Left as u32),
            VirtualKey::Right => Some(keysym::XK_Right as u32),
            VirtualKey::Up => Some(keysym::XK_Up as u32),
            VirtualKey::Down => Some(keysym::XK_Down as u32),
            
            _ => None,
        }
    }
    
    /// Type text using X11 XTest extension
    #[cfg(target_os = "linux")]
    async fn type_text_x11(&self, text: &str, delay_ms: u64) -> Result<()> {
        // Process each character
        for ch in text.chars() {
            // Check if we should stop
            if !*self.injection_active.lock() {
                return Ok(());
            }
            
            // Type the character synchronously to avoid holding display pointer across await
            self.type_char_x11_sync(ch)?;
            
            // Delay between keystrokes
            if delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
        
        Ok(())
    }
    
    /// Type a single character using X11 (synchronous)
    #[cfg(target_os = "linux")]
    fn type_char_x11_sync(&self, ch: char) -> Result<()> {
        let display = self.x11_display
            .context("X11 display not initialized")?;
        
        unsafe {
            // Check if XTest extension is available
            let mut event_base = 0;
            let mut error_base = 0;
            let mut major = 0;
            let mut minor = 0;
            
            if XTestQueryExtension(display, &mut event_base, &mut error_base, 
                                 &mut major, &mut minor) == 0 {
                return Err(anyhow::anyhow!("XTest extension not available"));
            }
            
            if ch as u32 > 127 {
                // For Unicode characters, try to type them directly
                self.type_unicode_char_x11_sync(ch, display)?;
            } else {
                // Map character to key presses
                let key_presses = self.keycode_mapper.char_to_keypresses(ch);
                
                for key_press in key_presses {
                    if let Some(keysym) = Self::virtual_key_to_keysym(key_press.key) {
                        let keycode = XKeysymToKeycode(display, keysym as u64);
                        if keycode == 0 {
                            warn!("No keycode for keysym {:x}", keysym);
                            continue;
                        }
                        
                        // Press modifiers
                        if key_press.modifiers.shift {
                            let shift_code = XKeysymToKeycode(display, keysym::XK_Shift_L as u64);
                            XTestFakeKeyEvent(display, shift_code as u32, True, 0);
                        }
                        if key_press.modifiers.control {
                            let ctrl_code = XKeysymToKeycode(display, keysym::XK_Control_L as u64);
                            XTestFakeKeyEvent(display, ctrl_code as u32, True, 0);
                        }
                        if key_press.modifiers.alt {
                            let alt_code = XKeysymToKeycode(display, keysym::XK_Alt_L as u64);
                            XTestFakeKeyEvent(display, alt_code as u32, True, 0);
                        }
                        if key_press.modifiers.meta {
                            let meta_code = XKeysymToKeycode(display, keysym::XK_Super_L as u64);
                            XTestFakeKeyEvent(display, meta_code as u32, True, 0);
                        }
                        
                        // Press and release key
                        XTestFakeKeyEvent(display, keycode as u32, True, 0);
                        XTestFakeKeyEvent(display, keycode as u32, False, 0);
                        
                        // Release modifiers
                        if key_press.modifiers.meta {
                            let meta_code = XKeysymToKeycode(display, keysym::XK_Super_L as u64);
                            XTestFakeKeyEvent(display, meta_code as u32, False, 0);
                        }
                        if key_press.modifiers.alt {
                            let alt_code = XKeysymToKeycode(display, keysym::XK_Alt_L as u64);
                            XTestFakeKeyEvent(display, alt_code as u32, False, 0);
                        }
                        if key_press.modifiers.control {
                            let ctrl_code = XKeysymToKeycode(display, keysym::XK_Control_L as u64);
                            XTestFakeKeyEvent(display, ctrl_code as u32, False, 0);
                        }
                        if key_press.modifiers.shift {
                            let shift_code = XKeysymToKeycode(display, keysym::XK_Shift_L as u64);
                            XTestFakeKeyEvent(display, shift_code as u32, False, 0);
                        }
                        
                        XFlush(display);
                    }
                }
            }
            
            XFlush(display);
        }
        
        Ok(())
    }
    
    /// Type a Unicode character using X11 (synchronous version)
    #[cfg(target_os = "linux")]
    fn type_unicode_char_x11_sync(&self, ch: char, display: *mut Display) -> Result<()> {
        unsafe {
            // For Unicode, we can try to use the keysym directly
            let keysym = ch as u32;
            let keycode = XKeysymToKeycode(display, keysym as u64);
            
            if keycode != 0 {
                XTestFakeKeyEvent(display, keycode as u32, True, 0);
                XTestFakeKeyEvent(display, keycode as u32, False, 0);
                XFlush(display);
            } else {
                // If no direct mapping, we might need to use compose sequences
                // or clipboard paste for complex Unicode
                debug!("No keycode mapping for Unicode character: {}", ch);
            }
        }
        
        Ok(())
    }
    
    /// Paste text using X11 clipboard
    #[cfg(target_os = "linux")]
    async fn paste_text_x11(&self, _text: &str, _restore_clipboard: bool) -> Result<()> {
        // For now, we'll simulate Ctrl+V after setting clipboard
        // This would need proper X11 clipboard handling
        let display = self.x11_display
            .context("X11 display not initialized")?;
        
        unsafe {
            // TODO: Implement proper X11 clipboard handling
            // For now, just simulate Ctrl+V
            let ctrl_code = XKeysymToKeycode(display, keysym::XK_Control_L as u64);
            let v_code = XKeysymToKeycode(display, keysym::XK_v as u64);
            
            XTestFakeKeyEvent(display, ctrl_code as u32, True, 0);
            XTestFakeKeyEvent(display, v_code as u32, True, 0);
            XTestFakeKeyEvent(display, v_code as u32, False, 0);
            XTestFakeKeyEvent(display, ctrl_code as u32, False, 0);
            
            XFlush(display);
        }
        
        Ok(())
    }
}

#[async_trait]
impl TextInjector for LinuxInjector {
    async fn inject_text(&self, text: &str, mode: &InjectionMode) -> Result<()> {
        match self.display_server {
            DisplayServer::X11 => {
                // Set injection active
                *self.injection_active.lock() = true;
                
                let result = match mode {
                    InjectionMode::Type { delay_ms, .. } => {
                        #[cfg(target_os = "linux")]
                        {
                            info!("Typing {} characters with {}ms delay", text.len(), delay_ms);
                            self.type_text_x11(text, (*delay_ms).into()).await
                        }
                        
                        #[cfg(not(target_os = "linux"))]
                        Err(anyhow::anyhow!("Type mode only available on Linux"))
                    }
                    InjectionMode::Paste { restore_clipboard, .. } => {
                        #[cfg(target_os = "linux")]
                        {
                            info!("Pasting {} characters", text.len());
                            self.paste_text_x11(text, *restore_clipboard).await
                        }
                        
                        #[cfg(not(target_os = "linux"))]
                        Err(anyhow::anyhow!("Paste mode only available on Linux"))
                    }
                    InjectionMode::Direct => {
                        // Direct mode uses paste without clipboard restoration
                        #[cfg(target_os = "linux")]
                        {
                            self.paste_text_x11(text, false).await
                        }
                        
                        #[cfg(not(target_os = "linux"))]
                        {
                            Err(anyhow::anyhow!("Direct mode only available on Linux"))
                        }
                    }
                };
                
                // Clear injection active flag
                *self.injection_active.lock() = false;
                
                result
            }
            DisplayServer::Wayland => {
                // Wayland has limited options for input simulation
                Err(anyhow::anyhow!("Text injection on Wayland is limited due to security restrictions"))
            }
            DisplayServer::Unknown => {
                Err(anyhow::anyhow!("Could not detect display server (X11 or Wayland)"))
            }
        }
    }
    
    async fn get_active_window(&self) -> Result<WindowInfo> {
        self.context_detector.get_active_window().await
    }
    
    fn is_available(&self) -> bool {
        matches!(self.display_server, DisplayServer::X11)
    }
    
    fn check_permissions(&self) -> PermissionStatus {
        match self.display_server {
            DisplayServer::X11 => {
                PermissionStatus {
                    available: true,
                    accessibility_granted: None,
                    has_required_privileges: true,
                    message: "Text injection is available on X11".to_string(),
                    required_actions: vec![],
                }
            }
            DisplayServer::Wayland => {
                PermissionStatus {
                    available: false,
                    accessibility_granted: None,
                    has_required_privileges: false,
                    message: "Text injection is limited on Wayland due to security model".to_string(),
                    required_actions: vec![
                        "Consider using X11 session for full functionality".to_string(),
                        "Or use clipboard-based injection mode".to_string(),
                    ],
                }
            }
            DisplayServer::Unknown => {
                PermissionStatus {
                    available: false,
                    accessibility_granted: None,
                    has_required_privileges: false,
                    message: "Could not detect display server".to_string(),
                    required_actions: vec![
                        "Ensure DISPLAY or WAYLAND_DISPLAY environment variable is set".to_string(),
                    ],
                }
            }
        }
    }
    
    async fn request_permissions(&self) -> Result<()> {
        // No special permissions needed on Linux/X11
        Ok(())
    }
    
    async fn stop_injection(&self) -> Result<()> {
        *self.injection_active.lock() = false;
        Ok(())
    }
}

impl Drop for LinuxInjector {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        if let Some(display) = self.x11_display {
            unsafe {
                XCloseDisplay(display);
            }
        }
    }
}