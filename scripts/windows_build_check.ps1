# Windows Build Pre-flight Check for BestMe
# Run this before building to ensure everything is set up correctly

Write-Host "BestMe Windows Build Pre-flight Check" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan

$hasErrors = $false

# Check Windows version
Write-Host "`nChecking Windows version..." -ForegroundColor Yellow
$os = Get-CimInstance Win32_OperatingSystem
if ($os.Version -lt "10.0") {
    Write-Host "ERROR: Windows 10 or later is required" -ForegroundColor Red
    $hasErrors = $true
} else {
    Write-Host "OK: Windows $($os.Caption) detected" -ForegroundColor Green
}

# Check for Visual Studio Build Tools
Write-Host "`nChecking for Visual Studio Build Tools..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsInstalls = & $vsWhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json | ConvertFrom-Json
    if ($vsInstalls.Count -gt 0) {
        Write-Host "OK: Visual Studio Build Tools found" -ForegroundColor Green
    } else {
        Write-Host "ERROR: Visual Studio C++ Build Tools not found" -ForegroundColor Red
        Write-Host "Install from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Yellow
        $hasErrors = $true
    }
} else {
    Write-Host "ERROR: Visual Studio not found" -ForegroundColor Red
    $hasErrors = $true
}

# Check for Rust
Write-Host "`nChecking for Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version
    Write-Host "OK: $rustVersion" -ForegroundColor Green
    
    # Check for Windows target
    $targets = rustup target list --installed
    if ($targets -contains "x86_64-pc-windows-msvc") {
        Write-Host "OK: Windows MSVC target installed" -ForegroundColor Green
    } else {
        Write-Host "WARNING: Installing Windows MSVC target..." -ForegroundColor Yellow
        rustup target add x86_64-pc-windows-msvc
    }
} catch {
    Write-Host "ERROR: Rust not found. Install from https://rustup.rs" -ForegroundColor Red
    $hasErrors = $true
}

# Check for Node.js
Write-Host "`nChecking for Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version
    $npmVersion = npm --version
    Write-Host "OK: Node.js $nodeVersion, npm $npmVersion" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Node.js not found. Install from https://nodejs.org" -ForegroundColor Red
    $hasErrors = $true
}

# Check for WebView2
Write-Host "`nChecking for WebView2 Runtime..." -ForegroundColor Yellow
$webView2 = Get-ItemProperty -Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue
if ($webView2) {
    Write-Host "OK: WebView2 Runtime installed" -ForegroundColor Green
} else {
    Write-Host "WARNING: WebView2 Runtime not found (will be installed by installer)" -ForegroundColor Yellow
}

# Check Windows SDK
Write-Host "`nChecking for Windows SDK..." -ForegroundColor Yellow
$sdkPath = "${env:ProgramFiles(x86)}\Windows Kits\10\Include"
if (Test-Path $sdkPath) {
    $sdkVersions = Get-ChildItem $sdkPath -Directory | Sort-Object Name -Descending
    if ($sdkVersions.Count -gt 0) {
        Write-Host "OK: Windows SDK found - $($sdkVersions[0].Name)" -ForegroundColor Green
    } else {
        Write-Host "ERROR: Windows SDK not found" -ForegroundColor Red
        $hasErrors = $true
    }
} else {
    Write-Host "ERROR: Windows SDK not found" -ForegroundColor Red
    $hasErrors = $true
}

# Check for potential antivirus issues
Write-Host "`nChecking Windows Defender exclusions..." -ForegroundColor Yellow
$projectPath = (Get-Location).Path
$exclusions = Get-MpPreference | Select-Object -ExpandProperty ExclusionPath
if ($exclusions -contains $projectPath) {
    Write-Host "OK: Project path is excluded from Windows Defender" -ForegroundColor Green
} else {
    Write-Host "WARNING: Consider adding project path to Windows Defender exclusions for faster builds" -ForegroundColor Yellow
    Write-Host "Run as Admin: Add-MpPreference -ExclusionPath '$projectPath'" -ForegroundColor Gray
}

# Check for required Windows features
Write-Host "`nChecking Windows features..." -ForegroundColor Yellow
$dotNetFeature = Get-WindowsOptionalFeature -Online -FeatureName "NetFx3" -ErrorAction SilentlyContinue
if ($dotNetFeature.State -eq "Enabled") {
    Write-Host "OK: .NET Framework 3.5 enabled" -ForegroundColor Green
} else {
    Write-Host "WARNING: .NET Framework 3.5 not enabled (may be needed for some tools)" -ForegroundColor Yellow
}

# Summary
Write-Host "`n=====================================" -ForegroundColor Cyan
if ($hasErrors) {
    Write-Host "RESULT: Build environment has errors!" -ForegroundColor Red
    Write-Host "Please fix the errors above before building." -ForegroundColor Red
    exit 1
} else {
    Write-Host "RESULT: Build environment is ready!" -ForegroundColor Green
    Write-Host "`nNext steps:" -ForegroundColor Cyan
    Write-Host "1. cd ui" -ForegroundColor White
    Write-Host "2. npm install" -ForegroundColor White
    Write-Host "3. npm run tauri build" -ForegroundColor White
}