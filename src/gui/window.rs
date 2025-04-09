use crate::audio::device::DeviceManager;
use crate::config::ConfigManager;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DestroyWindow,
    RegisterClassExA, ShowWindow, SW_HIDE, SW_SHOW, 
    WM_CREATE, WM_DESTROY, WM_PAINT, WNDCLASSEXA, WS_EX_LAYERED, WS_EX_TOPMOST,
    WS_POPUP, CW_USEDEFAULT, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASS_STYLES,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, PAINTSTRUCT,
};
use windows::core::PCSTR;

/// Window size constants
const WINDOW_WIDTH: i32 = 600;
const WINDOW_HEIGHT: i32 = 200;

/// Transcription window
pub struct TranscriptionWindow {
    /// Window handle
    hwnd: HWND,
    
    #[allow(dead_code)]
    config_manager: Arc<Mutex<ConfigManager>>,
    
    #[allow(dead_code)]
    device_manager: Arc<DeviceManager>,
    
    /// Window visibility
    visible: bool,
}

impl TranscriptionWindow {
    /// Create a new transcription window
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
            hbrBackground: unsafe { windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::WHITE_BRUSH).into() },
            lpszMenuName: PCSTR::null(),
            lpszClassName: PCSTR(b"BestMeTranscriptionWindow\0".as_ptr()),
            hIconSm: Default::default(),
        };
        
        unsafe {
            RegisterClassExA(&window_class);
        }
        
        // Get screen dimensions for centering the window
        let screen_width = unsafe { windows::Win32::Graphics::Gdi::GetSystemMetrics(windows::Win32::Graphics::Gdi::SM_CXSCREEN) };
        let screen_height = unsafe { windows::Win32::Graphics::Gdi::GetSystemMetrics(windows::Win32::Graphics::Gdi::SM_CYSCREEN) };
        
        let x = (screen_width - WINDOW_WIDTH) / 2;
        let y = (screen_height - WINDOW_HEIGHT) / 2;
        
        // Create window with proper styles for visibility
        let hwnd = unsafe {
            CreateWindowExA(
                WINDOW_EX_STYLE(WS_EX_TOPMOST.0),
                PCSTR(b"BestMeTranscriptionWindow\0".as_ptr()),
                PCSTR(b"BestMe - Speech to Text\0".as_ptr()),
                windows::Win32::UI::WindowsAndMessaging::WS_OVERLAPPEDWINDOW,
                x,
                y,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                None,
                None,
                instance,
                Some(std::ptr::null()),
            )
        };
        
        if hwnd.0 == 0 {
            anyhow::bail!("Failed to create window");
        }
        
        Ok(Self {
            hwnd,
            config_manager,
            device_manager,
            visible: false,
        })
    }
    
    /// Show the window
    pub fn show(&mut self) -> Result<()> {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
        }
        self.visible = true;
        Ok(())
    }
    
    /// Hide the window
    pub fn hide(&mut self) -> Result<()> {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
        self.visible = false;
        Ok(())
    }
    
    /// Toggle window visibility
    pub fn toggle(&mut self) -> Result<()> {
        if self.visible {
            self.hide()
        } else {
            self.show()
        }
    }
    
    /// Start transcription
    pub fn start_transcription(&mut self) -> Result<()> {
        // Implementation will be added in Phase 4
        Ok(())
    }
    
    /// Stop transcription
    pub fn stop_transcription(&mut self) -> Result<()> {
        // Implementation will be added in Phase 4
        Ok(())
    }
    
    /// Window procedure
    extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: windows::Win32::Foundation::WPARAM, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::LRESULT {
        match msg {
            WM_CREATE => {
                // Window creation
                windows::Win32::Foundation::LRESULT(0)
            },
            WM_PAINT => {
                // Paint the window with some text
                let mut ps = PAINTSTRUCT::default();
                unsafe {
                    let hdc = BeginPaint(hwnd, &mut ps);
                    
                    // Create a font
                    let font = windows::Win32::Graphics::Gdi::CreateFontA(
                        24, 0, 0, 0, 
                        windows::Win32::Graphics::Gdi::FW_NORMAL, 
                        0, 0, 0, 
                        windows::Win32::Graphics::Gdi::DEFAULT_CHARSET, 
                        windows::Win32::Graphics::Gdi::OUT_DEFAULT_PRECIS,
                        windows::Win32::Graphics::Gdi::CLIP_DEFAULT_PRECIS,
                        windows::Win32::Graphics::Gdi::DEFAULT_QUALITY,
                        windows::Win32::Graphics::Gdi::DEFAULT_PITCH | windows::Win32::Graphics::Gdi::FF_DONTCARE,
                        PCSTR(b"Arial\0".as_ptr()),
                    );
                    
                    let old_font = windows::Win32::Graphics::Gdi::SelectObject(hdc, font);
                    
                    // Set text color
                    windows::Win32::Graphics::Gdi::SetTextColor(hdc, 0x00000000); // Black
                    
                    // Set transparent background
                    windows::Win32::Graphics::Gdi::SetBkMode(hdc, windows::Win32::Graphics::Gdi::TRANSPARENT);
                    
                    // Display welcome message
                    let message = b"BestMe - Speech to Text App\0";
                    let rect = windows::Win32::Foundation::RECT {
                        left: 20,
                        top: 20,
                        right: WINDOW_WIDTH,
                        bottom: WINDOW_HEIGHT,
                    };
                    
                    windows::Win32::Graphics::Gdi::DrawTextA(
                        hdc,
                        PCSTR(message.as_ptr()),
                        -1,
                        &rect,
                        windows::Win32::Graphics::Gdi::DT_LEFT | windows::Win32::Graphics::Gdi::DT_TOP,
                    );
                    
                    // Add status message
                    let status = b"Ready to transcribe. Start speaking...\0";
                    let status_rect = windows::Win32::Foundation::RECT {
                        left: 20,
                        top: 70,
                        right: WINDOW_WIDTH,
                        bottom: WINDOW_HEIGHT,
                    };
                    
                    windows::Win32::Graphics::Gdi::DrawTextA(
                        hdc,
                        PCSTR(status.as_ptr()),
                        -1,
                        &status_rect,
                        windows::Win32::Graphics::Gdi::DT_LEFT | windows::Win32::Graphics::Gdi::DT_TOP,
                    );
                    
                    // Clean up
                    windows::Win32::Graphics::Gdi::SelectObject(hdc, old_font);
                    windows::Win32::Graphics::Gdi::DeleteObject(font);
                    
                    EndPaint(hwnd, &ps);
                }
                windows::Win32::Foundation::LRESULT(0)
            },
            WM_DESTROY => {
                // Window destruction
                unsafe {
                    windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                }
                windows::Win32::Foundation::LRESULT(0)
            },
            _ => unsafe {
                DefWindowProcA(hwnd, msg, wparam, lparam)
            },
        }
    }
}

impl Drop for TranscriptionWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd);
        }
    }
} 
