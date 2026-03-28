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
    # shellcheck source=/dev/null
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
    echo "[DRY-RUN] $*"
  else
    "$@"
  fi
}

# Retry wrapper with exponential backoff
# Usage: retry_cmd <retries> <initial_delay_seconds> -- <cmd> [args...]
retry_cmd() {
  local retries="$1"; shift
  local delay="$1"; shift
  if [ "$1" != "--" ]; then
    echo "retry_cmd usage: retry_cmd <retries> <initial_delay_seconds> -- <cmd> [args...]"
    return 2
  fi
  shift
  local attempt=0
  local max_retries=$retries
  local current_delay=$delay
  while true; do
    if run_or_dry "$@"; then
      return 0
    fi
    attempt=$((attempt+1))
    if [ $attempt -gt $max_retries ]; then
      echo "Command failed after $max_retries retries: $*"
      return 1
    fi
    echo "Retry #$attempt in ${current_delay}s..."
    sleep "$current_delay"
    current_delay=$(( current_delay * 2 ))
    if [ $current_delay -gt 30 ]; then
      current_delay=30
    fi
  done
}
