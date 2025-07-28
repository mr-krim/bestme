# Fixing Windows API Errors

## The Problem
The Windows crate API has changed, causing type mismatches:
- `VIRTUAL_KEY` now requires wrapping u16 values
- `KEYEVENTF` is no longer a function
- String pointers need `PWSTR` wrapper
- Multiple versions of windows-core causing conflicts

## Quick Solution 1: Disable Problematic Features

Edit `Cargo.toml` line 15:
```toml
# Remove "text-injection" from default
default = ["tauri-2", "config", "audio", "transcribe", "storage", "commands", "ai-all"]
```

Then build:
```cmd
cargo clean
cargo tauri build
```

## Quick Solution 2: Minimal Build

Run the batch file:
```cmd
MINIMAL-WINDOWS-BUILD.bat
```

This builds just the GUI without text injection or transcription.

## Full Solution: Fix the API Issues

To fix all issues properly, we need to:

1. Update Windows API calls to use new types
2. Fix the windows-core version conflicts
3. Update the type conversions

For now, use the minimal build to test the GUI, then we can fix the API issues properly.