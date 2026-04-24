#!/bin/bash

# StrellerMinds Smart Contracts - Rollback to v1 Script
# Rolls back from v2 to v1 contracts

set -e

# Default values
NETWORK="testnet"
BACKUP_DIR="backups"
FORCE_ROLLBACK=false
SKIP_BACKUP=false
DRY_RUN=false

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
StrellerMinds Rollback to v1 Script

Usage: $0 [OPTIONS]

OPTIONS:
    --network <network>        Target network (local|testnet|mainnet) [default: testnet]
    --backup-dir <dir>         Backup directory [default: backups]
    --force-rollback          Force rollback without additional confirmation
    --skip-backup            Skip creating backup before rollback (not recommended)
    --dry-run                 Simulate rollback without executing
    --help                     Show this help message

EXAMPLES:
    $0 --network testnet
    $0 --network mainnet --dry-run
    $0 --force-rollback --skip-backup

WARNING: This script will rollback to v1 contracts. Use with caution!
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --backup-dir)
            BACKUP_DIR="$2"
            shift 2
            ;;
        --force-rollback)
            FORCE_ROLLBACK=true
            shift
            ;;
        --skip-backup)
            SKIP_BACKUP=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
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

# Validate network
case $NETWORK in
    local|testnet|mainnet)
        ;;
    *)
        error "Invalid network: $NETWORK. Must be local, testnet, or mainnet"
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

# Confirmation prompt unless forced
if [ "$FORCE_ROLLBACK" = false ] && [ "$DRY_RUN" = false ]; then
    echo
    warning "WARNING: This will rollback all contracts from v2 to v1"
    warning "This action cannot be undone easily and may result in data loss"
    echo
    read -p "Are you sure you want to continue? (type 'rollback' to confirm): " confirmation
    
    if [ "$confirmation" != "rollback" ]; then
        log "Rollback cancelled by user"
        exit 0
    fi
fi

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Function to create emergency backup
create_emergency_backup() {
    if [ "$SKIP_BACKUP" = true ]; then
        warning "Skipping backup creation as requested"
        return 0
    fi
    
    log "Creating emergency backup of current v2 state..."
    
    local backup_file="$BACKUP_DIR/emergency_backup_v2_$(date +%Y%m%d_%H%M%S).tar.gz"
    local temp_dir="/tmp/streller_backup_$$"
    
    mkdir -p "$temp_dir"
    
    # Export current contract states
    local contracts=("analytics" "token" "shared")
    for contract in "${contracts[@]}"; do
        local v2_id_var="${contract^^}_CONTRACT_ID_V2"
        local v2_id="${!v2_id_var}"
        
        if [ -n "$v2_id" ]; then
            log "Backing up $contract contract..."
            soroban contract read \
                --id "$v2_id" \
                --rpc-url "$SOROBAN_RPC_URL" \
                --secret-key "$STELLAR_SECRET_KEY" \
                --output "$temp_dir/${contract}_v2_state.json" || {
                warning "Failed to backup $contract contract"
            }
        fi
    done
    
    # Create backup archive
    tar -czf "$backup_file" -C "$temp_dir" .
    rm -rf "$temp_dir"
    
    success "Emergency backup created: $backup_file"
    echo "$backup_file"
}

