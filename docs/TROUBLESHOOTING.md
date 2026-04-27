# Troubleshooting Guide for StrellerMinds Smart Contracts

## Table of Contents
- [Common Errors](#common-errors)
- [Build & Compilation Issues](#build--compilation-issues)
- [Deployment Problems](#deployment-problems)
- [Runtime Errors](#runtime-errors)
- [Performance Issues](#performance-issues)
- [Certificate Versioning](#certificate-versioning)
- [API Key Rotation](#api-key-rotation)
- [Log Interpretation](#log-interpretation)
- [Support Contacts](#support-contacts)
- [FAQ](#faq)

---

## Common Errors

### Error: Contract Already Initialized

**Symptom:**
```
CertificateError::AlreadyInitialized
```

**Cause:** Attempting to initialize a contract that has already been initialized.

**Solution:**
```bash
# Check if contract is already initialized
stellar contract invoke --id <contract_id> --network testnet -- is_initialized

# Skip initialization if already done
```

### Error: Unauthorized Access

**Symptom:**
```
CertificateError::Unauthorized
CertificateError::ApproverNotAuthorized
```

**Cause:** Calling a function with an address that doesn't have the required permissions.

**Solution:**
1. Verify the caller's role:
```bash
stellar contract invoke --id <contract_id> --network testnet -- get_admin
```

2. Ensure you're using the correct signer:
```bash
stellar keys use <admin_key_name>
```

3. Re-authorize if needed:
```bash
stellar contract invoke --id <contract_id> --network testnet \
  --source <admin_key> -- initialize --admin <admin_address>
```

### Error: Template Not Found

**Symptom:**
```
CertificateError::TemplateNotFound
CertificateError::TemplateVersionNotFound
```

**Cause:** Referencing a template ID or version that doesn't exist.

**Solution:**
```bash
# List all available templates
stellar contract invoke --id <contract_id> --network testnet -- get_template_list

# Check template version history
stellar contract invoke --id <contract_id> --network testnet -- \
  get_template_version_history --template_id "<template_id>"

# Verify the template exists before using it
stellar contract invoke --id <contract_id> --network testnet -- \
  get_template --template_id "<template_id>"
```

---

## Build & Compilation Issues

### Issue: WASM Build Fails

**Symptom:**
```
error[E0432]: unresolved import
```

**Solutions:**

1. **Check Rust Toolchain:**
```bash
rustup show
rustup target add wasm32-unknown-unknown
```

2. **Verify Dependencies:**
```bash
cargo check
cargo build --target wasm32-unknown-unknown
```

3. **Clean and Rebuild:**
```bash
cargo clean
cargo build --target wasm32-unknown-unknown --release
```

4. **Check Cargo.toml:**
Ensure all dependencies are correctly specified:
```toml
[dependencies]
soroban-sdk = "22.0.0"
shared = { path = "../shared" }
```

### Issue: Compilation Errors After Updates

**Symptom:**
Type mismatches or missing fields after pulling updates.

**Solution:**
```bash
# Update dependencies
cargo update

# Rebuild workspace
cargo build --target wasm32-unknown-unknown --workspace

# Run tests to verify
cargo test --target wasm32-unknown-unknown
```

---

## Deployment Problems

### Issue: Deployment Timeout

**Symptom:**
```
Error: Transaction failed: Timeout
```

**Solutions:**

1. **Optimize WASM Size:**
```bash
./scripts/optimize_wasm.sh
```

2. **Check Network Status:**
```bash
stellar network status --network testnet
```

3. **Increase Timeout (if needed):**
```bash
stellar contract deploy --wasm <file.wasm> \
  --network testnet \
  --timeout 300
```

### Issue: Insufficient Balance

**Symptom:**
```
Error: Insufficient balance for transaction fees
```

**Solution:**
```bash
# Check account balance
stellar account show --account <address> --network testnet

# Fund account on testnet
stellar account fund --account <address> --network testnet
```

---

## Runtime Errors

### Issue: Certificate Issuance Fails

**Symptom:**
Certificate creation returns an error.

**Debugging Steps:**

1. **Check Multi-Sig Configuration:**
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_multisig_config --course_id "<course_id>"
```

2. **Verify Approvals:**
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_multisig_request --request_id <request_id>
```

3. **Check Audit Trail:**
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_multisig_audit_trail --request_id <request_id>
```

### Issue: Template Rollback Fails

**Symptom:**
```
CertificateError::TemplateRollbackFailed
```

**Solution:**
1. Verify target version exists:
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_template_at_version --template_id "<id>" --version <version>
```

2. Check version history:
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_template_version_history --template_id "<id>"
```

3. Ensure admin permissions:
```bash
stellar contract invoke --id <contract_id> --network testnet -- get_admin
```

---

## Performance Issues

### Issue: Slow Contract Execution

**Symptom:** Transaction takes longer than expected.

**Solutions:**

1. **Check WASM Size:**
```bash
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

2. **Run Performance Metrics:**
```bash
./scripts/deploy_metrics.sh
```

3. **Optimize Contract:**
```bash
./scripts/optimize_wasm.sh
```

4. **Review Storage Patterns:**
- Use instance storage for frequently accessed data
- Use persistent storage only for critical data
- Minimize storage reads/writes in loops

### Issue: Cold Start Time > 500ms

**Solutions:**

1. **Enable Provisioned Concurrency** (for Lambda deployments)

2. **Reduce WASM Size:**
```bash
wasm-opt -Oz input.wasm -o output.wasm
```

3. **Implement Code Splitting:**
See [CODE_SPLITTING_STRATEGY.md](CODE_SPLITTING_STRATEGY.md)

4. **Lazy Loading:**
Initialize complex features only when needed.

---

## Certificate Versioning

### Issue: Version Conflict

**Symptom:** Multiple versions causing confusion.

**Solution:**
```bash
# Check current version
stellar contract invoke --id <contract_id> --network testnet -- \
  get_template --template_id "<id>"

# View all versions
stellar contract invoke --id <contract_id> --network testnet -- \
  get_template_version_history --template_id "<id>"
```

### Issue: Migration Problems

**Solution:**
1. Validate source and target versions exist
2. Test migration on testnet first
3. Monitor migration progress:
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  migrate_template_certificates \
  --template_id "<id>" \
  --from_version 1 \
  --to_version 2
```

---

## API Key Rotation

### Issue: Rotation Failure

**Symptom:** API key rotation doesn't complete.

**Solution:**

1. **Check Rotation Status:**
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_api_key_status
```

2. **Verify Dual Key Setup:**
```bash
stellar contract invoke --id <contract_id> --network testnet -- \
  get_active_api_keys
```

3. **Manual Rotation:**
```bash
stellar contract invoke --id <contract_id> --network testnet \
  --source <admin_key> -- \
  rotate_api_key --new_key "<new_key>"
```

### Issue: Deprecated Key Still Active

**Solution:**
```bash
# Force deprecation
stellar contract invoke --id <contract_id> --network testnet \
  --source <admin_key> -- \
  deprecate_api_key --key_id "<old_key_id>"
```

---

## Log Interpretation

### Understanding Soroban Logs

**Format:**
```
[timestamp] [level] [contract_id] message
```

**Common Log Levels:**
- `INFO`: Normal operation
- `WARN`: Potential issue
- `ERROR`: Operation failed
- `DEBUG`: Detailed debugging info

### Key Events to Monitor

1. **Certificate Issued:**
```
event: certification_issued
data: { certificate_id, student, course_id }
```

2. **Template Version Created:**
```
event: template_version_created
data: { template_id, version, created_by }
```

3. **API Key Rotated:**
```
event: api_key_rotated
data: { old_key_id, new_key_id, timestamp }
```

### Debug Mode

Enable detailed logging:
```bash
export SORBAN_LOG=debug
stellar contract invoke --id <contract_id> --network testnet -- <function>
```

---

## Support Contacts

### Community Support
- **GitHub Issues:** https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/issues
- **Discord:** [Join our Discord](https://discord.gg/strellerminds)
- **Forum:** https://forum.strellerminds.com

### Documentation
- **Main Docs:** https://docs.strellerminds.com
- **API Reference:** [API.md](API.md)
- **Architecture:** [ARCHITECTURE.md](ARCHITECTURE.md)

### Emergency Contacts
For critical production issues:
- **Email:** support@strellerminds.com
- **Priority Support:** Available for enterprise customers

---

## FAQ

### General Questions

**Q: How do I check my contract version?**
```bash
stellar contract version --id <contract_id>
```

**Q: Can I revert a deployed contract?**
A: Yes, if you have the proxy contract set up:
```bash
stellar contract invoke --id <proxy_id> --network testnet -- \
  rollback --to_version <version>
```

**Q: How do I backup contract state?**
A: Export all relevant data:
```bash
stellar contract export --id <contract_id> --network testnet > backup.json
```

### Certificate Questions

**Q: How do I create a new template version?**
```bash
stellar contract invoke --id <contract_id> --network testnet \
  --source <admin_key> -- \
  create_template_version \
  --template_id "<id>" \
  --fields "[...]" \
  --changelog "Updated fields"
```

**Q: Can I rollback a certificate?**
A: Certificates themselves cannot be rolled back, but you can:
1. Revoke the certificate
2. Reissue with corrected data
3. Use template rollback for future certificates

**Q: How many template versions are kept?**
A: All versions are kept indefinitely for audit purposes.

### Performance Questions

**Q: What's the maximum WASM size?**
A: Stellar recommends < 64KB for optimal performance, but supports up to 1MB.

**Q: How do I improve deployment speed?**
1. Optimize WASM size
2. Use code splitting
3. Deploy during off-peak hours
4. Consider provisioned concurrency

**Q: Why is my contract slow?**
Common causes:
- Large WASM size
- Excessive storage operations
- Complex computations
- Network latency

### Security Questions

**Q: How often should API keys be rotated?**
A: Recommended every 90 days, or immediately if compromised.

**Q: What happens during key rotation?**
1. New key is generated
2. Both keys are active (grace period)
3. Old key is deprecated
4. Alerts are sent to administrators

**Q: How do I report a security vulnerability?**
A: Email security@strellerminds.com or use our bug bounty program.

---

## Quick Reference Commands

### Build & Test
```bash
# Build all contracts
./scripts/build.sh

# Run tests
cargo test

# Check code quality
cargo clippy
```

### Deploy
```bash
# Deploy to testnet
./scripts/deploy.sh --network testnet --contract <name> --wasm <path>

# Deploy to mainnet
./scripts/deploy.sh --network mainnet --contract <name> --wasm <path>
```

### Monitor
```bash
# Check performance
./scripts/deploy_metrics.sh

# View optimization report
cat target/optimized/optimization_report.txt
```

---

## Contributing

Found an issue not covered here? Please:
1. Check existing GitHub issues
2. Create a new issue with detailed description
3. Submit a PR to improve this guide

**Remember:** Always test changes on testnet before deploying to mainnet!
