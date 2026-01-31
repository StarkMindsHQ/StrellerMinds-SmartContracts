#![no_std]
mod interface;
mod types;

#[cfg(test)]
mod test;

use shared::access_control::AccessControl;
use shared::roles::Permission;
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Address, Env, String, Symbol, Vec,
};

#[contract]
pub struct Token;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InsufficientBalance = 4,
    Unauthorized = 5,
    InvalidInput = 6,
    FlashLoanActive = 7,
    FlashLoanNotRepaid = 8,
    SlippageExceeded = 9,
    PoolNotFound = 10,
    InsufficientLiquidity = 11,
    LockPeriodNotExpired = 12,
    InvalidRecipient = 13,
    BatchOperationFailed = 14,
}

#[contractimpl]
impl Token {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if admin_exists(&env) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();
        // Initialize centralized RBAC (grants SuperAdmin to admin)
        let _ = AccessControl::initialize(&env, &admin);
        env.storage().instance().set(&ADMIN_KEY, &admin);

        Ok(())
    }

    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
        let admin = get_admin(&env)?;
        admin.require_auth();
        // RBAC: require token mint permission
        if AccessControl::require_permission(&env, &admin, &Permission::MintTokens).is_err() {
            return Err(Error::Unauthorized);
        }

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let balance = get_balance(&env, &to);
        set_balance(&env, &to, balance + amount);

        Ok(())
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        get_balance(&env, &id)
    }

    pub fn approve(env: Env, from: Address, spender: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        if amount < 0 {
            return Err(Error::InvalidAmount);
        }

        set_allowance(&env, &from, &spender, amount);
        Ok(())
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        get_allowance(&env, &from, &spender)
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        spender.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let allowance = get_allowance(&env, &from, &spender);
        if allowance < amount {
            return Err(Error::InsufficientBalance);
        }

        let from_balance = get_balance(&env, &from);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        set_allowance(&env, &from, &spender, allowance - amount);
        set_balance(&env, &from, from_balance - amount);
        let to_balance = get_balance(&env, &to);
        set_balance(&env, &to, to_balance + amount);

        Ok(())
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let from_balance = get_balance(&env, &from);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        set_balance(&env, &from, from_balance - amount);
        let to_balance = get_balance(&env, &to);
        set_balance(&env, &to, to_balance + amount);

        Ok(())
    }

    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let from_balance = get_balance(&env, &from);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        set_balance(&env, &from, from_balance - amount);
        Ok(())
    }

    // Simplified incentive methods that just return basic values
    pub fn reward_course_completion(
        env: Env,
        user: Address,
        _course_id: String,
        completion_percentage: u32,
    ) -> Result<i128, Error> {
        if completion_percentage == 0 {
            return Err(Error::InvalidInput);
        }

        // Simple reward calculation: 100 tokens per 10% completion
        let reward = (completion_percentage / 10) as i128 * 100;

        let balance = get_balance(&env, &user);
        set_balance(&env, &user, balance + reward);

        Ok(reward)
    }

    pub fn reward_module_completion(
        env: Env,
        user: Address,
        _course_id: String,
        _module_id: String,
        completion_percentage: u32,
    ) -> Result<i128, Error> {
        if completion_percentage == 0 {
            return Err(Error::InvalidInput);
        }

        // Simple reward calculation: 50 tokens per 10% completion
        let reward = (completion_percentage / 10) as i128 * 50;

        let balance = get_balance(&env, &user);
        set_balance(&env, &user, balance + reward);

        Ok(reward)
    }

    pub fn create_achievement(
        env: Env,
        _title: String,
        _description: String,
        reward_amount: i128,
    ) -> Result<String, Error> {
        if reward_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Simple achievement ID generation
        let achievement_id = String::from_str(&env, "achievement_1");
        Ok(achievement_id)
    }

    pub fn check_achievements(env: Env, _user: Address) -> Result<Vec<String>, Error> {
        // Return empty list for now
        let achievements = Vec::new(&env);
        Ok(achievements)
    }

    pub fn create_staking_pool(env: Env, _name: String, apy: u32) -> Result<String, Error> {
        if apy == 0 {
            return Err(Error::InvalidInput);
        }

        // Simple pool ID generation
        let pool_id = String::from_str(&env, "pool_1");
        Ok(pool_id)
    }

    pub fn stake_tokens(
        env: Env,
        user: Address,
        _pool_id: String,
        amount: i128,
    ) -> Result<String, Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let balance = get_balance(&env, &user);
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        // Simple staking: just reduce balance
        set_balance(&env, &user, balance - amount);

        let stake_id = String::from_str(&env, "stake_1");
        Ok(stake_id)
    }

    pub fn burn_for_upgrade(
        env: Env,
        user: Address,
        _course_id: String,
        _module_id: String,
        amount: i128,
        _upgrade_type: String,
    ) -> Result<String, Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let balance = get_balance(&env, &user);
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        // Burn tokens for upgrade
        set_balance(&env, &user, balance - amount);

        let burn_id = String::from_str(&env, "burn_1");
        Ok(burn_id)
    }

    // ==================== BATCH OPERATIONS ====================

    /// Batch transfer to multiple recipients with gas optimization
    pub fn batch_transfer(
        env: Env,
        from: Address,
        recipients: Vec<Address>,
        amounts: Vec<i128>,
    ) -> Result<u32, Error> {
        from.require_auth();

        if recipients.len() != amounts.len() {
            return Err(Error::InvalidInput);
        }

        if recipients.is_empty() {
            return Err(Error::InvalidInput);
        }

        // Calculate total amount needed
        let mut total_amount: i128 = 0;
        for i in 0..amounts.len() {
            let amt = amounts.get(i).unwrap();
            if amt <= 0 {
                return Err(Error::InvalidAmount);
            }
            total_amount += amt;
        }

        // Check sender has sufficient balance
        let from_balance = get_balance(&env, &from);
        if from_balance < total_amount {
            return Err(Error::InsufficientBalance);
        }

        // Perform batch transfer
        set_balance(&env, &from, from_balance - total_amount);

        let mut success_count: u32 = 0;
        for i in 0..recipients.len() {
            let recipient = recipients.get(i).unwrap();
            let amount = amounts.get(i).unwrap();

            let recipient_balance = get_balance(&env, &recipient);
            set_balance(&env, &recipient, recipient_balance + amount);
            success_count += 1;
        }

        Ok(success_count)
    }

    // ==================== FLASH LOAN PROTECTION ====================

    /// Execute a flash loan with protection mechanisms
    pub fn flash_loan(
        env: Env,
        borrower: Address,
        amount: i128,
        fee_bps: u32, // basis points (100 = 1%)
    ) -> Result<(), Error> {
        borrower.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Check if another flash loan is active
        if env.storage().instance().has(&FLASH_LOAN_ACTIVE_KEY) {
            return Err(Error::FlashLoanActive);
        }

        // Mark flash loan as active
        env.storage().instance().set(&FLASH_LOAN_ACTIVE_KEY, &true);

        let initial_balance = get_balance(&env, &borrower);

        // Transfer loan amount to borrower
        set_balance(&env, &borrower, initial_balance + amount);

        // Calculate fee (amount * fee_bps / 10000)
        let fee = (amount * fee_bps as i128) / 10000;
        let repay_amount = amount + fee;

        // Store expected repayment
        env.storage()
            .instance()
            .set(&FLASH_LOAN_AMOUNT_KEY, &repay_amount);
        env.storage()
            .instance()
            .set(&FLASH_LOAN_BORROWER_KEY, &borrower);

        Ok(())
    }

    /// Repay flash loan - must be called in same transaction
    pub fn repay_flash_loan(env: Env, borrower: Address) -> Result<(), Error> {
        borrower.require_auth();

        if !env.storage().instance().has(&FLASH_LOAN_ACTIVE_KEY) {
            return Err(Error::InvalidInput);
        }

        let expected_borrower: Address = env
            .storage()
            .instance()
            .get(&FLASH_LOAN_BORROWER_KEY)
            .unwrap();

        if borrower != expected_borrower {
            return Err(Error::Unauthorized);
        }

        let repay_amount: i128 = env
            .storage()
            .instance()
            .get(&FLASH_LOAN_AMOUNT_KEY)
            .unwrap();

        let borrower_balance = get_balance(&env, &borrower);
        if borrower_balance < repay_amount {
            return Err(Error::FlashLoanNotRepaid);
        }

        // Deduct repayment
        set_balance(&env, &borrower, borrower_balance - repay_amount);

        // Clear flash loan state
        env.storage().instance().remove(&FLASH_LOAN_ACTIVE_KEY);
        env.storage().instance().remove(&FLASH_LOAN_AMOUNT_KEY);
        env.storage().instance().remove(&FLASH_LOAN_BORROWER_KEY);

        Ok(())
    }

    // ==================== ADVANCED STAKING POOLS ====================

    /// Create advanced staking pool with yield farming
    pub fn create_advanced_staking_pool(
        env: Env,
        pool_id: String,
        name: String,
        apy_bps: u32,       // basis points (1000 = 10%)
        lock_duration: u64, // seconds
        min_stake: i128,
        compound_enabled: bool,
    ) -> Result<(), Error> {
        let admin = get_admin(&env)?;
        admin.require_auth();

        if apy_bps == 0 || min_stake <= 0 {
            return Err(Error::InvalidInput);
        }

        let pool = AdvancedStakingPool {
            id: pool_id.clone(),
            name,
            apy_bps,
            lock_duration,
            min_stake,
            total_staked: 0,
            total_rewards_paid: 0,
            compound_enabled,
            created_at: env.ledger().timestamp(),
            is_active: true,
        };

        env.storage()
            .instance()
            .set(&(STAKING_POOL_KEY, pool_id), &pool);

        Ok(())
    }

    /// Stake tokens with yield farming
    pub fn stake_advanced(
        env: Env,
        user: Address,
        pool_id: String,
        amount: i128,
        auto_compound: bool,
    ) -> Result<(), Error> {
        user.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut pool: AdvancedStakingPool = env
            .storage()
            .instance()
            .get(&(STAKING_POOL_KEY, pool_id.clone()))
            .ok_or(Error::PoolNotFound)?;

        if !pool.is_active {
            return Err(Error::InvalidInput);
        }

        if amount < pool.min_stake {
            return Err(Error::InvalidAmount);
        }

        let user_balance = get_balance(&env, &user);
        if user_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        // Create or update stake
        let stake = AdvancedStake {
            user: user.clone(),
            pool_id: pool_id.clone(),
            amount,
            staked_at: env.ledger().timestamp(),
            unlock_at: env.ledger().timestamp() + pool.lock_duration,
            rewards_earned: 0,
            last_claim: env.ledger().timestamp(),
            auto_compound,
        };

        // Update balances
        set_balance(&env, &user, user_balance - amount);
        pool.total_staked += amount;

        env.storage()
            .instance()
            .set(&(STAKING_POOL_KEY, pool_id.clone()), &pool);
        env.storage()
            .instance()
            .set(&(USER_STAKE_KEY, user.clone(), pool_id), &stake);

        Ok(())
    }

    /// Calculate and claim staking rewards
    pub fn claim_staking_rewards(env: Env, user: Address, pool_id: String) -> Result<i128, Error> {
        user.require_auth();

        let mut stake: AdvancedStake = env
            .storage()
            .instance()
            .get(&(USER_STAKE_KEY, user.clone(), pool_id.clone()))
            .ok_or(Error::InvalidInput)?;

        let pool: AdvancedStakingPool = env
            .storage()
            .instance()
            .get(&(STAKING_POOL_KEY, pool_id.clone()))
            .ok_or(Error::PoolNotFound)?;

        let current_time = env.ledger().timestamp();
        let time_staked = current_time - stake.last_claim;

        // Calculate rewards: (amount * apy_bps * time_staked) / (10000 * SECONDS_PER_YEAR)
        let seconds_per_year: u64 = 31536000;
        let rewards = (stake.amount * pool.apy_bps as i128 * time_staked as i128)
            / (10000 * seconds_per_year as i128);

        if rewards > 0 {
            if stake.auto_compound && pool.compound_enabled {
                // Auto-compound: add rewards to staked amount
                stake.amount += rewards;
            } else {
                // Pay out rewards
                let user_balance = get_balance(&env, &user);
                set_balance(&env, &user, user_balance + rewards);
            }

            stake.rewards_earned += rewards;
            stake.last_claim = current_time;

            env.storage()
                .instance()
                .set(&(USER_STAKE_KEY, user, pool_id), &stake);
        }

        Ok(rewards)
    }

    /// Unstake tokens (after lock period)
    pub fn unstake_advanced(env: Env, user: Address, pool_id: String) -> Result<i128, Error> {
        user.require_auth();

        let stake: AdvancedStake = env
            .storage()
            .instance()
            .get(&(USER_STAKE_KEY, user.clone(), pool_id.clone()))
            .ok_or(Error::InvalidInput)?;

        let current_time = env.ledger().timestamp();
        if current_time < stake.unlock_at {
            return Err(Error::LockPeriodNotExpired);
        }

        let mut pool: AdvancedStakingPool = env
            .storage()
            .instance()
            .get(&(STAKING_POOL_KEY, pool_id.clone()))
            .ok_or(Error::PoolNotFound)?;

        // Calculate final rewards
        let time_staked = current_time - stake.last_claim;
        let seconds_per_year: u64 = 31536000;
        let final_rewards = (stake.amount * pool.apy_bps as i128 * time_staked as i128)
            / (10000 * seconds_per_year as i128);

        let total_return = stake.amount + final_rewards;

        // Update balances
        let user_balance = get_balance(&env, &user);
        set_balance(&env, &user, user_balance + total_return);

        pool.total_staked -= stake.amount;
        pool.total_rewards_paid += final_rewards;

        // Remove stake
        env.storage()
            .instance()
            .remove(&(USER_STAKE_KEY, user, pool_id.clone()));
        env.storage()
            .instance()
            .set(&(STAKING_POOL_KEY, pool_id), &pool);

        Ok(total_return)
    }

    // ==================== AUTOMATED MARKET MAKER (AMM) ====================

    /// Create liquidity pool for AMM
    pub fn create_liquidity_pool(
        env: Env,
        token_a: Address,
        token_b: Address,
        fee_bps: u32, // basis points (30 = 0.3%)
    ) -> Result<String, Error> {
        let admin = get_admin(&env)?;
        admin.require_auth();

        let pool_count: u32 = env.storage().instance().get(&POOL_COUNT_KEY).unwrap_or(0);

        let pool_id = String::from_str(&env, "amm_pool_");

        let pool = LiquidityPool {
            id: pool_id.clone(),
            token_a,
            token_b,
            reserve_a: 0,
            reserve_b: 0,
            fee_bps,
            total_shares: 0,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&(AMM_POOL_KEY, pool_id.clone()), &pool);
        env.storage()
            .instance()
            .set(&POOL_COUNT_KEY, &(pool_count + 1));

        Ok(pool_id)
    }

    /// Add liquidity to AMM pool
    pub fn add_liquidity(
        env: Env,
        pool_id: String,
        user: Address,
        amount_a: i128,
        amount_b: i128,
        min_shares: i128,
    ) -> Result<i128, Error> {
        user.require_auth();

        if amount_a <= 0 || amount_b <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut pool: LiquidityPool = env
            .storage()
            .instance()
            .get(&(AMM_POOL_KEY, pool_id.clone()))
            .ok_or(Error::PoolNotFound)?;

        let user_balance = get_balance(&env, &user);
        if user_balance < amount_a + amount_b {
            return Err(Error::InsufficientBalance);
        }

        // Calculate shares
        let shares = if pool.total_shares == 0 {
            // First liquidity provider
            int_sqrt(amount_a * amount_b)
        } else {
            // Subsequent providers - maintain ratio
            let share_a = (amount_a * pool.total_shares) / pool.reserve_a;
            let share_b = (amount_b * pool.total_shares) / pool.reserve_b;
            if share_a < share_b {
                share_a
            } else {
                share_b
            }
        };

        if shares < min_shares {
            return Err(Error::SlippageExceeded);
        }

        // Update pool
        pool.reserve_a += amount_a;
        pool.reserve_b += amount_b;
        pool.total_shares += shares;

        // Deduct from user
        set_balance(&env, &user, user_balance - amount_a - amount_b);

        // Store user's LP shares
        let current_shares = get_lp_shares(&env, &user, &pool_id);
        set_lp_shares(&env, &user, &pool_id, current_shares + shares);

        env.storage()
            .instance()
            .set(&(AMM_POOL_KEY, pool_id), &pool);

        Ok(shares)
    }

    /// Swap tokens with slippage protection
    pub fn swap(
        env: Env,
        pool_id: String,
        user: Address,
        amount_in: i128,
        token_in_is_a: bool,
        min_amount_out: i128,
    ) -> Result<i128, Error> {
        user.require_auth();

        if amount_in <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut pool: LiquidityPool = env
            .storage()
            .instance()
            .get(&(AMM_POOL_KEY, pool_id.clone()))
            .ok_or(Error::PoolNotFound)?;

        // Apply fee
        let amount_in_with_fee = amount_in - ((amount_in * pool.fee_bps as i128) / 10000);

        // Calculate output using constant product formula: x * y = k
        let (reserve_in, reserve_out) = if token_in_is_a {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };

        let amount_out = (reserve_out * amount_in_with_fee) / (reserve_in + amount_in_with_fee);

        if amount_out < min_amount_out {
            return Err(Error::SlippageExceeded);
        }

        if amount_out > reserve_out {
            return Err(Error::InsufficientLiquidity);
        }

        // Update reserves
        if token_in_is_a {
            pool.reserve_a += amount_in;
            pool.reserve_b -= amount_out;
        } else {
            pool.reserve_b += amount_in;
            pool.reserve_a -= amount_out;
        }

        // Update balances
        let user_balance = get_balance(&env, &user);
        if user_balance < amount_in {
            return Err(Error::InsufficientBalance);
        }
        set_balance(&env, &user, user_balance - amount_in + amount_out);

        env.storage()
            .instance()
            .set(&(AMM_POOL_KEY, pool_id), &pool);

        Ok(amount_out)
    }

    /// Get quote for swap without executing
    pub fn get_swap_quote(
        env: Env,
        pool_id: String,
        amount_in: i128,
        token_in_is_a: bool,
    ) -> Result<i128, Error> {
        if amount_in <= 0 {
            return Err(Error::InvalidAmount);
        }

        let pool: LiquidityPool = env
            .storage()
            .instance()
            .get(&(AMM_POOL_KEY, pool_id))
            .ok_or(Error::PoolNotFound)?;

        let amount_in_with_fee = amount_in - ((amount_in * pool.fee_bps as i128) / 10000);

        let (reserve_in, reserve_out) = if token_in_is_a {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };

        let amount_out = (reserve_out * amount_in_with_fee) / (reserve_in + amount_in_with_fee);

        Ok(amount_out)
    }
}

