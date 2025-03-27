@echo off
REM Setup script for BestMe on Windows
REM This script installs dependencies and sets up the project

echo ===================================
echo BestMe Project Setup - Windows
echo ===================================

REM Check for Node.js
echo Checking for Node.js...
where node >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: Node.js not found. Please install Node.js from https://nodejs.org/
    echo After installing Node.js, run this script again.
    pause
    exit /b 1
)

echo Node.js found: 
node --version

REM Check for npm
echo Checking for npm...
where npm >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: npm not found. Please install Node.js from https://nodejs.org/
    echo After installing Node.js, run this script again.
    pause
    exit /b 1
)

echo npm found:
npm --version

REM Check for Rust
echo Checking for Rust...
where rustc >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: Rust not found. Please install Rust from https://rustup.rs/
    echo After installing Rust, run this script again.
    pause
    exit /b 1
)

echo Rust found:
rustc --version

REM Check for Cargo
echo Checking for Cargo...
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: Cargo not found. Please install Rust from https://rustup.rs/
    echo After installing Rust, run this script again.
    pause
    exit /b 1
)

echo Cargo found:
cargo --version

REM Install Tauri CLI
echo Installing Tauri CLI...
cargo install tauri-cli
if %ERRORLEVEL% neq 0 (
    echo WARNING: Failed to install Tauri CLI. You may need to install it manually.
) else (
    echo Tauri CLI installed successfully.
)

REM Install Node.js dependencies
echo Installing Node.js dependencies...
npm install
if %ERRORLEVEL% neq 0 (
    echo ERROR: Failed to install Node.js dependencies.
    pause
    exit /b 1
)

echo Dependencies installed successfully.

REM Set up development environment
echo Setting up development environment...

REM Check for Rust analyzer
rustup component add rust-analyzer
if %ERRORLEVEL% neq 0 (
    echo WARNING: Failed to install rust-analyzer. You might want to install it manually.
) else (
    echo rust-analyzer installed successfully.
)

REM Set up batch scripts as executable
echo Setting up batch scripts...
attrib +x run_default.bat
attrib +x run_voice.bat
attrib +x run_debug.bat
attrib +x run_dev.bat
attrib +x test_voice_commands.bat

echo ===================================
echo Setup complete!
echo ===================================
echo.
echo You can now run BestMe using one of the following commands:
echo.
echo   run_default.bat - Run with default settings
echo   run_voice.bat   - Run with voice commands enabled
echo   run_debug.bat   - Run in debug mode
echo.
echo For more information, see README.md and SCRIPTS.md
echo.
pause 
