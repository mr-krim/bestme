use crate::audio::device::DeviceManager;
use crate::config::ConfigManager;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExA, DefWindowProcA, DestroyWindow,
    RegisterClassExA, HMENU, WM_APP, WM_DESTROY,
    WNDCLASSEXA, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASS_STYLES,
    AppendMenuA, MF_STRING, MF_SEPARATOR, LoadIconW, IDI_APPLICATION,
    GetCursorPos, SetForegroundWindow, TrackPopupMenu, TPM_RIGHTBUTTON,
    PostMessageA, WM_RBUTTONUP, WM_LBUTTONUP, LoadCursorW, IDC_ARROW,
};
use windows::Win32::Foundation::{HWND, POINT, WPARAM, LPARAM, LRESULT};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconA, NOTIFYICONDATAA, NIF_ICON, NIF_MESSAGE, 
    NIF_TIP, NIM_ADD, NIM_DELETE,
};
use windows::core::PCSTR;

/// System tray manager
pub struct SystemTray {
    /// Window handle
    hwnd: HWND,
    
    /// Menu handle
    menu: HMENU,
    
    /// Configuration manager
    #[allow(dead_code)]
    config_manager: Arc<Mutex<ConfigManager>>,
    
    /// Device manager
    #[allow(dead_code)]
    device_manager: Arc<Mutex<DeviceManager>>,
}

/// Tray icon message ID
const TRAY_ICON_MESSAGE: u32 = WM_APP + 1;

/// Tray icon menu IDs
const MENU_SETTINGS: u32 = 1;
const MENU_START: u32 = 2;
const MENU_STOP: u32 = 3;
const MENU_EXIT: u32 = 4;

impl SystemTray {
    /// Create a new system tray
    pub fn new(config_manager: Arc<Mutex<ConfigManager>>, device_manager: Arc<Mutex<DeviceManager>>) -> Result<Self> {
        // Register window class for tray icon
        let instance = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleA(None).unwrap() };
        
        let window_class = WNDCLASSEXA {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(Self::wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: Default::default(),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap() },
            hbrBackground: Default::default(),
            lpszMenuName: PCSTR::null(),
            lpszClassName: PCSTR(b"BestMeTrayIcon\0".as_ptr()),
            hIconSm: Default::default(),
        };
        
        unsafe {
            RegisterClassExA(&window_class);
        }
        
        // Create a hidden window to handle tray messages
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
                Some(instance),
                Some(std::ptr::null()),
            )
        };
        
        let hwnd = hwnd?;
        if hwnd.0.is_null() {
            anyhow::bail!("Failed to create tray window");
        }
        
        // Create system tray icon
        let mut nid = NOTIFYICONDATAA::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAA>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = TRAY_ICON_MESSAGE;
        
        // Load icon
        nid.hIcon = unsafe {
            LoadIconW(None, IDI_APPLICATION).unwrap()
        };
        
        // Set tooltip
        let tip = b"BestMe Transcription\0";
        unsafe {
            std::ptr::copy_nonoverlapping(
                tip.as_ptr() as *const i8,
                nid.szTip.as_mut_ptr(),
                tip.len(),
            );
        }
        
        // Add notification icon
        let result = unsafe {
            Shell_NotifyIconA(NIM_ADD, &nid)
        };
        
        if !result.as_bool() {
            anyhow::bail!("Failed to add tray icon");
        }
        
        // Create popup menu
        let menu = unsafe { CreatePopupMenu().unwrap() };
        
        // Add menu items
        unsafe {
            AppendMenuA(
                menu,
                MF_STRING,
                MENU_START as usize,
                PCSTR(b"Start Transcription\0".as_ptr()),
            );
            
            AppendMenuA(
                menu,
                MF_STRING,
                MENU_STOP as usize,
                PCSTR(b"Stop Transcription\0".as_ptr()),
            );
            
            AppendMenuA(
                menu,
                MF_SEPARATOR,
                0,
                PCSTR::null(),
            );
            
            AppendMenuA(
                menu,
                MF_STRING,
                MENU_SETTINGS as usize,
                PCSTR(b"Settings\0".as_ptr()),
            );
            
            AppendMenuA(
                menu,
                MF_SEPARATOR,
                0,
                PCSTR::null(),
            );
            
            AppendMenuA(
                menu,
                MF_STRING,
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
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            TRAY_ICON_MESSAGE => {
                match lparam.0 as u32 {
                    WM_RBUTTONUP => {
                        // Show context menu
                        unsafe {
                            let mut point = POINT::default();
                            GetCursorPos(&mut point);
                            
                            SetForegroundWindow(hwnd);
                            
                            let flags = TPM_RIGHTBUTTON;
                            let _tpm_result = TrackPopupMenu(
                                HMENU(std::ptr::null_mut()), // This should ideally be the menu handle
                                flags,
                                point.x,
                                point.y,
                                Some(0),
                                hwnd,
                                None,
                            );
                            
                            PostMessageA(Some(hwnd), 0, WPARAM(0), LPARAM(0));
                        }
                        LRESULT(0)
                    },
                    WM_LBUTTONUP => {
                        // Toggle transcription window
                        LRESULT(0)
                    },
                    _ => LRESULT(0),
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
                }
                
                LRESULT(0)
            },
            _ => unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) },
        }
    }
}

impl Drop for SystemTray {
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
