# Community Contract Integration Guide

This document provides guidance on integrating the Community contract with other contracts in the StrellerMinds ecosystem.

## Architecture Overview

The Community contract is designed to work alongside:
- **Gamification Contract**: For XP rewards and achievements
- **Analytics Contract**: For tracking engagement metrics
- **Token Contract**: For token-based rewards
- **Learning Contract**: For course-specific discussions

## Integration Points

### 1. Gamification Contract Integration

The Community contract awards XP for various activities. These should trigger calls to the Gamification contract:

```rust
// In forum.rs, mentorship.rs, knowledge.rs, community_events.rs
fn award_xp(env: &Env, user: &Address, xp: u32) {
    // Call gamification contract
    let gamification_client = GamificationClient::new(env, &gamification_contract_id);
    
    let activity = ActivityRecord {
        activity_type: ActivityType::PeerHelped,
        course_id: String::from_str(env, ""),
        module_id: String::from_str(env, ""),
        score: 0,
        time_spent: 0,
        timestamp: env.ledger().timestamp(),
    };
    
    let _ = gamification_client.record_activity(user, &activity);
}
```

### 2. Token Rewards Integration

For knowledge contributions, the contract should distribute token rewards:

```rust
// In knowledge.rs
fn award_tokens(env: &Env, user: &Address, tokens: i128) {
    // Call token contract to transfer rewards
    let token_client = TokenClient::new(env, &token_contract_id);
    token_client.transfer(&treasury_address, user, &tokens);
}
```

### 3. Analytics Integration

Community metrics should be reported to the Analytics contract:

```rust
// In analytics.rs
pub fn sync_to_analytics(env: &Env) {
    let analytics_client = AnalyticsClient::new(env, &analytics_contract_id);
    let metrics = Self::get_community_metrics(env);
    
    // Report community engagement metrics
    analytics_client.record_community_metrics(&metrics);
}
```

### 4. Cross-Contract Event Listening

The Community contract emits events that other contracts can listen to:

```rust
// Events emitted by Community contract
- post_new: New forum post created
- reply_new: New reply added
- solution: Solution marked
- mntr_reg: Mentor registered
- mntr_req: Mentorship requested
- contrib: Contribution submitted
- approved: Contribution approved
- event_new: Event created
- event_reg: User registered for event
- report: Content reported
- proposal: Proposal created
- vote: Vote cast
```

## Configuration

### Contract Addresses

Store contract addresses in the Community contract storage:

```rust
#[derive(Clone)]
#[contracttype]
pub enum ContractAddressKey {
    GamificationContract,
    AnalyticsContract,
    TokenContract,
    LearningContract,
}
```

### Initialization with Dependencies

```rust
pub fn initialize_with_deps(
    env: Env,
    admin: Address,
    gamification_contract: Address,
    analytics_contract: Address,
    token_contract: Address,
) -> Result<(), Error> {
    // Standard initialization
    Self::initialize(env.clone(), admin)?;
    
    // Store contract addresses
    env.storage().instance().set(
        &ContractAddressKey::GamificationContract,
        &gamification_contract
    );
    env.storage().instance().set(
        &ContractAddressKey::AnalyticsContract,
        &analytics_contract
    );
    env.storage().instance().set(
        &ContractAddressKey::TokenContract,
        &token_contract
    );
    
    Ok(())
}
```

## Data Flow Examples

### Example 1: User Creates Post and Earns XP

```
1. User calls create_post()
2. Community contract creates post
3. Community contract calls Gamification.record_activity()
4. Gamification awards XP and checks achievements
5. Analytics contract records engagement metric
6. Event emitted: post_new
```

### Example 2: Contribution Approved and Rewarded

```
1. Moderator calls review_contribution(approve=true)
2. Community contract approves contribution
3. Community contract calls Gamification.record_activity()
4. Community contract calls Token.transfer() for reward
5. Analytics contract records contribution metric
6. Event emitted: approved
```

### Example 3: Mentorship Session Completed

```
1. Mentor calls complete_session()
2. Community contract records session
3. Community contract awards XP to both mentor and mentee
4. Gamification contract updates profiles
5. Analytics contract records mentorship metric
6. Event emitted: session
```

