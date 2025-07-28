//! Windows implementation of text injection using SendInput

use anyhow::Result;
use async_trait::async_trait;
use log::{debug, warn};
use std::sync::Arc;
use parking_lot::Mutex;

use super::{TextInjector, InjectionMode, PermissionStatus, WindowInfo};
use super::context::ContextDetector;
use super::keycodes::{VirtualKey as VKey, GenericKeycodeMapper, KeyPress, Modifiers};

// Import our compatibility layer
#[cfg(target_os = "windows")]

/// Windows text injector using SendInput API
pub struct WindowsInjector {
    context_detector: ContextDetector,
    keycode_mapper: GenericKeycodeMapper,
    injection_active: Arc<Mutex<bool>>,
}

impl WindowsInjector {
    /// Create a new Windows injector
    pub fn new() -> Result<Self> {
        Ok(Self {
            context_detector: ContextDetector::new(),
            keycode_mapper: GenericKeycodeMapper::new(),
            injection_active: Arc::new(Mutex::new(false)),
        })
    }
    
    /// Convert VirtualKey to Windows virtual key code
    fn virtual_key_to_vk(&self, key: VKey) -> u16 {
        match key {
            // Letters
            VKey::A => 0x41,
            VKey::B => 0x42,
            VKey::C => 0x43,
            VKey::D => 0x44,
            VKey::E => 0x45,
            VKey::F => 0x46,
            VKey::G => 0x47,
            VKey::H => 0x48,
            VKey::I => 0x49,
            VKey::J => 0x4A,
            VKey::K => 0x4B,
            VKey::L => 0x4C,
            VKey::M => 0x4D,
            VKey::N => 0x4E,
            VKey::O => 0x4F,
            VKey::P => 0x50,
            VKey::Q => 0x51,
            VKey::R => 0x52,
            VKey::S => 0x53,
            VKey::T => 0x54,
            VKey::U => 0x55,
            VKey::V => 0x56,
            VKey::W => 0x57,
            VKey::X => 0x58,
            VKey::Y => 0x59,
            VKey::Z => 0x5A,
            
            // Numbers
            VKey::Num0 => 0x30,
            VKey::Num1 => 0x31,
            VKey::Num2 => 0x32,
            VKey::Num3 => 0x33,
            VKey::Num4 => 0x34,
            VKey::Num5 => 0x35,
            VKey::Num6 => 0x36,
            VKey::Num7 => 0x37,
            VKey::Num8 => 0x38,
            VKey::Num9 => 0x39,
            
            // Function keys
            VKey::F1 => 0x70,
            VKey::F2 => 0x71,
            VKey::F3 => 0x72,
            VKey::F4 => 0x73,
            VKey::F5 => 0x74,
            VKey::F6 => 0x75,
            VKey::F7 => 0x76,
            VKey::F8 => 0x77,
            VKey::F9 => 0x78,
            VKey::F10 => 0x79,
            VKey::F11 => 0x7A,
            VKey::F12 => 0x7B,
            
            // Modifiers
            VKey::Shift => 0x10,
            VKey::Control => 0x11,
            VKey::Alt => 0x12,
            VKey::Meta => 0x5B, // Left Windows key
            
            // Special keys
            VKey::Space => 0x20,
            VKey::Enter => 0x0D,
            VKey::Tab => 0x09,
            VKey::Backspace => 0x08,
            VKey::Delete => 0x2E,
            VKey::Escape => 0x1B,
            VKey::CapsLock => 0x14,
            VKey::NumLock => 0x90,
            VKey::ScrollLock => 0x91,
            
            // Navigation
            VKey::Left => 0x25,
            VKey::Right => 0x27,
            VKey::Up => 0x26,
            VKey::Down => 0x28,
            VKey::Home => 0x24,
            VKey::End => 0x23,
            VKey::PageUp => 0x21,
            VKey::PageDown => 0x22,
            
            // Punctuation
            VKey::Minus => 0xBD,
            VKey::Equals => 0xBB,
            VKey::LeftBracket => 0xDB,
            VKey::RightBracket => 0xDD,
            VKey::Semicolon => 0xBA,
            VKey::Quote => 0xDE,
            VKey::Backslash => 0xDC,
            VKey::Comma => 0xBC,
            VKey::Period => 0xBE,
            VKey::Slash => 0xBF,
            VKey::Grave => 0xC0,
            
            // Numpad
            VKey::Numpad0 => 0x60,
            VKey::Numpad1 => 0x61,
            VKey::Numpad2 => 0x62,
            VKey::Numpad3 => 0x63,
            VKey::Numpad4 => 0x64,
            VKey::Numpad5 => 0x65,
            VKey::Numpad6 => 0x66,
            VKey::Numpad7 => 0x67,
            VKey::Numpad8 => 0x68,
            VKey::Numpad9 => 0x69,
            VKey::NumpadMultiply => 0x6A,
            VKey::NumpadAdd => 0x6B,
            VKey::NumpadSubtract => 0x6D,
            VKey::NumpadDecimal => 0x6E,
            VKey::NumpadDivide => 0x6F,
        }
    }
    
