//! macOS implementation of text injection using Core Graphics

use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::sync::Arc;
use parking_lot::Mutex;

use super::{TextInjector, InjectionMode, PermissionStatus, WindowInfo};
use super::context::ContextDetector;
use super::keycodes::{VirtualKey, GenericKeycodeMapper, KeyPress};

#[cfg(target_os = "macos")]
use {
    core_graphics::{
        event::{CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGKeyCode},
        event_source::{CGEventSource, CGEventSourceStateID},
    },
    cocoa::{
        base::{id, nil, YES, NO},
        foundation::{NSAutoreleasePool, NSString, NSDictionary, NSNumber},
        appkit::{NSPasteboard, NSPasteboardTypeString},
    },
    objc::{msg_send, sel, sel_impl, class},
    std::{ffi::c_void, thread, time::Duration},
};

#[cfg(target_os = "macos")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: id) -> bool;
    fn CGEventKeyboardSetUnicodeString(event: core_graphics::sys::CGEventRef, 
                                      length: i64, 
                                      string: *const u16);
}

/// macOS text injector using Core Graphics Event Services
pub struct MacOSInjector {
    context_detector: ContextDetector,
    keycode_mapper: GenericKeycodeMapper,
    injection_active: Arc<Mutex<bool>>,
    #[cfg(target_os = "macos")]
    event_source: Option<CGEventSource>,
}

