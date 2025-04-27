#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, symbol_short, Address, BytesN, Env, Symbol, Vec, Map, String};

// Storage keys
const INITIALIZED: Symbol = symbol_short!("INIT");
const CERTIFICATES: Symbol = symbol_short!("CERT");
const USER_CERTS: Symbol = symbol_short!("UCERT");
const INSTRUCTORS: Symbol = symbol_short!("INST");
const ADMIN: Symbol = symbol_short!("ADMIN");

// Certificate metadata structure
#[derive(Clone)]
pub struct CertificateMetadata {
    pub course_id: String,
    pub student_id: Address,
    pub instructor_id: Address,
    pub issue_date: u64,
    pub metadata_uri: String,
    pub token_id: BytesN<32>,     // Unique NFT identifier
    pub title: String,            // Certificate title
    pub description: String,      // Certificate description
    pub status: CertificateStatus // Certificate status (Active/Revoked)
}

#[derive(Clone, Eq, PartialEq)]
pub enum CertificateStatus {
    Active,
    Revoked
}

// Use the contracterror macro to define errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    CertificateAlreadyExists = 2,
    CertificateNotFound = 3,
    NotInitialized = 4,
    Unauthorized = 5,
    NotInstructor = 6,
    InvalidTokenId = 7,
    InvalidMetadata = 8,
    CertificateRevoked = 9,
    TransferNotAllowed = 10,
}

#[contract]
pub struct Certificate;

#[contractimpl]
impl Certificate {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        // Check if already initialized
        if env.storage().instance().has(&INITIALIZED) {
            return Err(Error::AlreadyInitialized);
        }
        
        // Require authorization from the admin
        admin.require_auth();
        
