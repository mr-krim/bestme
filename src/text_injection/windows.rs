//! Windows implementation of text injection using SendInput

use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::sync::Arc;
use parking_lot::Mutex;

use super::{TextInjector, InjectionMode, PermissionStatus, WindowInfo};
use super::context::ContextDetector;
use super::keycodes::{VirtualKey, GenericKeycodeMapper, KeyPress, Modifiers};

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
    fn virtual_key_to_vk(&self, key: VirtualKey) -> u16 {
        match key {
            // Letters
            VirtualKey::A => 0x41,
            VirtualKey::B => 0x42,
            VirtualKey::C => 0x43,
            VirtualKey::D => 0x44,
            VirtualKey::E => 0x45,
            VirtualKey::F => 0x46,
            VirtualKey::G => 0x47,
            VirtualKey::H => 0x48,
            VirtualKey::I => 0x49,
            VirtualKey::J => 0x4A,
            VirtualKey::K => 0x4B,
            VirtualKey::L => 0x4C,
            VirtualKey::M => 0x4D,
            VirtualKey::N => 0x4E,
            VirtualKey::O => 0x4F,
            VirtualKey::P => 0x50,
            VirtualKey::Q => 0x51,
            VirtualKey::R => 0x52,
            VirtualKey::S => 0x53,
            VirtualKey::T => 0x54,
            VirtualKey::U => 0x55,
            VirtualKey::V => 0x56,
            VirtualKey::W => 0x57,
            VirtualKey::X => 0x58,
            VirtualKey::Y => 0x59,
            VirtualKey::Z => 0x5A,
            
            // Numbers
            VirtualKey::Num0 => 0x30,
            VirtualKey::Num1 => 0x31,
            VirtualKey::Num2 => 0x32,
            VirtualKey::Num3 => 0x33,
            VirtualKey::Num4 => 0x34,
            VirtualKey::Num5 => 0x35,
            VirtualKey::Num6 => 0x36,
            VirtualKey::Num7 => 0x37,
            VirtualKey::Num8 => 0x38,
            VirtualKey::Num9 => 0x39,
            
            // Function keys
            VirtualKey::F1 => 0x70,
            VirtualKey::F2 => 0x71,
            VirtualKey::F3 => 0x72,
            VirtualKey::F4 => 0x73,
            VirtualKey::F5 => 0x74,
            VirtualKey::F6 => 0x75,
            VirtualKey::F7 => 0x76,
            VirtualKey::F8 => 0x77,
            VirtualKey::F9 => 0x78,
            VirtualKey::F10 => 0x79,
            VirtualKey::F11 => 0x7A,
            VirtualKey::F12 => 0x7B,
            
            // Modifiers
            VirtualKey::Shift => 0x10,
            VirtualKey::Control => 0x11,
            VirtualKey::Alt => 0x12,
            VirtualKey::Meta => 0x5B, // Left Windows key
            
            // Special keys
            VirtualKey::Space => 0x20,
            VirtualKey::Enter => 0x0D,
            VirtualKey::Tab => 0x09,
            VirtualKey::Backspace => 0x08,
            VirtualKey::Delete => 0x2E,
            VirtualKey::Escape => 0x1B,
            VirtualKey::CapsLock => 0x14,
            VirtualKey::NumLock => 0x90,
            VirtualKey::ScrollLock => 0x91,
            
            // Navigation
            VirtualKey::Left => 0x25,
            VirtualKey::Right => 0x27,
            VirtualKey::Up => 0x26,
            VirtualKey::Down => 0x28,
            VirtualKey::Home => 0x24,
            VirtualKey::End => 0x23,
            VirtualKey::PageUp => 0x21,
            VirtualKey::PageDown => 0x22,
            
            // Punctuation
            VirtualKey::Minus => 0xBD,
            VirtualKey::Equals => 0xBB,
            VirtualKey::LeftBracket => 0xDB,
            VirtualKey::RightBracket => 0xDD,
            VirtualKey::Semicolon => 0xBA,
            VirtualKey::Quote => 0xDE,
            VirtualKey::Backslash => 0xDC,
            VirtualKey::Comma => 0xBC,
            VirtualKey::Period => 0xBE,
            VirtualKey::Slash => 0xBF,
            VirtualKey::Grave => 0xC0,
            
            // Numpad
            VirtualKey::Numpad0 => 0x60,
            VirtualKey::Numpad1 => 0x61,
            VirtualKey::Numpad2 => 0x62,
            VirtualKey::Numpad3 => 0x63,
            VirtualKey::Numpad4 => 0x64,
            VirtualKey::Numpad5 => 0x65,
            VirtualKey::Numpad6 => 0x66,
            VirtualKey::Numpad7 => 0x67,
            VirtualKey::Numpad8 => 0x68,
            VirtualKey::Numpad9 => 0x69,
            VirtualKey::NumpadMultiply => 0x6A,
            VirtualKey::NumpadAdd => 0x6B,
            VirtualKey::NumpadSubtract => 0x6D,
            VirtualKey::NumpadDecimal => 0x6E,
            VirtualKey::NumpadDivide => 0x6F,
        }
    }
    
    /// Simulate a key press using SendInput
    #[cfg(target_os = "windows")]
    async fn send_key_press(&self, key_press: &KeyPress) -> Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        use windows::Win32::UI::WindowsAndMessaging::*;
        
        let mut inputs = Vec::new();
        
        // Press modifier keys if needed
        if key_press.modifiers.shift {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: self.virtual_key_to_vk(VirtualKey::Shift),
                        wScan: 0,
                        dwFlags: KEYEVENTF(0),
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
                        wVk: self.virtual_key_to_vk(VirtualKey::Control),
                        wScan: 0,
                        dwFlags: KEYEVENTF(0),
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
                        wVk: self.virtual_key_to_vk(VirtualKey::Alt),
                        wScan: 0,
                        dwFlags: KEYEVENTF(0),
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
                    wVk: vk_code,
                    wScan: 0,
                    dwFlags: KEYEVENTF(0),
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
                    wVk: vk_code,
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
                        wVk: self.virtual_key_to_vk(VirtualKey::Alt),
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
                        wVk: self.virtual_key_to_vk(VirtualKey::Control),
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
                        wVk: self.virtual_key_to_vk(VirtualKey::Shift),
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
        use windows::Win32::UI::WindowsAndMessaging::*;
        
        let mut inputs = Vec::new();
        
        // Convert char to UTF-16
        let mut utf16_buf = [0u16; 2];
        let utf16_slice = ch.encode_utf16(&mut utf16_buf);
        
        for &code_unit in utf16_slice {
            // Send Unicode character using KEYEVENTF_UNICODE flag
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: 0, // Must be 0 for Unicode
                        wScan: code_unit,
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
                        wVk: 0,
                        wScan: code_unit,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
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
                .context("Failed to set clipboard")?;
            
            // Send Ctrl+V
            let paste_key = KeyPress {
                key: VirtualKey::V,
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
        assert_eq!(injector.virtual_key_to_vk(VirtualKey::A), 0x41);
        assert_eq!(injector.virtual_key_to_vk(VirtualKey::Z), 0x5A);
        
        // Test number mapping
        assert_eq!(injector.virtual_key_to_vk(VirtualKey::Num0), 0x30);
        assert_eq!(injector.virtual_key_to_vk(VirtualKey::Num9), 0x39);
        
        // Test special keys
        assert_eq!(injector.virtual_key_to_vk(VirtualKey::Space), 0x20);
        assert_eq!(injector.virtual_key_to_vk(VirtualKey::Enter), 0x0D);
    }
}