# Function to check v1 contract availability
check_v1_availability() {
    log "Checking v1 contract availability..."
    
    local contracts=("analytics" "token" "shared")
    local missing_contracts=()
    
    for contract in "${contracts[@]}"; do
        local v1_id_var="${contract^^}_CONTRACT_ID_V1"
        local v1_id="${!v1_id_var}"
        
        if [ -z "$v1_id" ]; then
            missing_contracts+=("$contract")
            continue
        fi
        
        if ! soroban contract read \
            --id "$v1_id" \
            --rpc-url "$SOROBAN_RPC_URL" \
            --secret-key "$STELLAR_SECRET_KEY" >/dev/null 2>&1; then
            missing_contracts+=("$contract")
        fi
    done
    
    if [ ${#missing_contracts[@]} -gt 0 ]; then
        error "Missing v1 contracts: ${missing_contracts[*]}"
        error "Cannot proceed with rollback"
        return 1
    fi
    
    success "All v1 contracts are available"
    return 0
}

# Function to pause v2 contracts
pause_v2_contracts() {
    log "Pausing v2 contracts..."
    
    local contracts=("analytics" "token" "shared")
    
    for contract in "${contracts[@]}"; do
        local v2_id_var="${contract^^}_CONTRACT_ID_V2"
        local v2_id="${!v2_id_var}"
        
        if [ -n "$v2_id" ]; then
            log "Pausing $contract v2 contract..."
            
            if [ "$DRY_RUN" = true ]; then
                log "[DRY RUN] Would pause $contract v2 contract"
            else
                soroban contract invoke \
                    --id "$v2_id" \
                    --rpc-url "$SOROBAN_RPC_URL" \
                    --secret-key "$STELLAR_SECRET_KEY" \
                    --wasm "target/${contract}.wasm" \
                    -- \
                    pause || {
                    warning "Failed to pause $contract v2 contract"
                }
            fi
        fi
    done
}

# Function to update proxy contracts
update_proxy_contracts() {
    log "Updating proxy contracts to point to v1..."
    
    local contracts=("analytics" "token" "shared")
    
    for contract in "${contracts[@]}"; do
        local proxy_id_var="${contract^^}_PROXY_CONTRACT_ID"
        local v1_id_var="${contract^^}_CONTRACT_ID_V1"
        local proxy_id="${!proxy_id_var}"
        local v1_id="${!v1_id_var}"
        
        if [ -n "$proxy_id" ] && [ -n "$v1_id" ]; then
            log "Updating $contract proxy contract..."
            
            if [ "$DRY_RUN" = true ]; then
                log "[DRY RUN] Would update $contract proxy to point to v1: $v1_id"
            else
                soroban contract invoke \
                    --id "$proxy_id" \
                    --rpc-url "$SOROBAN_RPC_URL" \
                    --secret-key "$STELLAR_SECRET_KEY" \
                    --wasm "target/proxy.wasm" \
                    -- \
                    upgrade \
                    --new-implementation "$v1_id" || {
                    error "Failed to update $contract proxy contract"
                    return 1
                }
            fi
        else
            warning "Proxy or v1 contract ID not set for $contract"
        fi
    done
    
    success "Proxy contracts updated"
}

# Function to restore v1 data if needed
restore_v1_data() {
    log "Checking if v1 data restoration is needed..."
    
    # Find the most recent v1 backup
    local latest_backup=$(find "$BACKUP_DIR" -name "*_v1_*.json" -type f | sort -r | head -1)
    
    if [ -n "$latest_backup" ]; then
        log "Found v1 backup: $latest_backup"
        
        read -p "Do you want to restore data from this backup? (y/N): " restore_choice
        
        if [ "$restore_choice" = "y" ] || [ "$restore_choice" = "Y" ]; then
            log "Restoring v1 data from backup..."
            
            local contracts=("analytics" "token" "shared")
            for contract in "${contracts[@]}"; do
                local v1_id_var="${contract^^}_CONTRACT_ID_V1"
                local v1_id="${!v1_id_var}"
                
                if [ -n "$v1_id" ]; then
                    if [ "$DRY_RUN" = true ]; then
                        log "[DRY RUN] Would restore $contract data from backup"
                    else
                        soroban contract invoke \
                            --id "$v1_id" \
                            --rpc-url "$SOROBAN_RPC_URL" \
                            --secret-key "$STELLAR_SECRET_KEY" \
                            --wasm "target/${contract}.wasm" \
                            -- \
                            restore_data \
                            --backup-file "$latest_backup" || {
                            warning "Failed to restore $contract data"
                        }
                    fi
                fi
            done
        else
            log "Skipping data restoration"
        fi
    else
        log "No v1 backup found, skipping data restoration"
    fi
}

# Function to verify rollback
verify_rollback() {
    log "Verifying rollback..."
    
    local contracts=("analytics" "token" "shared")
    local verification_errors=0
    
    for contract in "${contracts[@]}"; do
        local v1_id_var="${contract^^}_CONTRACT_ID_V1"
        local v1_id="${!v1_id_var}"
        
        if [ -n "$v1_id" ]; then
            log "Verifying $contract v1 contract..."
            
            if soroban contract invoke \
                --id "$v1_id" \
                --rpc-url "$SOROBAN_RPC_URL" \
                --secret-key "$STELLAR_SECRET_KEY" \
                --wasm "target/${contract}.wasm" \
                -- \
                health_check 2>/dev/null; then
                success "$contract v1 contract is healthy"
            else
                error "$contract v1 contract health check failed"
                ((verification_errors++))
            fi
        fi
    done
    
    if [ $verification_errors -eq 0 ]; then
        success "Rollback verification passed"
        return 0
    else
        error "Rollback verification failed with $verification_errors errors"
        return 1
    fi
}

# Function to create rollback report
create_rollback_report() {
    local report_file="$BACKUP_DIR/rollback_report_$(date +%Y%m%d_%H%M%S).json"
    
    log "Creating rollback report..."
    
    cat > "$report_file" << EOF
{
    "rollback": {
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "network": "$NETWORK",
        "from_version": "v2",
        "to_version": "v1",
        "dry_run": $DRY_RUN,
        "force_rollback": $FORCE_ROLLBACK,
        "skip_backup": $SKIP_BACKUP
    },
    "contracts": {
        "analytics": {
            "v1_contract_id": "$ANALYTICS_CONTRACT_ID_V1",
            "v2_contract_id": "$ANALYTICS_CONTRACT_ID_V2",
            "status": "rolled_back"
        },
        "token": {
            "v1_contract_id": "$TOKEN_CONTRACT_ID_V1",
            "v2_contract_id": "$TOKEN_CONTRACT_ID_V2",
            "status": "rolled_back"
        },
        "shared": {
            "v1_contract_id": "$SHARED_CONTRACT_ID_V1",
            "v2_contract_id": "$SHARED_CONTRACT_ID_V2",
            "status": "rolled_back"
        }
    },
    "backup_created": $([ "$SKIP_BACKUP" = false ] && echo "true" || echo "false")
}
EOF
    
    success "Rollback report created: $report_file"
}

# Main rollback function
main() {
    log "Starting StrellerMinds rollback from v2 to v1"
    log "Network: $NETWORK"
    log "Backup directory: $BACKUP_DIR"
    
    if [ "$DRY_RUN" = true ]; then
        log "Running in dry-run mode - no changes will be made"
    fi
    
    # Create emergency backup
    local backup_file=""
    if [ "$SKIP_BACKUP" = false ]; then
        backup_file=$(create_emergency_backup)
    fi
    
    # Check v1 availability
    if ! check_v1_availability; then
        error "Cannot proceed with rollback"
        exit 1
    fi
    
    # Pause v2 contracts
    pause_v2_contracts
    
    # Update proxy contracts
    update_proxy_contracts
    
    # Restore v1 data if needed
    restore_v1_data
    
    # Verify rollback
    if [ "$DRY_RUN" = false ]; then
        if ! verify_rollback; then
            error "Rollback verification failed"
            error "Please check the system and try again"
            exit 1
        fi
    fi
    
    # Create rollback report
    create_rollback_report
    
    success "Rollback completed successfully!"
    
    if [ "$DRY_RUN" = false ]; then
        log "System is now running v1 contracts"
        if [ -n "$backup_file" ]; then
            log "Emergency backup saved to: $backup_file"
        fi
        log "Please test all functionality to ensure proper operation"
    else
        log "Dry-run completed. No actual changes were made."
    fi
}

# Execute main function
main
