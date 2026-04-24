#!/bin/bash

# StrellerMinds Smart Contracts - Export Contract State Script
# Exports the current state of contracts for backup/migration

set -e

# Default values
NETWORK="testnet"
CONTRACT_ID=""
OUTPUT_FILE=""
FORMAT="json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
StrellerMinds Contract State Export Script

Usage: $0 [OPTIONS]

OPTIONS:
    --network <network>        Target network (local|testnet|mainnet) [default: testnet]
    --contract-id <id>         Contract ID to export (required)
    --output-file <file>       Output file path (required)
    --format <format>          Output format (json|binary) [default: json]
    --help                     Show this help message

EXAMPLES:
    $0 --network testnet --contract-id ABC123... --output-file backup.json
    $0 --network mainnet --contract-id DEF456... --output-file backup.bin --format binary

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --contract-id)
            CONTRACT_ID="$2"
            shift 2
            ;;
        --output-file)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --format)
            FORMAT="$2"
            shift 2
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate arguments
if [ -z "$CONTRACT_ID" ]; then
    error "Contract ID is required"
    show_help
    exit 1
fi

if [ -z "$OUTPUT_FILE" ]; then
    error "Output file is required"
    show_help
    exit 1
fi

# Validate network
case $NETWORK in
    local|testnet|mainnet)
        ;;
    *)
        error "Invalid network: $NETWORK. Must be local, testnet, or mainnet"
        exit 1
        ;;
esac

# Validate format
case $FORMAT in
    json|binary)
        ;;
    *)
        error "Invalid format: $FORMAT. Must be json or binary"
        exit 1
        ;;
esac

# Load environment variables
if [ -f ".env.$NETWORK" ]; then
    source ".env.$NETWORK"
    log "Loaded environment variables from .env.$NETWORK"
else
    error "Environment file .env.$NETWORK not found"
    exit 1
fi

# Check required environment variables
required_vars=("STELLAR_SECRET_KEY" "SOROBAN_RPC_URL")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        error "Environment variable $var is not set"
        exit 1
    fi
done

# Create output directory if needed
output_dir=$(dirname "$OUTPUT_FILE")
mkdir -p "$output_dir"

# Function to export contract state
export_contract_state() {
    log "Exporting contract state..."
    log "Contract ID: $CONTRACT_ID"
    log "Network: $NETWORK"
    log "Output file: $OUTPUT_FILE"
    log "Format: $FORMAT"
    
    # Verify contract exists
    log "Verifying contract accessibility..."
    if ! soroban contract read \
        --id "$CONTRACT_ID" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" >/dev/null 2>&1; then
        error "Contract is not accessible or does not exist"
        exit 1
    fi
    
    # Export contract state
    log "Exporting contract state..."
    
    case $FORMAT in
        json)
            if soroban contract read \
                --id "$CONTRACT_ID" \
                --rpc-url "$SOROBAN_RPC_URL" \
                --secret-key "$STELLAR_SECRET_KEY" \
                --output "$OUTPUT_FILE"; then
                success "Contract state exported successfully to $OUTPUT_FILE"
            else
                error "Failed to export contract state"
                exit 1
            fi
            ;;
        binary)
            if soroban contract read \
                --id "$CONTRACT_ID" \
                --rpc-url "$SOROBAN_RPC_URL" \
                --secret-key "$STELLAR_SECRET_KEY" \
                --output "$OUTPUT_FILE" \
                --binary; then
                success "Contract state exported successfully to $OUTPUT_FILE"
            else
                error "Failed to export contract state"
                exit 1
            fi
            ;;
    esac
    
    # Verify export file
    if [ -f "$OUTPUT_FILE" ]; then
        local file_size=$(stat -f%z "$OUTPUT_FILE" 2>/dev/null || stat -c%s "$OUTPUT_FILE" 2>/dev/null || echo "0")
        if [ "$file_size" -gt 0 ]; then
            success "Export file created successfully ($file_size bytes)"
        else
            error "Export file is empty"
            exit 1
        fi
    else
        error "Export file was not created"
        exit 1
    fi
}

# Function to create export metadata
create_export_metadata() {
    local metadata_file="${OUTPUT_FILE}.meta.json"
    
    log "Creating export metadata..."
    
    cat > "$metadata_file" << EOF
{
    "export": {
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "network": "$NETWORK",
        "contract_id": "$CONTRACT_ID",
        "format": "$FORMAT",
        "file_size": $(stat -f%z "$OUTPUT_FILE" 2>/dev/null || stat -c%s "$OUTPUT_FILE" 2>/dev/null || echo "0"),
        "checksum": "$(sha256sum "$OUTPUT_FILE" | cut -d' ' -f1)"
    },
    "environment": {
        "soroban_rpc_url": "$SOROBAN_RPC_URL",
        "stellar_network": "$STELLAR_NETWORK_PASSPHRASE"
    }
}
EOF
    
    success "Export metadata created: $metadata_file"
}

# Main export function
main() {
    log "Starting contract state export"
    
    # Export contract state
    export_contract_state
    
    # Create metadata
    create_export_metadata
    
    success "Contract state export completed successfully!"
    log "Files created:"
    log "  - State data: $OUTPUT_FILE"
    log "  - Metadata: ${OUTPUT_FILE}.meta.json"
}

# Execute main function
main
