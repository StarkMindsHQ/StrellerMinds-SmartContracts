#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Vec, TryFromVal, Val, IntoVal, ConstructorArgs, Map as SorobanMap};

mod certificate; // Certificate data structure
mod auth;        // Authentication utilities
mod storage;     // Storage patterns
mod events;      // Event emission
mod error;       // Error types
mod test;        // Test utilities

use certificate::CertificateData;
use error::{Error, MintResult};
use crate::events::emit_error_event;
// use shared::reentrancy_guard::ReentrancyLock;

#[contract]
pub struct CertificateContract;

impl IntoVal<Env, Vec<Val>> for CertificateContract {
    fn into_val(&self, env: &Env) -> Vec<Val> {
        Vec::new(env)
    }
}

impl ConstructorArgs for CertificateContract {}

#[contractimpl]
impl CertificateContract {
    // Initialize the contract with an admin and max batch size
    pub fn initialize(env: &Env, admin: Address, max_batch_size: u32) -> Result<(), Error> {
        if storage::is_initialized(env) {
            return Err(Error::AlreadyInitialized);
        }
        
        storage::initialize(env, &admin, max_batch_size);
        events::emit_contract_initialized(env, &admin, max_batch_size);
        
        Ok(())
    }
    
    // Add an authorized issuer
    pub fn add_issuer(env: &Env, admin: Address, issuer: Address) -> Result<(), Error> {
        admin.require_auth();
        
        if !auth::is_admin(env, &admin) {
            return Err(Error::Unauthorized);
        }
        
        storage::add_issuer(env, &issuer);
        events::emit_issuer_added(env, &admin, &issuer);
        
        Ok(())
    }
    
    // Remove an authorized issuer
    pub fn remove_issuer(env: &Env, admin: Address, issuer: Address) -> Result<(), Error> {
        admin.require_auth();
        
        if !auth::is_admin(env, &admin) {
            return Err(Error::Unauthorized);
        }
        
        storage::remove_issuer(env, &issuer);
        events::emit_issuer_removed(env, &admin, &issuer);
        
        Ok(())
    }
    
    // Mint a single certificate
    pub fn mint_single_certificate(
        env: &Env,
        issuer: Address,
        owner: Address,
        certificate: CertificateData
    ) -> Result<(), Error> {
        // let _guard = ReentrancyLock::new(env);
        issuer.require_auth();
        
        if !auth::is_issuer(env, &issuer) {
            emit_error_event(env, "mint_single_certificate", Error::Unauthorized as u32, Error::Unauthorized.message(), Some(certificate.id));
            return Err(Error::Unauthorized);
        }
        
        // Validate certificate data
        if !certificate.validate(env) {
            emit_error_event(env, "mint_single_certificate", Error::InvalidTimeRange as u32, Error::InvalidTimeRange.message(), Some(certificate.id));
            return Err(Error::InvalidTimeRange);
        }
        
        // Check for duplicate certificate
        if storage::certificate_exists(env, certificate.id) {
            emit_error_event(env, "mint_single_certificate", Error::DuplicateCertificate as u32, Error::DuplicateCertificate.message(), Some(certificate.id));
            return Err(Error::DuplicateCertificate);
        }
        
        // Save certificate with one retry on storage error
        match storage::save_certificate(env, &owner, &certificate) {
            Ok(()) => {},
            Err(e) if e == Error::StorageError => {
                emit_error_event(env, "mint_single_certificate", e as u32, "Storage error, retrying once", Some(certificate.id));
                // Retry once
                match storage::save_certificate(env, &owner, &certificate) {
                    Ok(()) => {},
                    Err(e2) => {
                        emit_error_event(env, "mint_single_certificate", e2 as u32, e2.message(), Some(certificate.id));
                        return Err(e2);
                    }
                }
            },
            Err(e) => {
                emit_error_event(env, "mint_single_certificate", e as u32, e.message(), Some(certificate.id));
                return Err(e);
            }
        }
        
        // Emit event
        events::emit_certificate_minted(env, &issuer, &owner, &certificate);
        
        Ok(())
    }
    
