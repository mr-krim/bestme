#!/bin/bash

# Code Coverage Report Script for BestMe
# This script analyzes test coverage across the codebase

echo "=================================================="
echo "           BestMe Code Coverage Report            "
echo "=================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to count tests and lines
count_coverage() {
    local dir=$1
    local module=$2
    
    # Count test functions
    local tests=$(find "$dir" -name "*.rs" -exec grep -c "#\[test\]" {} \; | awk '{s+=$1} END {print s}')
    tests=${tests:-0}
    
    # Count test modules
    local test_mods=$(find "$dir" -name "*.rs" -exec grep -c "#\[cfg(test)\]" {} \; | awk '{s+=$1} END {print s}')
    test_mods=${test_mods:-0}
    
    # Count total lines (excluding tests)
    local total_lines=$(find "$dir" -name "*.rs" -exec grep -v -E "^\s*#\[test\]|^\s*#\[cfg\(test\)\]|^\s*//" {} \; | wc -l)
    
    # Count source files
    local files=$(find "$dir" -name "*.rs" | wc -l)
    
    # Estimate coverage based on test density
    local coverage=0
    if [ $total_lines -gt 0 ]; then
        # Rough estimate: each test covers ~20 lines on average
        local covered_lines=$((tests * 20))
        coverage=$((covered_lines * 100 / total_lines))
        [ $coverage -gt 100 ] && coverage=100
    fi
    
    # Determine color based on coverage
    local color=$RED
    [ $coverage -ge 50 ] && color=$YELLOW
    [ $coverage -ge 80 ] && color=$GREEN
    
    printf "%-30s ${color}%3d%%${NC} | %4d tests | %4d modules | %6d lines | %3d files\n" \
           "$module" "$coverage" "$tests" "$test_mods" "$total_lines" "$files"
}

# Header
printf "%-30s %4s | %10s | %12s | %12s | %9s\n" "Module" "Cov%" "Tests" "Test Modules" "Lines" "Files"
echo "----------------------------------------------------------------------------------------"

# Core modules
count_coverage "src/audio" "Audio"
count_coverage "src/ai" "AI"
count_coverage "src/storage" "Storage"
count_coverage "src/text_injection" "Text Injection"
count_coverage "src/gui" "GUI"
count_coverage "src/config" "Config"

echo "----------------------------------------------------------------------------------------"

# Total statistics
total_tests=$(find src -name "*.rs" -exec grep -c "#\[test\]" {} \; | awk '{s+=$1} END {print s}')
total_test_mods=$(find src -name "*.rs" -exec grep -c "#\[cfg(test)\]" {} \; | awk '{s+=$1} END {print s}')
total_lines=$(find src -name "*.rs" -not -path "*/tests/*" | xargs wc -l | tail -1 | awk '{print $1}')
total_files=$(find src -name "*.rs" | wc -l)

# Calculate overall coverage
covered_lines=$((total_tests * 20))
overall_coverage=$((covered_lines * 100 / total_lines))
[ $overall_coverage -gt 100 ] && overall_coverage=100

# Determine overall color
overall_color=$RED
[ $overall_coverage -ge 50 ] && overall_color=$YELLOW
[ $overall_coverage -ge 80 ] && overall_color=$GREEN

printf "%-30s ${overall_color}%3d%%${NC} | %4d tests | %4d modules | %6d lines | %3d files\n" \
       "TOTAL" "$overall_coverage" "$total_tests" "$total_test_mods" "$total_lines" "$total_files"

echo ""
echo "=================================================="
echo "             Test Distribution                    "
echo "=================================================="
echo ""

# Find files with most tests
echo "Top 5 Most Tested Files:"
find src -name "*.rs" -exec bash -c 'echo "$(grep -c "#\[test\]" "$1") $1"' _ {} \; | \
    sort -nr | head -5 | while read count file; do
    printf "  %3d tests - %s\n" "$count" "${file#src/}"
done

echo ""
echo "Files Without Tests:"
files_without_tests=0
for file in $(find src -name "*.rs" -not -path "*/tests/*" -not -path "*/bin/*"); do
    if ! grep -q "#\[test\]" "$file" && ! grep -q "#\[cfg(test)\]" "$file"; then
        # Skip files that are pure trait definitions or very small
        lines=$(wc -l < "$file")
        if [ $lines -gt 50 ]; then
            echo "  - ${file#src/}"
            ((files_without_tests++))
        fi
    fi
done

echo ""
echo "=================================================="
echo "              Coverage Summary                    "
echo "=================================================="
echo ""

echo "Overall Statistics:"
echo "  Total Test Functions:  $total_tests"
echo "  Total Test Modules:    $total_test_mods"
echo "  Total Lines of Code:   $total_lines"
echo "  Total Source Files:    $total_files"
echo "  Files Without Tests:   $files_without_tests"
printf "  Estimated Coverage:    ${overall_color}%d%%${NC}\n" "$overall_coverage"

echo ""
echo "Coverage Legend:"
echo -e "  ${GREEN}■${NC} 80%+ Excellent"
echo -e "  ${YELLOW}■${NC} 50-79% Good"
echo -e "  ${RED}■${NC} <50% Needs Improvement"

echo ""
echo "Note: This is an estimated coverage based on test density."
echo "For exact coverage, use 'cargo tarpaulin' when build issues are resolved."