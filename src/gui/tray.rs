use crate::audio::device::DeviceManager;
use crate::config::ConfigManager;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExA, DefWindowProcA, DestroyWindow,
    RegisterClassExA, HMENU, WM_APP, WM_DESTROY,
    WNDCLASSEXA, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASS_STYLES,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconA, NOTIFYICONDATAA, NIF_ICON, NIF_MESSAGE, 
    NIF_TIP, NIM_ADD, NIM_DELETE,
};
use windows::core::PCSTR;

/// Tray icon message ID
const TRAY_ICON_MESSAGE: u32 = WM_APP + 1;

/// Tray icon menu IDs
const MENU_SETTINGS: u32 = 1;
const MENU_START: u32 = 2;
const MENU_STOP: u32 = 3;
const MENU_EXIT: u32 = 4;

/// Tray icon
pub struct TrayIcon {
    /// Window handle
    hwnd: HWND,
    
    /// Menu handle
    menu: HMENU,
    
    /// Configuration manager
    config_manager: Arc<Mutex<ConfigManager>>,
    
    /// Device manager
    device_manager: Arc<DeviceManager>,
}

impl TrayIcon {
    /// Create a new tray icon
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
            lpszClassName: PCSTR(b"BestMeTrayIcon\0".as_ptr()),
            hIconSm: Default::default(),
        };
        
        unsafe {
            RegisterClassExA(&window_class);
        }
        
        // Create window
        let hwnd = unsafe {
            CreateWindowExA(
                WINDOW_EX_STYLE(0),
                PCSTR(b"BestMeTrayIcon\0".as_ptr()),
                PCSTR(b"BestMe Tray\0".as_ptr()),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                None,
                None,
                instance,
                Some(std::ptr::null()),
            )
        };
        
        if hwnd.0 == 0 {
            anyhow::bail!("Failed to create tray window");
        }
        
        // Create tray icon
        let mut nid = NOTIFYICONDATAA::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAA>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = TRAY_ICON_MESSAGE;
        
        // Load icon
        nid.hIcon = unsafe {
            let icon_id = windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION;
            windows::Win32::UI::WindowsAndMessaging::LoadIconW(
                None,
                icon_id,
            )
            .unwrap()
        };
        
        // Set tooltip
        let tip = b"BestMe Transcription\0";
        unsafe {
            std::ptr::copy_nonoverlapping(
                tip.as_ptr(),
                nid.szTip.as_mut_ptr(),
                tip.len(),
            );
        }
        
        // Add notification icon
        unsafe {
            Shell_NotifyIconA(NIM_ADD, &nid);
        }
        
        // Create popup menu
        let menu = unsafe { CreatePopupMenu().unwrap() };
        
        // Add menu items
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::AppendMenuA(
                menu,
                windows::Win32::UI::WindowsAndMessaging::MF_STRING,
                MENU_START as usize,
                PCSTR(b"Start Transcription\0".as_ptr()),
            );
            
            windows::Win32::UI::WindowsAndMessaging::AppendMenuA(
                menu,
                windows::Win32::UI::WindowsAndMessaging::MF_STRING,
                MENU_STOP as usize,
                PCSTR(b"Stop Transcription\0".as_ptr()),
            );
            
            windows::Win32::UI::WindowsAndMessaging::AppendMenuA(
                menu,
                windows::Win32::UI::WindowsAndMessaging::MF_SEPARATOR,
                0,
                PCSTR::null(),
            );
            
            windows::Win32::UI::WindowsAndMessaging::AppendMenuA(
                menu,
                windows::Win32::UI::WindowsAndMessaging::MF_STRING,
                MENU_SETTINGS as usize,
                PCSTR(b"Settings\0".as_ptr()),
            );
            
            windows::Win32::UI::WindowsAndMessaging::AppendMenuA(
                menu,
                windows::Win32::UI::WindowsAndMessaging::MF_SEPARATOR,
                0,
                PCSTR::null(),
            );
            
            windows::Win32::UI::WindowsAndMessaging::AppendMenuA(
                menu,
                windows::Win32::UI::WindowsAndMessaging::MF_STRING,
                MENU_EXIT as usize,
                PCSTR(b"Exit\0".as_ptr()),
            );
        }
        
        Ok(Self {
            hwnd,
            menu,
            config_manager,
            device_manager,
        })
    }
    
    /// Window procedure
    extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        match msg {
            TRAY_ICON_MESSAGE => {
                match lparam.0 as u32 {
                    windows::Win32::UI::WindowsAndMessaging::WM_RBUTTONUP => {
                        // Show context menu
                        unsafe {
                            let mut point = windows::Win32::Foundation::POINT::default();
                            windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point);
                            
                            windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
                            
                            let flags = windows::Win32::UI::WindowsAndMessaging::TPM_RIGHTBUTTON;
                            let _tpm_result = windows::Win32::UI::WindowsAndMessaging::TrackPopupMenu(
                                // Get menu from class instance
                                // This is a simplification; we'd need to store the menu handle
                                // in a static or window property in a real implementation
                                HMENU(0),
                                flags,
                                point.x,
                                point.y,
                                0,
                                hwnd,
                                None,
                            );
                            
                            windows::Win32::UI::WindowsAndMessaging::PostMessageA(hwnd, 0, windows::Win32::Foundation::WPARAM(0), windows::Win32::Foundation::LPARAM(0));
                        }
                        windows::Win32::Foundation::LRESULT(0)
                    },
                    windows::Win32::UI::WindowsAndMessaging::WM_LBUTTONUP => {
                        // Toggle transcription window
                        windows::Win32::Foundation::LRESULT(0)
                    },
                    _ => windows::Win32::Foundation::LRESULT(0),
                }
            },
            windows::Win32::UI::WindowsAndMessaging::WM_COMMAND => {
                let command_id = wparam.0 as u32 & 0xFFFF;
                match command_id {
                    MENU_START => {
                        // Start transcription
                        windows::Win32::Foundation::LRESULT(0)
                    },
                    MENU_STOP => {
                        // Stop transcription
                        windows::Win32::Foundation::LRESULT(0)
                    },
                    MENU_SETTINGS => {
                        // Show settings dialog
                        windows::Win32::Foundation::LRESULT(0)
                    },
                    MENU_EXIT => {
                        // Exit application
                        unsafe {
                            windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                        }
                        windows::Win32::Foundation::LRESULT(0)
                    },
                    _ => unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) },
                }
            },
            WM_DESTROY => {
                // Remove tray icon
                let mut nid = NOTIFYICONDATAA::default();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAA>() as u32;
                nid.hWnd = hwnd;
                nid.uID = 1;
                
                unsafe {
                    Shell_NotifyIconA(NIM_DELETE, &nid);
                    windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                }
                
                windows::Win32::Foundation::LRESULT(0)
            },
            _ => unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) },
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        // Remove tray icon
        let mut nid = NOTIFYICONDATAA::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAA>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;
        
        unsafe {
            Shell_NotifyIconA(NIM_DELETE, &nid);
            DestroyWindow(self.hwnd);
        }
    }
} 
