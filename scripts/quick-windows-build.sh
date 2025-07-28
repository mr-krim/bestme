#!/bin/bash
# Quick Windows build for testing

echo "Quick Windows Build for Testing"
echo "==============================="

# Since we're on Linux, we'll build a portable version that can be tested on Windows

# Build frontend first
echo "Building frontend..."
cd ui
npm run build
cd ..

# Build Windows executable without bundling (faster)
echo "Building Windows executable..."
cd src-tauri

# Try to build for Windows
if rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    echo "Building with x86_64-pc-windows-gnu target..."
    cargo build --release --target x86_64-pc-windows-gnu
else
    echo "Adding Windows GNU target..."
    rustup target add x86_64-pc-windows-gnu
    cargo build --release --target x86_64-pc-windows-gnu
fi

cd ..

# Create portable package
echo "Creating portable Windows package..."
mkdir -p dist/windows-portable

# Copy the executable
if [ -f "src-tauri/target/x86_64-pc-windows-gnu/release/bestme-tauri.exe" ]; then
    cp src-tauri/target/x86_64-pc-windows-gnu/release/bestme-tauri.exe dist/windows-portable/BestMe.exe
    echo "✓ Copied executable"
else
    echo "✗ Windows executable not found"
    exit 1
fi

# Copy configuration
cp -r config dist/windows-portable/
echo "✓ Copied configuration"

# Create a README for Windows users
cat > dist/windows-portable/README.txt << 'EOF'
BestMe - Portable Windows Version
=================================

This is a portable version of BestMe that doesn't require installation.

Requirements:
- Windows 10 or later
- WebView2 Runtime (usually pre-installed on Windows 10/11)
  If not installed, download from: https://go.microsoft.com/fwlink/p/?LinkId=2124703

How to run:
1. Double-click BestMe.exe
2. Allow Windows Defender if prompted
3. The app should start!

First Run:
- Models will be downloaded automatically when you first use transcription
- This may take a few minutes depending on your internet speed

Features to test:
- Click the microphone button to start recording
- Speak clearly and watch for transcription
- Try the settings menu
- Test AI text enhancement
- Check system tray functionality

Troubleshooting:
- If the app doesn't start, install WebView2 Runtime
- If no audio devices found, check Windows privacy settings
- Logs are saved in %APPDATA%\bestme\logs\

Enjoy using BestMe!
EOF

# Create a batch file to run with console output
cat > dist/windows-portable/run-with-console.bat << 'EOF'
@echo off
echo Starting BestMe with console output...
echo =====================================
BestMe.exe
pause
EOF

# Package everything
echo "Creating ZIP archive..."
cd dist/windows-portable
if command -v zip &> /dev/null; then
    zip -r ../BestMe-Windows-Portable.zip *
    echo "✓ Created ZIP: dist/BestMe-Windows-Portable.zip"
else
    echo "! ZIP command not found, skipping archive"
fi
cd ../..

echo ""
echo "================================="
echo "Windows Portable Build Complete!"
echo "================================="
echo ""
echo "Package location: dist/windows-portable/"
echo "Archive: dist/BestMe-Windows-Portable.zip"
echo ""
echo "To test on Windows:"
echo "1. Copy the 'dist/windows-portable' folder to Windows"
echo "2. Run BestMe.exe"
echo "3. Or run 'run-with-console.bat' to see debug output"
echo ""
echo "File size: $(du -h dist/windows-portable/BestMe.exe | cut -f1)"

# Also try to build proper installer if possible
echo ""
echo "For a proper installer, you can also try:"
echo "cargo tauri build --target x86_64-pc-windows-gnu"