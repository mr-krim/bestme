// Windows API fixes for text injection
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::Foundation::*;

// Helper to convert u16 to VIRTUAL_KEY
pub fn make_virtual_key(code: u16) -> VIRTUAL_KEY {
    VIRTUAL_KEY(code)
}

// Constants for keyboard event flags
pub const KEYEVENTF_NONE: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);
pub const KEYEVENTF_EXTENDEDKEY: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0x0001);
pub const KEYEVENTF_KEYUP: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0x0002);
pub const KEYEVENTF_UNICODE: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0x0004);
pub const KEYEVENTF_SCANCODE: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0x0008);