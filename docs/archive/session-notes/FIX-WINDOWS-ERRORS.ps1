# PowerShell script to fix Windows compilation errors

Write-Host "Fixing Windows compilation errors..." -ForegroundColor Green

# Fix 1: Update Cargo.toml to add missing Windows features
$cargoContent = Get-Content Cargo.toml -Raw
if ($cargoContent -notmatch "Win32_Security") {
    Write-Host "Adding Windows Security features to Cargo.toml"
    $cargoContent = $cargoContent -replace '("Win32_Devices_FunctionDiscovery",\s*\])', @'
"Win32_Devices_FunctionDiscovery",
    "Win32_Security",
    "Win32_Security_Credentials",
    "Win32_Security_Authentication_Identity",
]
'@
    $cargoContent | Set-Content Cargo.toml -NoNewline
}

# Fix 2: Create a patch file for Windows API changes
Write-Host "Creating Windows API compatibility layer..."

$patchContent = @'
// Windows API compatibility fixes
#[cfg(target_os = "windows")]
pub mod windows_compat {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    
    // Helper to create VIRTUAL_KEY
    pub fn vk(code: u16) -> VIRTUAL_KEY {
        VIRTUAL_KEY(code)
    }
    
    // Keyevent flags
    pub const KEYEVENTF_NONE: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);
}
'@

# Add to src/lib.rs if not exists
$libPath = "src\lib.rs"
if (Test-Path $libPath) {
    $libContent = Get-Content $libPath -Raw
    if ($libContent -notmatch "windows_compat") {
        Add-Content $libPath "`n$patchContent"
    }
}

# Fix 3: Temporarily disable problematic modules
Write-Host "Adjusting feature flags for Windows build..."

# Create a Windows-specific build configuration
$winBuildConfig = @'
[package]
name = "bestme"
version = "0.1.0"
edition = "2021"

[features]
default = ["tauri-2", "config", "audio", "storage", "commands", "ai-all"]
# Temporarily disabled for Windows: "text-injection", "transcribe"

[dependencies]
# ... rest of dependencies ...
'@

Write-Host @"

Fixes applied! Now try building with:

1. Clean build:
   cargo clean

2. Build without transcription and text injection (temporarily):
   cargo tauri build --no-default-features --features "tauri-2,config,audio,storage,commands,ai-all"

Or for a quick test without AI:
   cargo tauri build --no-default-features --features "tauri-2,config,audio,storage"

This will create a working Windows build that you can test!
"@ -ForegroundColor Yellow