    /// Helper to split a batch into optimal sub-batches based on gas estimation
    pub fn split_into_optimal_batches(
        env: &Env,
        owners: Vec<Address>,
        certificates: Vec<CertificateData>,
        target_gas_limit: u64
    ) -> Vec<(Vec<Address>, Vec<CertificateData>)> {
        let mut batches = Vec::new(env);
        let mut start = 0u32;
         let total = certificates.len();
        while start < total {
            let remaining = total - start;
            let batch_owners = owners.slice(start..total);
            let batch_certs = certificates.slice(start..total);
            let (_, optimal_size) = Self::estimate_gas_for_batch(
                env,
                Address::from_str(&env, "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM"), // dummy issuer for estimation
                batch_owners.clone(),
                batch_certs.clone(),
                target_gas_limit,
            );
            let end = (start + optimal_size).min(total);
            batches.push_back((owners.slice(start..end), certificates.slice(start..end)));
            start = end;
        }
        batches
    }

    // Mint multiple certificates in a batch (dynamically optimized)
    pub fn mint_batch_certificates_dynamic(
        env: &Env,
        issuer: Address,
        owners: Vec<Address>,
        certificates: Vec<CertificateData>,
        target_gas_limit: u64
    ) -> Vec<MintResult> {
        let mut all_results = Vec::new(env);
        let batches = Self::split_into_optimal_batches(env, owners, certificates, target_gas_limit);
        for (batch_owners, batch_certs) in batches.iter() {
            let results = Self::mint_batch_certificates(env, issuer.clone(), batch_owners, batch_certs);
            for r in results.iter() {
                all_results.push_back(r);
            }
        }
        all_results
    }
    
    // Revoke a certificate
    pub fn revoke_certificate(
        env: &Env,
        issuer: Address,
        certificate_id: u64
    ) -> Result<(), Error> {
        // let _guard = ReentrancyLock::new(env);
        issuer.require_auth();
        
        if !auth::is_issuer(env, &issuer) {
            return Err(Error::Unauthorized);
        }
        
        // Revoke the certificate
        storage::revoke_certificate(env, certificate_id)?;
        
        // Emit event
        events::emit_certificate_revoked(env, &issuer, certificate_id);
        
        Ok(())
    }
    
    // Get certificate data
    pub fn get_certificate(env: &Env, certificate_id: u64) -> Option<CertificateData> {
        storage::get_certificate(env, certificate_id)
    }
    
    // Get certificates owned by an address
    pub fn get_owner_certificates(env: &Env, owner: Address) -> Vec<u64> {
        storage::get_owner_certificates(env, &owner)
    }
    
    // Check if an address is an authorized issuer
    pub fn is_issuer(env: &Env, address: Address) -> bool {
        storage::is_issuer(env, &address)
    }

    /// Estimate gas usage for a given batch size and return the optimal batch size for a target gas limit
    pub fn estimate_gas_for_batch(
        _env: &Env,
        _issuer: Address,
        _owners: Vec<Address>,
        certificates: Vec<CertificateData>,
        target_gas_limit: u64
    ) -> (u64, u32) {
        // Simulate the batch minting logic to estimate gas usage
        // For demonstration, we use a simple linear model: base + per_certificate * batch_size
        // In practice, you would use actual gas metering APIs if available
        let base_gas: u64 = 10_000; // base cost for batch operation
        let per_certificate_gas: u64 = 5_000; // estimated cost per certificate
        let batch_size = certificates.len() as u32;
        let estimated_gas = base_gas + per_certificate_gas * batch_size as u64;

        // Calculate the largest batch size that fits within the target gas limit
        let mut optimal_batch_size = batch_size;
        if estimated_gas > target_gas_limit {
            optimal_batch_size = ((target_gas_limit - base_gas) / per_certificate_gas) as u32;
            // Ensure optimal_batch_size is at least 1 and not more than the original batch_size
            optimal_batch_size = optimal_batch_size.max(1).min(batch_size);
        }
        
        // Recalculate estimated gas for the optimal batch size
        let final_estimated_gas = base_gas + per_certificate_gas * optimal_batch_size as u64;
        (final_estimated_gas, optimal_batch_size)
    }
}

impl TryFromVal<Env, Val> for MintResult {
    type Error = soroban_sdk::Error;

