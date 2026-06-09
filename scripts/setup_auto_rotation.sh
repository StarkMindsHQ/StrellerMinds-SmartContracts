#!/bin/bash
# Setup automated API key rotation using cron jobs
# This script configures scheduled rotation for zero-downtime key management

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRON_SETUP_LOG="$PROJECT_ROOT/target/cron_setup.log"

mkdir -p "$PROJECT_ROOT/target"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== Automated API Key Rotation Setup ===${NC}"
echo ""

# Function to log
log() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo "[$timestamp] $1" | tee -a "$CRON_SETUP_LOG"
}

# Check if crontab is available
if ! command -v crontab &> /dev/null; then
    echo -e "${RED}Error: crontab is not available${NC}"
    echo "Please install cron or use an alternative scheduler"
    exit 1
fi

echo "This script will set up automated API key rotation."
echo ""
echo "Configuration:"
echo "  - Rotation frequency: Every 90 days (default)"
echo "  - Alert frequency: Daily check for expiring keys"
echo "  - Grace period: 7 days"
echo ""

echo "Enter contract ID:"
read -r contract_id

echo "Enter network (testnet/mainnet):"
read -r network
network=${network:-testnet}

echo "Enter admin source (key name):"
read -r admin_source

echo ""
echo -e "${YELLOW}Select rotation schedule:${NC}"
echo "1) Every 90 days (recommended)"
echo "2) Every 60 days"
echo "3) Every 30 days"
echo "4) Custom schedule"
echo ""
read -r schedule_choice

case $schedule_choice in
    1)
        rotation_days=90
        ;;
    2)
        rotation_days=60
        ;;
    3)
        rotation_days=30
        ;;
    4)
        echo "Enter custom rotation interval in days:"
        read -r rotation_days
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

# Create rotation wrapper script
ROTATION_SCRIPT="$PROJECT_ROOT/scripts/auto_rotate.sh"
cat > "$ROTATION_SCRIPT" << 'EOF'
#!/bin/bash
# Automated rotation wrapper script
# Called by cron job

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
AUTO_ROTATION_LOG="$PROJECT_ROOT/target/auto_rotation.log"

mkdir -p "$PROJECT_ROOT/target"

timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
echo "[$timestamp] Starting automated rotation check" >> "$AUTO_ROTATION_LOG"

# Configuration (update these values)
CONTRACT_ID="__CONTRACT_ID__"
NETWORK="__NETWORK__"
ADMIN_SOURCE="__ADMIN_SOURCE__"

# Run scheduled rotation
bash "$SCRIPT_DIR/rotate_api_keys.sh" << INPUT
2
$CONTRACT_ID
$NETWORK
$ADMIN_SOURCE
INPUT

exit_code=$?

if [ $exit_code -eq 0 ]; then
    echo "[$timestamp] Automated rotation completed successfully" >> "$AUTO_ROTATION_LOG"
else
    echo "[$timestamp] ERROR: Automated rotation failed with exit code $exit_code" >> "$AUTO_ROTATION_LOG"
    # Send alert email (configure as needed)
    # echo "API key rotation failed" | mail -s "URGENT: Key Rotation Failed" admin@example.com
fi

exit $exit_code
EOF

# Replace placeholders
sed -i "s/__CONTRACT_ID__/$contract_id/g" "$ROTATION_SCRIPT"
sed -i "s/__NETWORK__/$network/g" "$ROTATION_SCRIPT"
sed -i "s/__ADMIN_SOURCE__/$admin_source/g" "$ROTATION_SCRIPT"

chmod +x "$ROTATION_SCRIPT"

# Create daily alert check script
ALERT_SCRIPT="$PROJECT_ROOT/scripts/check_expiring_keys.sh"
cat > "$ALERT_SCRIPT" << EOF
#!/bin/bash
# Daily check for expiring keys

SCRIPT_DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="\$(cd "\$SCRIPT_DIR/.." && pwd)"
ALERT_LOG="\$PROJECT_ROOT/target/alert_check.log"

mkdir -p "\$PROJECT_ROOT/target"

timestamp=\$(date -u +"%Y-%m-%dT%H:%M:%SZ")
echo "[\$timestamp] Checking for expiring keys" >> "\$ALERT_LOG"

# Configuration
CONTRACT_ID="$contract_id"
NETWORK="$network"

# Check expiring keys
stellar contract invoke --id "\$CONTRACT_ID" --network "\$NETWORK" -- \\
    get_keys_expiring_soon >> "\$ALERT_LOG" 2>&1

exit_code=\$?

if [ \$exit_code -eq 0 ]; then
    echo "[\$timestamp] Alert check completed" >> "\$ALERT_LOG"
else
    echo "[\$timestamp] ERROR: Alert check failed" >> "\$ALERT_LOG"
fi

exit \$exit_code
EOF

chmod +x "$ALERT_SCRIPT"

# Set up cron jobs
echo ""
echo -e "${YELLOW}Setting up cron jobs...${NC}"

# Create temporary crontab file
TEMP_CRON=$(mktemp)

# Export current crontab (if exists)
crontab -l > "$TEMP_CRON" 2>/dev/null || true

# Add rotation cron job (runs every $rotation_days days at 2:00 AM)
rotation_cron="0 2 */$rotation_days * * $ROTATION_SCRIPT >> $PROJECT_ROOT/target/cron_rotation.log 2>&1"
echo "$rotation_cron" >> "$TEMP_CRON"

# Add daily alert check (runs every day at 9:00 AM)
alert_cron="0 9 * * * $ALERT_SCRIPT >> $PROJECT_ROOT/target/cron_alerts.log 2>&1"
echo "$alert_cron" >> "$TEMP_CRON"

# Install new crontab
crontab "$TEMP_CRON"
rm -f "$TEMP_CRON"

log "SUCCESS: Cron jobs configured"

echo ""
echo -e "${GREEN}✓ Automated rotation setup complete!${NC}"
echo ""
echo "Configuration Summary:"
echo "  Contract ID: $contract_id"
echo "  Network: $network"
echo "  Rotation Interval: Every $rotation_days days"
echo "  Rotation Time: 2:00 AM UTC"
echo "  Alert Check: Daily at 9:00 AM UTC"
echo ""
echo "Scripts created:"
echo "  - $ROTATION_SCRIPT"
echo "  - $ALERT_SCRIPT"
echo ""
echo "Log files:"
echo "  - Rotation log: $PROJECT_ROOT/target/auto_rotation.log"
echo "  - Alert log: $PROJECT_ROOT/target/alert_check.log"
echo "  - Setup log: $CRON_SETUP_LOG"
echo ""
echo "View cron jobs:"
echo "  crontab -l"
echo ""
echo "Remove cron jobs:"
echo "  crontab -e  # Then remove the lines"
echo ""
echo -e "${YELLOW}Important Notes:${NC}"
echo "  1. Ensure stellar CLI is in PATH for cron jobs"
echo "  2. Configure email alerts in auto_rotate.sh"
echo "  3. Test rotation manually before relying on automation"
echo "  4. Monitor log files regularly"
echo "  5. Update admin source credentials as needed"
echo ""
echo -e "${GREEN}Setup completed successfully!${NC}"
