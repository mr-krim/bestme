@echo off
REM Script to build and run BestMe in debug mode with verbose logging

REM Enable voice commands (optional, comment out if not needed)
set ENABLE_VOICE_COMMANDS=true

REM Set environment variables for verbose logging
set RUST_LOG=debug,voice_commands=trace,transcribe=trace

REM Build and run in development mode
echo Building and running BestMe in debug mode...
echo Logging level: %RUST_LOG%
if defined ENABLE_VOICE_COMMANDS (
    echo Voice commands: enabled
) else (
    echo Voice commands: disabled
)

REM Check if npm is installed
where npm >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Error: npm is not installed or not in the PATH
    echo Please install Node.js and npm before running this script
    exit /b 1
)

REM Check if cargo is installed
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Error: cargo is not installed or not in the PATH
    echo Please install Rust and Cargo before running this script
    exit /b 1
)

REM Install dependencies if needed
if not exist "node_modules" (
    echo Installing npm dependencies...
    call npm install
)

REM Run with tauri
echo Starting Tauri development server in debug mode...
call npm run tauri dev -- --features verbose

REM If Tauri fails, try to run with cargo directly
if %ERRORLEVEL% neq 0 (
    echo Tauri run failed, trying with cargo directly...
    cd src-tauri
    cargo run --features dev,verbose
) 
