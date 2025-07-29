# Fix common Windows build issues for BestMe

Write-Host "Fixing common Windows build issues..." -ForegroundColor Cyan

# 1. Fix Windows crate version mismatch
Write-Host "`nFixing Windows crate versions..." -ForegroundColor Yellow
$cargoToml = Get-Content "Cargo.toml" -Raw

# Update windows crate to match the locked version
$cargoToml = $cargoToml -replace 'windows = \{ version = "0\.58\.0"', 'windows = { version = "0.61"'
Set-Content "Cargo.toml" -Value $cargoToml -NoNewline

# 2. Clear Cargo cache for Windows dependencies
Write-Host "`nClearing Cargo cache..." -ForegroundColor Yellow
if (Test-Path "$env:USERPROFILE\.cargo\registry\cache") {
    Remove-Item "$env:USERPROFILE\.cargo\registry\cache\*windows*" -Force -Recurse -ErrorAction SilentlyContinue
}

# 3. Set required environment variables
Write-Host "`nSetting environment variables..." -ForegroundColor Yellow
$env:RUST_BACKTRACE = "1"
$env:CARGO_INCREMENTAL = "0"  # Disable incremental compilation for clean builds

# 4. Install/Update Windows-specific dependencies
Write-Host "`nUpdating Rust toolchain..." -ForegroundColor Yellow
rustup update
rustup target add x86_64-pc-windows-msvc

# 5. Fix potential permission issues
Write-Host "`nFixing file permissions..." -ForegroundColor Yellow
icacls . /grant "$env:USERNAME:(OI)(CI)F" /T /Q

# 6. Clean previous build artifacts
Write-Host "`nCleaning build artifacts..." -ForegroundColor Yellow
if (Test-Path "target") {
    Remove-Item "target" -Recurse -Force -ErrorAction SilentlyContinue
}
if (Test-Path "ui/dist") {
    Remove-Item "ui/dist" -Recurse -Force -ErrorAction SilentlyContinue
}
if (Test-Path "ui/node_modules") {
    Write-Host "Cleaning node_modules (this may take a while)..." -ForegroundColor Yellow
    Remove-Item "ui/node_modules" -Recurse -Force -ErrorAction SilentlyContinue
}

# 7. Fix potential SSL certificate issues
Write-Host "`nConfiguring SSL certificates..." -ForegroundColor Yellow
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# 8. Create required directories
Write-Host "`nCreating required directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "src-tauri/icons" | Out-Null
New-Item -ItemType Directory -Force -Path "src-tauri/resources" | Out-Null

# 9. Download WebView2 bootstrapper
Write-Host "`nDownloading WebView2 bootstrapper..." -ForegroundColor Yellow
$webView2Url = "https://go.microsoft.com/fwlink/p/?LinkId=2124703"
$webView2Path = "src-tauri/resources/WebView2Setup.exe"
if (-not (Test-Path $webView2Path)) {
    try {
        Invoke-WebRequest -Uri $webView2Url -OutFile $webView2Path
        Write-Host "OK: WebView2 bootstrapper downloaded" -ForegroundColor Green
    } catch {
        Write-Host "WARNING: Could not download WebView2 bootstrapper" -ForegroundColor Yellow
    }
}

# 10. Fix Cargo.lock if it exists
if (Test-Path "Cargo.lock") {
    Write-Host "`nUpdating Cargo.lock..." -ForegroundColor Yellow
    cargo update
}

Write-Host "`n=====================================" -ForegroundColor Cyan
Write-Host "Fixes applied!" -ForegroundColor Green
Write-Host "`nRecommended next steps:" -ForegroundColor Cyan
Write-Host "1. Run: .\scripts\windows_build_check.ps1" -ForegroundColor White
Write-Host "2. cd ui" -ForegroundColor White
Write-Host "3. npm install" -ForegroundColor White
Write-Host "4. npm run tauri build" -ForegroundColor White

Write-Host "`nIf you still encounter errors:" -ForegroundColor Yellow
Write-Host "- Run Visual Studio Installer and ensure 'Desktop development with C++' is installed" -ForegroundColor Gray
Write-Host "- Restart your terminal/PowerShell after installing Visual Studio" -ForegroundColor Gray
Write-Host "- Try running as Administrator if permission errors persist" -ForegroundColor Gray