    /// Simulate a key press using SendInput
    #[cfg(target_os = "windows")]
    async fn send_key_press(&self, key_press: &KeyPress) -> Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        use crate::text_injection::windows_types::compat::*;
        
        let mut inputs = Vec::new();
        
        // Press modifier keys if needed
        if key_press.modifiers.shift {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: create_virtual_key(self.virtual_key_to_vk(VKey::Shift)),
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
        
        if key_press.modifiers.control {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: create_virtual_key(self.virtual_key_to_vk(VKey::Control)),
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
        
        if key_press.modifiers.alt {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: create_virtual_key(self.virtual_key_to_vk(VKey::Alt)),
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
        
        // Press the main key
        let vk_code = self.virtual_key_to_vk(key_press.key);
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: create_virtual_key(vk_code),
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        
        // Release the main key
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: create_virtual_key(vk_code),
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        
        // Release modifier keys in reverse order
        if key_press.modifiers.alt {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: create_virtual_key(self.virtual_key_to_vk(VKey::Alt)),
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
        
        if key_press.modifiers.control {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: create_virtual_key(self.virtual_key_to_vk(VKey::Control)),
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
        
        if key_press.modifiers.shift {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: create_virtual_key(self.virtual_key_to_vk(VKey::Shift)),
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
        
        // Send all inputs
        unsafe {
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent == 0 {
                return Err(anyhow::anyhow!("SendInput failed"));
            }
        }
        
        Ok(())
    }
    
    /// Placeholder implementation for non-Windows platforms
    #[cfg(not(target_os = "windows"))]
    async fn send_key_press(&self, _key_press: &KeyPress) -> Result<()> {
        Err(anyhow::anyhow!("Windows text injection not available on this platform"))
    }
    
    /// Send Unicode character using Windows Unicode input
    #[cfg(target_os = "windows")]
    async fn send_unicode_char(&self, ch: char) -> Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        
        let mut inputs = Vec::new();
        
        // Convert char to UTF-16
        let mut utf16_buf = [0u16; 2];
        let utf16_slice = ch.encode_utf16(&mut utf16_buf);
        
        for code_unit in utf16_slice {
            // Send Unicode character using KEYEVENTF_UNICODE flag
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0), // Must be 0 for Unicode
                        wScan: *code_unit,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
            
            // Key up event
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: *code_unit,
                        dwFlags: KEYBD_EVENT_FLAGS(KEYEVENTF_UNICODE.0 | KEYEVENTF_KEYUP.0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
        
        // Send all inputs
        unsafe {
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent == 0 {
                return Err(anyhow::anyhow!("SendInput failed for Unicode character"));
            }
        }
        
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    async fn send_unicode_char(&self, _ch: char) -> Result<()> {
        Err(anyhow::anyhow!("Unicode input not available on this platform"))
    }

    /// Type text character by character
    async fn type_text(&self, text: &str, delay_ms: u32) -> Result<()> {
        for ch in text.chars() {
            // Check if injection should stop
            if !*self.injection_active.lock() {
                debug!("Injection stopped by user");
                break;
            }
            
            // Get key presses for this character
            if let Some(key_presses) = self.keycode_mapper.get_keypresses(ch) {
                if key_presses.is_empty() {
                    // Empty vec indicates Unicode input needed
                    debug!("Using Unicode input for character: {:?}", ch);
                    self.send_unicode_char(ch).await?;
                } else {
                    // Normal key press simulation
                    for key_press in key_presses {
                        self.send_key_press(key_press).await?;
                    }
                }
            } else {
                // Character not in map at all, try Unicode input
                warn!("No key mapping for character: {:?}, using Unicode input", ch);
                self.send_unicode_char(ch).await?;
            }
            
            // Delay between keystrokes
            if delay_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms as u64)).await;
            }
        }
        
        Ok(())
    }
    
    /// Paste text using clipboard
    async fn paste_text(&self, text: &str, restore_clipboard: bool) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use clipboard_win::{formats, get_clipboard, set_clipboard};
            
            // Save current clipboard if needed
            let original_content = if restore_clipboard {
                get_clipboard::<String, _>(formats::Unicode).ok()
            } else {
                None
            };
            
            // Set clipboard content
            set_clipboard(formats::Unicode, text)
                .map_err(|e| anyhow::anyhow!("Failed to set clipboard: {:?}", e))?;
            
            // Send Ctrl+V
            let paste_key = KeyPress {
                key: VKey::V,
                modifiers: Modifiers {
                    control: true,
                    ..Default::default()
                },
            };
            
            self.send_key_press(&paste_key).await?;
            
            // Small delay to ensure paste completes
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Restore original clipboard if requested
            if restore_clipboard {
                if let Some(content) = original_content {
                    let _ = set_clipboard(formats::Unicode, &content);
                }
            }
            
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(anyhow::anyhow!("Clipboard paste not implemented for this platform"))
        }
    }
}

#[async_trait]
impl TextInjector for WindowsInjector {
    async fn inject_text(&self, text: &str, mode: &InjectionMode) -> Result<()> {
        // Set injection active
        *self.injection_active.lock() = true;
        
        let result = match mode {
            InjectionMode::Type { delay_ms, .. } => {
                self.type_text(text, *delay_ms).await
            }
            InjectionMode::Paste { restore_clipboard, .. } => {
                self.paste_text(text, *restore_clipboard).await
            }
            InjectionMode::Direct => {
                Err(anyhow::anyhow!("Direct injection mode not yet implemented"))
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
        // Windows text injection is generally available
        // Could check for specific blockers here
        true
    }
    
    fn check_permissions(&self) -> PermissionStatus {
        PermissionStatus {
            available: true,
            accessibility_granted: None, // Not applicable on Windows
            has_required_privileges: true, // SendInput doesn't require special privileges
            message: "Text injection is available on Windows".to_string(),
            required_actions: vec![],
        }
    }
    
    async fn request_permissions(&self) -> Result<()> {
        // No special permissions needed on Windows
        Ok(())
    }
    
    async fn stop_injection(&self) -> Result<()> {
        *self.injection_active.lock() = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_virtual_key_mapping() {
        let injector = WindowsInjector::new().unwrap();
        
        // Test letter mapping
        assert_eq!(injector.virtual_key_to_vk(VKey::A), 0x41);
        assert_eq!(injector.virtual_key_to_vk(VKey::Z), 0x5A);
        
        // Test number mapping
        assert_eq!(injector.virtual_key_to_vk(VKey::Num0), 0x30);
        assert_eq!(injector.virtual_key_to_vk(VKey::Num9), 0x39);
        
        // Test special keys
        assert_eq!(injector.virtual_key_to_vk(VKey::Space), 0x20);
        assert_eq!(injector.virtual_key_to_vk(VKey::Enter), 0x0D);
    }
}