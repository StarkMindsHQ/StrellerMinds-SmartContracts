#!/bin/bash
# Mobile-optimized contract deployment script
# Supports WASM compression, offline bundle preparation, and bandwidth monitoring

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/common.sh
# shellcheck disable=SC1091
source "$SCRIPT_DIR/common.sh"

NETWORK=""
export DRY_RUN=false
CONTRACT=""
WASM_PATH=""
COMPRESS=true
OFFLINE_PREP=false
OUTPUT_DIR=""

print_mobile_usage() {
  echo "Usage: $0 --contract <name> --wasm <path> [options]"
  echo ""
  echo "Options:"
  echo "  --network <network>   Target network (testnet, mainnet, localnet)"
  echo "  --contract <name>     Contract name"
  echo "  --wasm <path>         Path to WASM file"
  echo "  --no-compress         Skip WASM compression"
  echo "  --offline-prep        Prepare offline deployment bundle"
  echo "  --output-dir <dir>    Output directory for offline bundle"
  echo "  --dry-run             Simulate deployment"
}

while [[ $# -gt 0 ]]; do
  case $1 in
    --network)      NETWORK="$2"; shift 2 ;;
    --contract)     CONTRACT="$2"; shift 2 ;;
    --wasm)         WASM_PATH="$2"; shift 2 ;;
    --no-compress)  COMPRESS=false; shift ;;
    --offline-prep) OFFLINE_PREP=true; shift ;;
    --output-dir)   OUTPUT_DIR="$2"; shift 2 ;;
    --dry-run)      export DRY_RUN=true; shift ;;
    *)              print_mobile_usage; exit 1 ;;
  esac
done

if [[ -z "$CONTRACT" || -z "$WASM_PATH" ]]; then
  print_mobile_usage
  exit 1
fi

if [[ ! -f "$WASM_PATH" ]]; then
  echo "Error: WASM file not found: $WASM_PATH"
  exit 1
fi

# Report original file size
ORIGINAL_SIZE=$(wc -c < "$WASM_PATH" | tr -d ' ')
echo "Original WASM size: ${ORIGINAL_SIZE} bytes"

DEPLOY_WASM="$WASM_PATH"

# Compress WASM for mobile deployment
if [[ "$COMPRESS" == "true" ]]; then
  OPTIMIZED_WASM="${WASM_PATH%.wasm}.optimized.wasm"

  if command -v soroban &> /dev/null; then
    echo "Compressing WASM with soroban optimize..."
    soroban contract optimize \
      --wasm "$WASM_PATH" \
      --wasm-out "$OPTIMIZED_WASM" 2>/dev/null || true
  fi

  if [[ ! -f "$OPTIMIZED_WASM" ]] && command -v wasm-opt &> /dev/null; then
    echo "Compressing WASM with wasm-opt..."
    wasm-opt -Oz "$WASM_PATH" -o "$OPTIMIZED_WASM"
  fi

  if [[ -f "$OPTIMIZED_WASM" ]]; then
    COMPRESSED_SIZE=$(wc -c < "$OPTIMIZED_WASM" | tr -d ' ')
    SAVINGS=$((ORIGINAL_SIZE - COMPRESSED_SIZE))
    echo "Compressed WASM size: ${COMPRESSED_SIZE} bytes (saved ${SAVINGS} bytes)"
    DEPLOY_WASM="$OPTIMIZED_WASM"
  else
    echo "Warning: Compression not available, deploying original WASM"
  fi
fi

# Bandwidth estimation
DEPLOY_SIZE=$(wc -c < "$DEPLOY_WASM" | tr -d ' ')
echo "Deployment payload: ${DEPLOY_SIZE} bytes"
echo "Estimated bandwidth: ~$((DEPLOY_SIZE * 2)) bytes (with protocol overhead)"

# Generate checksum
if command -v sha256sum &> /dev/null; then
  CHECKSUM=$(sha256sum "$DEPLOY_WASM" | awk '{print $1}')
elif command -v shasum &> /dev/null; then
  CHECKSUM=$(shasum -a 256 "$DEPLOY_WASM" | awk '{print $1}')
else
  CHECKSUM="unavailable"
fi
echo "WASM SHA256: $CHECKSUM"

# Prepare offline bundle if requested
if [[ "$OFFLINE_PREP" == "true" ]]; then
  BUNDLE_DIR="${OUTPUT_DIR:-./deployment-bundle}"
  mkdir -p "$BUNDLE_DIR"
  cp "$DEPLOY_WASM" "$BUNDLE_DIR/${CONTRACT}.wasm"

  cat > "$BUNDLE_DIR/manifest.json" <<EOF
{
  "contract": "$CONTRACT",
  "network": "${NETWORK:-pending}",
  "wasm_file": "${CONTRACT}.wasm",
  "original_size": $ORIGINAL_SIZE,
  "deploy_size": $DEPLOY_SIZE,
  "sha256": "$CHECKSUM",
  "compressed": $COMPRESS,
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo "unknown")"
}
EOF

  if [[ "$CHECKSUM" != "unavailable" ]]; then
    echo "$CHECKSUM  ${CONTRACT}.wasm" > "$BUNDLE_DIR/checksum.sha256"
  fi

  echo "Offline deployment bundle created: $BUNDLE_DIR"
  echo "Transfer this bundle and deploy with:"
  echo "  $0 --network <network> --contract $CONTRACT --wasm $BUNDLE_DIR/${CONTRACT}.wasm"
  exit 0
fi

# Require network for live deployment
if [[ -z "$NETWORK" ]]; then
  echo "Error: --network required for deployment (or use --offline-prep)"
  print_mobile_usage
  exit 1
fi

load_env "$NETWORK"

echo "Deploying $CONTRACT to $NETWORK..."
DEPLOY_CMD=(soroban contract deploy \
  --wasm "$DEPLOY_WASM" \
  --network "$NETWORK" \
  --contract-name "$CONTRACT")
run_or_dry "${DEPLOY_CMD[@]}"

echo "Deployment complete."
echo "  Contract:  $CONTRACT"
echo "  Network:   $NETWORK"
echo "  Size:      ${DEPLOY_SIZE} bytes"
echo "  Checksum:  $CHECKSUM"
