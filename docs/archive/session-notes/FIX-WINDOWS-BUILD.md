# Fix Windows Build - libclang Error

## The Issue
Whisper-rs needs Clang/LLVM to build its bindings on Windows.

## Solution 1: Install LLVM (Recommended)

### In PowerShell (as Administrator):
```powershell
# Install LLVM using winget
winget install LLVM.LLVM

# Or download manually from:
# https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/LLVM-17.0.6-win64.exe
```

### After installation:
1. **Restart PowerShell**
2. **Set environment variable**:
```powershell
# Add to current session
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"

# Or add permanently (requires admin):
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")
```

3. **Try building again**:
```powershell
cargo tauri build
```

## Solution 2: Quick Fix - Disable Whisper (Temporary)

If you just want to test the GUI without audio transcription:

### Edit `Cargo.toml`:
```toml
# Change this line:
default = ["tauri-2", "config", "audio", "transcribe", "storage", "commands", "text-injection", "ai-all"]

# To this (remove "transcribe"):
default = ["tauri-2", "config", "audio", "storage", "commands", "text-injection", "ai-all"]
```

### Edit `src-tauri/Cargo.toml`:
```toml
# Comment out or change:
default = ["custom-protocol", "audio-plugin", "transcribe-plugin", "voice-command-plugin"]

# To:
default = ["custom-protocol", "audio-plugin", "voice-command-plugin"]
```

Then build:
```powershell
cargo tauri build
```

## Solution 3: Use Pre-built Whisper (Alternative)

### In `Cargo.toml`, change:
```toml
whisper-rs = { version = "0.11.1", optional = true }
```

### To:
```toml
whisper-rs = { version = "0.11.1", optional = true, features = ["download-binaries"] }
```

## Solution 4: Install Visual Studio Build Tools (If not already)

Sometimes the C++ toolchain is incomplete:

```powershell
# Install C++ build tools
winget install Microsoft.VisualStudio.2022.BuildTools

# During installation, select:
# - Desktop development with C++
# - Windows 10/11 SDK
# - MSVC v143
```

## Quick Test Without Transcription

To quickly test if everything else works:

1. **Disable transcription features temporarily**
2. **Build and test GUI, settings, etc.**
3. **Fix transcription after confirming base app works**

## After Fixing

Once LLVM is installed and PATH is set:
```powershell
# Clean build
cargo clean

# Build again
cargo tauri build
```

The build should complete successfully and create:
- `src-tauri\target\release\bundle\msi\BestMe_0.1.0_x64_en-US.msi`
- `src-tauri\target\release\bestme-tauri.exe`

---

**Note**: LLVM is about 500MB download. If you're in a hurry, use Solution 2 to test the GUI first!