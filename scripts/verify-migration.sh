#!/bin/bash

# StrellerMinds Smart Contracts - Migration Verification Script
# Verifies that migration from v1 to v2 was successful

set -e

# Default values
NETWORK="testnet"
BACKUP_DIR="backups"
DETAILED_REPORT=false
QUICK_CHECK=false

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
StrellerMinds Migration Verification Script

Usage: $0 [OPTIONS]

OPTIONS:
    --network <network>        Target network (local|testnet|mainnet) [default: testnet]
    --backup-dir <dir>         Backup directory [default: backups]
    --detailed-report          Generate detailed verification report
    --quick-check             Perform quick verification only
    --help                     Show this help message

EXAMPLES:
    $0 --network testnet
    $0 --network mainnet --detailed-report
    $0 --quick-check

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
        --detailed-report)
            DETAILED_REPORT=true
            shift
            ;;
        --quick-check)
            QUICK_CHECK=true
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

# Verification results
VERIFICATION_RESULTS=()
TOTAL_CHECKS=0
PASSED_CHECKS=0

# Function to record verification result
record_result() {
    local test_name="$1"
    local result="$2"
    local details="$3"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if [ "$result" = "PASS" ]; then
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        success "$test_name: $result"
    else
        error "$test_name: $result"
    fi
    
    VERIFICATION_RESULTS+=("{\"test\": \"$test_name\", \"result\": \"$result\", \"details\": \"$details\"}")
}

# Function to check contract accessibility
check_contract_accessibility() {
    local contract_name="$1"
    local contract_id="$2"
    
    log "Checking $contract_name contract accessibility..."
    
    if soroban contract read \
        --id "$contract_id" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" >/dev/null 2>&1; then
        record_result "Contract Accessibility - $contract_name" "PASS" "Contract is accessible"
        return 0
    else
        record_result "Contract Accessibility - $contract_name" "FAIL" "Contract is not accessible"
        return 1
    fi
}

# Function to check contract version
check_contract_version() {
    local contract_name="$1"
    local contract_id="$2"
    local expected_version="$3"
    
    log "Checking $contract_name contract version..."
    
    local version=$(soroban contract invoke \
        --id "$contract_id" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" \
        --wasm "target/${contract_name}.wasm" \
        -- \
        get_version 2>/dev/null || echo "unknown")
    
    if [ "$version" = "$expected_version" ]; then
        record_result "Contract Version - $contract_name" "PASS" "Version: $version"
        return 0
    else
        record_result "Contract Version - $contract_name" "FAIL" "Expected: $expected_version, Got: $version"
        return 1
    fi
}

# Function to check data integrity
check_data_integrity() {
    local contract_name="$1"
    local v1_id="$2"
    local v2_id="$3"
    
    log "Checking $contract_name data integrity..."
    
    # Get data counts from both contracts
    local v1_count=$(soroban contract invoke \
        --id "$v1_id" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" \
        --wasm "target/${contract_name}.wasm" \
        -- \
        get_data_count 2>/dev/null || echo "0")
    
    local v2_count=$(soroban contract invoke \
        --id "$v2_id" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" \
        --wasm "target/${contract_name}.wasm" \
        -- \
        get_data_count 2>/dev/null || echo "0")
    
    if [ "$v1_count" = "$v2_count" ] && [ "$v1_count" != "0" ]; then
        record_result "Data Integrity - $contract_name" "PASS" "V1: $v1_count, V2: $v2_count"
        return 0
    else
        record_result "Data Integrity - $contract_name" "FAIL" "V1: $v1_count, V2: $v2_count"
        return 1
    fi
}

# Function to check contract functionality
check_contract_functionality() {
    local contract_name="$1"
    local contract_id="$2"
    
    log "Checking $contract_name contract functionality..."
    
    # Test basic functionality
    case "$contract_name" in
        "analytics")
            # Test analytics specific functionality
            local result=$(soroban contract invoke \
                --id "$contract_id" \
                --rpc-url "$SOROBAN_RPC_URL" \
                --secret-key "$STELLAR_SECRET_KEY" \
                --wasm "target/analytics.wasm" \
                -- \
                health_check 2>/dev/null || echo "fail")
            ;;
        "token")
            # Test token specific functionality
            local result=$(soroban contract invoke \
                --id "$contract_id" \
                --rpc-url "$SOROBAN_RPC_URL" \
                --secret-key "$STELLAR_SECRET_KEY" \
                --wasm "target/token.wasm" \
                -- \
                health_check 2>/dev/null || echo "fail")
            ;;
        "shared")
            # Test shared specific functionality
            local result=$(soroban contract invoke \
                --id "$contract_id" \
                --rpc-url "$SOROBAN_RPC_URL" \
                --secret-key "$STELLAR_SECRET_KEY" \
                --wasm "target/shared.wasm" \
                -- \
                health_check 2>/dev/null || echo "fail")
            ;;
        *)
            local result="fail"
            ;;
    esac
    
    if [ "$result" = "ok" ]; then
        record_result "Contract Functionality - $contract_name" "PASS" "Basic functionality test passed"
        return 0
    else
        record_result "Contract Functionality - $contract_name" "FAIL" "Basic functionality test failed"
        return 1
    fi
}

