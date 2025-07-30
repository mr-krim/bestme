//! Windows type compatibility layer for different versions of the windows crate

#[cfg(target_os = "windows")]
pub mod compat {
    use windows::Win32::UI::Input::KeyboardAndMouse::{KEYBD_EVENT_FLAGS, VIRTUAL_KEY};
    
    // Create type aliases for compatibility
    pub type VirtualKey = u16;
    pub type KeyEventFlags = u32;
    
    // Re-export the actual constants (only used ones)
    #[allow(unused_imports)] // These are used via wildcard import in windows.rs
    pub use windows::Win32::UI::Input::KeyboardAndMouse::{
        KEYEVENTF_KEYUP, 
        KEYEVENTF_UNICODE,
    };
    
    // Helper function to create KEYEVENTF from our flag type
    pub fn create_keyeventf(flags: KeyEventFlags) -> KEYBD_EVENT_FLAGS {
        KEYBD_EVENT_FLAGS(flags)
    }
    
    // Helper to create VIRTUAL_KEY from u16
    pub fn create_virtual_key(key: u16) -> VIRTUAL_KEY {
        VIRTUAL_KEY(key)
    }
}