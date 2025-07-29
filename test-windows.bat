@echo off
echo Testing BestMe on Windows...
echo.

REM Set environment variables for debug logging
set RUST_LOG=debug,voice_commands=trace,transcribe=trace
set ENABLE_VOICE_COMMANDS=true

echo Logging level: %RUST_LOG%
echo Voice commands: %ENABLE_VOICE_COMMANDS%
echo.

REM Navigate to project directory
cd /d "%~dp0"

REM Check if the built executable exists
if exist "target\release\bestme-tauri.exe" (
    echo Running release build...
    target\release\bestme-tauri.exe
) else if exist "target\debug\bestme-tauri.exe" (
    echo Running debug build...
    target\debug\bestme-tauri.exe
) else (
    echo No build found. Building application...
    cargo build --release
    if %errorlevel% neq 0 (
        echo Build failed!
        pause
        exit /b 1
    )
    echo Build successful. Running application...
    target\release\bestme-tauri.exe
)

pause