# Function to check migration metadata
check_migration_metadata() {
    local contract_name="$1"
    local contract_id="$2"
    
    log "Checking $contract_name migration metadata..."
    
    local migration_info=$(soroban contract invoke \
        --id "$contract_id" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --secret-key "$STELLAR_SECRET_KEY" \
        --wasm "target/${contract_name}.wasm" \
        -- \
        get_migration_info 2>/dev/null || echo "{}")
    
    # Check if migration info exists and is valid
    if echo "$migration_info" | grep -q "migration_timestamp"; then
        record_result "Migration Metadata - $contract_name" "PASS" "Migration metadata found"
        return 0
    else
        record_result "Migration Metadata - $contract_name" "FAIL" "Migration metadata missing"
        return 1
    fi
}

# Function to perform detailed verification
perform_detailed_verification() {
    log "Performing detailed verification..."
    
    local contracts=("analytics" "token" "shared")
    
    for contract in "${contracts[@]}"; do
        local v1_id_var="${contract^^}_CONTRACT_ID_V1"
        local v2_id_var="${contract^^}_CONTRACT_ID_V2"
        
        check_contract_accessibility "$contract" "${!v2_id_var}"
        check_contract_version "$contract" "${!v2_id_var}" "2.0.0"
        check_data_integrity "$contract" "${!v1_id_var}" "${!v2_id_var}"
        check_contract_functionality "$contract" "${!v2_id_var}"
        check_migration_metadata "$contract" "${!v2_id_var}"
    done
}

# Function to perform quick verification
perform_quick_verification() {
    log "Performing quick verification..."
    
    local contracts=("analytics" "token" "shared")
    
    for contract in "${contracts[@]}"; do
        local v2_id_var="${contract^^}_CONTRACT_ID_V2"
        
        check_contract_accessibility "$contract" "${!v2_id_var}"
        check_contract_functionality "$contract" "${!v2_id_var}"
    done
}

# Function to generate verification report
generate_verification_report() {
    local report_file="$BACKUP_DIR/verification_report_$(date +%Y%m%d_%H%M%S).json"
    
    log "Generating verification report..."
    
    cat > "$report_file" << EOF
{
    "verification": {
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "network": "$NETWORK",
        "total_checks": $TOTAL_CHECKS,
        "passed_checks": $PASSED_CHECKS,
        "success_rate": "$(echo "scale=2; $PASSED_CHECKS * 100 / $TOTAL_CHECKS" | bc -l)%",
        "detailed_report": $DETAILED_REPORT,
        "quick_check": $QUICK_CHECK
    },
    "results": [
$(printf ',\n        %s' "${VERIFICATION_RESULTS[@]}" | tail -c +3)
    ],
    "summary": {
        "status": "$([ $PASSED_CHECKS -eq $TOTAL_CHECKS ] && echo "SUCCESS" || echo "PARTIAL_SUCCESS")",
        "recommendation": "$([ $PASSED_CHECKS -eq $TOTAL_CHECKS ] && echo "Migration verified successfully" || echo "Review failed checks and take corrective action")"
    }
}
EOF
    
    success "Verification report generated: $report_file"
    
    # Display summary
    echo
    log "Verification Summary:"
    log "Total Checks: $TOTAL_CHECKS"
    log "Passed Checks: $PASSED_CHECKS"
    log "Success Rate: $(echo "scale=2; $PASSED_CHECKS * 100 / $TOTAL_CHECKS" | bc -l)%"
    
    if [ $PASSED_CHECKS -eq $TOTAL_CHECKS ]; then
        success "All verification checks passed!"
    else
        warning "Some verification checks failed. Please review the report."
    fi
}

# Main verification function
main() {
    log "Starting StrellerMinds migration verification"
    log "Network: $NETWORK"
    log "Backup directory: $BACKUP_DIR"
    
    if [ "$DETAILED_REPORT" = true ]; then
        log "Running detailed verification"
    elif [ "$QUICK_CHECK" = true ]; then
        log "Running quick verification"
    else
        log "Running standard verification"
    fi
    
    # Perform verification
    if [ "$QUICK_CHECK" = true ]; then
        perform_quick_verification
    else
        perform_detailed_verification
    fi
    
    # Generate report
    generate_verification_report
    
    # Exit with appropriate code
    if [ $PASSED_CHECKS -eq $TOTAL_CHECKS ]; then
        success "Migration verification completed successfully!"
        exit 0
    else
        error "Migration verification completed with issues"
        exit 1
    fi
}

# Execute main function
main
