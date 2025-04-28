#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, symbol_short, Address, BytesN, Env, Symbol, Vec, contracttype};

// Storage keys
const INITIALIZED: Symbol = symbol_short!("INIT");
const CERTIFICATES: Symbol = symbol_short!("CERT");
const USER_CERTS: Symbol = symbol_short!("UCERT");

// Use the contracterror macro to define errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    CertificateAlreadyExists = 2,
    CertificateNotFound = 3,
    NotInitialized = 4,
    CertificateExpired = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateData {
    // Unix timestamp for expiry (0 means no expiry/permanent)
    pub expiry_date: u64,
    pub is_active: bool,
}

#[contract]
pub struct Certificate;

#[contractimpl]
impl Certificate {
    pub fn initialize(env: Env) -> Result<(), Error> {
        // Check if already initialized
        if env.storage().instance().has(&INITIALIZED) {
            return Err(Error::AlreadyInitialized);
        }
        
        // Mark as initialized
        env.storage().instance().set(&INITIALIZED, &true);
        
        Ok(())
    }

    pub fn issue_certificate(env: Env, certificate_id: BytesN<32>, expiry_date: u64) -> Result<(), Error> {
        // Check if initialized
        if !env.storage().instance().has(&INITIALIZED) {
            return Err(Error::NotInitialized);
        }
        
        // Create a storage key for this certificate
        let key = (CERTIFICATES, certificate_id.clone());
        
        // Check if certificate already exists
        if env.storage().instance().has(&key) {
            return Err(Error::CertificateAlreadyExists);
        }
        
        // Store certificate with expiry date (0 means no expiry)
        let cert_data = CertificateData {
            expiry_date,
            is_active: true,
        };
        
        env.storage().instance().set(&key, &cert_data);
        
        Ok(())
    }

    pub fn is_certificate_expired(env: Env, certificate_id: BytesN<32>) -> Result<bool, Error> {
        // Create a storage key for this certificate
        let key = (CERTIFICATES, certificate_id.clone());
        
        // Check if certificate exists
        if !env.storage().instance().has(&key) {
            return Err(Error::CertificateNotFound);
        }
        
        // Get certificate data
        let cert_data: CertificateData = env.storage().instance().get(&key).unwrap();
        
        // If expiry_date is 0, the certificate is permanent and never expires
        if cert_data.expiry_date == 0 {
            return Ok(false);
        }
        
        // Check if certificate has expired
        let current_timestamp = env.ledger().timestamp();
        Ok(current_timestamp > cert_data.expiry_date)
    }

    pub fn verify_certificate(env: Env, certificate_id: BytesN<32>) -> Result<(), Error> {
        // Create a storage key for this certificate
        let key = (CERTIFICATES, certificate_id.clone());
        
        // Check if certificate exists
        if !env.storage().instance().has(&key) {
            return Err(Error::CertificateNotFound);
        }
        
        // Get certificate data
        let cert_data: CertificateData = env.storage().instance().get(&key).unwrap();
        
        // Check if certificate is active
        if !cert_data.is_active {
            return Err(Error::CertificateNotFound);
        }
        
        // Check if certificate has expired
        if Self::is_certificate_expired(env.clone(), certificate_id)? {
            return Err(Error::CertificateExpired);
        }
        
        Ok(())
    }

    pub fn revoke_certificate(env: Env, certificate_id: BytesN<32>) -> Result<(), Error> {
        // Create a storage key for this certificate
        let key = (CERTIFICATES, certificate_id.clone());
        
        // Check if certificate exists
        if !env.storage().instance().has(&key) {
            return Err(Error::CertificateNotFound);
        }
        
        // Remove certificate
        env.storage().instance().remove(&key);
        
        Ok(())
    }

    pub fn track_certificates(env: Env, user_address: Address) -> Vec<BytesN<32>> {
        let key = (USER_CERTS, user_address.clone());
        
        // Check if user has any certificates
        if env.storage().instance().has(&key) {
            // Get existing certificates
            env.storage().instance().get(&key).unwrap()
        } else {
            // Return empty vector
            Vec::new(&env)
        }
    }
    
