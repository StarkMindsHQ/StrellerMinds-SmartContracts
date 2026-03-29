#!/bin/bash

# Pre-Release Validation Script
# Performs final validation checks before creating a release

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Pre-Release Validation Checklist     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo

# Check if version tag is provided
if [ -z "$1" ]; then
    echo -e "${YELLOW}Usage: $0 <version>${NC}"
    echo "Example: $0 v1.2.3"
    exit 1
fi

VERSION="$1"
echo -e "${BLUE}Validating release version: ${GREEN}$VERSION${NC}"
echo

VALIDATION_PASSED=true

# 1. Check version format
echo -e "${BLUE}[1/10] Checking version format...${NC}"
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
    echo -e "${RED}✗ Invalid version format. Should be vX.Y.Z or vX.Y.Z-prerelease+build${NC}"
    VALIDATION_PASSED=false
else
    echo -e "${GREEN}✓ Version format is valid${NC}"
fi
echo

# 2. Check if tag already exists
echo -e "${BLUE}[2/10] Checking for existing tags...${NC}"
if git rev-parse "$VERSION" >/dev/null 2>&1; then
    echo -e "${RED}✗ Tag '$VERSION' already exists${NC}"
    VALIDATION_PASSED=false
else
    echo -e "${GREEN}✓ Tag does not exist (good)${NC}"
fi
echo

# 3. Check current branch
echo -e "${BLUE}[3/10] Checking current branch...${NC}"
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "master" ]; then
    echo -e "${YELLOW}⚠ Not on main/master branch. Current branch: $CURRENT_BRANCH${NC}"
    echo -e "${YELLOW}  Consider switching to main branch before releasing${NC}"
else
    echo -e "${GREEN}✓ On release branch ($CURRENT_BRANCH)${NC}"
fi
echo

# 4. Check for uncommitted changes
echo -e "${BLUE}[4/10] Checking for uncommitted changes...${NC}"
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}✗ You have uncommitted changes${NC}"
    git status --short
    VALIDATION_PASSED=false
else
    echo -e "${GREEN}✓ No uncommitted changes${NC}"
fi
echo

# 5. Run tests
echo -e "${BLUE}[5/10] Running test suite...${NC}"
if [ -f "$SCRIPT_DIR/release-test.sh" ]; then
    if "$SCRIPT_DIR/release-test.sh" --quick; then
        echo -e "${GREEN}✓ Quick tests passed${NC}"
    else
        echo -e "${RED}✗ Quick tests failed${NC}"
        VALIDATION_PASSED=false
    fi
else
    echo -e "${YELLOW}⚠ Release test script not found, skipping${NC}"
fi
echo

# 6. Build all contracts
echo -e "${BLUE}[6/10] Building all contracts...${NC}"
if [ -f "$SCRIPT_DIR/build.sh" ]; then
    if "$SCRIPT_DIR/build.sh"; then
        echo -e "${GREEN}✓ All contracts built successfully${NC}"
    else
        echo -e "${RED}✗ Build failed${NC}"
        VALIDATION_PASSED=false
    fi
else
    echo -e "${YELLOW}⚠ Build script not found, attempting direct build...${NC}"
    if cargo build --workspace --target wasm32-unknown-unknown --release; then
        echo -e "${GREEN}✓ Direct build succeeded${NC}"
    else
        echo -e "${RED}✗ Direct build failed${NC}"
        VALIDATION_PASSED=false
    fi
fi
echo

# 7. Check CHANGELOG
echo -e "${BLUE}[7/10] Checking CHANGELOG...${NC}"
if [ -f "$PROJECT_ROOT/CHANGELOG.md" ]; then
    if grep -q "\[Unreleased\]" "$PROJECT_ROOT/CHANGELOG.md"; then
        echo -e "${GREEN}✓ CHANGELOG has Unreleased section${NC}"
    else
        echo -e "${YELLOW}⚠ CHANGELOG missing Unreleased section${NC}"
    fi
else
    echo -e "${YELLOW}⚠ CHANGELOG.md not found${NC}"
fi
echo

# 8. Check documentation
echo -e "${BLUE}[8/10] Checking documentation...${NC}"
MISSING_DOCS=0
for doc_file in README.md docs/*.md; do
    if [ ! -f "$doc_file" ]; then
        MISSING_DOCS=$((MISSING_DOCS + 1))
    fi
done

if [ $MISSING_DOCS -eq 0 ]; then
    echo -e "${GREEN}✓ Documentation files present${NC}"
else
    echo -e "${YELLOW}⚠ $MISSING_DOCS documentation files missing${NC}"
fi
echo

# 9. Security audit check
echo -e "${BLUE}[9/10] Running security checks...${NC}"
if command -v cargo-audit &> /dev/null; then
    if cargo audit --quiet 2>/dev/null; then
        echo -e "${GREEN}✓ No security vulnerabilities found${NC}"
    else
        echo -e "${RED}✗ Security vulnerabilities detected${NC}"
        echo -e "${YELLOW}  Run 'cargo audit' for details${NC}"
        VALIDATION_PASSED=false
    fi
else
    echo -e "${YELLOW}⚠ cargo-audit not installed, skipping${NC}"
fi
echo

# 10. Git history check
echo -e "${BLUE}[10/10] Checking recent commits...${NC}"
RECENT_COMMITS=$(git log --oneline -10)
echo "Recent commits:"
echo "$RECENT_COMMITS" | head -5
if echo "$RECENT_COMMITS" | grep -qi "WIP\|TODO\|FIXME"; then
    echo -e "${YELLOW}⚠ Found WIP/TODO/FIXME in recent commits${NC}"
else
    echo -e "${GREEN}✓ No obvious WIP commits${NC}"
fi
echo

# Summary
echo "╔════════════════════════════════════════╗"
if [ "$VALIDATION_PASSED" = true ]; then
    echo -e "${GREEN}║  ✓ Pre-Release Validation PASSED       ║${NC}"
    echo "╚════════════════════════════════════════╝"
    echo
    echo -e "${GREEN}Ready to create release: $VERSION${NC}"
    echo
    echo "Next steps:"
    echo "1. git tag -a $VERSION -m 'Release $VERSION'"
    echo "2. git push origin $VERSION"
    echo "3. GitHub Actions will create the release automatically"
    exit 0
else
    echo -e "${RED}║  ✗ Pre-Release Validation FAILED       ║${NC}"
    echo "╚════════════════════════════════════════╝"
    echo
    echo -e "${RED}Please fix the issues above before releasing${NC}"
    exit 1
fi
