#!/bin/bash

# Run tests for BestMe enhanced features
echo "Running BestMe enhanced feature tests..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Change to project root
cd "$(dirname "$0")/.." || exit 1

echo -e "${YELLOW}Running unit tests for enhanced features...${NC}"
~/.cargo/bin/cargo test audio::tests::enhanced_features_tests -- --nocapture

echo -e "\n${YELLOW}Running integration tests for streaming...${NC}"
~/.cargo/bin/cargo test audio::tests::streaming_integration_tests -- --nocapture

echo -e "\n${YELLOW}Running all audio tests...${NC}"
~/.cargo/bin/cargo test audio:: -- --nocapture

echo -e "\n${GREEN}Test run complete!${NC}"