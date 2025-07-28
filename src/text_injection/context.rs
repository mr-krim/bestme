//! Context detection for active window and application information

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use log::warn;

#[cfg(target_os = "windows")]
use windows::{
    Win32::{
        Foundation::{HWND, MAX_PATH},
        UI::WindowsAndMessaging::*,
        System::Threading::*,
    },
    core::PWSTR,
};

#[cfg(target_os = "macos")]
use {
    cocoa::{base::id, foundation::NSString},
    objc::{msg_send, sel, sel_impl, class},
};

#[cfg(target_os = "linux")]
use {
    x11::xlib::*,
    std::{ffi::CStr, ptr},
};

/// Information about the currently active window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    /// Window title
    pub title: String,
    /// Application name
    pub app_name: String,
    /// Full path to the application executable
    pub app_path: Option<String>,
    /// Process ID of the application
    pub pid: Option<u32>,
    /// Whether the application is running with elevated privileges
    pub is_elevated: bool,
    /// Whether the window is in focus
    pub is_focused: bool,
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            title: String::new(),
            app_name: "Unknown".to_string(),
            app_path: None,
            pid: None,
            is_elevated: false,
            is_focused: false,
        }
    }
}

/// Context detector for monitoring active windows
pub struct ContextDetector {
    #[cfg(target_os = "linux")]
    x11_display: Option<*mut Display>,
}

// Mark ContextDetector as Send + Sync since we handle thread safety internally
unsafe impl Send for ContextDetector {}
unsafe impl Sync for ContextDetector {}

