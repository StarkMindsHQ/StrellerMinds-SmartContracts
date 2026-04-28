#![no_std]

pub mod errors;
pub mod events;
pub mod storage;
pub mod storage_optimizer;
pub mod two_factor_integration;
pub mod types;

#[cfg(test)]
mod test;

use errors::CertificateError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Vec};
use types::CertificateStatus;

#[contract]
pub struct CertificateContract;

#[contractimpl]
impl CertificateContract {
    /// Initialize the certificate contract with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), CertificateError> {
        if storage::is_initialized(&env) {
            return Err(CertificateError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_initialized(&env);
        Ok(())
    }

    /// Scan all issued certificates and remove storage entries for those that have
    /// passed their `expiry_date`, freeing ledger memory (fixes #439).
    ///
    /// Only the contract admin may call this function.
    /// Returns the number of expired certificate entries that were cleaned up.
    pub fn cleanup_expired_certificates(
        env: Env,
        caller: Address,
    ) -> Result<u32, CertificateError> {
        require_initialized(&env)?;
        require_admin(&env, &caller)?;

        let now = env.ledger().timestamp();
        let all_ids = storage::get_all_certificates(&env);
        let mut remaining: Vec<BytesN<32>> = Vec::new(&env);
        let mut cleaned: u32 = 0;

        for cert_id in all_ids.iter() {
            match storage::get_certificate(&env, &cert_id) {
                Some(cert) => {
                    if cert.expiry_date > 0 && now >= cert.expiry_date {
                        // Remove the storage entry to release ledger memory
                        storage::remove_certificate(&env, &cert_id);
                        cleaned += 1;
                    } else {
                        remaining.push_back(cert_id);
                    }
                }
                None => {
                    // Already removed; drop from index silently
                }
            }
        }

        storage::set_all_certificates(&env, &remaining);
        Ok(cleaned)
    }

    /// Return the number of certificates currently tracked in the global index.
    pub fn get_certificate_count(env: Env) -> u32 {
        storage::get_all_certificates(&env).len()
    }
}

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────
fn require_admin(env: &Env, caller: &Address) -> Result<(), CertificateError> {
    caller.require_auth();
    let admin = storage::get_admin(env);
    if *caller != admin {
        return Err(CertificateError::Unauthorized);
    }
    Ok(())
}

fn require_initialized(env: &Env) -> Result<(), CertificateError> {
    if !storage::is_initialized(env) {
        return Err(CertificateError::NotInitialized);
    }
    Ok(())
}
