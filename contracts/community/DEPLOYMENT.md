# Community Contract Deployment Guide

## Prerequisites

- Stellar CLI installed
- Soroban SDK 22.0.0 or later
- Rust toolchain
- Access to Stellar testnet/mainnet
- Admin wallet with sufficient XLM

## Build

### Development Build
```bash
cargo build --manifest-path contracts/community/Cargo.toml
```

### Production Build
```bash
cargo build --manifest-path contracts/community/Cargo.toml --release --target wasm32-unknown-unknown
```

### Optimize WASM
```bash
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/community.wasm \
  --wasm-out target/wasm32-unknown-unknown/release/community_optimized.wasm
```

## Deploy to Testnet

### 1. Set Network Configuration
```bash
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

### 2. Configure Identity
```bash
soroban config identity generate admin
soroban config identity address admin
```

### 3. Fund Account
```bash
soroban config identity fund admin --network testnet
```

### 4. Deploy Contract
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/community_optimized.wasm \
  --source admin \
  --network testnet
```

Save the contract ID output.

### 5. Initialize Contract
```bash
ADMIN_ADDRESS=$(soroban config identity address admin)
CONTRACT_ID=<your-contract-id>

soroban contract invoke \
  --id $CONTRACT_ID \
  --source admin \
  --network testnet \
  -- initialize \
  --admin $ADMIN_ADDRESS
```

## Deploy to Mainnet

### 1. Set Network Configuration
```bash
soroban config network add mainnet \
  --rpc-url https://soroban-mainnet.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

### 2. Use Production Identity
```bash
soroban config identity generate mainnet-admin
# Or import existing key
soroban config identity import mainnet-admin <secret-key>
```

### 3. Deploy Contract
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/community_optimized.wasm \
  --source mainnet-admin \
  --network mainnet
```

### 4. Initialize Contract
```bash
ADMIN_ADDRESS=$(soroban config identity address mainnet-admin)
CONTRACT_ID=<your-contract-id>

soroban contract invoke \
  --id $CONTRACT_ID \
  --source mainnet-admin \
  --network mainnet \
  -- initialize \
  --admin $ADMIN_ADDRESS
```

## Post-Deployment Configuration

### 1. Configure Contract Settings
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source admin \
  --network testnet \
  -- update_config \
  --admin $ADMIN_ADDRESS \
  --config '{
    "post_xp_reward": 10,
    "reply_xp_reward": 5,
    "solution_xp_reward": 50,
    "contribution_base_xp": 100,
    "contribution_base_tokens": 1000,
    "mentor_session_xp": 75,
    "event_attendance_xp": 25,
    "min_reputation_to_moderate": 500,
    "max_reports_per_day": 10,
    "vote_weight_threshold": 100
  }'
```

### 2. Add Initial Moderators
```bash
MODERATOR_ADDRESS=<moderator-address>

soroban contract invoke \
  --id $CONTRACT_ID \
  --source admin \
  --network testnet \
  -- add_moderator \
  --admin $ADMIN_ADDRESS \
  --moderator $MODERATOR_ADDRESS \
  --role Moderator
```

### 3. Verify Deployment
```bash
# Check configuration
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_config

# Check metrics
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_community_metrics
```

## Integration with Other Contracts

### Link Gamification Contract
```bash
GAMIFICATION_CONTRACT_ID=<gamification-contract-id>

# This would require adding a setter function in the contract
# For now, integration is done at the application layer
```

### Link Analytics Contract
```bash
ANALYTICS_CONTRACT_ID=<analytics-contract-id>

# Similar to gamification, integration at application layer
```

## Testing Deployment

### 1. Create Test Post
```bash
USER_ADDRESS=$(soroban config identity address test-user)

soroban contract invoke \
  --id $CONTRACT_ID \
  --source test-user \
  --network testnet \
  -- create_post \
  --author $USER_ADDRESS \
  --category General \
  --title "Test Post" \
  --content "This is a test post" \
  --tags '[]' \
  --course_id ""
```

### 2. Register Test Mentor
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source test-user \
  --network testnet \
  -- register_mentor \
  --mentor $USER_ADDRESS \
  --expertise_areas '["Rust", "Blockchain"]' \
  --expertise_level Expert \
  --max_mentees 5 \
  --bio "Experienced developer"
```

### 3. Create Test Event
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source test-user \
  --network testnet \
  -- create_event \
  --organizer $USER_ADDRESS \
  --event_type Workshop \
  --title "Soroban Workshop" \
  --description "Learn Soroban development" \
  --start_time 1700000000 \
  --end_time 1700010000 \
  --max_participants 50 \
  --is_public true \
  --xp_reward 25
```

## Monitoring

### Contract Events
Monitor contract events using Stellar's event streaming:

```bash
soroban events \
  --start-ledger <ledger-number> \
  --id $CONTRACT_ID \
  --network testnet
```

### Key Events to Monitor
- `post_new`: New posts
- `reply_new`: New replies
- `solution`: Solutions marked
- `mntr_reg`: Mentors registered
- `contrib`: Contributions submitted
- `event_new`: Events created
- `report`: Content reported
- `proposal`: Proposals created

## Backup and Recovery

### Export Contract State
```bash
# Use Stellar's state archival features
soroban contract fetch \
  --id $CONTRACT_ID \
  --network testnet \
  --out-file community_state_backup.json
```

### Contract Upgrade
```bash
# Deploy new version
NEW_CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/community_v2_optimized.wasm \
  --source admin \
  --network testnet)

# Migrate data (application-specific logic required)
# Update references in dependent systems
```

## Security Checklist

- [ ] Admin keys stored securely
- [ ] Multi-sig setup for admin operations (if applicable)
- [ ] Rate limiting configured appropriately
- [ ] Moderator permissions assigned correctly
- [ ] Contract addresses verified
- [ ] Integration endpoints tested
- [ ] Event monitoring configured
- [ ] Backup procedures established
- [ ] Incident response plan documented
- [ ] Access logs reviewed

## Cost Estimation

### Deployment Costs (Testnet)
- Contract deployment: ~0.5 XLM
- Initialization: ~0.1 XLM
- Configuration: ~0.05 XLM per update

### Operational Costs
- Post creation: ~0.01 XLM
- Reply creation: ~0.005 XLM
- Contribution submission: ~0.02 XLM
- Event creation: ~0.015 XLM
- Proposal creation: ~0.02 XLM

Note: Mainnet costs may vary based on network conditions.

## Troubleshooting

### Common Issues

**Issue**: Contract deployment fails
```bash
# Check account balance
soroban config identity fund admin --network testnet

# Verify WASM file
ls -lh target/wasm32-unknown-unknown/release/community_optimized.wasm
```

**Issue**: Initialization fails
```bash
# Verify admin address
soroban config identity address admin

# Check if already initialized
soroban contract invoke --id $CONTRACT_ID --network testnet -- get_config
```

**Issue**: Function calls fail
```bash
# Check authorization
# Ensure correct source identity is used
# Verify function parameters match contract interface
```

## Support and Resources

- Contract source: `contracts/community/src/`
- Tests: `contracts/community/src/tests.rs`
- Documentation: `contracts/community/README.md`
- Integration guide: `contracts/community/INTEGRATION.md`
- Soroban docs: https://soroban.stellar.org/docs

## Maintenance Schedule

- **Daily**: Monitor events and metrics
- **Weekly**: Review reports and moderation actions
- **Monthly**: Analyze engagement trends
- **Quarterly**: Review and update configuration
- **Annually**: Plan major upgrades

## Contact

For deployment support, contact the StrellerMinds development team.
