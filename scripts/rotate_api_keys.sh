#!/bin/bash
# API Key Rotation Script - Automated key rotation with zero downtime
# Implements: Schedule-based rotation, Dual key support, Gradual deprecation, Alert system

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ROTATION_LOG="$PROJECT_ROOT/target/api_key_rotation.log"

mkdir -p "$PROJECT_ROOT/target"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== API Key Rotation Script ===${NC}"
echo ""

# Logging function
log() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo "[$timestamp] $1" | tee -a "$ROTATION_LOG"
}

# Function to check keys expiring soon
check_expiring_keys() {
    local contract_id=$1
    local network=$2
    
    echo -e "${YELLOW}Checking for keys expiring soon...${NC}"
    log "INFO: Checking expiring keys for contract $contract_id on $network"
    
    # Get keys expiring within alert period
    stellar contract invoke --id "$contract_id" --network "$network" -- \
        get_keys_expiring_soon 2>&1 | tee -a "$ROTATION_LOG"
    
    if [ $? -eq 0 ]; then
        log "INFO: Expiring keys check completed"
    else
        log "ERROR: Failed to check expiring keys"
    fi
}

# Function to rotate a single API key
rotate_key() {
    local contract_id=$1
    local network=$2
    local old_key_id=$3
    local admin_source=$4
    local reason=${5:-"Scheduled rotation"}
    
    echo -e "${YELLOW}Rotating key: $old_key_id${NC}"
    log "INFO: Starting rotation for key $old_key_id"
    log "INFO: Reason: $reason"
    
    # Generate new key hash (in production, use secure random generation)
    local new_key_hash=$(openssl rand -hex 32)
    
    # Perform rotation
    stellar contract invoke --id "$contract_id" \
        --network "$network" \
        --source "$admin_source" \
        -- rotate_api_key \
        --old_key_id "$old_key_id" \
        --new_key_hash "$new_key_hash" \
        --reason "$reason" 2>&1 | tee -a "$ROTATION_LOG"
    
    if [ $? -eq 0 ]; then
        log "SUCCESS: Key rotation completed for $old_key_id"
        echo -e "${GREEN}✓ Key rotated successfully${NC}"
    else
        log "ERROR: Key rotation failed for $old_key_id"
        echo -e "${RED}✗ Key rotation failed${NC}"
        return 1
    fi
}

# Function to verify dual key support
verify_dual_keys() {
    local contract_id=$1
    local network=$2
    
    echo -e "${YELLOW}Verifying dual key support...${NC}"
    log "INFO: Verifying dual key configuration"
    
    # Get active keys
    local active_keys=$(stellar contract invoke --id "$contract_id" \
        --network "$network" -- get_active_keys 2>&1)
    
    echo "Active keys: $active_keys"
    log "INFO: Active keys: $active_keys"
    
    # Count active keys
    local key_count=$(echo "$active_keys" | grep -o "key_[0-9]*" | wc -l)
    
    if [ "$key_count" -ge 2 ]; then
        echo -e "${GREEN}✓ Dual key support active ($key_count keys)${NC}"
        log "INFO: Dual key support verified with $key_count keys"
    else
        echo -e "${YELLOW}⚠ Only $key_count key(s) active${NC}"
        log "WARN: Less than 2 keys active"
    fi
}

# Function to send alert (placeholder for actual alerting)
send_alert() {
    local alert_type=$1
    local message=$2
    
    log "ALERT [$alert_type]: $message"
    
    # In production, integrate with:
    # - Email notifications
    # - Slack/Teams webhooks
    # - PagerDuty
    # - SMS alerts
    
    echo -e "${RED}🔔 ALERT [$alert_type]: $message${NC}"
}

