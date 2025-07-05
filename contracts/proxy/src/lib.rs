#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env, Address, BytesN, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Implementation,
    Admin,
    RollbackStack,
}

#[contract]
pub struct Proxy;

#[contractimpl]
impl Proxy {
    /// Initialize proxy with admin and implementation address
    pub fn initialize(env: Env, admin: Address, implementation: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Implementation, &implementation);
        env.storage().instance().set(&DataKey::RollbackStack, &Vec::<Address>::new(&env));
    }

    /// Upgrade implementation (admin only)
    pub fn upgrade(env: Env, new_implementation: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        let current: Address = env.storage().instance().get(&DataKey::Implementation).unwrap();
        let mut stack: Vec<Address> = env.storage().instance().get(&DataKey::RollbackStack).unwrap();
        stack.push_back(current.clone());
        env.storage().instance().set(&DataKey::RollbackStack, &stack);
        env.storage().instance().set(&DataKey::Implementation, &new_implementation);
    }

    /// Rollback to previous implementation (admin only)
    pub fn rollback(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        let mut stack: Vec<Address> = env.storage().instance().get(&DataKey::RollbackStack).unwrap();
        let prev = stack.pop_back().expect("No previous implementation");
        env.storage().instance().set(&DataKey::RollbackStack, &stack);
        env.storage().instance().set(&DataKey::Implementation, &prev);
    }

    /// Get current implementation address
    pub fn get_implementation(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Implementation).unwrap()
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }
}

// Note: Actual call delegation is handled by Soroban host, not in userland Rust.
// For a real proxy, you would use Soroban's host functions to forward calls.
