use crate::audio::device::DeviceManager;
use crate::config::ConfigManager;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::Win32::Foundation::{HWND, RECT, COLORREF};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DestroyWindow,
    RegisterClassExA, ShowWindow, SW_HIDE, SW_SHOW, 
    WM_CREATE, WM_DESTROY, WM_PAINT, WNDCLASSEXA, WS_EX_TOPMOST,
    WS_OVERLAPPEDWINDOW, WINDOW_EX_STYLE, WNDCLASS_STYLES,
    GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, PAINTSTRUCT, GetStockObject, WHITE_BRUSH, HBRUSH,
    CreateFontA, SelectObject, SetTextColor, SetBkMode, TRANSPARENT, HGDIOBJ,
    DrawTextA, DeleteObject, DT_LEFT, DT_TOP,
    DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
    CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, DEFAULT_PITCH, FF_DONTCARE,
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
            hbrBackground: unsafe { HBRUSH(GetStockObject(WHITE_BRUSH).0) },
            lpszMenuName: PCSTR::null(),
            lpszClassName: PCSTR(b"BestMeTranscriptionWindow\0".as_ptr()),
            hIconSm: Default::default(),
        };
        
        unsafe {
            RegisterClassExA(&window_class);
        }
        
        // Get screen dimensions for centering the window
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        
        let x = (screen_width - WINDOW_WIDTH) / 2;
        let y = (screen_height - WINDOW_HEIGHT) / 2;
        
        // Create window with proper styles for visibility
        let hwnd = unsafe {
            CreateWindowExA(
                WINDOW_EX_STYLE(WS_EX_TOPMOST.0),
                PCSTR(b"BestMeTranscriptionWindow\0".as_ptr()),
                PCSTR(b"BestMe - Speech to Text\0".as_ptr()),
                WS_OVERLAPPEDWINDOW,
                x,
                y,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                None,
                None,
                Some(instance),
                Some(std::ptr::null()),
            )
        };
        
        let hwnd = hwnd?;
        if hwnd.0.is_null() {
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
                    let font = CreateFontA(
                        24,             // Height
                        0,              // Width
                        0,              // Escapement
                        0,              // Orientation
                        400,            // Weight (400 = normal)
                        0,              // Italic
                        0,              // Underline
                        0,              // StrikeOut
                        DEFAULT_CHARSET,               // CharSet
                        OUT_DEFAULT_PRECIS,            // OutPrecision
                        CLIP_DEFAULT_PRECIS,           // ClipPrecision
                        DEFAULT_QUALITY,               // Quality
                        (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32, // PitchAndFamily
                        PCSTR(b"Arial\0".as_ptr()),
                    );
                    
                    let old_font = SelectObject(hdc, HGDIOBJ(font.0));
                    
                    // Set text color
                    SetTextColor(hdc, COLORREF(0x00000000)); // Black
                    
                    // Set transparent background
                    SetBkMode(hdc, TRANSPARENT);
                    
                    // Display welcome message
                    let message = b"BestMe - Speech to Text App\0";
                    let mut rect = RECT {
                        left: 20,
                        top: 20,
                        right: WINDOW_WIDTH,
                        bottom: WINDOW_HEIGHT,
                    };
                    
                    // Convert message to a mutable slice
                    let mut message_bytes = message.to_vec();
                    DrawTextA(hdc, &mut message_bytes, &mut rect, DT_LEFT | DT_TOP);
                    
                    // Add status message
                    let status = b"Ready to transcribe. Start speaking...\0";
                    let mut status_rect = RECT {
                        left: 20,
                        top: 70,
                        right: WINDOW_WIDTH,
                        bottom: WINDOW_HEIGHT,
                    };
                    
                    // Convert status to a mutable slice
                    let mut status_bytes = status.to_vec();
                    DrawTextA(hdc, &mut status_bytes, &mut status_rect, DT_LEFT | DT_TOP);
                    
                    // Clean up
                    SelectObject(hdc, old_font);
                    DeleteObject(HGDIOBJ(font.0));
                    
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