impl MacOSInjector {
    /// Create a new macOS injector
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "macos")]
        let event_source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .context("Failed to create CGEventSource")?;
        
        Ok(Self {
            context_detector: ContextDetector::new(),
            keycode_mapper: GenericKeycodeMapper::new(),
            injection_active: Arc::new(Mutex::new(false)),
            #[cfg(target_os = "macos")]
            event_source: Some(event_source),
        })
    }
    
    /// Check if accessibility permissions are granted
    fn check_accessibility_permission() -> bool {
        #[cfg(target_os = "macos")]
        {
            unsafe {
                let trusted = AXIsProcessTrustedWithOptions(nil);
                trusted
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        false
    }
    
    /// Request accessibility permissions
    fn request_accessibility_permission() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            unsafe {
                // Create dictionary with prompt option
                let key = NSString::alloc(nil).init_str("AXTrustedCheckOptionPrompt");
                let value: id = msg_send![class!(NSNumber), numberWithBool:YES];
                let options: id = msg_send![class!(NSDictionary), dictionaryWithObject:value forKey:key];
                
                AXIsProcessTrustedWithOptions(options);
                Ok(())
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        Err(anyhow::anyhow!("Not running on macOS"))
    }
    
    /// Convert our virtual key to macOS keycode
    #[cfg(target_os = "macos")]
    fn virtual_key_to_cgkeycode(key: VirtualKey) -> Option<CGKeyCode> {
        match key {
            VirtualKey::A => Some(0x00),
            VirtualKey::S => Some(0x01),
            VirtualKey::D => Some(0x02),
            VirtualKey::F => Some(0x03),
            VirtualKey::H => Some(0x04),
            VirtualKey::G => Some(0x05),
            VirtualKey::Z => Some(0x06),
            VirtualKey::X => Some(0x07),
            VirtualKey::C => Some(0x08),
            VirtualKey::V => Some(0x09),
            VirtualKey::B => Some(0x0B),
            VirtualKey::Q => Some(0x0C),
            VirtualKey::W => Some(0x0D),
            VirtualKey::E => Some(0x0E),
            VirtualKey::R => Some(0x0F),
            VirtualKey::Y => Some(0x10),
            VirtualKey::T => Some(0x11),
            VirtualKey::Key1 => Some(0x12),
            VirtualKey::Key2 => Some(0x13),
            VirtualKey::Key3 => Some(0x14),
            VirtualKey::Key4 => Some(0x15),
            VirtualKey::Key6 => Some(0x16),
            VirtualKey::Key5 => Some(0x17),
            VirtualKey::Equal => Some(0x18),
            VirtualKey::Key9 => Some(0x19),
            VirtualKey::Key7 => Some(0x1A),
            VirtualKey::Minus => Some(0x1B),
            VirtualKey::Key8 => Some(0x1C),
            VirtualKey::Key0 => Some(0x1D),
            VirtualKey::RightBracket => Some(0x1E),
            VirtualKey::O => Some(0x1F),
            VirtualKey::U => Some(0x20),
            VirtualKey::LeftBracket => Some(0x21),
            VirtualKey::I => Some(0x22),
            VirtualKey::P => Some(0x23),
            VirtualKey::Return => Some(0x24),
            VirtualKey::L => Some(0x25),
            VirtualKey::J => Some(0x26),
            VirtualKey::Quote => Some(0x27),
            VirtualKey::K => Some(0x28),
            VirtualKey::Semicolon => Some(0x29),
            VirtualKey::Backslash => Some(0x2A),
            VirtualKey::Comma => Some(0x2B),
            VirtualKey::Slash => Some(0x2C),
            VirtualKey::N => Some(0x2D),
            VirtualKey::M => Some(0x2E),
            VirtualKey::Period => Some(0x2F),
            VirtualKey::Tab => Some(0x30),
            VirtualKey::Space => Some(0x31),
            VirtualKey::Grave => Some(0x32),
            VirtualKey::Delete => Some(0x33),
            VirtualKey::Escape => Some(0x35),
            VirtualKey::RightCommand => Some(0x36),
            VirtualKey::Command => Some(0x37),
            VirtualKey::Shift => Some(0x38),
            VirtualKey::CapsLock => Some(0x39),
            VirtualKey::Option => Some(0x3A),
            VirtualKey::Control => Some(0x3B),
            VirtualKey::RightShift => Some(0x3C),
            VirtualKey::RightOption => Some(0x3D),
            VirtualKey::RightControl => Some(0x3E),
            VirtualKey::Function => Some(0x3F),
            VirtualKey::F17 => Some(0x40),
            VirtualKey::VolumeUp => Some(0x48),
            VirtualKey::VolumeDown => Some(0x49),
            VirtualKey::Mute => Some(0x4A),
            VirtualKey::F18 => Some(0x4F),
            VirtualKey::F19 => Some(0x50),
            VirtualKey::F20 => Some(0x5A),
            VirtualKey::F5 => Some(0x60),
            VirtualKey::F6 => Some(0x61),
            VirtualKey::F7 => Some(0x62),
            VirtualKey::F3 => Some(0x63),
            VirtualKey::F8 => Some(0x64),
            VirtualKey::F9 => Some(0x65),
            VirtualKey::F11 => Some(0x67),
            VirtualKey::F13 => Some(0x69),
            VirtualKey::F16 => Some(0x6A),
            VirtualKey::F14 => Some(0x6B),
            VirtualKey::F10 => Some(0x6D),
            VirtualKey::F12 => Some(0x6F),
            VirtualKey::F15 => Some(0x71),
            VirtualKey::Help => Some(0x72),
            VirtualKey::Home => Some(0x73),
            VirtualKey::PageUp => Some(0x74),
            VirtualKey::ForwardDelete => Some(0x75),
            VirtualKey::F4 => Some(0x76),
            VirtualKey::End => Some(0x77),
            VirtualKey::F2 => Some(0x78),
            VirtualKey::PageDown => Some(0x79),
            VirtualKey::F1 => Some(0x7A),
            VirtualKey::LeftArrow => Some(0x7B),
            VirtualKey::RightArrow => Some(0x7C),
            VirtualKey::DownArrow => Some(0x7D),
            VirtualKey::UpArrow => Some(0x7E),
            _ => None,
        }
    }
    
    /// Type text using Core Graphics events
    #[cfg(target_os = "macos")]
    async fn type_text_cg(&self, text: &str, delay_ms: u64) -> Result<()> {
        let event_source = self.event_source.as_ref()
            .context("Event source not initialized")?;
        
        for ch in text.chars() {
            // Check if we should stop
            if !*self.injection_active.lock() {
                return Ok(());
            }
            
            // For special characters, we might need to use Unicode input
            if ch as u32 > 127 {
                self.type_unicode_char(ch, event_source).await?;
            } else {
                // Map character to key presses
                let key_presses = self.keycode_mapper.char_to_keypresses(ch);
                
                for key_press in key_presses {
                    if let Some(keycode) = Self::virtual_key_to_cgkeycode(key_press.key) {
                        self.send_key_event(keycode, key_press.shift, key_press.ctrl, 
                                          key_press.alt, key_press.meta, event_source)?;
                    }
                }
            }
            
            // Delay between keystrokes
            if delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
        
        Ok(())
    }
    
    /// Send a key event
    #[cfg(target_os = "macos")]
    fn send_key_event(&self, keycode: CGKeyCode, shift: bool, ctrl: bool, 
                     alt: bool, cmd: bool, source: &CGEventSource) -> Result<()> {
        let mut flags = CGEventFlags::empty();
        if shift { flags |= CGEventFlags::CGEventFlagShift; }
        if ctrl { flags |= CGEventFlags::CGEventFlagControl; }
        if alt { flags |= CGEventFlags::CGEventFlagAlternate; }
        if cmd { flags |= CGEventFlags::CGEventFlagCommand; }
        
        // Create key down event
        let key_down = CGEvent::new_keyboard_event(source.clone(), keycode, true)
            .context("Failed to create key down event")?;
        key_down.set_flags(flags);
        key_down.post(CGEventTapLocation::HID);
        
        // Small delay between down and up
        thread::sleep(Duration::from_micros(100));
        
        // Create key up event
        let key_up = CGEvent::new_keyboard_event(source.clone(), keycode, false)
            .context("Failed to create key up event")?;
        key_up.set_flags(flags);
        key_up.post(CGEventTapLocation::HID);
        
        Ok(())
    }
    
    /// Type a Unicode character
    #[cfg(target_os = "macos")]
    async fn type_unicode_char(&self, ch: char, source: &CGEventSource) -> Result<()> {
        let mut utf16_buffer = [0u16; 2];
        let utf16_slice = ch.encode_utf16(&mut utf16_buffer);
        
        for &code_unit in utf16_slice {
            let event = CGEvent::new_keyboard_event(source.clone(), 0, true)
                .context("Failed to create Unicode event")?;
            
            // For Unicode characters on macOS, we'll use the string directly
            unsafe {
                let string = NSString::alloc(nil).init_str(&ch.to_string());
                CGEventKeyboardSetUnicodeString(event.as_concrete_TypeRef(), 
                                               ch.len_utf16() as i64, 
                                               string as *const u16);
            }
            
            event.post(CGEventTapLocation::HID);
            thread::sleep(Duration::from_micros(100));
        }
        
        Ok(())
    }
    
    /// Paste text using the clipboard
    #[cfg(target_os = "macos")]
    async fn paste_text(&self, text: &str, restore_clipboard: bool) -> Result<()> {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            
            // Get the general pasteboard
            let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
            
            // Save current clipboard if requested
            let original_content = if restore_clipboard {
                let string_type = NSString::alloc(nil).init_str("public.utf8-plain-text");
                let types: id = msg_send![pasteboard, types];
                let has_string: bool = msg_send![types, containsObject: string_type];
                
                if has_string {
                    let content: id = msg_send![pasteboard, stringForType: string_type];
                    let c_str: *const i8 = msg_send![content, UTF8String];
                    if c_str.is_null() {
                        None
                    } else {
                        Some(c_str)
                    }
                } else {
                    None
                }
            } else {
                None
            };
            
            // Clear and set new content
            let _: () = msg_send![pasteboard, clearContents];
            let ns_string = NSString::alloc(nil).init_str(text);
            let string_type = NSString::alloc(nil).init_str("public.utf8-plain-text");
            let _: bool = msg_send![pasteboard, setString:ns_string forType:string_type];
            
            // Simulate Cmd+V
            let source = self.event_source.as_ref()
                .context("Event source not initialized")?;
            self.send_key_event(0x09, false, false, false, true, source)?; // 0x09 is 'V'
            
            // Wait a bit for paste to complete
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // Restore original clipboard if requested
            if restore_clipboard {
                if let Some(original) = original_content {
                    let _: () = msg_send![pasteboard, clearContents];
                    let original_str = std::ffi::CStr::from_ptr(original).to_string_lossy();
                    let original_string = NSString::alloc(nil).init_str(&original_str);
                    let string_type = NSString::alloc(nil).init_str("public.utf8-plain-text");
                    let _: bool = msg_send![pasteboard, setString:original_string 
                                                      forType:string_type];
                }
            }
            
            pool.drain();
        }
        
        Ok(())
    }
}

#[async_trait]
impl TextInjector for MacOSInjector {
    async fn inject_text(&self, text: &str, mode: &InjectionMode) -> Result<()> {
        // Check permissions first
        if !Self::check_accessibility_permission() {
            return Err(anyhow::anyhow!("Accessibility permissions required for text injection on macOS"));
        }
        
        // Set injection active
        *self.injection_active.lock() = true;
        
        let result = match mode {
            InjectionMode::Type { delay_ms, .. } => {
                #[cfg(target_os = "macos")]
                {
                    info!("Typing {} characters with {}ms delay", text.len(), delay_ms);
                    self.type_text_cg(text, *delay_ms).await
                }
                
                #[cfg(not(target_os = "macos"))]
                Err(anyhow::anyhow!("Type mode only available on macOS"))
            }
            InjectionMode::Paste { restore_clipboard, .. } => {
                #[cfg(target_os = "macos")]
                {
                    info!("Pasting {} characters", text.len());
                    self.paste_text(text, *restore_clipboard).await
                }
                
                #[cfg(not(target_os = "macos"))]
                Err(anyhow::anyhow!("Paste mode only available on macOS"))
            }
            InjectionMode::Direct => {
                // Direct mode uses paste without clipboard restoration
                #[cfg(target_os = "macos")]
                self.paste_text(text, false).await
                
                #[cfg(not(target_os = "macos"))]
                Err(anyhow::anyhow!("Direct mode only available on macOS"))
            }
        };
        
        // Clear injection active flag
        *self.injection_active.lock() = false;
        
        result
    }
    
    async fn get_active_window(&self) -> Result<WindowInfo> {
        self.context_detector.get_active_window().await
    }
    
    fn is_available(&self) -> bool {
        // Check if we're on macOS
        cfg!(target_os = "macos")
    }
    
    fn check_permissions(&self) -> PermissionStatus {
        let has_accessibility = Self::check_accessibility_permission();
        
        PermissionStatus {
            available: cfg!(target_os = "macos"),
            accessibility_granted: Some(has_accessibility),
            has_required_privileges: has_accessibility,
            message: if has_accessibility {
                "Text injection is available with accessibility permissions".to_string()
            } else {
                "Accessibility permissions required for text injection".to_string()
            },
            required_actions: if has_accessibility {
                vec![]
            } else {
                vec![
                    "Grant accessibility permissions in System Preferences".to_string(),
                    "Go to System Preferences > Security & Privacy > Privacy > Accessibility".to_string(),
                    "Add BestMe to the list of allowed applications".to_string(),
                ]
            },
        }
    }
    
    async fn request_permissions(&self) -> Result<()> {
        Self::request_accessibility_permission()
    }
    
    async fn stop_injection(&self) -> Result<()> {
        *self.injection_active.lock() = false;
        Ok(())
    }
}