// ==================== STORAGE KEYS ====================

const BALANCE_KEY: Symbol = symbol_short!("BALANCE");
const ALLOWANCE_KEY: Symbol = symbol_short!("ALLOW");
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const FLASH_LOAN_ACTIVE_KEY: Symbol = symbol_short!("FL_ACT");
const FLASH_LOAN_AMOUNT_KEY: Symbol = symbol_short!("FL_AMT");
const FLASH_LOAN_BORROWER_KEY: Symbol = symbol_short!("FL_BORR");
const STAKING_POOL_KEY: Symbol = symbol_short!("STK_POOL");
const USER_STAKE_KEY: Symbol = symbol_short!("USR_STK");
const AMM_POOL_KEY: Symbol = symbol_short!("AMM_POOL");
const POOL_COUNT_KEY: Symbol = symbol_short!("PL_CNT");
const LP_SHARES_KEY: Symbol = symbol_short!("LP_SHR");

// ==================== DATA STRUCTURES ====================

use soroban_sdk::contracttype;

#[derive(Clone)]
#[contracttype]
struct AdvancedStakingPool {
    id: String,
    name: String,
    apy_bps: u32,
    lock_duration: u64,
    min_stake: i128,
    total_staked: i128,
    total_rewards_paid: i128,
    compound_enabled: bool,
    created_at: u64,
    is_active: bool,
}

