#![no_std]


use soroban_sdk::{contract, contractimpl, contracterror, symbol_short, Address, BytesN, Env, Symbol, Vec, Map, String};

// Storage keys
const INITIALIZED: Symbol = symbol_short!("INIT");
const CERTIFICATES: Symbol = symbol_short!("CERT");
const USER_CERTS: Symbol = symbol_short!("UCERT");
const INSTRUCTORS: Symbol = symbol_short!("INST");
const ADMIN: Symbol = symbol_short!("ADMIN");

// Certificate metadata structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateMetadata {
    pub course_id: String,
    pub student_id: Address,
    pub instructor_id: Address,
    pub issue_date: u64,
    pub metadata_uri: String,
    pub token_id: BytesN<32>,     // Unique NFT identifier
    pub title: String,            // Certificate title
    pub description: String,      // Certificate description
    pub status: CertificateStatus, // Certificate status (Active/Revoked)
    pub expiry_date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateStatus {
    Active,
    Revoked
}


// Event names
const EVENT_CERTIFICATE_MINTED: Symbol = symbol_short!("CertMintd");
const EVENT_CERTIFICATE_REVOKED: Symbol = symbol_short!("CertRevtd");
const EVENT_CERTIFICATE_UPDATED: Symbol = symbol_short!("CertUpdtd");

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
    CertificateExpired = 11,
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
        metadata_uri: String,
        expiry_date: u64
    ) -> Result<(), Error> {
        // Check if initialized
        if !env.storage().instance().has(&INITIALIZED) {
            return Err(Error::NotInitialized);
        }
        
        // For testing purposes, we'll use the contract's address as instructor
        // In production, this would be the caller's address with proper authentication
        let instructor = env.current_contract_address();
        
        if !Self::is_instructor(&env, &instructor) {
            return Err(Error::NotInstructor);
        }

        // Validate inputs - check if any strings are empty
        // Note that in Soroban, strings are already byte vectors so we can use len()
        if title.len() == 0 || description.len() == 0 || metadata_uri.len() == 0 {
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
            status: CertificateStatus::Active,
            expiry_date
        };

        // Store certificate metadata
        env.storage().instance().set(&key, &metadata);

        // Add to user's certificates
        Self::add_user_certificate(env.clone(), student.clone(), certificate_id.clone())?;

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

    pub fn is_certificate_expired(env: Env, certificate_id: BytesN<32>) -> Result<bool, Error> {
        // Create a storage key for this certificate
        let key = (CERTIFICATES, certificate_id.clone());

        // Check if certificate exists
        if !env.storage().instance().has(&key) {
            return Err(Error::CertificateNotFound);
        }

        // Get certificate data
        let metadata: CertificateMetadata = env.storage().instance().get(&key).unwrap();

        // If expiry_date is 0, the certificate is permanent and never expires
        if metadata.expiry_date == 0 {
            return Ok(false);
        }

        // Check if certificate has expired
        let current_timestamp = env.ledger().timestamp();
        Ok(current_timestamp > metadata.expiry_date)
    }

    pub fn verify_certificate(env: Env, certificate_id: BytesN<32>) -> Result<CertificateMetadata, Error> {
        // Create a storage key for this certificate
        let key = (CERTIFICATES, certificate_id.clone());
        
        // Check if certificate exists
        if !env.storage().instance().has(&key) {
            return Err(Error::CertificateNotFound);
        }
        
        // Get certificate metadata
        let metadata: CertificateMetadata = env.storage().instance().get(&key).unwrap();
        
        // Check if certificate is active
        if metadata.status != CertificateStatus::Active {
            return Err(Error::CertificateRevoked);
        }
        
        // Check if certificate has expired
        if Self::is_certificate_expired(env.clone(), certificate_id)? {
            return Err(Error::CertificateExpired);
        }
        
        Ok(metadata)
    }

    pub fn revoke_certificate(env: Env, certificate_id: BytesN<32>) -> Result<(), Error> {
        // Check if initialized
        if !env.storage().instance().has(&INITIALIZED) {
            return Err(Error::NotInitialized);
        }
        
        // Get certificate key
        let key = (CERTIFICATES, certificate_id.clone());
        
        // Check if certificate exists
        if !env.storage().instance().has(&key) {
            return Err(Error::CertificateNotFound);
        }
        
        // For testing purposes, use contract address as the caller
        // In production, this would be the authenticated caller
        let caller = env.current_contract_address();
        
        // Get certificate metadata
        let mut metadata: CertificateMetadata = env.storage().instance().get(&key).unwrap();
        
        // Verify caller is either admin or the instructor who issued the certificate
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        
        if caller != admin && caller != metadata.instructor_id {
            return Err(Error::Unauthorized);
        }
        
        // Update certificate status to revoked
        metadata.status = CertificateStatus::Revoked;
        
        // Save updated metadata
        env.storage().instance().set(&key, &metadata);
        
        // Emit certificate revoked event
        env.events().publish(
            (Symbol::new(&env, "certificate_revoked"), certificate_id.clone()),
            (metadata.student_id, caller)
        );
        
        // Emit CertificateRevoked event
        env.events().publish(
            (EVENT_CERTIFICATE_REVOKED,),  
            (
               certificate_id, 
               None::<Address>,
               symbol_short!("REVOKED"),
               env.ledger().timestamp()),  
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
        // Create storage key for user certificates
        let key = (USER_CERTS, user_address.clone());
        
        // Get or create user certificates list
        let mut user_certs = if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Vec::new(&env)
        };
        
        // Add certificate to user's list
        user_certs.push_back(certificate_id.clone());
        
        // Store updated list
        env.storage().instance().set(&key, &user_certs);

        // Emit event when certificate is added to the user
        env.events().publish(
            (EVENT_CERTIFICATE_UPDATED,),  
            (
               certificate_id, 
               user_certs,
               None::<Address>,
               symbol_short!("REVOKED"),
               env.ledger().timestamp()),  
        );
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
        let instructor = Address::random(&env);
        let student = Address::random(&env);
        
        // Initialize
        client.initialize(&admin).unwrap();
        
        // Add instructor
        admin.require_auth_for_testing();
        client.add_instructor(&instructor).unwrap();
        
        // Create certificate ID
        let cert_id = BytesN::random(&env);
        
        // Mint certificate (with no expiry)
        instructor.require_auth_for_testing();
        client.mint_certificate(
            &cert_id,
            &String::from_str(&env, "COURSE-101"),
            &student,
            &String::from_str(&env, "Certificate Title"),
            &String::from_str(&env, "Certificate Description"),
            &String::from_str(&env, "https://example.com/metadata"),
            0
        ).unwrap();
        
        // Verify certificate
        let metadata = client.verify_certificate(&cert_id).unwrap();
        assert_eq!(metadata.status, CertificateStatus::Active);
        
        // Revoke certificate
        instructor.require_auth_for_testing();
        client.revoke_certificate(&cert_id).unwrap();
        
        // Verify certificate should now fail due to revocation
        let result = client.verify_certificate(&cert_id);
        assert_eq!(result, Err(Error::CertificateRevoked));
    }
    
    #[test]
    fn test_certificate_expiry() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        // Create admin and instructor accounts
        let admin = Address::random(&env);
        let instructor = Address::random(&env);
        let student = Address::random(&env);
        
        // Initialize contract
        client.initialize(&admin).unwrap();
        
        // Add instructor
        admin.require_auth_for_testing();
        client.add_instructor(&instructor).unwrap();
        
        // Create certificate ID
        let cert_id = BytesN::random(&env);
        
        // Get current timestamp
        let current_timestamp = env.ledger().timestamp();
        
        // Mint certificate with future expiry date (current time + 1 day in seconds)
        let expiry_date = current_timestamp + 86400;
        instructor.require_auth_for_testing();
        client.mint_certificate(
            &cert_id,
            &String::from_str(&env, "COURSE-101"),
            &student,
            &String::from_str(&env, "Certificate Title"),
            &String::from_str(&env, "Certificate Description"),
            &String::from_str(&env, "https://example.com/metadata"),
            expiry_date
        ).unwrap();
        
        // Verify certificate is not expired
        assert_eq!(client.is_certificate_expired(&cert_id), Ok(false));
        
        // Verify certificate works
        let metadata = client.verify_certificate(&cert_id).unwrap();
        assert_eq!(metadata.status, CertificateStatus::Active);
        
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
        
        // Create admin and instructor accounts
        let admin = Address::random(&env);
        let instructor = Address::random(&env);
        let student = Address::random(&env);
        
        // Initialize contract
        client.initialize(&admin).unwrap();
        
        // Add instructor
        admin.require_auth_for_testing();
        client.add_instructor(&instructor).unwrap();
        
        // Create certificate ID
        let cert_id = BytesN::random(&env);
        
        // Mint permanent certificate (expiry date = 0)
        instructor.require_auth_for_testing();
        client.mint_certificate(
            &cert_id,
            &String::from_str(&env, "COURSE-101"),
            &student,
            &String::from_str(&env, "Certificate Title"),
            &String::from_str(&env, "Certificate Description"),
            &String::from_str(&env, "https://example.com/metadata"),
            0
        ).unwrap();
        
        // Fast forward time significantly (1 year in seconds)
        let current_timestamp = env.ledger().timestamp();
        env.ledger().set_timestamp(current_timestamp + 31536000);
        
        // Verify certificate is still not expired
        assert_eq!(client.is_certificate_expired(&cert_id), Ok(false));
        
        // Verify certificate should still work
        let metadata = client.verify_certificate(&cert_id).unwrap();
        assert_eq!(metadata.status, CertificateStatus::Active);
    }
    
    #[test]
    fn test_user_certificates() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Certificate);
        let client = CertificateClient::new(&env, &contract_id);
        
        // Create admin and instructor accounts
        let admin = Address::random(&env);
        let instructor = Address::random(&env);
        let student = Address::random(&env);
        
        // Initialize contract
        client.initialize(&admin).unwrap();
        
        // Add instructor
        admin.require_auth_for_testing();
        client.add_instructor(&instructor).unwrap();
        
        // Create certificate IDs
        let cert_id1 = BytesN::random(&env);
        let cert_id2 = BytesN::random(&env);
        
        // Mint certificates
        instructor.require_auth_for_testing();
        client.mint_certificate(
            &cert_id1,
            &String::from_str(&env, "COURSE-101"),
            &student,
            &String::from_str(&env, "Certificate 1"),
            &String::from_str(&env, "Certificate Description 1"),
            &String::from_str(&env, "https://example.com/metadata1"),
            0
        ).unwrap();
        
        client.mint_certificate(
            &cert_id2,
            &String::from_str(&env, "COURSE-102"),
            &student,
            &String::from_str(&env, "Certificate 2"),
            &String::from_str(&env, "Certificate Description 2"),
            &String::from_str(&env, "https://example.com/metadata2"),
            0
        ).unwrap();
        
        // Get user certificates
        let user_certs = client.track_certificates(&student);
        
        // Verify user has both certificates
        assert_eq!(user_certs.len(), 2);
        assert!(user_certs.contains(&cert_id1));
        assert!(user_certs.contains(&cert_id2));
    }
}