## Security Considerations

### 1. Authorization

All cross-contract calls should verify authorization:

```rust
// Only admin can set contract addresses
pub fn set_gamification_contract(
    env: Env,
    admin: Address,
    contract: Address,
) -> Result<(), Error> {
    admin.require_auth();
    CommunityStorage::require_admin(&env, &admin)?;
    
    env.storage().instance().set(
        &ContractAddressKey::GamificationContract,
        &contract
    );
    Ok(())
}
```

### 2. Reentrancy Protection

Avoid reentrancy issues by following checks-effects-interactions pattern:

```rust
pub fn create_post(...) -> Result<u64, Error> {
    // 1. Checks
    author.require_auth();
    
    // 2. Effects (update state)
    let post_id = create_post_internal(...);
    
    // 3. Interactions (external calls)
    award_xp(&env, &author, config.post_xp_reward);
    
    Ok(post_id)
}
```

### 3. Error Handling

Handle cross-contract call failures gracefully:

```rust
fn award_xp(env: &Env, user: &Address, xp: u32) {
    if let Some(gamification_addr) = get_gamification_contract(env) {
        let client = GamificationClient::new(env, &gamification_addr);
        
        // Don't fail the main operation if XP award fails
        let _ = client.try_record_activity(user, &activity);
    }
}
```

## Testing Integration

### Mock Contracts for Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    fn setup_full_ecosystem() -> (
        Env,
        CommunityClient,
        GamificationClient,
        AnalyticsClient,
        TokenClient,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        
        // Deploy all contracts
        let community = deploy_community(&env);
        let gamification = deploy_gamification(&env);
        let analytics = deploy_analytics(&env);
        let token = deploy_token(&env);
        
        // Link contracts
        community.initialize_with_deps(
            &admin,
            &gamification.address,
            &analytics.address,
            &token.address,
        );
        
        (env, community, gamification, analytics, token)
    }
    
    #[test]
    fn test_post_creation_awards_xp() {
        let (env, community, gamification, _, _) = setup_full_ecosystem();
        
        // Create post
        community.create_post(...);
        
        // Verify XP was awarded
        let profile = gamification.get_user_profile(&user);
        assert!(profile.total_xp > 0);
    }
}
```

## Deployment Sequence

1. Deploy Token contract
2. Deploy Gamification contract
3. Deploy Analytics contract
4. Deploy Community contract
5. Initialize Community with dependency addresses
6. Configure cross-contract permissions
7. Fund treasury for rewards

## Monitoring and Maintenance

### Health Checks

```rust
pub fn health_check(env: Env) -> HealthStatus {
    HealthStatus {
        is_initialized: CommunityStorage::is_initialized(&env),
        gamification_connected: check_gamification_connection(&env),
        analytics_connected: check_analytics_connection(&env),
        token_connected: check_token_connection(&env),
        total_posts: get_counter(&env, CommunityKey::PostCounter),
        total_users: count_active_users(&env),
    }
}
```

### Metrics to Monitor

- Post creation rate
- Reply response time
- Mentorship acceptance rate
- Contribution approval rate
- Event attendance rate
- Report resolution time
- Proposal participation rate
- Cross-contract call success rate

## Upgrade Strategy

When upgrading the Community contract:

1. Deploy new version
2. Migrate critical data (posts, contributions, mentorships)
3. Update contract addresses in dependent contracts
4. Test integration thoroughly
5. Switch traffic to new contract
6. Archive old contract data

## Best Practices

1. **Idempotency**: Make operations idempotent where possible
2. **Event Emission**: Always emit events for state changes
3. **Gas Optimization**: Batch operations when possible
4. **Error Messages**: Provide clear error messages
5. **Documentation**: Keep integration docs updated
6. **Versioning**: Use semantic versioning for contract updates
7. **Testing**: Maintain comprehensive integration tests
8. **Monitoring**: Set up alerts for critical metrics

## Support

For integration support:
- Review contract source code in `contracts/community/src/`
- Check test examples in `contracts/community/src/tests.rs`
- Refer to main README at `contracts/community/README.md`
