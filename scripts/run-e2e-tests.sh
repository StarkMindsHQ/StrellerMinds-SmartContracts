#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
E2E_DIR="${PROJECT_ROOT}/e2e"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🧪 Running E2E Test Suite..."
echo ""

# Cleanup function
cleanup() {
  if [ "${SKIP_CLEANUP}" != "true" ]; then
    echo ""
    echo "${YELLOW}Cleaning up localnet...${NC}"
    "${SCRIPT_DIR}/localnet-down.sh"
  fi
}

trap cleanup EXIT

# Parse arguments
SKIP_BUILD=false
SKIP_CLEANUP=false
CI_MODE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --skip-build)
      SKIP_BUILD=true
      shift
      ;;
    --skip-cleanup)
      SKIP_CLEANUP=true
      shift
      ;;
    --ci)
      CI_MODE=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--skip-build] [--skip-cleanup] [--ci]"
      exit 1
      ;;
  esac
done

# Step 1: Build contracts if needed
if [ "${SKIP_BUILD}" != "true" ]; then
  echo "📦 Building contracts..."
  cd "${PROJECT_ROOT}"
  ./scripts/build.sh
  echo ""
fi

# Step 2: Start localnet
echo "🚀 Starting localnet..."
"${SCRIPT_DIR}/localnet-up.sh"
echo ""

# Step 3: Install E2E dependencies
echo "📥 Installing E2E test dependencies..."
cd "${E2E_DIR}"
if [ ! -d "node_modules" ]; then
  npm install
else
  echo "Dependencies already installed"
fi
echo ""

# Step 4: Deploy contracts
echo "🚢 Deploying contracts to localnet..."
npm run deploy:local
echo ""

# Step 5: Run tests
echo "🧪 Running E2E tests..."
if [ "${CI_MODE}" = "true" ]; then
  npm run test:ci
else
  npm test
fi

TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -eq 0 ]; then
  echo ""
  echo "${GREEN}✅ All E2E tests passed!${NC}"
else
  echo ""
  echo "${RED}❌ E2E tests failed${NC}"
  exit $TEST_EXIT_CODE
fi
