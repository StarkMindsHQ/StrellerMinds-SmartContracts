#!/bin/bash

# Enhanced Testnet Deployment Script for Stellar/Soroban Contracts
# Supports argument parsing, validation, dry-run, and safety features

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="$PROJECT_ROOT/target"
WASM_DIR="$TARGET_DIR/wasm32-unknown-unknown/release"

# Network-specific configuration
NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org"
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Default values
CONTRACT=""
DRY_RUN=false
PROFILE=""
VERBOSE=false

# Required tools and environment variables
REQUIRED_TOOLS=("soroban" "stellar" "jq")
REQUIRED_ENV_VARS=("STELLAR_SECRET_KEY")

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Print usage information
print_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Deploy Stellar/Soroban smart contracts to TESTNET with safety checks and validation.

OPTIONS:
    --contract CONTRACT     Deploy specific contract (optional, deploys all if not specified)
    --dry-run              Preview deployment without executing (show commands only)
    --profile PROFILE      AWS profile or config profile to use (optional)
    --verbose              Enable verbose output
    --help                 Show this help message

EXAMPLES:
    $(basename "$0") --dry-run
    $(basename "$0") --contract my_contract
    $(basename "$0") --contract my_contract --verbose --dry-run

ENVIRONMENT VARIABLES:
    STELLAR_SECRET_KEY     Your Stellar secret key for deployment (required)
    SOROBAN_RPC_URL       Custom RPC URL (optional, uses testnet default)

EOF
}

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --contract)
                CONTRACT="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --profile)
                PROFILE="$2"
                shift 2
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help)
                print_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done
}

# Check if required tools are installed
check_dependencies() {
    log_step "Checking required dependencies..."
    
    local missing_tools=()
    
    for tool in "${REQUIRED_TOOLS[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            missing_tools+=("$tool")
        fi
    done
    
    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_error "Please install the missing tools and try again"
        exit 1
    fi
    
    log_success "All required tools are installed"
}

# Validate environment variables
validate_environment() {
    log_step "Validating environment variables..."
    
    local missing_vars=()
    
    for var in "${REQUIRED_ENV_VARS[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            missing_vars+=("$var")
        fi
    done
    
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        log_error "Missing required environment variables: ${missing_vars[*]}"
        log_error "Please set the missing variables and try again"
        exit 1
    fi
    
    log_success "Environment variables validated"
}

# Check if WASM files exist
validate_wasm_files() {
    log_step "Checking for compiled WASM files..."
    
    if [[ ! -d "$WASM_DIR" ]]; then
        log_error "WASM directory not found: $WASM_DIR"
        log_error "Please run 'cargo build --release --target wasm32-unknown-unknown' first"
        exit 1
    fi
    
    local wasm_files=("$WASM_DIR"/*.optimized.wasm)
    
    if [[ ! -f "${wasm_files[0]}" ]]; then
        log_error "No optimized WASM files found in $WASM_DIR"
        log_error "Please build and optimize your contracts first"
        exit 1
    fi
    
    log_success "WASM files found and ready for deployment"
}

# Configure Soroban network
configure_network() {
    local rpc_url="$RPC_URL"
    
    if [[ -n "${SOROBAN_RPC_URL:-}" ]]; then
        rpc_url="$SOROBAN_RPC_URL"
        log_info "Using custom RPC URL: $rpc_url"
    fi
    
    log_step "Configuring Soroban network: $NETWORK"
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "[DRY-RUN] Would execute: soroban config network add --global $NETWORK --rpc-url $rpc_url --network-passphrase \"$NETWORK_PASSPHRASE\""
    else
        soroban config network add --global "$NETWORK" \
            --rpc-url "$rpc_url" \
            --network-passphrase "$NETWORK_PASSPHRASE"
        log_success "Network configured successfully"
    fi
}

# Deploy a single contract
deploy_contract() {
    local wasm_file="$1"
    local contract_name
    contract_name=$(basename "$wasm_file" .optimized.wasm)
    
    log_step "Deploying contract: $contract_name"
    
    if [[ "$VERBOSE" == true ]]; then
        log_info "WASM file: $wasm_file"
        log_info "Network: $NETWORK"
    fi
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "[DRY-RUN] Would execute: soroban contract deploy --wasm \"$wasm_file\" --source \"\$STELLAR_SECRET_KEY\" --network $NETWORK"
        log_info "[DRY-RUN] Would save contract ID to: target/$contract_name.$NETWORK.id"
    else
        local contract_id
        contract_id=$(soroban contract deploy \
            --wasm "$wasm_file" \
            --source "$STELLAR_SECRET_KEY" \
            --network "$NETWORK")
        
        if [[ -n "$contract_id" ]]; then
            log_success "Contract deployed: $contract_name"
            log_info "Contract ID: $contract_id"
            
            # Save contract ID to file
            echo "$contract_id" > "$TARGET_DIR/$contract_name.$NETWORK.id"
            log_info "Contract ID saved to: target/$contract_name.$NETWORK.id"
        else
            log_error "Failed to deploy contract: $contract_name"
            exit 1
        fi
    fi
}

# Deploy contracts
deploy_contracts() {
    log_step "Starting contract deployment to $NETWORK"
    
    # Create target directory if it doesn't exist
    mkdir -p "$TARGET_DIR"
    
    if [[ -n "$CONTRACT" ]]; then
        # Deploy specific contract
        local wasm_file="$WASM_DIR/$CONTRACT.optimized.wasm"
        if [[ ! -f "$wasm_file" ]]; then
            log_error "Contract WASM file not found: $wasm_file"
            exit 1
        fi
        deploy_contract "$wasm_file"
    else
        # Deploy all contracts
        local deployed_count=0
        for wasm_file in "$WASM_DIR"/*.optimized.wasm; do
            if [[ -f "$wasm_file" ]]; then
                deploy_contract "$wasm_file"
                ((deployed_count++))
            fi
        done
        
        if [[ $deployed_count -eq 0 ]]; then
            log_error "No WASM files found for deployment"
            exit 1
        fi
        
        log_success "Deployed $deployed_count contracts to $NETWORK"
    fi
}

# Main execution function
main() {
    echo "🚀 Testnet Deployment Script for Stellar/Soroban"
    echo "================================================"
    
    # Parse arguments
    parse_arguments "$@"
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "🔍 DRY-RUN MODE: Commands will be displayed but not executed"
    fi
    
    # Run all checks
    check_dependencies
    validate_environment
    validate_wasm_files
    
    # Configure network
    configure_network
    
    # Deploy contracts
    deploy_contracts
    
    # Success message
    echo
    if [[ "$DRY_RUN" == true ]]; then
        log_success "✅ Testnet dry-run completed successfully!"
        log_info "All checks passed. Run without --dry-run to execute deployment."
    else
        log_success "✅ Testnet deployment completed successfully!"
        log_info "All contracts have been deployed to testnet"
    fi
}

# Run main function with all arguments
main "$@"