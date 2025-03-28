@echo off
REM Setup script for BestMe on Windows
REM This script installs dependencies and sets up the project

SETLOCAL EnableDelayedExpansion

REM Colors for output
SET "GREEN=[32m"
SET "YELLOW=[33m"
SET "RED=[31m"
SET "BLUE=[34m"
SET "NC=[0m"

echo %BLUE%===================================%NC%
echo %GREEN%BestMe Project Setup - Windows%NC%
echo %BLUE%===================================%NC%

REM Navigate to project root
cd %~dp0\..

REM Check for Node.js
echo Checking for Node.js...
where node >nul 2>nul
IF %ERRORLEVEL% NEQ 0 (
    echo %RED%ERROR: Node.js not found. Please install Node.js from https://nodejs.org/%NC%
    echo After installing Node.js, run this script again.
    exit /b 1
)

FOR /F "tokens=*" %%a in ('node --version') do SET NODE_VERSION=%%a
echo Node.js found: %NODE_VERSION%

REM Check for npm
echo Checking for npm...
where npm >nul 2>nul
IF %ERRORLEVEL% NEQ 0 (
    echo %RED%ERROR: npm not found. Please install Node.js from https://nodejs.org/%NC%
    echo After installing Node.js, run this script again.
    exit /b 1
)

FOR /F "tokens=*" %%a in ('npm --version') do SET NPM_VERSION=%%a
echo npm found: %NPM_VERSION%

REM Check for Rust
echo Checking for Rust...
where rustc >nul 2>nul
IF %ERRORLEVEL% NEQ 0 (
    echo %RED%ERROR: Rust not found. Please install Rust from https://rustup.rs/%NC%
    echo After installing Rust, run this script again.
    exit /b 1
)

FOR /F "tokens=*" %%a in ('rustc --version') do SET RUST_VERSION=%%a
echo Rust found: %RUST_VERSION%

REM Check for Cargo
echo Checking for Cargo...
where cargo >nul 2>nul
IF %ERRORLEVEL% NEQ 0 (
    echo %RED%ERROR: Cargo not found. Please install Rust from https://rustup.rs/%NC%
    echo After installing Rust, run this script again.
    exit /b 1
)

FOR /F "tokens=*" %%a in ('cargo --version') do SET CARGO_VERSION=%%a
echo Cargo found: %CARGO_VERSION%

REM Install Tauri CLI
echo Installing Tauri CLI...
cargo install tauri-cli
IF %ERRORLEVEL% NEQ 0 (
    echo %YELLOW%WARNING: Failed to install Tauri CLI. You may need to install it manually.%NC%
)

REM Install Node.js dependencies
echo Installing Node.js dependencies...
call npm install
IF %ERRORLEVEL% NEQ 0 (
    echo %RED%ERROR: Failed to install Node.js dependencies.%NC%
    exit /b 1
)

echo Dependencies installed successfully.

REM Set up development environment
echo Setting up development environment...

REM Install Rust analyzer
rustup component add rust-analyzer
IF %ERRORLEVEL% NEQ 0 (
    echo %YELLOW%WARNING: Failed to install rust-analyzer. You might want to install it manually.%NC%
)

REM Set up batch scripts as executable
echo Setting up batch scripts...
attrib +x scripts\run_default.bat
attrib +x scripts\run_voice.bat
attrib +x scripts\run_debug.bat
attrib +x scripts\run_dev.bat
attrib +x scripts\test_voice_commands.bat

echo %BLUE%===================================%NC%
echo %GREEN%Setup complete!%NC%
echo %BLUE%===================================%NC%
echo.
echo You can now run BestMe using one of the following commands:
echo.
echo   scripts\run_default.bat - Run with default settings
echo   scripts\run_voice.bat   - Run with voice commands enabled
echo   scripts\run_debug.bat   - Run in debug mode
echo.
echo For more information, see README.md and SCRIPTS.md

ENDLOCAL 
