#!/bin/bash
# Script to set up permissions and environment for BestMe

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}====================================${NC}"
echo -e "${GREEN}BestMe Development Environment Setup${NC}"
echo -e "${BLUE}====================================${NC}"

# Navigate to project root
cd "$(dirname "$0")/.." || { echo "Failed to navigate to project root"; exit 1; }

# Check for required dependencies
check_dependency() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${YELLOW}Warning: $1 is not installed or not in the PATH${NC}"
        echo "This may cause issues with development."
        return 1
    else
        echo -e "${GREEN}✓ $1 is installed${NC}"
        return 0
    fi
}

echo -e "\n${BLUE}Checking for required dependencies...${NC}"
check_dependency "cargo"
check_dependency "rustc"
check_dependency "npm"
check_dependency "node"

# Check if rust is up to date
echo -e "\n${BLUE}Checking Rust version...${NC}"
rustc --version
echo -e "${YELLOW}Note: Rust 1.70+ is recommended for development${NC}"

# Check if tauri-cli is installed
if ! cargo install --list | grep -q "tauri-cli"; then
    echo -e "\n${YELLOW}Warning: tauri-cli is not installed${NC}"
    read -p "Would you like to install it now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Installing tauri-cli...${NC}"
        cargo install tauri-cli
    else
        echo -e "${YELLOW}Skipping tauri-cli installation. You will need it to run the app.${NC}"
    fi
else
    echo -e "${GREEN}✓ tauri-cli is installed${NC}"
fi

# Check for platform-specific dependencies
echo -e "\n${BLUE}Checking platform-specific dependencies...${NC}"
OS="$(uname -s)"
case "${OS}" in
    Linux*)
        echo "Linux detected, checking for GTK and WebKit dependencies..."
        
        # Check for common Linux dependencies
        MISSING_DEPS=()
        for dep in "libgtk-3-dev" "libwebkit2gtk-4.0-dev" "libappindicator3-dev" "librsvg2-dev" "patchelf"; do
            if ! dpkg -s "$dep" &> /dev/null; then
                MISSING_DEPS+=("$dep")
            fi
        done
        
        if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
            echo -e "${YELLOW}Warning: The following dependencies are missing:${NC}"
            for dep in "${MISSING_DEPS[@]}"; do
                echo "  - $dep"
            done
            echo -e "${YELLOW}You may need to install them to build the app.${NC}"
            echo "On Ubuntu/Debian, you can install them with:"
            echo "sudo apt update && sudo apt install ${MISSING_DEPS[*]}"
        else
            echo -e "${GREEN}✓ All required Linux dependencies are installed${NC}"
        fi
        ;;
    Darwin*)
        echo "macOS detected, checking for Xcode Command Line Tools..."
        if ! xcode-select -p &> /dev/null; then
            echo -e "${YELLOW}Warning: Xcode Command Line Tools not found${NC}"
            echo "You can install them by running: xcode-select --install"
        else
            echo -e "${GREEN}✓ Xcode Command Line Tools are installed${NC}"
        fi
        ;;
    CYGWIN*|MINGW*|MSYS*|Windows*)
        echo "Windows detected. Ensure you have the following installed:"
        echo "  - Visual Studio Build Tools"
        echo "  - WebView2 Runtime"
        echo -e "${YELLOW}Please refer to the Tauri documentation for Windows-specific setup.${NC}"
        ;;
    *)
        echo -e "${YELLOW}Unknown operating system. Please ensure you have the required dependencies for Tauri.${NC}"
        ;;
esac

# Set up permissions for scripts
echo -e "\n${BLUE}Setting up script permissions...${NC}"
chmod +x scripts/run_default.sh scripts/run_voice.sh scripts/run_debug.sh scripts/run_dev.sh scripts/test_voice_commands.sh scripts/refresh_voice.sh

# Set up environment variables
echo -e "\n${BLUE}Setting up environment variables...${NC}"
ENV_FILE=".env"
if [ ! -f "$ENV_FILE" ]; then
    echo "Creating default .env file..."
    cat > "$ENV_FILE" << EOF
# BestMe Environment Variables
RUST_LOG=info
EOF
    echo -e "${GREEN}✓ Created .env file${NC}"
else
    echo -e "${GREEN}✓ Using existing .env file${NC}"
fi

# Install npm dependencies
echo -e "\n${BLUE}Installing frontend dependencies...${NC}"
if [ ! -d "ui/node_modules" ]; then
    echo "Installing npm packages..."
    cd ui && npm install && cd ..
    echo -e "${GREEN}✓ Frontend dependencies installed${NC}"
else
    echo -e "${GREEN}✓ Frontend dependencies already installed${NC}"
fi

echo -e "\n${GREEN}Setup complete! You can now run the application:${NC}"
echo -e "  ${GREEN}./scripts/run_default.sh${NC} - Run with default settings"
echo -e "  ${GREEN}./scripts/run_voice.sh${NC}   - Run with voice commands enabled"
echo -e "  ${GREEN}./scripts/run_debug.sh${NC}   - Run in debug mode"
echo -e "  ${GREEN}./scripts/refresh_voice.sh${NC} - Auto-rebuild when voice command files change"
echo
echo -e "For more information, see README.md and SCRIPTS.md"
echo 
