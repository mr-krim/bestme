#!/bin/bash
# Setup script for BestMe on Linux/macOS
# This script installs dependencies and sets up the project

set -e  # Exit on error

# Terminal colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color
BLUE='\033[0;34m'

echo -e "${BLUE}===================================${NC}"
echo -e "${GREEN}BestMe Project Setup - Linux/macOS${NC}"
echo -e "${BLUE}===================================${NC}"

# Check for Node.js
echo -e "Checking for Node.js..."
if ! command -v node &> /dev/null; then
    echo -e "${RED}ERROR: Node.js not found. Please install Node.js from https://nodejs.org/${NC}"
    echo -e "After installing Node.js, run this script again."
    exit 1
fi

echo -e "Node.js found: $(node --version)"

# Check for npm
echo -e "Checking for npm..."
if ! command -v npm &> /dev/null; then
    echo -e "${RED}ERROR: npm not found. Please install Node.js from https://nodejs.org/${NC}"
    echo -e "After installing Node.js, run this script again."
    exit 1
fi

echo -e "npm found: $(npm --version)"

# Check for Rust
echo -e "Checking for Rust..."
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}ERROR: Rust not found. Please install Rust from https://rustup.rs/${NC}"
    echo -e "After installing Rust, run this script again."
    exit 1
fi

echo -e "Rust found: $(rustc --version)"

# Check for Cargo
echo -e "Checking for Cargo..."
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}ERROR: Cargo not found. Please install Rust from https://rustup.rs/${NC}"
    echo -e "After installing Rust, run this script again."
    exit 1
fi

echo -e "Cargo found: $(cargo --version)"

# Check for inotify-tools (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "Checking for inotify-tools..."
    if ! command -v inotifywait &> /dev/null; then
        echo -e "${YELLOW}WARNING: inotify-tools not found. This is needed for refresh_voice.sh${NC}"
        echo -e "Would you like to install inotify-tools now? (y/n)"
        read -r answer
        if [[ "$answer" == "y" ]]; then
            if command -v apt-get &> /dev/null; then
                sudo apt-get update
                sudo apt-get install -y inotify-tools
            elif command -v yum &> /dev/null; then
                sudo yum install -y inotify-tools
            elif command -v dnf &> /dev/null; then
                sudo dnf install -y inotify-tools
            elif command -v pacman &> /dev/null; then
                sudo pacman -S inotify-tools
            else
                echo -e "${YELLOW}Could not automatically install inotify-tools.${NC}"
                echo -e "Please install it manually using your package manager."
            fi
        fi
    else
        echo -e "inotify-tools found: $(inotifywait --version | head -n 1)"
    fi
fi

# Install Tauri CLI
echo -e "Installing Tauri CLI..."
cargo install tauri-cli || {
    echo -e "${YELLOW}WARNING: Failed to install Tauri CLI. You may need to install it manually.${NC}"
}

# Install Node.js dependencies
echo -e "Installing Node.js dependencies..."
npm install || {
    echo -e "${RED}ERROR: Failed to install Node.js dependencies.${NC}"
    exit 1
}

echo -e "Dependencies installed successfully."

# Set up development environment
echo -e "Setting up development environment..."

# Install Rust analyzer
rustup component add rust-analyzer || {
    echo -e "${YELLOW}WARNING: Failed to install rust-analyzer. You might want to install it manually.${NC}"
}

# Set up script permissions
echo -e "Setting up script permissions..."
chmod +x run_default.sh run_voice.sh run_debug.sh run_dev.sh test_voice_commands.sh refresh_voice.sh

echo -e "${BLUE}===================================${NC}"
echo -e "${GREEN}Setup complete!${NC}"
echo -e "${BLUE}===================================${NC}"
echo
echo -e "You can now run BestMe using one of the following commands:"
echo
echo -e "  ${GREEN}./run_default.sh${NC} - Run with default settings"
echo -e "  ${GREEN}./run_voice.sh${NC}   - Run with voice commands enabled"
echo -e "  ${GREEN}./run_debug.sh${NC}   - Run in debug mode"
echo -e "  ${GREEN}./refresh_voice.sh${NC} - Auto-rebuild when voice command files change"
echo
echo -e "For more information, see README.md and SCRIPTS.md"
echo 
