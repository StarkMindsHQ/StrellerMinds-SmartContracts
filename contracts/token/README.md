# Token Contract

## Overview
A comprehensive token management system with advanced incentive mechanisms for educational platforms. This contract handles token minting, transfers, burning, and implements a sophisticated incentive system including achievements, staking pools, and referral programs to gamify the learning experience.

## Interface

### Core Token Functions
```rust
// Initialize the contract with admin
fn initialize(env: Env, admin: Address) -> Result<(), Error>

// Initialize incentive system
fn initialize_incentives(env: Env, admin: Address) -> Result<(), Error>

// Mint tokens to an address
fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error>

// Get token balance
fn balance(env: Env, id: Address) -> i128

// Transfer tokens between addresses
fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error>

// Burn tokens from an address
fn burn(env: Env, from: Address, amount: i128) -> Result<(), Error>
```

### Incentive System Functions
```rust
// Reward course completion
fn reward_course_completion(env: Env, user: Address, course_id: String, completion_percentage: u32) -> Result<i128, Error>

// Reward module completion
fn reward_module_completion(env: Env, user: Address, course_id: String, module_id: String) -> Result<i128, Error>

// Create achievement
fn create_achievement(env: Env, admin: Address, achievement: Achievement) -> Result<String, Error>

// Check user achievements
fn check_achievements(env: Env, user: Address) -> Result<Vec<String>, Error>

// Create staking pool
fn create_staking_pool(env: Env, admin: Address, pool: StakingPool) -> Result<String, Error>

// Stake tokens in a pool
fn stake_tokens(env: Env, user: Address, pool_id: String, amount: i128) -> Result<(), Error>

// Burn tokens for certificate upgrade
fn burn_for_upgrade(env: Env, user: Address, amount: i128, certificate_id: String, upgrade_type: String) -> Result<String, Error>
```

## Events

### Token Events
- `tokens_minted`: Emitted when tokens are minted
- `tokens_transferred`: Emitted when tokens are transferred
- `tokens_burned`: Emitted when tokens are burned

### Incentive Events
- `course_completed`: Emitted when course completion is rewarded
- `module_completed`: Emitted when module completion is rewarded
- `achievement_earned`: Emitted when user earns an achievement
- `tokens_staked`: Emitted when tokens are staked
- `certificate_upgraded`: Emitted when certificate is upgraded using tokens

## Configuration

### Token Configuration
- **Token Symbol**: Educational platform token
- **Decimal Places**: Standard token decimal configuration
- **Admin Controls**: Admin-only minting and configuration

### Incentive Configuration
```rust
pub struct TokenomicsConfig {
    pub course_completion_reward: i128,
    pub module_completion_reward: i128,
    pub achievement_bonus_multiplier: u32,
    pub staking_apy_percentage: u32,
    pub referral_bonus_percentage: u32,
}
```

### Achievement Configuration
```rust
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub reward_amount: i128,
    pub criteria: AchievementCriteria,
    pub category: AchievementCategory,
}
```

### Staking Pool Configuration
```rust
pub struct StakingPool {
    pub id: String,
    pub name: String,
    pub apy_percentage: u32,
    pub min_stake_amount: i128,
    pub lock_period_seconds: u64,
    pub max_total_stake: Option<i128>,
}
```

## Testing

### Running Tests
```bash
# Run all tests for token contract
cargo test --package token

# Run specific test modules
cargo test --package token incentive_tests
cargo test --package token incentive_integration_tests
cargo test --package token test::test_token_operations
cargo test --package token test::test_incentive_system
```

### Test Coverage
- **Token Operation Tests**: Mint, transfer, burn functionality
- **Incentive Tests**: Course and module completion rewards
- **Achievement Tests**: Achievement creation and earning
- **Staking Tests**: Staking pool creation and token staking
- **Integration Tests**: Complete incentive workflow testing
- **Edge Case Tests**: Boundary conditions and error handling

## Deployment