    pub fn add_user_certificate(env: Env, user_address: Address, certificate_id: BytesN<32>) -> Result<(), Error> {
        // Verify certificate exists
        Self::verify_certificate(env.clone(), certificate_id.clone())?;
        
        // Create storage key for user certificates
        let key = (USER_CERTS, user_address.clone());
        
        // Get or create user certificates list
        let mut user_certs = if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Vec::new(&env)
        };
        
        // Add certificate to user's list
        user_certs.push_back(certificate_id);
        
        // Store updated list
        env.storage().instance().set(&key, &user_certs);
        
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, BytesN as _};
    
    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        // Test successful initialization
        let result = client.initialize();
        assert!(result.is_ok());
        
        // Test re-initialization (should fail)
        let result = client.initialize();
        assert_eq!(result, Err(Error::AlreadyInitialized));
    }
    
    #[test]
    fn test_certificate_lifecycle() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        // Initialize
        client.initialize().unwrap();
        
        // Create a certificate ID
        let cert_id = BytesN::random(&env);
        
        // Issue permanent certificate (no expiry)
        client.issue_certificate(&cert_id, 0).unwrap();
        
        // Verify certificate
        client.verify_certificate(&cert_id).unwrap();
        
        // Try to issue same certificate again (should fail)
        let result = client.issue_certificate(&cert_id, 0);
        assert_eq!(result, Err(Error::CertificateAlreadyExists));
        
        // Revoke certificate
        client.revoke_certificate(&cert_id).unwrap();
        
        // Verify revoked certificate (should fail)
        let result = client.verify_certificate(&cert_id);
        assert_eq!(result, Err(Error::CertificateNotFound));
    }
    
    #[test]
    fn test_certificate_expiry() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        // Initialize
        client.initialize().unwrap();
        
        // Create a certificate ID
        let cert_id = BytesN::random(&env);
        
        // Get current timestamp
        let current_timestamp = env.ledger().timestamp();
        
        // Issue certificate with future expiry date (current time + 1 day in seconds)
        let expiry_date = current_timestamp + 86400;
        client.issue_certificate(&cert_id, expiry_date).unwrap();
        
        // Verify certificate is not expired
        assert_eq!(client.is_certificate_expired(&cert_id), Ok(false));
        
        // Verify certificate
        client.verify_certificate(&cert_id).unwrap();
        
        // Fast forward time to after expiry date
        env.ledger().set_timestamp(expiry_date + 1);
        
        // Verify certificate is now expired
        assert_eq!(client.is_certificate_expired(&cert_id), Ok(true));
        
        // Verify certificate should now fail due to expiration
        let result = client.verify_certificate(&cert_id);
        assert_eq!(result, Err(Error::CertificateExpired));
    }
    
    #[test]
    fn test_permanent_certificate() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        // Initialize
        client.initialize().unwrap();
        
        // Create a certificate ID
        let cert_id = BytesN::random(&env);
        
        // Issue permanent certificate (expiry date = 0)
        client.issue_certificate(&cert_id, 0).unwrap();
        
        // Fast forward time significantly (1 year in seconds)
        let current_timestamp = env.ledger().timestamp();
        env.ledger().set_timestamp(current_timestamp + 31536000);
        
        // Verify certificate is still not expired
        assert_eq!(client.is_certificate_expired(&cert_id), Ok(false));
        
        // Verify certificate should still work
        client.verify_certificate(&cert_id).unwrap();
    }
    
    #[test]
    fn test_user_certificates() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        // Initialize
        client.initialize().unwrap();
        
        // Create a user
        let user = Address::random(&env);
        
        // Create certificate IDs
        let cert_id1 = BytesN::random(&env);
        let cert_id2 = BytesN::random(&env);
        
        // Issue certificates
        client.issue_certificate(&cert_id1, 0).unwrap();
        client.issue_certificate(&cert_id2, 0).unwrap();
        
        // Add certificates to user
        client.add_user_certificate(&user, &cert_id1).unwrap();
        client.add_user_certificate(&user, &cert_id2).unwrap();
        
        // Get user certificates
        let user_certs = client.track_certificates(&user);
        
        // Verify user has both certificates
        assert_eq!(user_certs.len(), 2);
        assert!(user_certs.contains(&cert_id1));
        assert!(user_certs.contains(&cert_id2));
    }
}
