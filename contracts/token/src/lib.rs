#![no_std]

mod types;
mod interface;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, contracterror, symbol_short, Address, Env, Symbol, String, Vec};
use shared::access_control::AccessControl;
use shared::roles::Permission;

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
        env.storage().instance().set(&AdminKey, &admin);
        
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

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) -> Result<(), Error> {
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
        course_id: String,
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
        course_id: String,
        module_id: String,
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
        title: String,
        description: String,
        reward_amount: i128,
    ) -> Result<String, Error> {
        if reward_amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        
        // Simple achievement ID generation
        let achievement_id = String::from_str(&env, "achievement_1");
        Ok(achievement_id)
    }

    pub fn check_achievements(env: Env, user: Address) -> Result<Vec<String>, Error> {
        // Return empty list for now
        let achievements = Vec::new(&env);
        Ok(achievements)
    }

    pub fn create_staking_pool(
        env: Env,
        name: String,
        apy: u32,
    ) -> Result<String, Error> {
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
        pool_id: String,
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
        course_id: String,
        module_id: String,
        amount: i128,
        upgrade_type: String,
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
}

// Storage keys
const BALANCE_KEY: Symbol = symbol_short!("BALANCE");
const ALLOWANCE_KEY: Symbol = symbol_short!("ALLOW");
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

fn get_allowance(env: &Env, from: &Address, spender: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&(ALLOWANCE_KEY, from, spender))
        .unwrap_or(0)
}

fn set_allowance(env: &Env, from: &Address, spender: &Address, amount: i128) {
    env.storage().instance().set(&(ALLOWANCE_KEY, from, spender), &amount);
}