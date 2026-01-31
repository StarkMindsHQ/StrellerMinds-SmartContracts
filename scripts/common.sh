#!/bin/bash
# Common functions for deployment scripts

set -e

# Print usage for scripts
print_usage() {
  echo "Usage: $0 --network <local|testnet|mainnet> [--dry-run] --contract <name> --wasm <path>"
}

# Load environment variables for the selected network
load_env() {
  local network="$1"
  local env_file=".env.$network"
  if [ -f "$env_file" ]; then
    set -a
    source "$env_file"
    set +a
  else
    echo "Error: Environment file $env_file not found."
    exit 1
  fi
}

# Dry-run wrapper
run_or_dry() {
  if [ "$DRY_RUN" = true ]; then
    echo "[DRY-RUN] $@"
  else
    eval "$@"
  fi
}
