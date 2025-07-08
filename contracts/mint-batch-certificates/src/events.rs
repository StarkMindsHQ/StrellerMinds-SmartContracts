use soroban_sdk::{Address, Env, Symbol};
use crate::certificate::CertificateData;

// Event topics
const CERTIFICATE_MINTED: &str = "CERTIFICATE_MINTED";
const CERTIFICATE_REVOKED: &str = "CERTIFICATE_REVOKED";
const BATCH_MINT_COMPLETED: &str = "BATCH_MINT_COMPLETED";
const ISSUER_ADDED: &str = "ISSUER_ADDED";
const ISSUER_REMOVED: &str = "ISSUER_REMOVED";
const CONTRACT_INITIALIZED: &str = "CONTRACT_INITIALIZED";
const ERROR_OCCURRED: &str = "ERROR_OCCURRED";

// Emit event when a certificate is minted
pub fn emit_certificate_minted(env: &Env, issuer: &Address, owner: &Address, certificate: &CertificateData) {
    let topics = (
        Symbol::new(env, CERTIFICATE_MINTED),
        issuer.clone(),
        owner.clone(),
        certificate.id,
    );
    
    // Convert CertificateData to a tuple for event emission
    let data = (
        certificate.id,
        certificate.metadata_hash.clone(),
        certificate.valid_from,
        certificate.valid_until,
        certificate.revocable,
        certificate.cert_type.to_u32(),
    );
    
    env.events().publish(topics, data);
}

// Emit event when a certificate is revoked
pub fn emit_certificate_revoked(env: &Env, revoker: &Address, certificate_id: u64) {
    let topics = (
        Symbol::new(env, CERTIFICATE_REVOKED),
        revoker.clone(),
        certificate_id,
    );
    
    env.events().publish(topics, ());
}

// Emit event when a batch mint operation is completed
pub fn emit_batch_mint_completed(
    env: &Env, 
    issuer: &Address, 
    total_count: u32, 
    success_count: u32, 
    failure_count: u32
) {
    let topics = (
        Symbol::new(env, BATCH_MINT_COMPLETED),
        issuer.clone(),
    );
    
    let data = (total_count, success_count, failure_count);
    env.events().publish(topics, data);
}

// Emit event when an issuer is added
pub fn emit_issuer_added(env: &Env, admin: &Address, issuer: &Address) {
    let topics = (
        Symbol::new(env, ISSUER_ADDED),
        admin.clone(),
    );
    
    env.events().publish(topics, issuer);
}

// Emit event when an issuer is removed
pub fn emit_issuer_removed(env: &Env, admin: &Address, issuer: &Address) {
    let topics = (
        Symbol::new(env, ISSUER_REMOVED),
        admin.clone(),
    );
    
    env.events().publish(topics, issuer);
}

// Emit event when the contract is initialized
pub fn emit_contract_initialized(env: &Env, admin: &Address, max_batch_size: u32) {
    let topics = (
        Symbol::new(env, CONTRACT_INITIALIZED),
        admin.clone(),
    );
    
    env.events().publish(topics, max_batch_size);
}

// Emit event when an error occurs
pub fn emit_error_event(env: &Env, function: &str, error_code: u32, error_message: &str, context: Option<u64>) {
    let topics = (
        Symbol::new(env, ERROR_OCCURRED),
        Symbol::new(env, function),
    );
    // Context can be certificate ID or None
    let data = (error_code, error_message, context);
    env.events().publish(topics, data);
}
