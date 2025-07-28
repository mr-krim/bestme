#!/bin/bash
# AppImage packaging script for BestMe

set -e

echo "Building BestMe AppImage..."
echo "============================"

# Check if we're on Linux
if [ "$(uname)" != "Linux" ]; then
    echo "Error: AppImage can only be built on Linux"
    exit 1
fi

# Set up directories
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/appimage"
APP_DIR="$BUILD_DIR/AppDir"

# Clean up old build
rm -rf "$BUILD_DIR"
mkdir -p "$APP_DIR"

# Build the Tauri app in release mode
echo "Building release binary..."
cd "$PROJECT_ROOT"
cargo tauri build

# Find the built binary
TAURI_BINARY="$PROJECT_ROOT/src-tauri/target/release/bestme-tauri"
if [ ! -f "$TAURI_BINARY" ]; then
    TAURI_BINARY="$PROJECT_ROOT/target/release/bestme-tauri"
fi

if [ ! -f "$TAURI_BINARY" ]; then
    echo "Error: Could not find bestme-tauri binary"
    echo "Please run 'cargo tauri build' first"
    exit 1
fi

# Create AppDir structure
mkdir -p "$APP_DIR/usr/bin"
mkdir -p "$APP_DIR/usr/lib"
mkdir -p "$APP_DIR/usr/share/applications"
mkdir -p "$APP_DIR/usr/share/icons/hicolor/256x256/apps"

# Copy binary
echo "Copying binary..."
cp "$TAURI_BINARY" "$APP_DIR/usr/bin/bestme"
chmod +x "$APP_DIR/usr/bin/bestme"

# Copy ONNX Runtime library
echo "Copying ONNX Runtime..."
ONNX_LIB=$(find "$PROJECT_ROOT/target" -name "libonnxruntime.so*" | head -1)
if [ -n "$ONNX_LIB" ]; then
    cp "$ONNX_LIB" "$APP_DIR/usr/lib/"
    # Create symlink if needed
    if [[ "$ONNX_LIB" == *.so.* ]]; then
        ln -sf "$(basename "$ONNX_LIB")" "$APP_DIR/usr/lib/libonnxruntime.so"
    fi
fi

# Create desktop file
cat > "$APP_DIR/usr/share/applications/bestme.desktop" << EOF
[Desktop Entry]
Name=BestMe
Comment=AI-Powered Speech-to-Text Application
Exec=bestme
Icon=bestme
Type=Application
Categories=AudioVideo;Audio;Utility;
Terminal=false
EOF

# Create AppRun script
cat > "$APP_DIR/AppRun" << 'EOF'
#!/bin/bash
# AppRun script for BestMe

# Get the directory where AppImage is mounted
HERE="$(dirname "$(readlink -f "${0}")")"

# Set library path
export LD_LIBRARY_PATH="$HERE/usr/lib:$LD_LIBRARY_PATH"

# Set GDK backend to X11 if running under Wayland
if [ "$XDG_SESSION_TYPE" = "wayland" ]; then
    export GDK_BACKEND=x11
fi

# Run the application
exec "$HERE/usr/bin/bestme" "$@"
EOF

chmod +x "$APP_DIR/AppRun"

# Create a simple icon if none exists
if [ ! -f "$PROJECT_ROOT/src-tauri/icons/256x256.png" ]; then
    echo "Creating placeholder icon..."
    # Create a simple colored square as placeholder
    convert -size 256x256 xc:'#4A90E2' \
            -fill white -gravity center \
            -pointsize 72 -annotate +0+0 'BM' \
            "$APP_DIR/usr/share/icons/hicolor/256x256/apps/bestme.png" 2>/dev/null || \
    echo "Warning: Could not create icon (ImageMagick not installed)"
else
    cp "$PROJECT_ROOT/src-tauri/icons/256x256.png" "$APP_DIR/usr/share/icons/hicolor/256x256/apps/bestme.png"
fi

# Copy icon to root for AppImage
if [ -f "$APP_DIR/usr/share/icons/hicolor/256x256/apps/bestme.png" ]; then
    cp "$APP_DIR/usr/share/icons/hicolor/256x256/apps/bestme.png" "$APP_DIR/bestme.png"
fi

# Download appimagetool if not present
APPIMAGETOOL="$BUILD_DIR/appimagetool-x86_64.AppImage"
if [ ! -f "$APPIMAGETOOL" ]; then
    echo "Downloading appimagetool..."
    wget -q https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage \
         -O "$APPIMAGETOOL"
    chmod +x "$APPIMAGETOOL"
fi

# Build AppImage
echo "Building AppImage..."
cd "$BUILD_DIR"
"$APPIMAGETOOL" --appimage-extract-and-run "$APP_DIR" "BestMe-x86_64.AppImage"

# Move to dist directory
DIST_DIR="$PROJECT_ROOT/dist"
mkdir -p "$DIST_DIR"
mv "$BUILD_DIR/BestMe-x86_64.AppImage" "$DIST_DIR/"

echo ""
echo "✅ AppImage created successfully!"
echo "📦 Output: $DIST_DIR/BestMe-x86_64.AppImage"
echo ""
echo "To run: $DIST_DIR/BestMe-x86_64.AppImage"