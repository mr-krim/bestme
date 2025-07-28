# Windows Build Guide

## Prerequisites
1. **LLVM**: `winget install LLVM.LLVM`
2. **CMake**: `winget install Kitware.CMake`
3. **Rust & Node.js**: Already installed

## Build Options

### Option 1: Minimal Build (Quickest)
```cmd
MINIMAL-WINDOWS-BUILD.bat
```

### Option 2: Build without problematic features
Edit `Cargo.toml` line 15 - remove `"text-injection"`:
```toml
default = ["tauri-2", "config", "audio", "transcribe", "storage", "commands", "ai-all"]
```
Then:
```cmd
cargo clean
cargo tauri build
```

### Option 3: Full build (after fixing APIs)
```cmd
cargo tauri build
```

## Known Issues
- Windows API version mismatches with `VIRTUAL_KEY`, `KEYEVENTF`
- Text injection module needs updating for Windows 0.58 crate
- Multiple windows-core versions causing conflicts

## Outputs
- MSI: `src-tauri\target\release\bundle\msi\`
- EXE: `src-tauri\target\release\bestme-tauri.exe`