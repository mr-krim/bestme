@echo off
REM Script to test voice commands in isolation

REM Enable voice commands
set ENABLE_VOICE_COMMANDS=true

REM Set verbose logging for voice commands
set RUST_LOG=info,voice_commands=debug

REM Colors for Windows console
set GREEN=[92m
set YELLOW=[93m
set BLUE=[94m
set NC=[0m

echo %BLUE%====================================%NC%
echo %GREEN%BestMe Voice Command Testing Utility%NC%
echo %BLUE%====================================%NC%

REM Check if cargo is installed
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo %YELLOW%Error: cargo is not installed or not in the PATH%NC%
    echo Please install Rust and Cargo before running this script
    exit /b 1
)

:menu
echo.
echo %GREEN%Choose a test option:%NC%
echo 1) Run voice command module test
echo 2) Run simulated test with sample phrases
echo 3) Exit
echo.

set /p choice=Enter your choice (1-3): 

if "%choice%"=="1" goto test_voice_commands
if "%choice%"=="2" goto simulate_test
if "%choice%"=="3" goto exit_script
echo %YELLOW%Invalid choice. Please try again.%NC%
goto menu

:test_voice_commands
echo.
echo %BLUE%Starting voice command module test...%NC%
echo %YELLOW%This will run just the voice command module with test input%NC%
echo Press Ctrl+C to exit the test
echo.

REM Navigate to the src-tauri directory
cd src-tauri

REM Run the test binary
echo %GREEN%Running voice command test...%NC%
cargo test --package bestme --lib plugin::voice_commands::tests::test_voice_command_detection -- --nocapture

REM Return to the original directory
cd ..

echo %GREEN%Test completed!%NC%
goto menu

:simulate_test
echo.
echo %BLUE%Simulating voice commands with test phrases...%NC%

REM Sample test phrases
echo %YELLOW%Sample test phrases:%NC%
echo   - Hey BestMe, what time is it
echo   - BestMe, start recording
echo   - BestMe, stop recording
echo   - BestMe, save transcript
echo   - BestMe, clear transcript
echo   - Random text that is not a command
echo   - Hey BestMe, switch to dark mode

echo.
echo %GREEN%Starting simulation...%NC%

REM Process each phrase
call :process_phrase "Hey BestMe, what time is it"
call :process_phrase "BestMe, start recording"
call :process_phrase "BestMe, stop recording"
call :process_phrase "BestMe, save transcript"
call :process_phrase "BestMe, clear transcript"
call :process_phrase "Random text that is not a command"
call :process_phrase "Hey BestMe, switch to dark mode"

echo.
echo %GREEN%Test completed!%NC%
goto menu

:process_phrase
echo.
echo %BLUE%Testing phrase:%NC% %~1
echo %~1 | findstr /i "BestMe" >nul
if %ERRORLEVEL% equ 0 (
    echo %~1 | findstr /i "start recording" >nul
    if %ERRORLEVEL% equ 0 (
        echo %GREEN%✓ Command detected:%NC% START_RECORDING
        goto :eof
    )
    
    echo %~1 | findstr /i "stop recording" >nul
    if %ERRORLEVEL% equ 0 (
        echo %GREEN%✓ Command detected:%NC% STOP_RECORDING
        goto :eof
    )
    
    echo %~1 | findstr /i "save transcript" >nul
    if %ERRORLEVEL% equ 0 (
        echo %GREEN%✓ Command detected:%NC% SAVE_TRANSCRIPT
        goto :eof
    )
    
    echo %~1 | findstr /i "clear transcript" >nul
    if %ERRORLEVEL% equ 0 (
        echo %GREEN%✓ Command detected:%NC% CLEAR_TRANSCRIPT
        goto :eof
    )
    
    echo %~1 | findstr /i "dark mode" >nul
    if %ERRORLEVEL% equ 0 (
        echo %GREEN%✓ Command detected:%NC% TOGGLE_THEME
        goto :eof
    )
    
    echo %YELLOW%? Potential command but action unknown%NC%
) else (
    echo %YELLOW%× No command detected%NC%
)
timeout /t 1 /nobreak >nul
goto :eof

:exit_script
echo %BLUE%Exiting...%NC%
exit /b 0 
