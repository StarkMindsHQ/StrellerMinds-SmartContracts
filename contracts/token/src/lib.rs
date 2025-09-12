#![no_std]

mod types;
mod incentives;
mod interface;

#[cfg(test)]
mod incentive_tests;

#[cfg(test)]
mod incentive_integration_tests;

use soroban_sdk::{contract, contractimpl, contracterror, symbol_short, Address, Env, Symbol, String, Vec};
use shared::reentrancy_guard::ReentrancyLock;
use types::{
    TokenReward, Achievement, UserAchievement, StakingPool, UserStake, BurnTransaction,
    TokenomicsConfig, UserStats, LeaderboardEntry, LeaderboardCategory, IncentiveEvent,
    GlobalStats, PremiumAccess, PremiumFeature, ReferralData
};
use incentives::IncentiveManager;
use interface::TokenTrait;

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
    AchievementNotFound = 7,
    StakingPoolNotFound = 8,
    InsufficientStake = 9,
    StakeLocked = 10,
    BurnTransactionNotFound = 11,
    EventNotFound = 12,
    ReferralNotFound = 13,
    ContractPaused = 14,
}

#[contractimpl]
impl Token {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        let _guard = ReentrancyLock::new(&env);
        if admin_exists(&env) {
            return Err(Error::AlreadyInitialized);
        }
        
        admin.require_auth();
        env.storage().instance().set(&AdminKey, &admin);
        
        // Initialize incentive system
        IncentiveManager::initialize(&env, &admin)?;
        
        Ok(())
    }

    pub fn initialize_incentives(env: Env, admin: Address) -> Result<(), Error> {
        let _guard = ReentrancyLock::new(&env);
        IncentiveManager::initialize(&env, &admin)
    }

    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
        let _guard = ReentrancyLock::new(&env);
        let admin = get_admin(&env)?;
        admin.require_auth();
        
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

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        let _guard = ReentrancyLock::new(&env);
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
        let _guard = ReentrancyLock::new(&env);
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

    // === Incentive System Methods ===

    pub fn reward_course_completion(
        env: Env,
        user: Address,
        course_id: String,
        completion_percentage: u32,
    ) -> Result<i128, Error> {
        let _guard = ReentrancyLock::new(&env);
        IncentiveManager::reward_course_completion(&env, &user, &course_id, completion_percentage)
    }

    pub fn reward_module_completion(
        env: Env,
        user: Address,
        course_id: String,
        module_id: String,
    ) -> Result<i128, Error> {
        let _guard = ReentrancyLock::new(&env);
        IncentiveManager::reward_module_completion(&env, &user, &course_id, &module_id)
    }

    pub fn create_achievement(
        env: Env,
        admin: Address,
        achievement: Achievement,
    ) -> Result<String, Error> {
        let _guard = ReentrancyLock::new(&env);
        IncentiveManager::create_achievement(&env, &admin, achievement)
    }

    pub fn check_achievements(env: Env, user: Address) -> Result<Vec<String>, Error> {
        let _guard = ReentrancyLock::new(&env);
        IncentiveManager::check_achievements(&env, &user)
    }

    pub fn create_staking_pool(
        env: Env,
        admin: Address,
        pool: StakingPool,
    ) -> Result<String, Error> {
        let _guard = ReentrancyLock::new(&env);
        IncentiveManager::create_staking_pool(&env, &admin, pool)
    }

    pub fn stake_tokens(
        env: Env,
        user: Address,
        pool_id: String,
        amount: i128,
    ) -> Result<(), Error> {
        let _guard = ReentrancyLock::new(&env);
        IncentiveManager::stake_tokens(&env, &user, &pool_id, amount)
    }

    pub fn burn_for_upgrade(
        env: Env,
        user: Address,
        amount: i128,
        certificate_id: String,
        upgrade_type: String,
    ) -> Result<String, Error> {
        let _guard = ReentrancyLock::new(&env);
        IncentiveManager::burn_for_upgrade(&env, &user, amount, &certificate_id, &upgrade_type)
    }
}

// Storage keys
const BALANCE_KEY: Symbol = symbol_short!("BALANCE");
const AdminKey: Symbol = symbol_short!("ADMIN");

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
    env.storage().instance().has(&AdminKey)
}

fn get_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&AdminKey)
        .ok_or(Error::NotInitialized)
}
