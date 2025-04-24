#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, symbol_short, Address, Env, Symbol};

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
}

#[contractimpl]
impl Token {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if admin_exists(&env) {
            return Err(Error::AlreadyInitialized);
        }
        
        admin.require_auth();
        env.storage().instance().set(&AdminKey, &admin);
        
        Ok(())
    }

    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
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
