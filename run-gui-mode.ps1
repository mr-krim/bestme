# BestMe GUI Mode Launch Script (PowerShell)
# This script ensures the application runs in GUI mode and stays visible

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "   BestMe Application - GUI Mode" -ForegroundColor Cyan
Write-Host "   Tauri 2.0 Windows Testing" -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""

# Set environment variables for debugging
$env:RUST_LOG = "info"
$env:RUST_BACKTRACE = "1"

Write-Host "Starting BestMe in GUI mode..." -ForegroundColor Green

# Check if application exists
if (-not (Test-Path ".\bestme.exe")) {
    Write-Host "Error: bestme.exe not found in current directory!" -ForegroundColor Red
    Write-Host "Make sure you're running this script from the same directory as the application." -ForegroundColor Yellow
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

# Additional parameters for GUI mode
$guiParams = @("--gui")

try {
    # Launch the application
    $process = Start-Process -FilePath ".\bestme.exe" -ArgumentList $guiParams -PassThru -Wait
    
    # Check exit code
    if ($process.ExitCode -ne 0) {
        Write-Host ""
        Write-Host "Application exited with error code: $($process.ExitCode)" -ForegroundColor Red
        Write-Host ""
        Write-Host "Please check the logs for more information." -ForegroundColor Yellow
        Write-Host "Logs may be found in: $env:APPDATA\bestme\logs" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Press any key to close this window..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    }
}
catch {
    Write-Host "Error launching BestMe application: $_" -ForegroundColor Red
    Write-Host "Press any key to close this window..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
} 
