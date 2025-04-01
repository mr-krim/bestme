#!/bin/bash
# BestMe Windows Packaging Script for WSL
# This script builds a standalone Windows package from WSL environment

set -e

echo "==============================================="
echo "   BestMe Windows Packaging Script (WSL)"
echo "   Tauri 2.0 Migration - Final Binary"
echo "==============================================="
echo ""

# Set variables
OUTPUT_DIR="./target/windows-release"
EXE_NAME="bestme.exe"
SOURCE_EXE="bestme"

# Build release version
echo "Building release version..."
cargo build --release
echo "[SUCCESS] Release build completed!"

# Create output directory
echo "Creating Windows package directory..."
mkdir -p "${OUTPUT_DIR}/assets"

# Copy application files
echo "Copying application files..."
cp "./target/release/${SOURCE_EXE}" "${OUTPUT_DIR}/${EXE_NAME}"
[ -f "./settings.cfg" ] && cp "./settings.cfg" "${OUTPUT_DIR}/settings.cfg"
[ -d "./assets" ] && cp -r "./assets/." "${OUTPUT_DIR}/assets/"
[ -d "./config" ] && cp -r "./config" "${OUTPUT_DIR}/config"
[ -f "./README-WINDOWS.md" ] && cp "./README-WINDOWS.md" "${OUTPUT_DIR}/README.txt"
[ -f "./docs/windows-testing-guide.md" ] && cp "./docs/windows-testing-guide.md" "${OUTPUT_DIR}/TESTING-GUIDE.txt"
[ -f "./LICENSE.txt" ] && cp "./LICENSE.txt" "${OUTPUT_DIR}/LICENSE.txt"
# Copy GUI scripts from scripts folder
[ -f "./scripts/run-gui-mode.bat" ] && cp "./scripts/run-gui-mode.bat" "${OUTPUT_DIR}/run-gui-mode.bat"
[ -f "./scripts/run-gui-mode.ps1" ] && cp "./scripts/run-gui-mode.ps1" "${OUTPUT_DIR}/run-gui-mode.ps1"

# Create version info
echo "Creating version info..."
cat > "${OUTPUT_DIR}/version.txt" << EOF
BestMe Application
Version: 0.1.0
Build Date: $(date '+%Y-%m-%d %H:%M:%S')
Platform: Windows 11
EOF

# Create launch script
echo "Creating launch script..."
cat > "${OUTPUT_DIR}/run-bestme.bat" << EOF
@echo off
echo Starting BestMe Application...
start bestme.exe
EOF

# Create PowerShell launch script
echo "Creating PowerShell launch script..."
cat > "${OUTPUT_DIR}/run-bestme.ps1" << EOF
Write-Host "Starting BestMe Application..." -ForegroundColor Green
Start-Process -FilePath ".\bestme.exe"
EOF

# Create Windows README
echo "Creating Windows README..."
cat > "${OUTPUT_DIR}/README-WINDOWS.txt" << EOF
BestMe - Windows 11 Testing Package
===================================

This is a standalone package for testing BestMe on Windows 11 following the Tauri 2.0 migration.

Quick Start:
-----------
1. Double-click run-bestme.bat or run-bestme.ps1 to start the application
2. Use the console interface to test audio device detection and recording
3. Follow the TESTING-GUIDE.txt for detailed testing procedures

Contents:
--------
- bestme.exe - Main application executable
- run-bestme.bat - Batch script to launch the application
- run-bestme.ps1 - PowerShell script to launch the application
- run-gui-mode.bat - Batch script to launch the GUI mode
- run-gui-mode.ps1 - PowerShell script to launch the GUI mode
- assets/ - Application assets and resources
- config/ - Application configuration files
- settings.cfg - Default configuration
- TESTING-GUIDE.txt - Detailed testing instructions
- LICENSE.txt - License information
- version.txt - Build version information

Notes:
------
- Audio permissions may need to be granted manually
- Windows Defender may ask for confirmation to run the application
- Use Windows 11 language settings to ensure proper language support

Reporting Issues:
----------------
1. Document the exact steps to reproduce any issues
2. Note your Windows version and hardware specifications
3. Include error messages or screenshots
4. Test on different audio devices if possible

EOF

echo ""
echo "==============================================="
echo "Windows package created successfully at:"
echo "${OUTPUT_DIR}"
echo "==============================================="
echo ""
echo "Next steps:"
echo "1. Copy the entire ${OUTPUT_DIR} folder to your Windows 11 machine"
echo "2. Run run-bestme.bat or run-bestme.ps1 to start the application"
echo "3. Follow the testing guide in TESTING-GUIDE.txt"
echo ""

# Create WSL export instructions
cat << EOF
To export the package to your Windows host:

1. From Windows, run:
   explorer.exe \$(wslpath -w $(pwd)/${OUTPUT_DIR})

2. Or copy this folder path to access from Windows:
   $(wslpath -w $(pwd)/${OUTPUT_DIR})
EOF 
