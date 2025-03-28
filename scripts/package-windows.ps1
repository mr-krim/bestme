# BestMe Windows Packaging Script
# This script builds a standalone Windows package for testing

# Set error action preference
$ErrorActionPreference = "Stop"

# Display banner
Write-Host "==============================================="
Write-Host "   BestMe Windows Packaging Script"
Write-Host "   Tauri 2.0 Migration - Final Binary"
Write-Host "==============================================="
Write-Host ""

# Set variables
$outputDir = ".\target\windows-release"
$exeName = "bestme.exe"

# Build release version
Write-Host "Building release version..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Release build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "[SUCCESS] Release build completed!" -ForegroundColor Green

# Create output directory
Write-Host "Creating Windows package directory..." -ForegroundColor Cyan
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}
if (-not (Test-Path "$outputDir\assets")) {
    New-Item -ItemType Directory -Path "$outputDir\assets" | Out-Null
}

# Copy application files
Write-Host "Copying application files..." -ForegroundColor Cyan
Copy-Item ".\target\release\$exeName" -Destination "$outputDir\$exeName"
Copy-Item ".\settings.cfg" -Destination "$outputDir\settings.cfg" -ErrorAction SilentlyContinue
Copy-Item ".\assets" -Destination "$outputDir\" -Recurse -Force -ErrorAction SilentlyContinue
Copy-Item ".\README-WINDOWS.md" -Destination "$outputDir\README.txt" -ErrorAction SilentlyContinue
Copy-Item ".\docs\windows-testing-guide.md" -Destination "$outputDir\TESTING-GUIDE.txt" -ErrorAction SilentlyContinue
Copy-Item ".\LICENSE.txt" -Destination "$outputDir\LICENSE.txt" -ErrorAction SilentlyContinue

# Create required DLL directory
Write-Host "Creating dependency directories..." -ForegroundColor Cyan
if (-not (Test-Path "$outputDir\bin")) {
    New-Item -ItemType Directory -Path "$outputDir\bin" | Out-Null
}

# Create version info
Write-Host "Creating version info..." -ForegroundColor Cyan
$versionInfo = @"
BestMe Application
Version: 0.1.0
Build Date: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
Platform: Windows 11
"@
$versionInfo | Out-File -FilePath "$outputDir\version.txt"

# Create launch script
Write-Host "Creating launch script..." -ForegroundColor Cyan
$launchScript = @"
@echo off
echo Starting BestMe Application...
start bestme.exe
"@
$launchScript | Out-File -FilePath "$outputDir\run-bestme.bat"

# Create PowerShell launch script
$psLaunchScript = @"
Write-Host "Starting BestMe Application..." -ForegroundColor Green
Start-Process -FilePath ".\bestme.exe"
"@
$psLaunchScript | Out-File -FilePath "$outputDir\run-bestme.ps1"

# Final message
Write-Host ""
Write-Host "===============================================" -ForegroundColor Green
Write-Host "Windows package created successfully at:" -ForegroundColor Green
Write-Host "$outputDir" -ForegroundColor Yellow
Write-Host "===============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Copy the entire $outputDir folder to your Windows 11 machine"
Write-Host "2. Run run-bestme.bat or run-bestme.ps1 to start the application"
Write-Host "3. Follow the testing guide in TESTING-GUIDE.txt" 
