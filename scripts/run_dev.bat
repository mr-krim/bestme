@echo off
REM Script to build and run BestMe with debug logging enabled

REM Set environment variables for logging
set RUST_LOG=debug

REM Build and run in development mode
echo Building and running BestMe with debug logging...
echo Logging level: %RUST_LOG%

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
echo Starting Tauri development server...
call npm run tauri dev

REM If Tauri fails, try to run with cargo directly
if %ERRORLEVEL% neq 0 (
    echo Tauri run failed, trying with cargo directly...
    cd src-tauri
    cargo run --features dev
) 
