# Quick build script to test GUI without Whisper
# Run this in PowerShell from D:\bestme directory

# Backup original files
Copy-Item Cargo.toml Cargo.toml.backup
Copy-Item src-tauri\Cargo.toml src-tauri\Cargo.toml.backup

# Modify root Cargo.toml to remove transcribe
$content = Get-Content Cargo.toml
$content = $content -replace 'default = \["tauri-2", "config", "audio", "transcribe", "storage", "commands", "text-injection", "ai-all"\]', 'default = ["tauri-2", "config", "audio", "storage", "commands", "text-injection", "ai-all"]'
$content | Set-Content Cargo.toml

# Modify src-tauri/Cargo.toml to remove transcribe-plugin
$content = Get-Content src-tauri\Cargo.toml
$content = $content -replace 'default = \["custom-protocol", "audio-plugin", "transcribe-plugin", "voice-command-plugin"\]', 'default = ["custom-protocol", "audio-plugin", "voice-command-plugin"]'
$content | Set-Content src-tauri\Cargo.toml

# Clean and build
cargo clean
cargo tauri build

# Restore original files after build
Copy-Item Cargo.toml.backup Cargo.toml -Force
Copy-Item src-tauri\Cargo.toml.backup src-tauri\Cargo.toml -Force