#!/bin/bash
# BestMe Packaging Script

set -e

echo "BestMe Packaging Script"
echo "======================"

# Detect OS
OS="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    OS="windows"
fi

echo "Detected OS: $OS"

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Build frontend
echo "Building frontend..."
cd ui
npm ci
npm run build
cd ..

# Build Tauri app
echo "Building Tauri application..."
cd src-tauri
cargo tauri build

# Package location
BUNDLE_DIR="target/release/bundle"
OUTPUT_DIR="../dist"

# Create output directory
mkdir -p $OUTPUT_DIR

# Copy bundles based on OS
echo "Packaging for $OS..."

case $OS in
    linux)
        # Copy AppImage
        if [ -f "$BUNDLE_DIR/appimage/bestme_*.AppImage" ]; then
            cp $BUNDLE_DIR/appimage/bestme_*.AppImage $OUTPUT_DIR/
            echo "✓ AppImage packaged"
        fi
        
        # Copy deb
        if [ -f "$BUNDLE_DIR/deb/bestme_*.deb" ]; then
            cp $BUNDLE_DIR/deb/bestme_*.deb $OUTPUT_DIR/
            echo "✓ Debian package created"
        fi
        
        # Copy rpm if exists
        if [ -f "$BUNDLE_DIR/rpm/bestme_*.rpm" ]; then
            cp $BUNDLE_DIR/rpm/bestme_*.rpm $OUTPUT_DIR/
            echo "✓ RPM package created"
        fi
        ;;
        
    macos)
        # Copy dmg
        if [ -f "$BUNDLE_DIR/dmg/bestme_*.dmg" ]; then
            cp $BUNDLE_DIR/dmg/bestme_*.dmg $OUTPUT_DIR/
            echo "✓ DMG created"
        fi
        
        # Copy app bundle
        if [ -d "$BUNDLE_DIR/macos/BestMe.app" ]; then
            cp -r $BUNDLE_DIR/macos/BestMe.app $OUTPUT_DIR/
            echo "✓ App bundle created"
        fi
        ;;
        
    windows)
        # Copy MSI
        if [ -f "$BUNDLE_DIR/msi/bestme_*.msi" ]; then
            cp $BUNDLE_DIR/msi/bestme_*.msi $OUTPUT_DIR/
            echo "✓ MSI installer created"
        fi
        
        # Copy NSIS installer
        if [ -f "$BUNDLE_DIR/nsis/bestme_*.exe" ]; then
            cp $BUNDLE_DIR/nsis/bestme_*.exe $OUTPUT_DIR/
            echo "✓ NSIS installer created"
        fi
        ;;
esac

# Create portable version
echo "Creating portable version..."
PORTABLE_DIR="$OUTPUT_DIR/bestme-portable-$OS"
mkdir -p $PORTABLE_DIR

# Copy binary
if [ "$OS" == "windows" ]; then
    cp target/release/bestme-tauri.exe $PORTABLE_DIR/bestme.exe
else
    cp target/release/bestme-tauri $PORTABLE_DIR/bestme
    chmod +x $PORTABLE_DIR/bestme
fi

# Copy resources
cp -r ../config $PORTABLE_DIR/
cp ../README.md $PORTABLE_DIR/
cp ../LICENSE $PORTABLE_DIR/

# Create run script for portable
if [ "$OS" != "windows" ]; then
    cat > $PORTABLE_DIR/run.sh << 'EOF'
#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$DIR"
./bestme
EOF
    chmod +x $PORTABLE_DIR/run.sh
fi

# Compress portable version
echo "Compressing portable version..."
cd $OUTPUT_DIR
if [ "$OS" == "windows" ]; then
    # Use zip for Windows
    zip -r bestme-portable-$OS.zip bestme-portable-$OS/
else
    # Use tar.gz for Unix-like systems
    tar -czf bestme-portable-$OS.tar.gz bestme-portable-$OS/
fi
rm -rf bestme-portable-$OS/

echo ""
echo "Packaging complete! Files available in: $OUTPUT_DIR"
echo ""
ls -la $OUTPUT_DIR/