    fn try_from_val(_env: &Env, _val: &Val) -> Result<Self, Self::Error> {
        // Implement conversion logic here
        Ok(MintResult::Success(0)) // Placeholder implementation
    }
}

impl IntoVal<Env, Val> for MintResult {
    fn into_val(&self, _env: &Env) -> Val {
        // Implement conversion logic here
        Val::from(0) // Placeholder implementation
    }
}

#[contractimpl]
impl CertificateContract {
    pub fn mint_batch_certificates(
        env: &Env,
        issuer: Address,
        owners: Vec<Address>,
        certificates: Vec<CertificateData>
    ) -> Vec<MintResult> {
        // let _guard = ReentrancyLock::new(env);
        issuer.require_auth();
        if !auth::is_issuer(env, &issuer) {
            emit_error_event(env, "mint_batch_certificates", Error::Unauthorized as u32, Error::Unauthorized.message(), None);
            env.panic_with_error(Error::Unauthorized);
        }
        let batch_size = certificates.len();
        let max_batch_size = storage::get_max_batch_size(env) as u32;
        if batch_size > max_batch_size as u32 {
            emit_error_event(env, "mint_batch_certificates", Error::BatchSizeTooLarge as u32, Error::BatchSizeTooLarge.message(), None);
            env.panic_with_error(Error::BatchSizeTooLarge);
        }
        if batch_size != owners.len() {
            emit_error_event(env, "mint_batch_certificates", Error::InvalidInput as u32, Error::InvalidInput.message(), None);
            env.panic_with_error(Error::InvalidInput);
        }
        let mut results = Vec::new(env);
        let mut success_count: u32 = 0;
        let mut failure_count: u32 = 0;
        let mut owner_cert_cache = SorobanMap::new(env);
        for i in 0..batch_size {
            let owner = owners.get(i).unwrap();
            let certificate = certificates.get(i).unwrap();
            let mut owner_certs = if let Some(certs) = owner_cert_cache.get(owner.clone()) {
                certs
            } else {
                let certs = storage::get_owner_certificates(env, &owner);
                owner_cert_cache.set(owner.clone(), certs.clone());
                certs
            };
            let result = match storage::certificate_exists(env, certificate.id) {
                true => {
                    emit_error_event(env, "mint_batch_certificates", Error::DuplicateCertificate as u32, Error::DuplicateCertificate.message(), Some(certificate.id));
                    Err(Error::DuplicateCertificate)
                },
                false => {
                    // Save certificate with one retry on storage error
                    match storage::save_certificate(env, &owner, &certificate) {
                        Ok(()) => {
                            owner_certs.push_back(certificate.id);
                            owner_cert_cache.set(owner.clone(), owner_certs.clone());
                            Ok(())
                        },
                        Err(e) if e == Error::StorageError => {
                            emit_error_event(env, "mint_batch_certificates", e as u32, "Storage error, retrying once", Some(certificate.id));
                            match storage::save_certificate(env, &owner, &certificate) {
                                Ok(()) => {
                                    owner_certs.push_back(certificate.id);
                                    owner_cert_cache.set(owner.clone(), owner_certs.clone());
                                    Ok(())
                                },
                                Err(e2) => {
                                    emit_error_event(env, "mint_batch_certificates", e2 as u32, e2.message(), Some(certificate.id));
                                    Err(e2)
                                }
                            }
                        },
                        Err(e) => {
                            emit_error_event(env, "mint_batch_certificates", e as u32, e.message(), Some(certificate.id));
                            Err(e)
                        }
                    }
                }
            };
            match result {
                Ok(()) => {
                    success_count += 1;
                    results.push_back(MintResult::Success(certificate.id));
                },
                Err(e) => {
                    failure_count += 1;
                    results.push_back(MintResult::Failure(certificate.id, e));
                }
            }
        }
        for (owner, certs) in owner_cert_cache.iter() {
            let owner_key = storage::get_owner_key(env, &owner);
            env.storage().persistent().set(&owner_key, &certs);
        }
        events::emit_batch_mint_completed(
            env, 
            &issuer, 
            batch_size, 
            success_count, 
            failure_count
        );
        results
    }
}