        // Store admin address and mark as initialized
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&INITIALIZED, &true);
        
        Ok(())
    }

    pub fn add_instructor(env: Env, instructor: Address) -> Result<(), Error> {
        // Get admin and check authorization
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap_or_else(|| {
            return Err(Error::NotInitialized);
        })?;
        admin.require_auth();
        
        // Add instructor to the list
        env.storage().instance().set(&(INSTRUCTORS, instructor.clone()), &true);
        
        Ok(())
    }

    pub fn remove_instructor(env: Env, instructor: Address) -> Result<(), Error> {
        // Get admin and check authorization
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap_or_else(|| {
            return Err(Error::NotInitialized);
        })?;
        admin.require_auth();
        
        // Remove instructor from the list
        env.storage().instance().remove(&(INSTRUCTORS, instructor.clone()));
        
        Ok(())
    }

    fn is_instructor(env: &Env, address: &Address) -> bool {
        env.storage().instance().has(&(INSTRUCTORS, address.clone()))
    }

    pub fn mint_certificate(
        env: Env,
        certificate_id: BytesN<32>,
        course_id: String,
        student: Address,
        title: String,
        description: String,
        metadata_uri: String
    ) -> Result<(), Error> {
        // Check if initialized
        if !env.storage().instance().has(&INITIALIZED) {
            return Err(Error::NotInitialized);
        }
        
        // Get instructor (caller) and verify authorization
        let instructor = env.invoker().unwrap_or_else(|| {
            return Err(Error::Unauthorized);
        })?;
        
        if !Self::is_instructor(&env, &instructor) {
            return Err(Error::NotInstructor);
        }
        
        // Validate inputs
        if title.is_empty() || description.is_empty() || metadata_uri.is_empty() {
            return Err(Error::InvalidMetadata);
        }
        
        // Create a storage key for this certificate
        let key = (CERTIFICATES, certificate_id.clone());
        
        // Check if certificate already exists
        if env.storage().instance().has(&key) {
            return Err(Error::CertificateAlreadyExists);
        }
        
        // Generate unique token ID for NFT
        let token_id = env.crypto().sha256(&env.ledger().sequence().to_be_bytes());
        
        // Create certificate metadata
        let metadata = CertificateMetadata {
            course_id,
            student_id: student.clone(),
            instructor_id: instructor.clone(),
            issue_date: env.ledger().timestamp(),
            metadata_uri,
            token_id: token_id.clone(),
            title,
            description,
            status: CertificateStatus::Active
        };
        
        // Store certificate metadata
        env.storage().instance().set(&key, &metadata);
        
        // Add to user's certificates
        Self::add_user_certificate(env.clone(), student.clone(), certificate_id)?;
        
        // Emit certificate minted event with enhanced metadata
        env.events().publish(
            (Symbol::new(&env, "nft_certificate_minted"),
             certificate_id.clone()),
            (
                metadata.clone(),
                student,
                instructor,
                token_id
            )
        );
        
        Ok(())
    }

    pub fn verify_certificate(env: Env, certificate_id: BytesN<32>) -> Result<CertificateMetadata, Error> {
        // Create a storage key for this certificate
        let key = (CERTIFICATES, certificate_id.clone());
        
        // Check if certificate exists and get metadata
        if let Some(metadata) = env.storage().instance().get::<_, CertificateMetadata>(&key) {
            Ok(metadata)
        } else {
            Err(Error::CertificateNotFound)
        }
    }

    pub fn revoke_certificate(env: Env, certificate_id: BytesN<32>) -> Result<(), Error> {
        // Get instructor (caller) and verify authorization
        let instructor = env.invoker().unwrap_or_else(|| {
            return Err(Error::Unauthorized);
        })?;
        
        if !Self::is_instructor(&env, &instructor) {
            return Err(Error::NotInstructor);
        }
        
        // Create a storage key for this certificate
        let key = (CERTIFICATES, certificate_id.clone());
        
        // Check if certificate exists and get metadata
        let mut metadata = env.storage().instance().get::<_, CertificateMetadata>(&key)
            .ok_or(Error::CertificateNotFound)?;
        
        // Update certificate status
        metadata.status = CertificateStatus::Revoked;
        
        // Store updated metadata
        env.storage().instance().set(&key, &metadata);
        
        // Emit enhanced certificate revoked event
        env.events().publish(
            (Symbol::new(&env, "nft_certificate_revoked"), certificate_id.clone()),
            (
                metadata.clone(),
                instructor.clone(),
                env.ledger().timestamp()
            )
        );
        
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
        let admin = Address::random(&env);
        
        // Test successful initialization
        let result = client.initialize(&admin);
        assert!(result.is_ok());
        
        // Test re-initialization (should fail)
        let result = client.initialize(&admin);
        assert_eq!(result, Err(Error::AlreadyInitialized));
    }
    
    #[test]
    fn test_certificate_lifecycle() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        let admin = Address::random(&env);
        
        // Initialize
        client.initialize(&admin).unwrap();
        
        // Create a certificate ID
        let cert_id = BytesN::random(&env);
        
        // Issue certificate
        client.issue_certificate(&cert_id).unwrap();
        
        // Verify certificate
        client.verify_certificate(&cert_id).unwrap();
        
        // Try to issue same certificate again (should fail)
        let result = client.issue_certificate(&cert_id);
        assert_eq!(result, Err(Error::CertificateAlreadyExists));
        
        // Revoke certificate
        client.revoke_certificate(&cert_id).unwrap();
        
        // Verify revoked certificate (should fail)
        let result = client.verify_certificate(&cert_id);
        assert_eq!(result, Err(Error::CertificateNotFound));
    }
    
    #[test]
    fn test_user_certificates() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        let admin = Address::random(&env);
        
        // Initialize
        client.initialize(&admin).unwrap();
        
        // Create a user
        let user = Address::random(&env);
        
        // Create certificate IDs
        let cert_id1 = BytesN::random(&env);
        let cert_id2 = BytesN::random(&env);
        
        // Issue certificates
        client.issue_certificate(&cert_id1).unwrap();
        client.issue_certificate(&cert_id2).unwrap();
        
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