impl ContextDetector {
    /// Create a new context detector
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        let x11_display = unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                warn!("Failed to open X11 display for context detection");
                None
            } else {
                Some(display)
            }
        };
        
        Self {
            #[cfg(target_os = "linux")]
            x11_display,
        }
    }
    
    /// Get information about the currently active window
    pub async fn get_active_window(&self) -> Result<WindowInfo> {
        // This would call platform-specific implementation
        #[cfg(target_os = "windows")]
        return self.get_active_window_windows().await;
        
        #[cfg(target_os = "macos")]
        return self.get_active_window_macos().await;
        
        #[cfg(target_os = "linux")]
        return self.get_active_window_linux().await;
    }
    
    #[cfg(target_os = "windows")]
    async fn get_active_window_windows(&self) -> Result<WindowInfo> {
        unsafe {
            // Get the foreground window
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 {
                return Err(anyhow::anyhow!("No active window found"));
            }
            
            // Get window title
            let mut title_buf = vec![0u16; 256];
            let title_len = GetWindowTextW(hwnd, &mut title_buf);
            let title = if title_len > 0 {
                String::from_utf16_lossy(&title_buf[..title_len as usize])
            } else {
                String::new()
            };
            
            // Get process ID
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            
            // Get process handle to get executable path
            let mut app_name = "Unknown".to_string();
            let mut app_path = None;
            let mut is_elevated = false;
            
            if let Ok(process) = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) {
                // Get executable path
                let mut path_buf = vec![0u16; MAX_PATH as usize];
                let mut path_len = MAX_PATH;
                
                if QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, 
                                            PWSTR(path_buf.as_mut_ptr()), &mut path_len).is_ok() {
                    let path = String::from_utf16_lossy(&path_buf[..path_len as usize]);
                    app_path = Some(path.clone());
                    
                    // Extract app name from path
                    if let Some(name) = path.split('\\').last() {
                        app_name = name.trim_end_matches(".exe").to_string();
                    }
                }
                
                // Check if process is elevated
                let mut token = windows::Win32::Foundation::HANDLE::default();
                if OpenProcessToken(process, TOKEN_QUERY, &mut token).is_ok() {
                    let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
                    let mut return_length = 0u32;
                    let elevation_ptr = &mut elevation as *mut _ as *mut std::ffi::c_void;
                    
                    if GetTokenInformation(token, TokenElevation, 
                                         Some(elevation_ptr),
                                         std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                                         &mut return_length).is_ok() {
                        is_elevated = elevation.TokenIsElevated != 0;
                    }
                    
                    let _ = CloseHandle(token);
                }
                
                let _ = CloseHandle(process);
            }
            
            Ok(WindowInfo {
                title,
                app_name,
                app_path,
                pid: Some(pid),
                is_elevated,
                is_focused: true,
            })
        }
    }
    
    #[cfg(target_os = "macos")]
    async fn get_active_window_macos(&self) -> Result<WindowInfo> {
        unsafe {
            let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
            let frontmost_app: id = msg_send![workspace, frontmostApplication];
            
            if frontmost_app.is_null() {
                return Err(anyhow::anyhow!("No active application found"));
            }
            
            // Get app name
            let localized_name: id = msg_send![frontmost_app, localizedName];
            let app_name = if !localized_name.is_null() {
                let c_str: *const i8 = msg_send![localized_name, UTF8String];
                if !c_str.is_null() {
                    std::ffi::CStr::from_ptr(c_str).to_string_lossy().to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };
            
            // Get bundle URL for app path
            let bundle_url: id = msg_send![frontmost_app, bundleURL];
            let app_path = if !bundle_url.is_null() {
                let path: id = msg_send![bundle_url, path];
                if !path.is_null() {
                    let c_str: *const i8 = msg_send![path, UTF8String];
                    if !c_str.is_null() {
                        Some(std::ffi::CStr::from_ptr(c_str).to_string_lossy().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            
            // Get PID
            let pid: i32 = msg_send![frontmost_app, processIdentifier];
            
            // Note: Getting window title on macOS requires additional Accessibility API calls
            // which are more complex. For now, we'll use the app name as title
            let title = app_name.clone();
            
            Ok(WindowInfo {
                title,
                app_name,
                app_path,
                pid: Some(pid as u32),
                is_elevated: false, // macOS doesn't have the same elevation concept
                is_focused: true,
            })
        }
    }
    
    #[cfg(target_os = "linux")]
    async fn get_active_window_linux(&self) -> Result<WindowInfo> {
        let display = self.x11_display
            .context("X11 display not initialized")?;
        
        unsafe {
            let root = XDefaultRootWindow(display);
            let mut focused_window = 0;
            let mut revert_to = 0;
            
            // Get the focused window
            XGetInputFocus(display, &mut focused_window, &mut revert_to);
            
            if focused_window == 0 || focused_window == 1 {
                // Try to get the active window property
                let atom_active = XInternAtom(display, 
                    b"_NET_ACTIVE_WINDOW\0".as_ptr() as *const i8, False);
                
                let mut actual_type = 0;
                let mut actual_format = 0;
                let mut n_items = 0;
                let mut bytes_after = 0;
                let mut prop: *mut u8 = ptr::null_mut();
                
                if XGetWindowProperty(display, root, atom_active, 0, 1, False,
                                    AnyPropertyType as u64, &mut actual_type,
                                    &mut actual_format, &mut n_items,
                                    &mut bytes_after, &mut prop) == Success as i32 
                    && n_items > 0 {
                    focused_window = *(prop as *const Window);
                    XFree(prop as *mut _);
                }
            }
            
            if focused_window == 0 || focused_window == 1 {
                return Err(anyhow::anyhow!("No active window found"));
            }
            
            // Get window title
            let mut title = String::new();
            let atom_name = XInternAtom(display, 
                b"_NET_WM_NAME\0".as_ptr() as *const i8, False);
            let atom_utf8 = XInternAtom(display, 
                b"UTF8_STRING\0".as_ptr() as *const i8, False);
            
            let mut actual_type = 0;
            let mut actual_format = 0;
            let mut n_items = 0;
            let mut bytes_after = 0;
            let mut prop: *mut u8 = ptr::null_mut();
            
            if XGetWindowProperty(display, focused_window, atom_name, 0, 1024, False,
                                atom_utf8, &mut actual_type, &mut actual_format,
                                &mut n_items, &mut bytes_after, &mut prop) == Success as i32 
                && n_items > 0 {
                let c_str = CStr::from_ptr(prop as *const i8);
                title = c_str.to_string_lossy().to_string();
                XFree(prop as *mut _);
            } else {
                // Fallback to WM_NAME
                let mut text_prop = XTextProperty {
                    value: ptr::null_mut(),
                    encoding: 0,
                    format: 0,
                    nitems: 0,
                };
                
                if XGetWMName(display, focused_window, &mut text_prop) != 0 
                    && !text_prop.value.is_null() {
                    let c_str = CStr::from_ptr(text_prop.value as *const i8);
                    title = c_str.to_string_lossy().to_string();
                    XFree(text_prop.value as *mut _);
                }
            }
            
            // Get window PID
            let mut pid = None;
            let atom_pid = XInternAtom(display, 
                b"_NET_WM_PID\0".as_ptr() as *const i8, False);
            
            let mut actual_type = 0;
            let mut actual_format = 0;
            let mut n_items = 0;
            let mut bytes_after = 0;
            let mut prop: *mut u8 = ptr::null_mut();
            
            if XGetWindowProperty(display, focused_window, atom_pid, 0, 1, False,
                                XA_CARDINAL, &mut actual_type, &mut actual_format,
                                &mut n_items, &mut bytes_after, &mut prop) == Success as i32 
                && n_items > 0 {
                pid = Some(*(prop as *const u32));
                XFree(prop as *mut _);
            }
            
            // Try to get app name from WM_CLASS
            let mut class_hint = XClassHint {
                res_name: ptr::null_mut(),
                res_class: ptr::null_mut(),
            };
            
            let mut app_name = "Unknown".to_string();
            if XGetClassHint(display, focused_window, &mut class_hint) != 0 {
                if !class_hint.res_class.is_null() {
                    let c_str = CStr::from_ptr(class_hint.res_class);
                    app_name = c_str.to_string_lossy().to_string();
                    XFree(class_hint.res_class as *mut _);
                }
                if !class_hint.res_name.is_null() {
                    XFree(class_hint.res_name as *mut _);
                }
            }
            
            Ok(WindowInfo {
                title,
                app_name,
                app_path: None, // Getting app path on Linux is complex
                pid,
                is_elevated: false, // Would need to check process uid
                is_focused: true,
            })
        }
    }
    
    /// Monitor for window focus changes
    pub async fn monitor_focus_changes<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(WindowInfo) + Send + 'static,
    {
        // Platform-specific implementation would set up event monitoring
        // For now, this is a placeholder
        let _ = callback;
        Ok(())
    }
}

impl Drop for ContextDetector {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        if let Some(display) = self.x11_display {
            unsafe {
                XCloseDisplay(display);
            }
        }
    }
}