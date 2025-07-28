# Installing LLVM on Windows for BestMe Build

## Step 1: Install LLVM

### Option A: Using winget (Recommended)
In PowerShell as Administrator:
```powershell
winget install LLVM.LLVM
```

### Option B: Manual Download
If winget fails, download directly:
1. Go to: https://github.com/llvm/llvm-project/releases/latest
2. Download: `LLVM-17.0.6-win64.exe` (or latest version)
3. Run installer
4. **IMPORTANT**: During installation, check "Add LLVM to system PATH"

## Step 2: Verify Installation

Close and reopen PowerShell, then verify:
```powershell
# Check if clang is accessible
clang --version

# Should output something like:
# clang version 17.0.6
# Target: x86_64-pc-windows-msvc
```

## Step 3: Set Environment Variable

### Temporary (for current session):
```powershell
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```

### Permanent (recommended):
```powershell
# As Administrator
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "Machine")

# Or for current user only:
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")
```

## Step 4: Clean and Rebuild

```powershell
# Navigate to project
cd D:\bestme

# Clean previous build attempts
cargo clean

# Rebuild with full functionality
cargo tauri build
```

## Troubleshooting

### If LLVM installed to different location:
```powershell
# Find where LLVM is installed
Get-ChildItem -Path "C:\Program Files" -Filter "LLVM" -Directory

# Update the path accordingly
$env:LIBCLANG_PATH = "C:\Your\Actual\LLVM\Path\bin"
```

### If build still fails:
1. **Check path**:
   ```powershell
   Test-Path "$env:LIBCLANG_PATH\libclang.dll"
   ```

2. **Try explicit path**:
   ```powershell
   $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
   $env:PATH = "$env:LIBCLANG_PATH;$env:PATH"
   ```

3. **Restart PowerShell** after setting variables

## Expected Build Time

- LLVM Download: ~500MB (5-10 minutes depending on internet)
- LLVM Installation: 2-3 minutes
- Cargo build: 10-15 minutes (first time)
- Subsequent builds: 2-3 minutes

## Success Indicators

When successful, you'll see:
```
Finished 2 bundles at:
  - src-tauri\target\release\bundle\msi\BestMe_0.1.0_x64_en-US.msi
  - src-tauri\target\release\bundle\nsis\BestMe_0.1.0_x64-setup.exe
```

## After Successful Build

1. **Install BestMe**:
   - Run the MSI installer
   - Or use the portable EXE

2. **Test Features**:
   - ✓ Application launches
   - ✓ Audio recording works
   - ✓ Speech-to-text transcription
   - ✓ AI text enhancement
   - ✓ Settings persistence

Good luck! The build should work perfectly once LLVM is installed.