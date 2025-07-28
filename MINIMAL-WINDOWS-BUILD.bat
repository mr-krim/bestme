@echo off
echo Building BestMe with minimal features for Windows...
echo.

:: Clean previous builds
cargo clean

:: Build with only essential features (no text injection, no transcription)
echo Building with core features only...
cd src-tauri
cargo build --release --no-default-features --features "custom-protocol"
cd ..

:: Check if build succeeded
if exist "target\release\bestme-tauri.exe" (
    echo.
    echo SUCCESS! Executable built at: target\release\bestme-tauri.exe
    echo.
    echo You can now run the app to test the GUI!
    echo Run: target\release\bestme-tauri.exe
) else (
    echo.
    echo Build failed. Trying even more minimal build...
    cargo build --release
)

pause