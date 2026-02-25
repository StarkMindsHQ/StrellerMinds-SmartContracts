#!/bin/bash
# Deployment rollback script for mobile-optimized contracts
# Reinstalls a previous WASM version to the network

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/common.sh
# shellcheck disable=SC1091
source "$SCRIPT_DIR/common.sh"

NETWORK=""
CONTRACT_ID=""
WASM_PATH=""
export DRY_RUN=false

print_rollback_usage() {
  echo "Usage: $0 --network <network> --contract-id <id> --wasm <path> [--dry-run]"
  echo ""
  echo "Rollback a deployment by installing a previous WASM version."
  echo ""
  echo "Options:"
  echo "  --network <network>      Target network (testnet, mainnet)"
  echo "  --contract-id <id>       Deployed contract ID"
  echo "  --wasm <path>            Path to previous WASM version to restore"
  echo "  --dry-run                Simulate rollback"
}

while [[ $# -gt 0 ]]; do
  case $1 in
    --network)      NETWORK="$2"; shift 2 ;;
    --contract-id)  CONTRACT_ID="$2"; shift 2 ;;
    --wasm)         WASM_PATH="$2"; shift 2 ;;
    --dry-run)      export DRY_RUN=true; shift ;;
    *)              print_rollback_usage; exit 1 ;;
  esac
done

if [[ -z "$NETWORK" || -z "$CONTRACT_ID" || -z "$WASM_PATH" ]]; then
  print_rollback_usage
  exit 1
fi

if [[ ! -f "$WASM_PATH" ]]; then
  echo "Error: WASM file not found: $WASM_PATH"
  exit 1
fi

load_env "$NETWORK"

WASM_SIZE=$(wc -c < "$WASM_PATH" | tr -d ' ')
echo "Rolling back contract $CONTRACT_ID on $NETWORK"
echo "  WASM file: $WASM_PATH"
echo "  WASM size: ${WASM_SIZE} bytes"

# Verify WASM checksum before rollback
if command -v sha256sum &> /dev/null; then
  CHECKSUM=$(sha256sum "$WASM_PATH" | awk '{print $1}')
elif command -v shasum &> /dev/null; then
  CHECKSUM=$(shasum -a 256 "$WASM_PATH" | awk '{print $1}')
else
  CHECKSUM="unavailable"
fi
echo "  SHA256: $CHECKSUM"

# Install previous WASM to the network
echo "Installing previous WASM version..."
INSTALL_CMD=(soroban contract install --wasm "$WASM_PATH" --network "$NETWORK")
run_or_dry "${INSTALL_CMD[@]}"

echo ""
echo "Rollback WASM installed to network."
echo "Trigger the contract upgrade via the contract's admin interface to complete rollback."
