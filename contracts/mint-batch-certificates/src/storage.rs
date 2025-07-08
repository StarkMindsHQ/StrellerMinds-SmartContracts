use soroban_sdk::{Address, Env, Map, Vec, Symbol, symbol_short, String as SorobanString};
use crate::certificate::CertificateData;
use crate::error::Error;

// Storage keys using symbol_short
const ADMIN: Symbol = symbol_short!("ADMIN");
const ISSUERS: Symbol = symbol_short!("ISSUERS");
const CONFIG: Symbol = symbol_short!("CONFIG");
const INITIALIZED: Symbol = symbol_short!("INIT");

// Configuration keys
const MAX_BATCH_SIZE: Symbol = symbol_short!("MAX_BS");

// Certificate and owner keys
const CERT_KEY: Symbol = symbol_short!("CERT");
const OWNER_KEY: Symbol = symbol_short!("OWNER");

// Initialize contract storage
pub fn initialize(env: &Env, admin: &Address, max_batch_size: u32) {
    if is_initialized(env) {
        env.panic_with_error(Error::AlreadyInitialized);
    }
    
    // Set admin
    env.storage().instance().set(&ADMIN, admin);
    
    // Set configuration
    let mut config = Map::new(env);
    config.set(MAX_BATCH_SIZE, max_batch_size);
    env.storage().instance().set(&CONFIG, &config);
    
    // Mark as initialized
    env.storage().instance().set(&INITIALIZED, &true);
}

// Check if contract is initialized
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INITIALIZED)
}

// Get admin address
pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&ADMIN).unwrap_or_else(|| env.panic_with_error(Error::NotInitialized))
}

// Get max batch size
pub fn get_max_batch_size(env: &Env) -> u32 {
    let config: Map<Symbol, u32> = env.storage().instance().get(&CONFIG).unwrap_or_else(|| env.panic_with_error(Error::NotInitialized));
    config.get(MAX_BATCH_SIZE).unwrap_or(10) // Default to 10 if not set
}

// Check if address is an issuer
pub fn is_issuer(env: &Env, address: &Address) -> bool {
    if !is_initialized(env) {
        env.panic_with_error(Error::NotInitialized);
    }
    
    if let Some(issuers) = env.storage().instance().get::<Symbol, Vec<Address>>(&ISSUERS) {
        issuers.contains(address)
    } else {
        false
    }
}

// Add an issuer
pub fn add_issuer(env: &Env, address: &Address) {
    if !is_initialized(env) {
        env.panic_with_error(Error::NotInitialized);
    }
    
    let mut issuers = env.storage().instance().get::<Symbol, Vec<Address>>(&ISSUERS).unwrap_or_else(|| Vec::new(env));
    
    if !issuers.contains(address) {
        issuers.push_back(address.clone());
        env.storage().instance().set(&ISSUERS, &issuers);
    }
}

// Remove an issuer
pub fn remove_issuer(env: &Env, address: &Address) {
    if !is_initialized(env) {
        env.panic_with_error(Error::NotInitialized);
    }
    
    if let Some(mut issuers) = env.storage().instance().get::<Symbol, Vec<Address>>(&ISSUERS) {
        let mut i = 0;
        while i < issuers.len() {
            if &issuers.get(i).unwrap() == address {
                issuers.remove(i);
                env.storage().instance().set(&ISSUERS, &issuers);
                return;
            }
            i += 1;
        }
    }
}

// Check if certificate exists
pub fn certificate_exists(env: &Env, id: u64) -> bool {
    if !is_initialized(env) {
        env.panic_with_error(Error::NotInitialized);
    }
    
    // Create a key for the certificate using its ID
    let key = get_certificate_key(env, id);
    
    env.storage().persistent().has(&key)
}

// Save certificate data
pub fn save_certificate(env: &Env, owner: &Address, certificate: &CertificateData) -> Result<(), Error> {
    if !is_initialized(env) {
        return Err(Error::NotInitialized);
    }
    
    if certificate_exists(env, certificate.id) {
        return Err(Error::DuplicateCertificate);
    }
    
    // Store certificate data
    let cert_key = get_certificate_key(env, certificate.id);
    env.storage().persistent().set(&cert_key, certificate);
    
    // Store certificate ownership
    let mut owner_certs = get_owner_certificates(env, owner);
    owner_certs.push_back(certificate.id);
    
    let owner_key = get_owner_key(env, owner);
    env.storage().persistent().set(&owner_key, &owner_certs);
    
    Ok(())
}

// Get certificate data
pub fn get_certificate(env: &Env, id: u64) -> Option<CertificateData> {
    if !is_initialized(env) {
        env.panic_with_error(Error::NotInitialized);
    }
    
    let key = get_certificate_key(env, id);
    env.storage().persistent().get(&key)
}

// Get certificates owned by an address
pub fn get_owner_certificates(env: &Env, owner: &Address) -> Vec<u64> {
    if !is_initialized(env) {
        env.panic_with_error(Error::NotInitialized);
    }
    
    let key = get_owner_key(env, owner);
    env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env))
}

// Revoke a certificate
pub fn revoke_certificate(env: &Env, id: u64) -> Result<(), Error> {
    if !is_initialized(env) {
        return Err(Error::NotInitialized);
    }
    
    let cert_key = get_certificate_key(env, id);
    
    if let Some(mut cert) = env.storage().persistent().get::<Symbol, CertificateData>(&cert_key) {
        if !cert.revocable {
            return Err(Error::CertificateNotRevocable);
        }
        
        cert.valid_until = env.ledger().timestamp();
        env.storage().persistent().set(&cert_key, &cert);
        
        Ok(())
    } else {
        Err(Error::CertificateNotFound)
    }
}

// Helper function to get certificate key
fn get_certificate_key(env: &Env, id: u64) -> Symbol {
    // Use a unique key for each certificate based on its ID
    let key_str = format!("CERT_{}", id);
    Symbol::from_str(env, &SorobanString::from_str(env, &key_str))
}

// Helper function to get owner key
fn get_owner_key(env: &Env, owner: &Address) -> Symbol {
    // Use a unique key for each owner based on their address
    let key_str = format!("OWNER_{}", owner.to_string());
    Symbol::from_str(env, &SorobanString::from_str(env, &key_str))
}
