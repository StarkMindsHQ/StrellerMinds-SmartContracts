use soroban_sdk::{Address, Env};
use crate::storage;
use crate::error::Error;

// Check if the address is the admin
pub fn is_admin(env: &Env, address: &Address) -> bool {
    if !storage::is_initialized(env) {
        env.panic_with_error(Error::NotInitialized);
    }
    
    let admin = storage::get_admin(env);
    &admin == address
}

// Check if the address is an issuer
pub fn is_issuer(env: &Env, address: &Address) -> bool {
    if !storage::is_initialized(env) {
        env.panic_with_error(Error::NotInitialized);
    }
    
    storage::is_issuer(env, address)
}

// Add an issuer (admin only)
#[allow(dead_code)]
pub fn add_issuer(env: &Env, admin: &Address, issuer: &Address) {
    if !is_admin(env, admin) {
        env.panic_with_error(Error::Unauthorized);
    }
    storage::add_issuer(env, issuer);
}

// Remove an issuer (admin only)
#[allow(dead_code)]
pub fn remove_issuer(env: &Env, admin: &Address, issuer: &Address) {
    if !is_admin(env, admin) {
        env.panic_with_error(Error::Unauthorized);
    }
    storage::remove_issuer(env, issuer);
}
