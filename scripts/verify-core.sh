#!/bin/bash

# Verification script for core functionality of BestMe application
# This script tests basic application functionality after Tauri 2.0 migration

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test status tracking
PASSED=0
FAILED=0
WARNINGS=0

echo -e "${YELLOW}=== BestMe Core Functionality Verification ===${NC}"
echo "Running verification tests for core functionality..."

# Function to track test results
report_test() {
    local test_name="$1"
    local status="$2"
    local message="$3"
    
    if [ "$status" -eq 0 ]; then
        echo -e "${GREEN}[PASS]${NC} $test_name"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}[FAIL]${NC} $test_name: $message"
        FAILED=$((FAILED + 1))
    fi
}

warn() {
    local test_name="$1"
    local message="$2"
    
    echo -e "${YELLOW}[WARN]${NC} $test_name: $message"
    WARNINGS=$((WARNINGS + 1))
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: This script must be run from the root directory of the BestMe project${NC}"
    exit 1
fi

echo -e "\n${YELLOW}=== Environment Verification ===${NC}"

# Check Rust installation
echo "Checking Rust installation..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    report_test "Rust installation" 0
    echo "  Rust version: $RUST_VERSION"
else
    report_test "Rust installation" 1 "Rust not found"
fi

# Check Node.js installation
echo "Checking Node.js installation..."
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    report_test "Node.js installation" 0
    echo "  Node.js version: $NODE_VERSION"
else
    report_test "Node.js installation" 1 "Node.js not found"
fi

# Check for required system libraries
echo "Checking system libraries..."
if pkg-config --list-all | grep -q "javascriptcoregtk-4.1"; then
    report_test "JavaScriptCore GTK" 0
else
    report_test "JavaScriptCore GTK" 1 "Library not found"
fi

if pkg-config --list-all | grep -q "libsoup-3.0"; then
    report_test "libsoup" 0
else
    report_test "libsoup" 1 "Library not found"
fi

echo -e "\n${YELLOW}=== Build Verification ===${NC}"

# Clean build
echo "Running clean build..."
cargo clean &> /dev/null
RUST_BACKTRACE=1 cargo build 2> build_errors.log

if [ $? -eq 0 ]; then
    report_test "Clean build" 0
else
    report_test "Clean build" 1 "Build failed, see build_errors.log for details"
fi

# Check for warnings
if grep -q "warning" build_errors.log; then
    warn "Build warnings" "$(grep -c "warning" build_errors.log) warnings found"
    echo "  Run 'cargo build' to see warnings"
fi

echo -e "\n${YELLOW}=== Plugin Initialization Verification ===${NC}"

# Check if plugins are properly initialized
echo "Checking plugin initialization..."
grep -q "initialize" src-tauri/src/plugin/audio.rs
if [ $? -eq 0 ]; then
    report_test "Audio plugin initialization" 0
else
    report_test "Audio plugin initialization" 1 "Initialize method not found"
fi

grep -q "initialize" src-tauri/src/plugin/transcribe.rs
if [ $? -eq 0 ]; then
    report_test "Transcription plugin initialization" 0
else
    report_test "Transcription plugin initialization" 1 "Initialize method not found"
fi

grep -q "initialize" src-tauri/src/plugin/voice_commands.rs
if [ $? -eq 0 ]; then
    report_test "Voice command plugin initialization" 0
else
    report_test "Voice command plugin initialization" 1 "Initialize method not found"
fi

echo -e "\n${YELLOW}=== Config File Verification ===${NC}"

# Check configuration files
if [ -f "config/config.json" ]; then
    report_test "Config file exists" 0
elif [ -f "config.json" ]; then
    warn "Config file location" "config.json found in root directory; should be moved to config/ directory"
    report_test "Config file exists" 0
else
    report_test "Config file exists" 1 "config.json not found in config/ or root directory"
fi

if [ -f "settings.cfg" ]; then
    report_test "Settings file exists" 0
else
    report_test "Settings file exists" 1 "settings.cfg not found"
fi

echo -e "\n${YELLOW}=== Frontend Verification ===${NC}"

# Check if frontend files exist
if [ -d "ui" ]; then
    report_test "UI directory exists" 0
else
    report_test "UI directory exists" 1 "ui directory not found"
fi

# Check package.json
if [ -f "ui/package.json" ] && grep -q "tauri" "ui/package.json"; then
    report_test "Tauri in package.json" 0
elif [ -f "package.json" ] && grep -q "tauri" "package.json"; then
    warn "package.json location" "package.json found in root directory; should be moved to ui/ directory"
    report_test "Tauri in package.json" 0
else
    warn "Tauri in package.json" "Tauri dependency might be missing in package.json"
fi

echo -e "\n${YELLOW}=== Test Summary ===${NC}"
echo -e "Tests passed: ${GREEN}$PASSED${NC}"
echo -e "Tests failed: ${RED}$FAILED${NC}"
echo -e "Warnings: ${YELLOW}$WARNINGS${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}Core verification completed successfully!${NC}"
    echo "Next steps:"
    echo "1. Run manual verification of audio recording"
    echo "2. Test transcription functionality"
    echo "3. Verify voice command execution"
    exit 0
else
    echo -e "\n${RED}Core verification completed with failures!${NC}"
    echo "Please fix the reported issues before proceeding."
    exit 1
fi 
