#!/bin/bash

# StrellerMinds Smart Contracts - Data Migration Script
# Migrates data from v1 to v2 contracts

set -e

# Default values
NETWORK="testnet"
FROM_VERSION="v1"
TO_VERSION="v2"
BACKUP_DIR="backups"
VALIDATION_ONLY=false
SKIP_VALIDATION=false

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
StrellerMinds Data Migration Script

Usage: $0 [OPTIONS]

OPTIONS:
    --network <network>        Target network (local|testnet|mainnet) [default: testnet]
    --from-version <version>   Source version [default: v1]
    --to-version <version>     Target version [default: v2]
    --backup-dir <dir>         Backup directory [default: backups]
    --validation-only          Only run validation, don't migrate
    --skip-validation          Skip data validation (not recommended)
    --help                     Show this help message

EXAMPLES:
    $0 --network testnet
    $0 --network mainnet --validation-only
    $0 --backup-dir /path/to/backups

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --from-version)
            FROM_VERSION="$2"
            shift 2
            ;;
        --to-version)
            TO_VERSION="$2"
            shift 2
            ;;
        --backup-dir)
            BACKUP_DIR="$2"
            shift 2
            ;;
        --validation-only)
            VALIDATION_ONLY=true
            shift
            ;;
        --skip-validation)
            SKIP_VALIDATION=true
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

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Function to export contract state
export_contract_state() {
    local contract_name="$1"
    local contract_id="$2"
    local output_file="$BACKUP_DIR/${contract_name}_${FROM_VERSION}_$(date +%Y%m%d_%H%M%S).json"
    
    log "Exporting $contract_name contract state..."
    
    if soroban contract read \
        --id "$contract_id" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" \
        --output "$output_file"; then
        success "Exported $contract_name state to $output_file"
        echo "$output_file"
    else
        error "Failed to export $contract_name state"
        return 1
    fi
}

# Function to validate contract compatibility
validate_compatibility() {
    log "Validating contract compatibility between $FROM_VERSION and $TO_VERSION..."
    
    # Check if contracts exist
    local contracts=("analytics" "token" "shared")
    for contract in "${contracts[@]}"; do
        local v1_id_var="${contract^^}_CONTRACT_ID_V1"
        local v2_id_var="${contract^^}_CONTRACT_ID_V2"
        
        if [ -z "${!v1_id_var}" ] || [ -z "${!v2_id_var}" ]; then
            error "Contract IDs not set for $contract"
            error "Set ${v1_id_var} and ${v2_id_var} in environment"
            return 1
        fi
        
        # Verify contracts are accessible
        if ! soroban contract read \
            --id "${!v1_id_var}" \
            --rpc-url "$SOROBAN_RPC_URL" \
            --secret-key "$STELLAR_SECRET_KEY" >/dev/null 2>&1; then
            error "Cannot access v1 $contract contract at ${!v1_id_var}"
            return 1
        fi
        
        if ! soroban contract read \
            --id "${!v2_id_var}" \
            --rpc-url "$SOROBAN_RPC_URL" \
            --secret-key "$STELLAR_SECRET_KEY" >/dev/null 2>&1; then
            error "Cannot access v2 $contract contract at ${!v2_id_var}"
            return 1
        fi
    done
    
    success "Contract compatibility validation passed"
    return 0
}

# Function to migrate analytics data
migrate_analytics_data() {
    log "Migrating analytics contract data..."
    
    local v1_id="$ANALYTICS_CONTRACT_ID_V1"
    local v2_id="$ANALYTICS_CONTRACT_ID_V2"
    
    # Export current data
    local backup_file=$(export_contract_state "analytics" "$v1_id")
    
    # Read and transform data
    log "Transforming analytics data for v2..."
    
    # Create migration data structure
    cat > /tmp/analytics_migration.json << EOF
{
    "migration_info": {
        "from_version": "$FROM_VERSION",
        "to_version": "$TO_VERSION",
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "network": "$NETWORK"
    },
    "data": $(cat "$backup_file")
}
EOF
    
    # Execute migration
    log "Executing analytics data migration..."
    
    if soroban contract invoke \
        --id "$v2_id" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" \
        --wasm target/analytics.wasm \
        -- \
        migrate_data \
        --migration-data @/tmp/analytics_migration.json; then
        success "Analytics data migration completed"
    else
        error "Analytics data migration failed"
        return 1
    fi
    
    # Cleanup
    rm -f /tmp/analytics_migration.json
}

# Function to migrate token data
migrate_token_data() {
    log "Migrating token contract data..."
    
    local v1_id="$TOKEN_CONTRACT_ID_V1"
    local v2_id="$TOKEN_CONTRACT_ID_V2"
    
    # Export current data
    local backup_file=$(export_contract_state "token" "$v1_id")
    
    # Read and transform data
    log "Transforming token data for v2..."
    
    # Create migration data structure
    cat > /tmp/token_migration.json << EOF
{
    "migration_info": {
        "from_version": "$FROM_VERSION",
        "to_version": "$TO_VERSION",
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "network": "$NETWORK"
    },
    "data": $(cat "$backup_file")
}
EOF
    
    # Execute migration
    log "Executing token data migration..."
    
    if soroban contract invoke \
        --id "$v2_id" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" \
        --wasm target/token.wasm \
        -- \
        migrate_data \
        --migration-data @/tmp/token_migration.json; then
        success "Token data migration completed"
    else
        error "Token data migration failed"
        return 1
    fi
    
    # Cleanup
    rm -f /tmp/token_migration.json
}

