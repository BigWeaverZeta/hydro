#!/bin/bash
# Script to verify test structure and documentation in the Hydro repository

set -e

echo "=================================="
echo "Hydro Test Verification"
echo "=================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counter for checks
checks_passed=0
checks_total=0

# Function to check if file exists
check_file() {
    checks_total=$((checks_total + 1))
    if [ -f "$1" ]; then
        echo -e "${GREEN}✓${NC} Found: $1"
        checks_passed=$((checks_passed + 1))
        return 0
    else
        echo -e "${YELLOW}✗${NC} Missing: $1"
        return 1
    fi
}

# Function to check if directory exists
check_dir() {
    checks_total=$((checks_total + 1))
    if [ -d "$1" ]; then
        echo -e "${GREEN}✓${NC} Directory exists: $1"
        checks_passed=$((checks_passed + 1))
        return 0
    else
        echo -e "${YELLOW}✗${NC} Directory missing: $1"
        return 1
    fi
}

# Function to count test files
count_tests() {
    local dir=$1
    local count=$(find "$dir" -name "*.rs" -type f | wc -l)
    echo -e "${BLUE}  → $count test files found${NC}"
}

echo "1. Checking new test directories..."
echo "-----------------------------------"
check_dir "hydro_std/tests" && count_tests "hydro_std/tests"
check_dir "hydro_build_utils/tests" && count_tests "hydro_build_utils/tests"
check_dir "tests" && count_tests "tests"
echo ""

echo "2. Checking hydro_std test files..."
echo "------------------------------------"
check_file "hydro_std/tests/quorum_tests.rs"
check_file "hydro_std/tests/membership_tests.rs"
check_file "hydro_std/tests/request_response_tests.rs"
check_file "hydro_std/tests/README.md"
echo ""

echo "3. Checking variadics test files..."
echo "------------------------------------"
check_file "variadics/tests/variadic_operations.rs"
echo ""

echo "4. Checking lattices test files..."
echo "-----------------------------------"
check_file "lattices/tests/lattice_properties.rs"
echo ""

echo "5. Checking hydro_build_utils test files..."
echo "--------------------------------------------"
check_file "hydro_build_utils/tests/macro_tests.rs"
echo ""

echo "6. Checking workspace-level test files..."
echo "------------------------------------------"
check_file "tests/workspace_integration.rs"
echo ""

echo "7. Checking documentation files..."
echo "-----------------------------------"
check_file "TESTING.md"
check_file "TEST_SUMMARY.md"
echo ""

echo "8. Checking existing test infrastructure..."
echo "--------------------------------------------"
check_dir "dfir_rs/tests" && count_tests "dfir_rs/tests"
check_dir "sinktools/tests" && count_tests "sinktools/tests"
check_dir "lattices/tests" && count_tests "lattices/tests"
check_dir "variadics/tests" && count_tests "variadics/tests"
echo ""

echo "9. Analyzing test file sizes..."
echo "--------------------------------"
if [ -f "hydro_std/tests/quorum_tests.rs" ]; then
    size=$(wc -l < "hydro_std/tests/quorum_tests.rs")
    echo -e "${BLUE}  → hydro_std/tests/quorum_tests.rs: $size lines${NC}"
fi
if [ -f "hydro_std/tests/membership_tests.rs" ]; then
    size=$(wc -l < "hydro_std/tests/membership_tests.rs")
    echo -e "${BLUE}  → hydro_std/tests/membership_tests.rs: $size lines${NC}"
fi
if [ -f "hydro_std/tests/request_response_tests.rs" ]; then
    size=$(wc -l < "hydro_std/tests/request_response_tests.rs")
    echo -e "${BLUE}  → hydro_std/tests/request_response_tests.rs: $size lines${NC}"
fi
if [ -f "variadics/tests/variadic_operations.rs" ]; then
    size=$(wc -l < "variadics/tests/variadic_operations.rs")
    echo -e "${BLUE}  → variadics/tests/variadic_operations.rs: $size lines${NC}"
fi
if [ -f "lattices/tests/lattice_properties.rs" ]; then
    size=$(wc -l < "lattices/tests/lattice_properties.rs")
    echo -e "${BLUE}  → lattices/tests/lattice_properties.rs: $size lines${NC}"
fi
if [ -f "hydro_build_utils/tests/macro_tests.rs" ]; then
    size=$(wc -l < "hydro_build_utils/tests/macro_tests.rs")
    echo -e "${BLUE}  → hydro_build_utils/tests/macro_tests.rs: $size lines${NC}"
fi
if [ -f "tests/workspace_integration.rs" ]; then
    size=$(wc -l < "tests/workspace_integration.rs")
    echo -e "${BLUE}  → tests/workspace_integration.rs: $size lines${NC}"
fi
echo ""

echo "10. Checking test module structure..."
echo "--------------------------------------"
echo "Checking for proper test modules..."
for file in hydro_std/tests/*.rs hydro_build_utils/tests/*.rs variadics/tests/variadic_operations.rs lattices/tests/lattice_properties.rs; do
    if [ -f "$file" ]; then
        if grep -q "#\[cfg(test)\]" "$file" || grep -q "#\[test\]" "$file"; then
            echo -e "${GREEN}✓${NC} $file contains tests"
            checks_passed=$((checks_passed + 1))
        else
            echo -e "${YELLOW}✗${NC} $file may not contain tests"
        fi
        checks_total=$((checks_total + 1))
    fi
done
echo ""

echo "11. Summary of test coverage..."
echo "--------------------------------"
total_test_files=$(find . -name "*.rs" -path "*/tests/*" -type f | wc -l)
echo -e "${BLUE}  → Total test files in workspace: $total_test_files${NC}"

new_test_files=7  # quorum, membership, request_response, variadic_operations, lattice_properties, macro_tests, workspace_integration
echo -e "${BLUE}  → New test files added: $new_test_files${NC}"

echo ""
echo "=================================="
echo "Verification Results"
echo "=================================="
echo -e "${GREEN}Checks passed: $checks_passed / $checks_total${NC}"

if [ $checks_passed -eq $checks_total ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Some checks did not pass${NC}"
    exit 1
fi
