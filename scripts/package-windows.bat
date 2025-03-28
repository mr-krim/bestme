@echo off
REM BestMe Windows Packaging Script
REM This script builds a standalone Windows package

echo ===============================================
echo    BestMe Windows Packaging Script
echo    Tauri 2.0 Migration - Final Binary
echo ===============================================
echo.

REM Set variables
set OUTPUT_DIR=.\target\windows-release
set EXE_NAME=bestme.exe

echo Building release version...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
  echo [ERROR] Release build failed!
  exit /b 1
)
echo [SUCCESS] Release build completed!

echo.
echo Creating Windows package directory...
if not exist %OUTPUT_DIR% mkdir %OUTPUT_DIR%
if not exist %OUTPUT_DIR%\assets mkdir %OUTPUT_DIR%\assets

echo.
echo Copying application files...
copy .\target\release\%EXE_NAME% %OUTPUT_DIR%\%EXE_NAME%
if %ERRORLEVEL% NEQ 0 (
  echo [ERROR] Failed to copy executable!
  exit /b 1
)

copy .\settings.cfg %OUTPUT_DIR%\settings.cfg
xcopy /E /I /Y .\assets %OUTPUT_DIR%\assets
copy .\README-WINDOWS.md %OUTPUT_DIR%\README.txt
copy .\docs\windows-testing-guide.md %OUTPUT_DIR%\TESTING-GUIDE.txt
copy .\LICENSE.txt %OUTPUT_DIR%\LICENSE.txt

echo.
echo Creating version info...
echo BestMe Application > %OUTPUT_DIR%\version.txt
echo Version: 0.1.0 >> %OUTPUT_DIR%\version.txt
echo Build Date: %DATE% %TIME% >> %OUTPUT_DIR%\version.txt
echo Platform: Windows 11 >> %OUTPUT_DIR%\version.txt

echo.
echo Creating launch script...
echo @echo off > %OUTPUT_DIR%\run-bestme.bat
echo echo Starting BestMe Application... >> %OUTPUT_DIR%\run-bestme.bat
echo start bestme.exe >> %OUTPUT_DIR%\run-bestme.bat

echo.
echo ===============================================
echo Windows package created successfully at:
echo %OUTPUT_DIR%
echo ===============================================
echo.
echo Next steps:
echo 1. Copy the entire %OUTPUT_DIR% folder to your Windows 11 machine
echo 2. Run run-bestme.bat to start the application
echo 3. Follow the testing guide in TESTING-GUIDE.txt
echo. 