#[derive(Clone)]
#[contracttype]
struct AdvancedStake {
    user: Address,
    pool_id: String,
    amount: i128,
    staked_at: u64,
    unlock_at: u64,
    rewards_earned: i128,
    last_claim: u64,
    auto_compound: bool,
}

#[derive(Clone)]
#[contracttype]
struct LiquidityPool {
    id: String,
    token_a: Address,
    token_b: Address,
    reserve_a: i128,
    reserve_b: i128,
    fee_bps: u32,
    total_shares: i128,
    created_at: u64,
}

// Helper functions
fn get_balance(env: &Env, id: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&(BALANCE_KEY, id))
        .unwrap_or(0)
}

fn set_balance(env: &Env, id: &Address, amount: i128) {
    env.storage().instance().set(&(BALANCE_KEY, id), &amount);
}

fn admin_exists(env: &Env) -> bool {
    env.storage().instance().has(&ADMIN_KEY)
}

fn get_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&ADMIN_KEY)
        .ok_or(Error::NotInitialized)
}

fn get_allowance(env: &Env, from: &Address, spender: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&(ALLOWANCE_KEY, from, spender))
        .unwrap_or(0)
}

fn set_allowance(env: &Env, from: &Address, spender: &Address, amount: i128) {
    env.storage()
        .instance()
        .set(&(ALLOWANCE_KEY, from, spender), &amount);
}

fn int_sqrt(n: i128) -> i128 {
    if n == 0 {
        return 0;
    }

    let mut x = n;
    let mut y = (x + 1) / 2;

    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }

    x
}

fn get_lp_shares(env: &Env, user: &Address, pool_id: &String) -> i128 {
    env.storage()
        .instance()
        .get(&(LP_SHARES_KEY, user.clone(), pool_id.clone()))
        .unwrap_or(0)
}

fn set_lp_shares(env: &Env, user: &Address, pool_id: &String, amount: i128) {
    env.storage()
        .instance()
        .set(&(LP_SHARES_KEY, user.clone(), pool_id.clone()), &amount);
}
