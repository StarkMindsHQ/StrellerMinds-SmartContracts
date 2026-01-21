#!/bin/bash
set -e

# Test help message
output=$(bash scripts/build.sh --help)
echo "$output" | grep -q "Usage: bash scripts/build.sh" && echo "Help message test: PASSED" || echo "Help message test: FAILED"

# Test build all contracts (simulate tools present)
if command -v cargo &> /dev/null && command -v soroban &> /dev/null; then
  bash scripts/build.sh || true
  [ -f build.log ] && echo "Log file creation test: PASSED" || echo "Log file creation test: FAILED"
else
  echo "Skipping build all contracts test: Required tools not installed."
fi

# Test build specific contract (simulate tools present)
if [ -d contracts/analytics ]; then
  if command -v cargo &> /dev/null && command -v soroban &> /dev/null; then
    bash scripts/build.sh analytics || true
    [ -f build.log ] && echo "Specific contract build log test: PASSED" || echo "Specific contract build log test: FAILED"
  else
    echo "Skipping specific contract build test: Required tools not installed."
  fi
else
  echo "Skipping specific contract build test: contracts/analytics not found."
fi

# Test error handling for missing cargo (simulate by unsetting PATH)
PATH_ORIG="$PATH"
export PATH=""
if bash scripts/build.sh 2>&1 | grep -q "Error: 'cargo' not found"; then
  echo "Missing cargo error test: PASSED"
else
  echo "Missing cargo error test: FAILED"
fi
export PATH="$PATH_ORIG"

# Clean up
rm -f build.log
