# BestMe Windows Build Script
# This script builds the BestMe application for Windows 11 testing

# Set error action preference to stop on any error
$ErrorActionPreference = "Stop"

# Display banner
Write-Host "==============================================="
Write-Host "   BestMe Windows 11 Build Script"
Write-Host "   Tauri 2.0 Migration Testing"
Write-Host "==============================================="
Write-Host ""

# Check Rust installation
Write-Host "Checking Rust installation..."
try {
    $rustVersion = rustc --version
    Write-Host "✓ Rust is installed: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust is not installed or not in PATH. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check cargo installation
Write-Host "Checking cargo installation..."
try {
    $cargoVersion = cargo --version
    Write-Host "✓ Cargo is installed: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Cargo is not installed or not in PATH." -ForegroundColor Red
    exit 1
}

# Check for WebView2
Write-Host "Checking WebView2 installation..."
$webview2Path = Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue
if ($webview2Path) {
    Write-Host "✓ WebView2 Runtime is installed" -ForegroundColor Green
} else {
    Write-Host "! WebView2 Runtime may not be installed. Installing WebView2 may be required." -ForegroundColor Yellow
}

# Clean previous builds
Write-Host "Cleaning previous builds..."
cargo clean
Write-Host "✓ Previous builds cleaned" -ForegroundColor Green

# Build the application in debug mode first
Write-Host "Building in debug mode..."
cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Debug build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Debug build completed successfully" -ForegroundColor Green

# Run tests
Write-Host "Running tests..."
cargo test
if ($LASTEXITCODE -ne 0) {
    Write-Host "! Some tests failed. Review the test output above." -ForegroundColor Yellow
} else {
    Write-Host "✓ All tests passed" -ForegroundColor Green
}

# Build release version
Write-Host "Building release version..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Release build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Release build completed successfully" -ForegroundColor Green

# Create output directory if it doesn't exist
$outputDir = ".\target\windows-package"
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

# Copy necessary files to output directory
Write-Host "Packaging application..."
Copy-Item -Path ".\target\release\bestme.exe" -Destination "$outputDir\bestme.exe"
Copy-Item -Path ".\assets" -Destination "$outputDir\assets" -Recurse -Force
Copy-Item -Path ".\settings.cfg" -Destination "$outputDir\settings.cfg" -Force
Copy-Item -Path ".\README.md" -Destination "$outputDir\README.md" -Force
Copy-Item -Path ".\docs\windows-testing-guide.md" -Destination "$outputDir\TESTING-GUIDE.md" -Force

# Create version info file
$versionInfo = @"
BestMe Application
Version: 0.1.0
Build Date: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
Platform: Windows 11
Tauri Version: 2.0.0
"@
$versionInfo | Out-File -FilePath "$outputDir\version.txt"

Write-Host "✓ Application packaged successfully to $outputDir" -ForegroundColor Green

# Display next steps
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Cyan
Write-Host "1. Navigate to the package directory: cd $outputDir"
Write-Host "2. Run the application: .\bestme.exe"
Write-Host "3. Follow the testing guide in TESTING-GUIDE.md"
Write-Host ""
Write-Host "For more detailed instructions, refer to docs/windows-testing-guide.md"
Write-Host ""
Write-Host "==============================================="
Write-Host "   Build and packaging complete"
Write-Host "===============================================" 
