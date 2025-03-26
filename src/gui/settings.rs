use crate::audio::device::DeviceManager;
use crate::config::ConfigManager;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DestroyWindow, RegisterClassExA,
    ShowWindow, SW_HIDE, SW_SHOW, WM_CREATE,
    WM_DESTROY, WNDCLASSEXA, WS_CAPTION, WS_SYSMENU,
    WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASS_STYLES,
};
use windows::core::PCSTR;

/// Settings dialog
pub struct SettingsDialog {
    /// Window handle
    hwnd: HWND,
    
    /// Configuration manager
    config_manager: Arc<Mutex<ConfigManager>>,
    
    /// Device manager
    device_manager: Arc<DeviceManager>,
}

impl SettingsDialog {
    /// Create a new settings dialog
    pub fn new(config_manager: Arc<Mutex<ConfigManager>>, device_manager: Arc<DeviceManager>) -> Result<Self> {
        // Register window class
        let instance = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleA(None).unwrap() };
        
        let window_class = WNDCLASSEXA {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(Self::wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: Default::default(),
            hCursor: unsafe { windows::Win32::UI::WindowsAndMessaging::LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW).unwrap() },
            hbrBackground: Default::default(),
            lpszMenuName: PCSTR::null(),
            lpszClassName: PCSTR(b"BestMeSettingsDialog\0".as_ptr()),
            hIconSm: Default::default(),
        };
        
        unsafe {
            RegisterClassExA(&window_class);
        }
        
        // Create window
        let hwnd = unsafe {
            CreateWindowExA(
                WINDOW_EX_STYLE(0),
                PCSTR(b"BestMeSettingsDialog\0".as_ptr()),
                PCSTR(b"BestMe Settings\0".as_ptr()),
                WINDOW_STYLE(WS_CAPTION.0 | WS_SYSMENU.0),
                100,
                100,
                400,
                500,
                None,
                None,
                instance,
                Some(std::ptr::null()),
            )
        };
        
        if hwnd.0 == 0 {
            anyhow::bail!("Failed to create settings dialog");
        }
        
        Ok(Self {
            hwnd,
            config_manager,
            device_manager,
        })
    }
    
    /// Show the dialog
    pub fn show(&mut self) -> Result<()> {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
        }
        Ok(())
    }
    
    /// Hide the dialog
    pub fn hide(&mut self) -> Result<()> {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
        Ok(())
    }
    
    /// Window procedure
    extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        match msg {
            WM_CREATE => {
                // Create controls
                windows::Win32::Foundation::LRESULT(0)
            },
            WM_DESTROY => {
                // Window destruction
                unsafe {
                    windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                }
                windows::Win32::Foundation::LRESULT(0)
            },
            _ => unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) },
        }
    }
}

impl Drop for SettingsDialog {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd);
        }
    }
} 
