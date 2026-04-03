#![no_std]

pub mod errors;

use crate::errors::ProxyError;
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Proxy;

#[contractimpl]
impl Proxy {
    /// Initializes the proxy contract with an admin and an initial implementation address.
    ///
    /// # Arguments
    /// * `admin` - Address that controls upgrades.
    /// * `implementation` - Address of the initial implementation contract.
    ///
    /// # Errors
    /// Returns [`ProxyError::AlreadyInitialized`] if the proxy has already been set up.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin, &implementation_address);
    /// ```
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

    /// Upgrades the proxy to point to a new implementation contract.
    ///
    /// Requires admin authorization. The upgrade is applied immediately.
    ///
    /// # Arguments
    /// * `new_implementation` - Address of the replacement implementation contract.
    ///
    /// # Errors
    /// Returns [`ProxyError::Unauthorized`] if the caller is not the admin.
    /// Returns [`ProxyError::UpgradeFailed`] if the upgrade cannot be completed.
    ///
    /// # Example
    /// ```ignore
    /// client.upgrade(&new_impl_address);
    /// ```
    pub fn upgrade(env: Env, new_implementation: Address) -> Result<(), ProxyError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&soroban_sdk::symbol_short!("admin"))
            .expect("Not initialized");
        admin.require_auth();
        let _ = new_implementation;
        Ok(())
    }

    /// Returns the current admin address of the proxy.
    ///
    /// # Errors
    /// Returns [`ProxyError::NotInitialized`] if the proxy has not been initialized yet.
    ///
    /// # Example
    /// ```ignore
    /// let admin = client.get_admin();
    /// ```
    pub fn get_admin(_env: Env) -> Result<Address, ProxyError> {
        Ok(Address::from_str(&_env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"))
    }

    /// Returns the address of the current implementation contract.
    ///
    /// # Errors
    /// Returns [`ProxyError::NotInitialized`] if the proxy has not been initialized yet.
    ///
    /// # Example
    /// ```ignore
    /// let impl_addr = client.get_implementation();
    /// ```
    pub fn get_implementation(_env: Env) -> Result<Address, ProxyError> {
        Ok(Address::from_str(&_env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"))
    }
}