# Function to perform scheduled rotation
scheduled_rotation() {
    local contract_id=$1
    local network=$2
    local admin_source=$3
    
    log "INFO: Starting scheduled rotation for $contract_id on $network"
    
    # Step 1: Check which keys need rotation
    check_expiring_keys "$contract_id" "$network"
    
    # Step 2: Get keys to rotate
    local keys_to_rotate=$(stellar contract invoke --id "$contract_id" \
        --network "$network" -- check_keys_need_rotation 2>&1)
    
    if [ -z "$keys_to_rotate" ] || [ "$keys_to_rotate" = "[]" ]; then
        log "INFO: No keys require rotation at this time"
        echo -e "${GREEN}✓ No keys need rotation${NC}"
        return 0
    fi
    
    # Step 3: Rotate each key
    echo "$keys_to_rotate" | grep -o "key_[0-9]*" | while read -r key_id; do
        rotate_key "$contract_id" "$network" "$key_id" "$admin_source" "Scheduled rotation"
        
        # Wait between rotations to avoid overwhelming the system
        sleep 2
    done
    
    # Step 4: Verify dual key support
    verify_dual_keys "$contract_id" "$network"
    
    # Step 5: Send completion alert
    send_alert "ROTATION_COMPLETE" "Scheduled rotation completed for $contract_id"
    
    log "INFO: Scheduled rotation completed successfully"
}

# Function to force rotation (emergency)
force_rotation() {
    local contract_id=$1
    local network=$2
    local old_key_id=$3
    local admin_source=$4
    
    log "WARN: Force rotation initiated for key $old_key_id"
    send_alert "EMERGENCY_ROTATION" "Force rotation of key $old_key_id on $contract_id"
    
    rotate_key "$contract_id" "$network" "$old_key_id" "$admin_source" "Emergency rotation"
    
    # Immediately revoke old key (skip grace period)
    stellar contract invoke --id "$contract_id" \
        --network "$network" \
        --source "$admin_source" \
        -- revoke_api_key \
        --key_id "$old_key_id" 2>&1 | tee -a "$ROTATION_LOG"
    
    log "SUCCESS: Force rotation completed"
}

# Function to check rotation status
rotation_status() {
    local contract_id=$1
    local network=$2
    
    echo -e "${GREEN}=== API Key Rotation Status ===${NC}"
    echo ""
    
    # Get rotation config
    echo "Rotation Configuration:"
    stellar contract invoke --id "$contract_id" --network "$network" -- \
        get_rotation_config 2>&1
    echo ""
    
    # Get active keys
    echo "Active Keys:"
    stellar contract invoke --id "$contract_id" --network "$network" -- \
        get_active_keys 2>&1
    echo ""
    
    # Get expiring keys
    echo "Keys Expiring Soon:"
    stellar contract invoke --id "$contract_id" --network "$network" -- \
        get_keys_expiring_soon 2>&1
    echo ""
    
    # Check rotation history
    echo "Recent Rotation Events:"
    tail -20 "$ROTATION_LOG"
}

# Main execution
echo "Select operation:"
echo "1) Check rotation status"
echo "2) Perform scheduled rotation"
echo "3) Force rotate specific key"
echo "4) Check expiring keys"
echo "5) Verify dual key support"
echo ""
read -r choice

echo ""
echo "Enter contract ID:"
read -r contract_id

echo "Enter network (testnet/mainnet):"
read -r network
network=${network:-testnet}

echo "Enter admin source (key name):"
read -r admin_source

case $choice in
    1)
        rotation_status "$contract_id" "$network"
        ;;
    2)
        scheduled_rotation "$contract_id" "$network" "$admin_source"
        ;;
    3)
        echo "Enter old key ID to rotate:"
        read -r old_key_id
        force_rotation "$contract_id" "$network" "$old_key_id" "$admin_source"
        ;;
    4)
        check_expiring_keys "$contract_id" "$network"
        ;;
    5)
        verify_dual_keys "$contract_id" "$network"
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}Operation completed!${NC}"
echo "Log file: $ROTATION_LOG"
