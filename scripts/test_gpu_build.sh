#!/bin/bash
# Test GPU compilation for all supported backends

echo "=== Testing GPU Backend Compilation ==="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Track results
TOTAL=0
PASSED=0

# Test CUDA
echo "Testing CUDA backend..."
TOTAL=$((TOTAL + 1))
if cargo check --features gpu-cuda 2>&1 | grep -q "error"; then
    echo -e "${RED}✗ CUDA compilation failed${NC}"
else
    echo -e "${GREEN}✓ CUDA compilation passed${NC}"
    PASSED=$((PASSED + 1))
fi
echo

# Test Metal (macOS only)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Testing Metal backend..."
    TOTAL=$((TOTAL + 1))
    if cargo check --features gpu-metal 2>&1 | grep -q "error"; then
        echo -e "${RED}✗ Metal compilation failed${NC}"
    else
        echo -e "${GREEN}✓ Metal compilation passed${NC}"
        PASSED=$((PASSED + 1))
    fi
    echo
else
    echo -e "${YELLOW}⚠ Metal backend skipped (not on macOS)${NC}"
    echo
fi

# Test Vulkan
echo "Testing Vulkan backend..."
TOTAL=$((TOTAL + 1))
if cargo check --features gpu-vulkan 2>&1 | grep -q "error"; then
    echo -e "${RED}✗ Vulkan compilation failed${NC}"
else
    echo -e "${GREEN}✓ Vulkan compilation passed${NC}"
    PASSED=$((PASSED + 1))
fi
echo

# Test HIP/ROCm
echo "Testing HIP/ROCm backend..."
TOTAL=$((TOTAL + 1))
if cargo check --features gpu-hipblas 2>&1 | grep -q "error"; then
    echo -e "${RED}✗ HIP/ROCm compilation failed${NC}"
else
    echo -e "${GREEN}✓ HIP/ROCm compilation passed${NC}"
    PASSED=$((PASSED + 1))
fi
echo

# Test all features together
echo "Testing all GPU backends together..."
TOTAL=$((TOTAL + 1))
if cargo check --features gpu-all 2>&1 | grep -q "error"; then
    echo -e "${RED}✗ Combined GPU compilation failed${NC}"
else
    echo -e "${GREEN}✓ Combined GPU compilation passed${NC}"
    PASSED=$((PASSED + 1))
fi
echo

# Summary
echo "=== Summary ==="
echo "Passed: $PASSED/$TOTAL"

if [ $PASSED -eq $TOTAL ]; then
    echo -e "${GREEN}All GPU backends compiled successfully!${NC}"
    exit 0
else
    echo -e "${RED}Some GPU backends failed to compile${NC}"
    exit 1
fi