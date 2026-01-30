#!/bin/bash

# Comprehensive CI validation script for StrellerMinds Smart Contracts
# Validates format, builds, and tests

set -e

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

FAILED=0

echo -e "${BLUE}=== StrellerMinds Smart Contracts CI Validation ===${NC}"
echo ""

# Step 1: Format Check
echo -e "${YELLOW}[1/5] Checking code formatting...${NC}"
if cargo fmt --all -- --check; then
    echo -e "${GREEN}✓ Format check passed${NC}"
else
    echo -e "${RED}✗ Format check failed - run 'cargo fmt --all'${NC}"
    FAILED=$((FAILED + 1))
fi
echo ""

# Step 2: Build shared library (tests)
echo -e "${YELLOW}[2/5] Building shared library with tests...${NC}"
if cargo build --lib --all-features 2>&1 | grep -v "warning:"; then
    echo -e "${GREEN}✓ Shared library build successful${NC}"
else
    echo -e "${RED}✗ Shared library build failed${NC}"
    FAILED=$((FAILED + 1))
fi
echo ""

# Step 3: Run library tests
echo -e "${YELLOW}[3/5] Running library tests...${NC}"
if cargo test --lib 2>&1; then
    echo -e "${GREEN}✓ Library tests passed${NC}"
else
    echo -e "${RED}✗ Library tests failed${NC}"
    FAILED=$((FAILED + 1))
fi
echo ""

# Step 4: Build WASM contracts
echo -e "${YELLOW}[4/5] Building WASM contracts (release)...${NC}"
cd contracts
for dir in */; do
    if [ -f "$dir/Cargo.toml" ]; then
        contract_name="${dir%/}"
        echo "  Building: $contract_name..."
        if cargo build --target wasm32-unknown-unknown --release --manifest-path "$dir/Cargo.toml" 2>&1 | grep -v "warning:"; then
            echo -e "  ${GREEN}✓ $contract_name compiled${NC}"
        else
            echo -e "  ${RED}✗ $contract_name failed${NC}"
            FAILED=$((FAILED + 1))
        fi
    fi
done
cd ..
echo ""

# Step 5: Documentation check
echo -e "${YELLOW}[5/5] Generating documentation...${NC}"
if cargo doc --no-deps --document-private-items 2>&1 | grep -v "warning:"; then
    echo -e "${GREEN}✓ Documentation generated successfully${NC}"
else
    echo -e "${RED}✗ Documentation generation failed${NC}"
    FAILED=$((FAILED + 1))
fi
echo ""

# Summary
echo -e "${BLUE}=== Validation Summary ===${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All checks passed! ✓${NC}"
    exit 0
else
    echo -e "${RED}$FAILED check(s) failed ✗${NC}"
    exit 1
fi
