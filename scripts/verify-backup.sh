#!/bin/bash

# StrellerMinds Smart Contracts - Verify Backup Script
# Verifies the integrity of contract state backups

set -e

# Default values
BACKUP_FILE=""
NETWORK="testnet"
DETAILED_CHECK=false

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
StrellerMinds Backup Verification Script

Usage: $0 [OPTIONS]

OPTIONS:
    --backup-file <file>       Backup file to verify (required)
    --network <network>        Network context for verification (local|testnet|mainnet) [default: testnet]
    --detailed-check          Perform detailed verification
    --help                     Show this help message

EXAMPLES:
    $0 --backup-file backup.json
    $0 --backup-file backup.bin --network mainnet --detailed-check

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --backup-file)
            BACKUP_FILE="$2"
            shift 2
            ;;
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --detailed-check)
            DETAILED_CHECK=true
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

# Validate arguments
if [ -z "$BACKUP_FILE" ]; then
    error "Backup file is required"
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

# Check if backup file exists
if [ ! -f "$BACKUP_FILE" ]; then
    error "Backup file not found: $BACKUP_FILE"
    exit 1
fi

# Load environment variables
if [ -f ".env.$NETWORK" ]; then
    source ".env.$NETWORK"
    log "Loaded environment variables from .env.$NETWORK"
else
    error "Environment file .env.$NETWORK not found"
    exit 1
fi

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

# Function to check file integrity
check_file_integrity() {
    log "Checking file integrity..."
    
    # Check if file exists and is readable
    if [ ! -r "$BACKUP_FILE" ]; then
        record_result "File Readability" "FAIL" "File is not readable"
        return 1
    fi
    
    # Check file size
    local file_size=$(stat -f%z "$BACKUP_FILE" 2>/dev/null || stat -c%s "$BACKUP_FILE" 2>/dev/null || echo "0")
    if [ "$file_size" -eq 0 ]; then
        record_result "File Size" "FAIL" "File is empty"
        return 1
    fi
    
    record_result "File Readability" "PASS" "File is readable"
    record_result "File Size" "PASS" "File size: $file_size bytes"
    
    # Calculate checksum
    local checksum=$(sha256sum "$BACKUP_FILE" | cut -d' ' -f1)
    record_result "Checksum" "PASS" "SHA256: $checksum"
    
    return 0
}

# Function to check backup format
check_backup_format() {
    log "Checking backup format..."
    
    # Determine file format
    local file_ext="${BACKUP_FILE##*.}"
    
    case $file_ext in
        json)
            log "Detected JSON format"
            
            # Validate JSON syntax
            if command -v jq >/dev/null 2>&1; then
                if jq empty "$BACKUP_FILE" 2>/dev/null; then
                    record_result "JSON Syntax" "PASS" "Valid JSON syntax"
                else
                    record_result "JSON Syntax" "FAIL" "Invalid JSON syntax"
                    return 1
                fi
            else
                warning "jq not available, skipping JSON syntax check"
                record_result "JSON Syntax" "SKIP" "jq not available"
            fi
            ;;
        bin|binary)
            log "Detected binary format"
            record_result "Binary Format" "PASS" "Binary file detected"
            ;;
        *)
            warning "Unknown file format: $file_ext"
            record_result "File Format" "WARN" "Unknown format: $file_ext"
            ;;
    esac
    
    return 0
}

# Function to check metadata
check_metadata() {
    local metadata_file="${BACKUP_FILE}.meta.json"
    
    log "Checking backup metadata..."
    
    if [ ! -f "$metadata_file" ]; then
        record_result "Metadata File" "FAIL" "Metadata file not found"
        return 1
    fi
    
    # Validate metadata JSON
    if command -v jq >/dev/null 2>&1; then
        if jq empty "$metadata_file" 2>/dev/null; then
            record_result "Metadata JSON" "PASS" "Valid metadata JSON"
            
            # Check required fields
            local required_fields=("timestamp" "network" "contract_id" "format" "file_size" "checksum")
            for field in "${required_fields[@]}"; do
                if jq -e ".export.$field" "$metadata_file" >/dev/null 2>&1; then
                    record_result "Metadata Field - $field" "PASS" "Field exists"
                else
                    record_result "Metadata Field - $field" "FAIL" "Field missing"
                fi
            done
        else
            record_result "Metadata JSON" "FAIL" "Invalid metadata JSON"
            return 1
        fi
    else
        warning "jq not available, skipping metadata validation"
        record_result "Metadata JSON" "SKIP" "jq not available"
    fi
    
    return 0
}