### Prerequisites
- Admin address for contract initialization
- Incentive system configuration
- Achievement definitions
- Staking pool configurations

### Deployment Steps
1. Deploy the token contract
2. Initialize with admin address
3. Initialize incentive system
4. Create initial achievements
5. Set up staking pools
6. Configure tokenomics parameters
7. Begin token operations and incentives

### Environment Setup
- Set admin address for contract management
- Configure tokenomics parameters
- Define achievement criteria and rewards
- Set up staking pool configurations
- Configure referral system parameters

## Usage Examples

### Basic Token Operations
```rust
// Mint tokens to user
client.mint(&user, &1000)?;

// Transfer tokens
client.transfer(&from, &to, &500)?;

// Burn tokens
client.burn(&user, &100)?;

// Check balance
let balance = client.balance(&user);
```

### Course Completion Rewards
```rust
let course_id = "BLOCKCHAIN101".to_string();
let completion_percentage = 100u32;
let reward = client.reward_course_completion(&user, &course_id, completion_percentage)?;
```

### Achievement System
```rust
let achievement = Achievement {
    id: "first_course".to_string(),
    name: "First Course Completed".to_string(),
    description: "Complete your first course".to_string(),
    reward_amount: 100,
    criteria: AchievementCriteria::CourseCompletion { course_count: 1 },
    category: AchievementCategory::Milestone,
};

let achievement_id = client.create_achievement(&admin, &achievement)?;
```

### Staking Operations
```rust
let pool = StakingPool {
    id: "beginner_pool".to_string(),
    name: "Beginner Staking Pool".to_string(),
    apy_percentage: 10,
    min_stake_amount: 100,
    lock_period_seconds: 2592000, // 30 days
    max_total_stake: Some(10000),
};

let pool_id = client.create_staking_pool(&admin, &pool)?;
client.stake_tokens(&user, &pool_id, &500)?;
```

### Certificate Upgrades
```rust
let upgrade_type = "premium".to_string();
let certificate_id = "cert_123".to_string();
let upgrade_cost = 200i128;

let upgrade_id = client.burn_for_upgrade(&user, &upgrade_cost, &certificate_id, &upgrade_type)?;
```

## Data Structures

### Token Balance Storage
- **Key**: `(BALANCE_KEY, address)`
- **Value**: `i128` (token balance)
- **Storage**: Instance storage for efficient access

### Achievement Storage
- **Key**: `achievement_id`
- **Value**: `Achievement` struct
- **Storage**: Persistent storage for achievement definitions

### Staking Storage
- **Key**: `(user_address, pool_id)`
- **Value**: `UserStake` struct
- **Storage**: Persistent storage for staking positions

### User Statistics
```rust
pub struct UserStats {
    pub total_tokens_earned: i128,
    pub courses_completed: u32,
    pub achievements_earned: u32,
    pub total_staked: i128,
    pub referral_count: u32,
}
```

## Incentive Mechanisms

### Course Completion Rewards
- **Base Reward**: Fixed amount for course completion
- **Percentage Bonus**: Additional reward based on completion percentage
- **Streak Bonuses**: Extra rewards for consecutive completions

### Achievement System
- **Milestone Achievements**: Course completion milestones
- **Streak Achievements**: Consecutive learning streaks
- **Excellence Achievements**: High performance rewards
- **Social Achievements**: Referral and community participation

### Staking Rewards
- **APY Rewards**: Annual percentage yield on staked tokens
- **Lock Periods**: Time-based staking with varying rewards
- **Pool Types**: Different staking pools with different risk/reward profiles

### Referral Program
- **Referral Bonuses**: Rewards for successful referrals
- **Multi-Level Rewards**: Rewards for referral chains
- **Activity Bonuses**: Additional rewards for active referrers

## Related Docs
- [Token Incentive System](../docs/TOKEN_INCENTIVE_SYSTEM.md)
- [Development Guide](../docs/development.md)
- [Security Documentation](../docs/security.md)