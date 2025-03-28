#!/bin/bash
# Script to test voice commands in isolation

# Enable voice commands
export ENABLE_VOICE_COMMANDS=true

# Set verbose logging for voice commands
export RUST_LOG=info,voice_commands=debug

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}====================================${NC}"
echo -e "${GREEN}BestMe Voice Command Testing Utility${NC}"
echo -e "${BLUE}====================================${NC}"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Error: cargo is not installed or not in the PATH${NC}"
    echo "Please install Rust and Cargo before running this script"
    exit 1
fi

# Navigate to project root
cd "$(dirname "$0")/.." || { echo -e "${YELLOW}Failed to navigate to project root${NC}"; exit 1; }

# Function to test voice commands
test_voice_commands() {
    echo -e "\n${BLUE}Starting voice command module test...${NC}"
    echo -e "${YELLOW}This will run just the voice command module with test input${NC}"
    echo -e "Press Ctrl+C to exit the test\n"
    
    # Navigate to the src-tauri directory
    cd src-tauri || { echo -e "${YELLOW}Failed to navigate to src-tauri directory${NC}"; exit 1; }
    
    # Run the test binary
    echo -e "${GREEN}Running voice command test...${NC}"
    cargo test --package bestme --lib plugin::voice_commands::tests::test_voice_command_detection -- --nocapture
    
    # Return to the original directory
    cd .. || { echo -e "${YELLOW}Failed to return to project root${NC}"; exit 1; }
}

# Function to run a simulated test with sample phrases
simulate_test() {
    echo -e "\n${BLUE}Simulating voice commands with test phrases...${NC}"
    
    # Sample test phrases
    TEST_PHRASES=(
        "Hey BestMe, what time is it"
        "BestMe, start recording"
        "BestMe, stop recording"
        "BestMe, save transcript"
        "BestMe, clear transcript"
        "Random text that is not a command"
        "Hey BestMe, switch to dark mode"
    )
    
    echo -e "${YELLOW}Sample test phrases:${NC}"
    for phrase in "${TEST_PHRASES[@]}"; do
        echo -e "  - ${phrase}"
    done
    
    echo -e "\n${GREEN}Starting simulation...${NC}"
    
    # Simulate processing each phrase
    for phrase in "${TEST_PHRASES[@]}"; do
        echo -e "\n${BLUE}Testing phrase:${NC} ${phrase}"
        # Simple simulation of command detection
        if [[ $phrase == *"BestMe"* ]]; then
            if [[ $phrase == *"start recording"* ]]; then
                echo -e "${GREEN}✓ Command detected:${NC} START_RECORDING"
            elif [[ $phrase == *"stop recording"* ]]; then
                echo -e "${GREEN}✓ Command detected:${NC} STOP_RECORDING"
            elif [[ $phrase == *"save transcript"* ]]; then
                echo -e "${GREEN}✓ Command detected:${NC} SAVE_TRANSCRIPT"
            elif [[ $phrase == *"clear transcript"* ]]; then
                echo -e "${GREEN}✓ Command detected:${NC} CLEAR_TRANSCRIPT"
            elif [[ $phrase == *"dark mode"* ]]; then
                echo -e "${GREEN}✓ Command detected:${NC} TOGGLE_THEME"
            else
                echo -e "${YELLOW}? Potential command but action unknown${NC}"
            fi
        else
            echo -e "${YELLOW}× No command detected${NC}"
        fi
        sleep 1
    done
}

# Main menu
echo -e "\n${GREEN}Choose a test option:${NC}"
echo "1) Run voice command module test"
echo "2) Run simulated test with sample phrases"
echo "3) Exit"

read -p "Enter your choice (1-3): " choice

case $choice in
    1)
        test_voice_commands
        ;;
    2)
        simulate_test
        ;;
    3)
        echo -e "${BLUE}Exiting...${NC}"
        exit 0
        ;;
    *)
        echo -e "${YELLOW}Invalid choice. Exiting...${NC}"
        exit 1
        ;;
esac

echo -e "\n${GREEN}Test completed!${NC}" 
