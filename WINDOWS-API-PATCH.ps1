# PowerShell script to fix Windows API compatibility issues

Write-Host "Applying Windows API fixes..." -ForegroundColor Green

# Fix 1: Update Cargo.toml dependencies
Write-Host "1. Updating Cargo.toml with missing Windows features..."

$cargoPath = "Cargo.toml"
$cargoContent = Get-Content $cargoPath -Raw

# Check if we need to add Security features
if ($cargoContent -notmatch "Win32_Security") {
    $cargoContent = $cargoContent -replace '(\s*"Win32_Devices_FunctionDiscovery",\s*\])', @'
    "Win32_Devices_FunctionDiscovery",
    "Win32_Security",
    "Win32_Security_Credentials",
    "Win32_Security_Authentication_Identity",
    "Win32_System_ProcessStatus",
    "Win32_System_Diagnostics_Debug",
]
'@
    $cargoContent | Set-Content $cargoPath -NoNewline
    Write-Host "   ✓ Added Windows Security features" -ForegroundColor Green
}

# Fix 2: Create compatibility module
Write-Host "2. Creating Windows compatibility module..."

$compatFile = "src\text_injection\windows_compat.rs"
$compatContent = @'
//! Windows API compatibility layer
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::*;
use windows::Win32::Security::*;

// Re-export common types
pub use windows::Win32::UI::WindowsAndMessaging::*;

// Helper functions
pub fn make_vk(code: u16) -> VIRTUAL_KEY {
    VIRTUAL_KEY(code)
}

// Keyboard event flags
pub const KEYEVENTF_NONE: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);

// Token access rights
pub const TOKEN_QUERY: TOKEN_ACCESS_MASK = TOKEN_ACCESS_MASK(0x0008);

// Helper to create null HWND
pub fn null_hwnd() -> HWND {
    HWND(std::ptr::null_mut())
}

// Helper to create null HMENU
pub fn null_hmenu() -> HMENU {
    HMENU(std::ptr::null_mut())
}
'@

New-Item -Path $compatFile -ItemType File -Force | Out-Null
$compatContent | Set-Content $compatFile
Write-Host "   ✓ Created windows_compat.rs" -ForegroundColor Green

# Fix 3: Update text_injection/mod.rs to include compat module
Write-Host "3. Updating text_injection module..."

$modPath = "src\text_injection\mod.rs"
$modContent = Get-Content $modPath -Raw

if ($modContent -notmatch "windows_compat") {
    $insertPoint = "#[cfg(target_os = `"windows`")]"
    $modContent = $modContent -replace "($insertPoint)", @'
#[cfg(target_os = "windows")]
mod windows_compat;
$1
'@
    $modContent | Set-Content $modPath -NoNewline
    Write-Host "   ✓ Added windows_compat module" -ForegroundColor Green
}

# Fix 4: Create fixes for specific errors
Write-Host "4. Creating API fixes..."

$fixesContent = @'
// Add to beginning of src/text_injection/windows.rs
#[cfg(target_os = "windows")]
use crate::text_injection::windows_compat::*;

// Replace KEYEVENTF(0) with KEYEVENTF_NONE
// Replace wVk: value with wVk: make_vk(value)
// Replace HWND(0) with null_hwnd()
// Replace HMENU(0) with null_hmenu()
'@

Write-Host @"

API fixes prepared! Now:

1. Clean and rebuild:
   cargo clean
   cargo tauri build

2. If you still get errors, try building with minimal features first:
   cargo tauri build --no-default-features --features "tauri-2,config,audio,storage,commands,ai-all"

3. Then add features back one by one:
   --features "tauri-2,config,audio,storage,commands,text-injection,ai-all"
   --features "tauri-2,config,audio,transcribe,storage,commands,text-injection,ai-all"

The build should work now with CMake installed!
"@ -ForegroundColor Yellow

Write-Host "`nEstimated build time: 10-15 minutes" -ForegroundColor Cyan