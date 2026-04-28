# Quick Start Guide - New Features

This guide helps you get started with the 4 newly implemented features.

---

## 1. Performance Optimization

### Optimize WASM Files
```bash
# Run optimization on all contracts
./scripts/optimize_wasm.sh

# View optimization report
cat target/optimized/optimization_report.txt
```

### Monitor Performance
```bash
# Check deployment metrics
./scripts/deploy_metrics.sh

# View metrics
cat target/metrics/deployment_metrics.json
```

### Code Splitting
Read the strategy: `docs/CODE_SPLITTING_STRATEGY.md`

**Deploy by tier:**
- Tier 1 (Core): shared, proxy, token
- Tier 2 (Educational): certificate, progress, analytics
- Tier 3 (Advanced): Deploy as needed

---

## 2. Certificate Versioning

### Create a Template Version
```bash
stellar contract invoke --id <contract_id> --network testnet \
  --source <admin_key> -- \
  create_template_version \
  --template_id "my_template" \
  --fields "[{\"field_name\": \"grade\", \"field_type\": \"Text\", \"is_required\": true}]" \
  --changelog "Added grade field"
```

### View Version History
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_template_version_history --template_id "my_template"
```

### Get Specific Version
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_template_at_version --template_id "my_template" --version 2
```

### Rollback to Previous Version
```bash
stellar contract invoke --id <contract_id> --network testnet \
  --source <admin_key> -- \
  rollback_template \
  --template_id "my_template" \
  --target_version 1
```

### Migrate Certificates
```bash
stellar contract invoke --id <contract_id> --network testnet \
  --source <admin_key> -- \
  migrate_template_certificates \
  --template_id "my_template" \
  --from_version 1 \
  --to_version 2
```

---

## 3. Troubleshooting Guide

### Access the Guide
```bash
# Open troubleshooting documentation
cat docs/TROUBLESHOOTING.md
```

### Common Quick Fixes

**Contract already initialized:**
```bash
stellar contract invoke --id <contract_id> --network testnet -- is_initialized
```

**Check admin address:**
```bash
stellar contract invoke --id <contract_id> --network testnet -- get_admin
```

**View certificate:**
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_certificate --certificate_id <cert_id>
```

**Check audit trail:**
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_multisig_audit_trail --request_id <request_id>
```

---

## 4. API Key Rotation

### Manual Rotation
```bash
# Run interactive rotation script
./scripts/rotate_api_keys.sh

# Follow prompts to:
# 1. Check rotation status
# 2. Perform scheduled rotation
# 3. Force rotate specific key
# 4. Check expiring keys
# 5. Verify dual key support
```

### Setup Automated Rotation
```bash
# Configure cron jobs for automatic rotation
./scripts/setup_auto_rotation.sh

# Follow prompts to set:
# - Contract ID
# - Network
# - Admin source
# - Rotation schedule (30/60/90 days)
```

### Check Key Status
```bash
# View active keys
stellar contract invoke --id <contract_id> --network testnet -- \
  get_active_keys

# Check keys expiring soon
stellar contract invoke --id <contract_id> --network testnet -- \
  get_keys_expiring_soon

# View rotation history
stellar contract invoke --id <contract_id> --network testnet -- \
  get_rotation_history --key_id "key_1"
```

### Rotation Configuration
```bash
# Get current rotation config
stellar contract invoke --id <contract_id> --network testnet -- \
  get_rotation_config

# Default settings:
# - Rotation interval: 90 days
# - Grace period: 7 days
# - Max keys per user: 3
# - Alert before expiry: 30 days
```

---

## Integration Examples

### Full Workflow Example

1. **Optimize and deploy contract:**
```bash
./scripts/optimize_wasm.sh
./scripts/deploy.sh --network testnet --contract certificate --wasm target/optimized/certificate.wasm
```

2. **Create certificate template with versioning:**
```bash
stellar contract invoke --id <contract_id> --network testnet \
  --source admin -- \
  create_template \
  --template_id "course_completion" \
  --name "Course Completion" \
  --description "Standard completion certificate" \
  --fields "[]"
```

3. **Set up API key rotation:**
```bash
./scripts/setup_auto_rotation.sh
```

4. **Monitor performance:**
```bash
./scripts/deploy_metrics.sh
cat target/metrics/deployment_metrics.json
```

---

## Troubleshooting

### Issues?

1. Check `docs/TROUBLESHOOTING.md` for solutions
2. View logs in `target/` directory
3. Run diagnostic commands from troubleshooting guide
4. Create GitHub issue if problem persists

### Log Files

- WASM optimization: `target/optimized/optimization_report.txt`
- Performance metrics: `target/metrics/deployment_metrics.json`
- API key rotation: `target/api_key_rotation.log`
- Auto rotation: `target/auto_rotation.log`
- Alert checks: `target/alert_check.log`

---

## Next Steps

1. ✅ All features implemented
2. 🔄 Test on testnet
3. 📝 Update documentation as needed
4. 🚀 Deploy to production

For detailed information, see `IMPLEMENTATION_SUMMARY.md`
