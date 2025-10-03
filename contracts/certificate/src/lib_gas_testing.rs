#![no_std]

#[cfg(test)]
mod gas_regression_tests_simple;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

// Minimal structs for gas testing
#[derive(Clone, Debug)]
pub struct SimpleCertificateParams {
    pub certificate_id: BytesN<32>,
    pub student: Address,
    pub course_id: String,
    pub title: String,
    pub description: String,
    pub metadata_uri: String,
    pub expiry_date: u64,
}

#[derive(Clone, Debug)]
pub enum SimpleCertificateError {
    AlreadyExists,
    NotFound,
    Unauthorized,
}

/// Minimal certificate contract for gas testing
pub trait GasTestCertificateTrait {
    fn initialize(env: Env, admin: Address) -> Result<(), SimpleCertificateError>;
    
    fn mint_certificate(
        env: Env,
        issuer: Address,
        params: SimpleCertificateParams,
    ) -> Result<(), SimpleCertificateError>;
    
    fn mint_certificates_batch(
        env: Env,
        issuer: Address,
        params_list: Vec<SimpleCertificateParams>,
    ) -> Result<(), SimpleCertificateError>;
    
    fn get_certificate_count(env: Env) -> u64;
}

#[contract]
pub struct GasTestCertificate;

#[contractimpl] 
impl GasTestCertificateTrait for GasTestCertificate {
    fn initialize(env: Env, admin: Address) -> Result<(), SimpleCertificateError> {
        admin.require_auth();
        
        // Simple storage to track initialization
        let key = "initialized";
        if env.storage().persistent().has(&key) {
            return Err(SimpleCertificateError::AlreadyExists);
        }
        env.storage().persistent().set(&key, &true);
        env.storage().persistent().set(&"admin", &admin);
        
        Ok(())
    }
    
    fn mint_certificate(
        env: Env,
        issuer: Address,
        params: SimpleCertificateParams,
    ) -> Result<(), SimpleCertificateError> {
        issuer.require_auth();
        
        // Check if certificate already exists
        if env.storage().persistent().has(&params.certificate_id) {
            return Err(SimpleCertificateError::AlreadyExists);
        }
        
        // Store certificate data (minimal)
        env.storage().persistent().set(&params.certificate_id, &params);
        
        // Update count
        let count_key = "count";
        let current_count: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);
        env.storage().persistent().set(&count_key, &(current_count + 1));
        
        Ok(())
    }
    
    fn mint_certificates_batch(
        env: Env,
        issuer: Address,
        params_list: Vec<SimpleCertificateParams>,
    ) -> Result<(), SimpleCertificateError> {
        issuer.require_auth();
        
        // Check all certificates don't exist first
        for params in params_list.iter() {
            if env.storage().persistent().has(&params.certificate_id) {
                return Err(SimpleCertificateError::AlreadyExists);
            }
        }
        
        // Mint all certificates
        for params in params_list.iter() {
            env.storage().persistent().set(&params.certificate_id, &params);
        }
        
        // Update count
        let count_key = "count";
        let current_count: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);
        env.storage().persistent().set(&count_key, &(current_count + params_list.len() as u64));
        
        Ok(())
    }
    
    fn get_certificate_count(env: Env) -> u64 {
        env.storage().persistent().get(&"count").unwrap_or(0)
    }
}