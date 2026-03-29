pub mod errors;

use crate::errors::ProxyError;
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Proxy;

#[contractimpl]
impl Proxy {
    pub fn initialize(
        env: Env,
        admin: Address,
        _implementation: Address,
    ) -> Result<(), ProxyError> {
        admin.require_auth();
        if env.storage().instance().has(&soroban_sdk::symbol_short!("admin")) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&soroban_sdk::symbol_short!("admin"), &admin);
        Ok(())
    }

    pub fn upgrade(env: Env, new_implementation: Address) -> Result<(), ProxyError> {
        let admin: Address = env.storage().instance().get(&soroban_sdk::symbol_short!("admin"))
            .expect("Not initialized");
        admin.require_auth();
        let _ = new_implementation;
        Ok(())
    }

    pub fn get_admin(_env: Env) -> Result<Address, ProxyError> {
        Ok(Address::from_str(&_env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"))
    }

    pub fn get_implementation(_env: Env) -> Result<Address, ProxyError> {
        Ok(Address::from_str(&_env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"))
    }
}
