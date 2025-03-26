use anyhow::Result;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION;

/// Load an icon from resources
pub fn load_icon(_resource_id: u16) -> Result<HICON> {
    let instance = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleA(None).unwrap() };
    
    // Convert resource ID to PCWSTR
    let icon_id = windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION; // Using builtin for now
    
    let icon = unsafe {
        windows::Win32::UI::WindowsAndMessaging::LoadIconW(
            instance,
            icon_id,
        )
    };
    
    if let Ok(icon) = icon {
        Ok(icon)
    } else {
        anyhow::bail!("Failed to load icon")
    }
}

/// Create a microphone icon
pub fn create_microphone_icon() -> Result<HICON> {
    // For now, we'll use the application icon
    // In a real implementation, we'd create a custom icon
    unsafe {
        Ok(windows::Win32::UI::WindowsAndMessaging::LoadIconW(
            None,
            IDI_APPLICATION,
        )
        .unwrap())
    }
} 
