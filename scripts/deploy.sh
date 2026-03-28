#!/bin/bash
# Hardened deployment script for Soroban smart contracts

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/common.sh
# shellcheck disable=SC1091
source "$SCRIPT_DIR/common.sh"

NETWORK=""
export DRY_RUN=false
CONTRACT=""
WASM_PATH=""
ROLLBACK_WASM=""
RETRIES=3
INITIAL_DELAY=2

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --network)
      NETWORK="$2"
      shift 2
      ;;
    --dry-run)
      export DRY_RUN=true
      shift
      ;;
    --contract)
      CONTRACT="$2"
      shift 2
      ;;
    --wasm)
      WASM_PATH="$2"
      shift 2
      ;;
    --rollback-wasm)
      ROLLBACK_WASM="$2"
      shift 2
      ;;
    --retries)
      RETRIES="$2"
      shift 2
      ;;
    --initial-delay)
      INITIAL_DELAY="$2"
      shift 2
      ;;
    *)
      print_usage
      exit 1
      ;;
  esac
done

if [[ -z "$NETWORK" || -z "$CONTRACT" || -z "$WASM_PATH" ]]; then
  print_usage
  exit 1
fi

load_env "$NETWORK"

# Example: Soroban CLI deploy command
DEPLOY_CMD=(soroban contract deploy --wasm "$WASM_PATH" --network "$NETWORK" --contract-name "$CONTRACT")
set +e
if ! retry_cmd "$RETRIES" "$INITIAL_DELAY" -- "${DEPLOY_CMD[@]}"; then
  echo "Deployment failed."
  if [[ -n "$ROLLBACK_WASM" ]]; then
    echo "Attempting rollback using WASM: $ROLLBACK_WASM"
    ROLLBACK_CMD=(soroban contract deploy --wasm "$ROLLBACK_WASM" --network "$NETWORK" --contract-name "$CONTRACT")
    if retry_cmd "$RETRIES" "$INITIAL_DELAY" -- "${ROLLBACK_CMD[@]}"; then
      echo "Rollback succeeded."
      exit 2
    else
      echo "Rollback failed."
      exit 3
    fi
  else
    echo "No rollback WASM provided. Exiting."
    exit 1
  fi
fi
set -e

echo "Deployment script completed."