# Function to verify checksum
verify_checksum() {
    local metadata_file="${BACKUP_FILE}.meta.json"
    
    if [ ! -f "$metadata_file" ]; then
        record_result "Checksum Verification" "SKIP" "No metadata file"
        return 0
    fi
    
    log "Verifying checksum..."
    
    if command -v jq >/dev/null 2>&1; then
        local expected_checksum=$(jq -r '.export.checksum' "$metadata_file" 2>/dev/null)
        local actual_checksum=$(sha256sum "$BACKUP_FILE" | cut -d' ' -f1)
        
        if [ "$expected_checksum" = "$actual_checksum" ]; then
            record_result "Checksum Verification" "PASS" "Checksums match"
        else
            record_result "Checksum Verification" "FAIL" "Checksum mismatch"
            record_result "Expected Checksum" "FAIL" "Expected: $expected_checksum"
            record_result "Actual Checksum" "FAIL" "Actual: $actual_checksum"
            return 1
        fi
    else
        warning "jq not available, skipping checksum verification"
        record_result "Checksum Verification" "SKIP" "jq not available"
    fi
    
    return 0
}

# Function to perform detailed content check
perform_detailed_check() {
    log "Performing detailed content check..."
    
    if [ "$DETAILED_CHECK" = false ]; then
        record_result "Detailed Content Check" "SKIP" "Not requested"
        return 0
    fi
    
    # Check if it's a JSON backup
    local file_ext="${BACKUP_FILE##*.}"
    if [ "$file_ext" != "json" ]; then
        record_result "Detailed Content Check" "SKIP" "Not a JSON file"
        return 0
    fi
    
    if command -v jq >/dev/null 2>&1; then
        # Check for expected data structure
        local has_data=$(jq -e '.data' "$BACKUP_FILE" >/dev/null 2>&1 && echo "true" || echo "false")
        local has_contract_info=$(jq -e '.contract_info' "$BACKUP_FILE" >/dev/null 2>&1 && echo "true" || echo "false")
        
        if [ "$has_data" = "true" ]; then
            record_result "Data Structure" "PASS" "Contains data field"
        else
            record_result "Data Structure" "WARN" "No data field found"
        fi
        
        if [ "$has_contract_info" = "true" ]; then
            record_result "Contract Info" "PASS" "Contains contract info"
        else
            record_result "Contract Info" "WARN" "No contract info found"
        fi
        
        # Count data entries if present
        local data_count=$(jq -r '.data | length // 0' "$BACKUP_FILE" 2>/dev/null || echo "0")
        record_result "Data Count" "PASS" "$data_count entries found"
    else
        warning "jq not available, skipping detailed content check"
        record_result "Detailed Content Check" "SKIP" "jq not available"
    fi
    
    return 0
}

# Function to generate verification report
generate_verification_report() {
    local report_file="${BACKUP_FILE}.verification_report.json"
    
    log "Generating verification report..."
    
    cat > "$report_file" << EOF
{
    "verification": {
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "backup_file": "$BACKUP_FILE",
        "network": "$NETWORK",
        "total_checks": $TOTAL_CHECKS,
        "passed_checks": $PASSED_CHECKS,
        "success_rate": "$(echo "scale=2; $PASSED_CHECKS * 100 / $TOTAL_CHECKS" | bc -l)%",
        "detailed_check": $DETAILED_CHECK
    },
    "results": [
$(printf ',\n        %s' "${VERIFICATION_RESULTS[@]}" | tail -c +3)
    ],
    "summary": {
        "status": "$([ $PASSED_CHECKS -eq $TOTAL_CHECKS ] && echo "SUCCESS" || echo "PARTIAL_SUCCESS")",
        "recommendation": "$([ $PASSED_CHECKS -eq $TOTAL_CHECKS ] && echo "Backup is valid and ready for use" || echo "Review failed checks and take corrective action")"
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
        success "Backup verification passed!"
    else
        warning "Some verification checks failed. Please review the report."
    fi
}

# Main verification function
main() {
    log "Starting backup verification"
    log "Backup file: $BACKUP_FILE"
    log "Network: $NETWORK"
    
    # Perform verification checks
    check_file_integrity || true
    check_backup_format || true
    check_metadata || true
    verify_checksum || true
    perform_detailed_check || true
    
    # Generate report
    generate_verification_report
    
    # Exit with appropriate code
    if [ $PASSED_CHECKS -eq $TOTAL_CHECKS ]; then
        success "Backup verification completed successfully!"
        exit 0
    else
        error "Backup verification completed with issues"
        exit 1
    fi
}

# Execute main function
main