# Function to migrate shared data
migrate_shared_data() {
    log "Migrating shared contract data..."
    
    local v1_id="$SHARED_CONTRACT_ID_V1"
    local v2_id="$SHARED_CONTRACT_ID_V2"
    
    # Export current data
    local backup_file=$(export_contract_state "shared" "$v1_id")
    
    # Read and transform data
    log "Transforming shared data for v2..."
    
    # Create migration data structure
    cat > /tmp/shared_migration.json << EOF
{
    "migration_info": {
        "from_version": "$FROM_VERSION",
        "to_version": "$TO_VERSION",
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "network": "$NETWORK"
    },
    "data": $(cat "$backup_file")
}
EOF
    
    # Execute migration
    log "Executing shared data migration..."
    
    if soroban contract invoke \
        --id "$v2_id" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" \
        --wasm target/shared.wasm \
        -- \
        migrate_data \
        --migration-data @/tmp/shared_migration.json; then
        success "Shared data migration completed"
    else
        error "Shared data migration failed"
        return 1
    fi
    
    # Cleanup
    rm -f /tmp/shared_migration.json
}

# Function to validate migration
validate_migration() {
    log "Validating migration results..."
    
    local contracts=("analytics" "token" "shared")
    local validation_errors=0
    
    for contract in "${contracts[@]}"; do
        local v2_id_var="${contract^^}_CONTRACT_ID_V2"
        local v2_id="${!v2_id_var}"
        
        log "Validating $contract contract..."
        
        # Check contract state
        if soroban contract invoke \
            --id "$v2_id" \
            --rpc-url "$SOROBAN_RPC_URL" \
            --secret-key "$STELLAR_SECRET_KEY" \
            --wasm "target/${contract}.wasm" \
            -- \
            validate_migration; then
            success "$contract validation passed"
        else
            error "$contract validation failed"
            ((validation_errors++))
        fi
    done
    
    if [ $validation_errors -eq 0 ]; then
        success "All contract validations passed"
        return 0
    else
        error "$validation_errors validation errors found"
        return 1
    fi
}

# Function to create migration report
create_migration_report() {
    local report_file="$BACKUP_DIR/migration_report_$(date +%Y%m%d_%H%M%S).json"
    
    log "Creating migration report..."
    
    cat > "$report_file" << EOF
{
    "migration": {
        "from_version": "$FROM_VERSION",
        "to_version": "$TO_VERSION",
        "network": "$NETWORK",
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "status": "completed",
        "validation_only": $VALIDATION_ONLY,
        "skip_validation": $SKIP_VALIDATION
    },
    "contracts": {
        "analytics": {
            "v1_contract_id": "$ANALYTICS_CONTRACT_ID_V1",
            "v2_contract_id": "$ANALYTICS_CONTRACT_ID_V2",
            "status": "migrated"
        },
        "token": {
            "v1_contract_id": "$TOKEN_CONTRACT_ID_V1",
            "v2_contract_id": "$TOKEN_CONTRACT_ID_V2",
            "status": "migrated"
        },
        "shared": {
            "v1_contract_id": "$SHARED_CONTRACT_ID_V1",
            "v2_contract_id": "$SHARED_CONTRACT_ID_V2",
            "status": "migrated"
        }
    },
    "backups": [
        $(find "$BACKUP_DIR" -name "*_${FROM_VERSION}_*.json" -type f | sed 's/^/"/;s/$/"/' | tr '\n' ',' | sed 's/,$//')
    ]
}
EOF
    
    success "Migration report created: $report_file"
}

# Main migration function
main() {
    log "Starting StrellerMinds data migration from $FROM_VERSION to $TO_VERSION"
    log "Network: $NETWORK"
    log "Backup directory: $BACKUP_DIR"
    
    if [ "$VALIDATION_ONLY" = true ]; then
        log "Running in validation-only mode"
    fi
    
    if [ "$SKIP_VALIDATION" = true ]; then
        warning "Skipping data validation (not recommended for production)"
    fi
    
    # Validate environment
    if ! validate_compatibility; then
        error "Environment validation failed"
        exit 1
    fi
    
    # Run validation only if requested
    if [ "$VALIDATION_ONLY" = true ]; then
        log "Validation-only mode: skipping actual migration"
        validate_migration
        create_migration_report
        success "Validation completed"
        exit 0
    fi
    
    # Perform migration
    log "Starting data migration..."
    
    # Migrate contracts in dependency order
    migrate_shared_data || exit 1
    migrate_token_data || exit 1
    migrate_analytics_data || exit 1
    
    # Validate migration unless skipped
    if [ "$SKIP_VALIDATION" = false ]; then
        validate_migration || exit 1
    fi
    
    # Create migration report
    create_migration_report
    
    success "Data migration completed successfully!"
    log "Please review the migration report and test the v2 contracts"
}

# Execute main function
main
