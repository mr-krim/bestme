@echo off
REM BestMe GUI Mode Launch Script
REM This script ensures the application runs in GUI mode and stays visible

echo ===============================================
echo    BestMe Application - GUI Mode
echo    Tauri 2.0 Windows Testing
echo ===============================================
echo.

REM Set environment variables for debugging if needed
set RUST_LOG=info
set RUST_BACKTRACE=1

echo Starting BestMe in GUI mode...

REM Launch application with --gui flag
REM The START command ensures it runs in a new window
REM The /WAIT parameter makes the batch file wait until the application exits
START /WAIT "" "bestme.exe" --gui

REM If application crashes immediately, this will keep the window open
if %ERRORLEVEL% NEQ 0 (
  echo.
  echo Application exited with error code: %ERRORLEVEL%
  echo.
  echo Please check the logs for more information.
  echo Logs may be found in: %APPDATA%\bestme\logs
  echo.
  echo Press any key to close this window...
  pause > nul
) 
