#!/bin/bash
# Build script for Windows packaging

echo "================================="
echo "Building BestMe for Windows"
echo "================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if we're on Windows/WSL or cross-compiling from Linux
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]] || [[ -n "$WSL_DISTRO_NAME" ]]; then
    echo -e "${GREEN}Building on Windows/WSL${NC}"
    TARGET="x86_64-pc-windows-msvc"
else
    echo -e "${YELLOW}Cross-compiling for Windows from Linux${NC}"
    TARGET="x86_64-pc-windows-gnu"
    
    # Check if cross-compilation tools are installed
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo -e "${RED}Error: MinGW cross-compiler not found${NC}"
        echo "Install with: sudo apt-get install mingw-w64"
        exit 1
    fi
fi

# Step 1: Add Windows target if not already added
echo -e "\n${YELLOW}1. Checking Rust target${NC}"
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Adding Windows target: $TARGET"
    rustup target add $TARGET
else
    echo -e "${GREEN}✓ Windows target already installed${NC}"
fi

# Step 2: Build frontend
echo -e "\n${YELLOW}2. Building frontend${NC}"
cd ui
if npm run build; then
    echo -e "${GREEN}✓ Frontend built successfully${NC}"
else
    echo -e "${RED}✗ Frontend build failed${NC}"
    exit 1
fi
cd ..

# Step 3: Build Tauri app for Windows
echo -e "\n${YELLOW}3. Building Tauri app for Windows${NC}"
cd src-tauri

# Set environment variables for Windows build
export TAURI_SIGNING_PRIVATE_KEY=""
export TAURI_SIGNING_PUBLIC_KEY=""

# Build command based on environment
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]] || [[ -n "$WSL_DISTRO_NAME" ]]; then
    # Native Windows build
    cargo tauri build --target $TARGET
else
    # Cross-compilation
    cargo tauri build --target $TARGET --bundles msi,nsis
fi

BUILD_STATUS=$?
cd ..

if [ $BUILD_STATUS -eq 0 ]; then
    echo -e "\n${GREEN}✓ Build completed successfully!${NC}"
    
    # Find the output files
    echo -e "\n${YELLOW}4. Package locations:${NC}"
    
    # Check for MSI installer
    MSI_PATH="src-tauri/target/$TARGET/release/bundle/msi/BestMe_0.1.0_x64_en-US.msi"
    if [ -f "$MSI_PATH" ]; then
        echo -e "${GREEN}✓ MSI Installer:${NC}"
        echo "  $MSI_PATH"
        echo "  Size: $(du -h "$MSI_PATH" | cut -f1)"
    fi
    
    # Check for NSIS installer
    NSIS_PATH="src-tauri/target/$TARGET/release/bundle/nsis/BestMe_0.1.0_x64-setup.exe"
    if [ -f "$NSIS_PATH" ]; then
        echo -e "${GREEN}✓ NSIS Installer:${NC}"
        echo "  $NSIS_PATH"
        echo "  Size: $(du -h "$NSIS_PATH" | cut -f1)"
    fi
    
    # Check for portable exe
    EXE_PATH="src-tauri/target/$TARGET/release/bestme-tauri.exe"
    if [ -f "$EXE_PATH" ]; then
        echo -e "${GREEN}✓ Portable EXE:${NC}"
        echo "  $EXE_PATH"
        echo "  Size: $(du -h "$EXE_PATH" | cut -f1)"
    fi
    
    # Create dist directory
    mkdir -p dist/windows
    
    # Copy files if they exist
    [ -f "$MSI_PATH" ] && cp "$MSI_PATH" dist/windows/
    [ -f "$NSIS_PATH" ] && cp "$NSIS_PATH" dist/windows/
    [ -f "$EXE_PATH" ] && cp "$EXE_PATH" dist/windows/
    
    # Copy required DLLs
    echo -e "\n${YELLOW}5. Copying required files${NC}"
    
    # Copy WebView2 loader if needed
    if [ -f "src-tauri/target/$TARGET/release/WebView2Loader.dll" ]; then
        cp "src-tauri/target/$TARGET/release/WebView2Loader.dll" dist/windows/
        echo "  ✓ WebView2Loader.dll"
    fi
    
    # Copy ONNX Runtime DLLs
    if ls src-tauri/target/$TARGET/release/onnxruntime*.dll 1> /dev/null 2>&1; then
        cp src-tauri/target/$TARGET/release/onnxruntime*.dll dist/windows/
        echo "  ✓ ONNX Runtime DLLs"
    fi
    
    echo -e "\n${GREEN}✅ Windows build complete!${NC}"
    echo -e "\nFiles ready in: ${YELLOW}dist/windows/${NC}"
    echo -e "\nTo test on Windows:"
    echo "1. Copy dist/windows/ folder to your Windows machine"
    echo "2. Install using the MSI or NSIS installer"
    echo "3. Or run the portable bestme-tauri.exe directly"
    
else
    echo -e "\n${RED}✗ Build failed!${NC}"
    echo "Check the error messages above"
    exit 1
fi