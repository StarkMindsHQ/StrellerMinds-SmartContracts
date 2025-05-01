use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::errors::CertificateError;
use crate::types::{CertificateMetadata, Permission, Role};

/// Interface for the Certificate contract.
pub trait CertificateTrait {
    /// Initialize the contract with an admin
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - Address to set as the admin
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if already initialized
    fn initialize(env: Env, admin: Address) -> Result<(), CertificateError>;

    /// Get the current admin address
    ///
    /// # Arguments
    /// * `env` - The contract environment
    ///
    /// # Returns
    /// * `Result<Address, CertificateError>` - Admin address if initialized, Error otherwise
    fn get_admin(env: Env) -> Result<Address, CertificateError>;

    /// Grant a role to a user
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - Address to grant role to
    /// * `role` - The role to grant
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized
    ///
    /// # Authentication
    /// * Requires authorization from admin
    fn grant_role(env: Env, user: Address, role: Role) -> Result<(), CertificateError>;

    /// Update a user's role
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - Address whose role to update
    /// * `new_role` - The new role
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or role not found
    ///
    /// # Authentication
    /// * Requires authorization from admin
    fn update_role(env: Env, user: Address, new_role: Role) -> Result<(), CertificateError>;

    /// Revoke a user's role
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - Address whose role to revoke
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or role not found
    ///
    /// # Authentication
    /// * Requires authorization from admin
    fn revoke_role(env: Env, user: Address) -> Result<(), CertificateError>;

    /// Get a user's role
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - Address to check
    ///
    /// # Returns
    /// * `Option<Role>` - The user's role if found, None otherwise
    fn get_role(env: Env, user: Address) -> Option<Role>;

    /// Check if a user has a permission
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - Address to check
    /// * `permission` - Permission to check
    ///
    /// # Returns
    /// * `bool` - True if user has permission, false otherwise
    fn has_permission(env: Env, user: Address, permission: Permission) -> bool;

    /// Mint a new certificate
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    /// * `course_id` - Course identifier
    /// * `student` - Address of the student
    /// * `title` - Title of the certificate
    /// * `description` - Description of the certificate
    /// * `metadata_uri` - URI for certificate metadata
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or invalid input
    ///
    /// # Authentication
    /// * Requires authorization from a user with Issue permission
    fn mint_certificate(
        env: Env,
        issuer: Address,
        certificate_id: BytesN<32>,
        course_id: String,
        student: Address,
        title: String,
        description: String,
        metadata_uri: String,
        expiry_date: u64,
    ) -> Result<(), CertificateError>;

    /// Check if a certificate is expired   
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    fn is_certificate_expired(env: Env, certificate_id: BytesN<32>) -> bool;

    /// Verify a certificate
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `Result<CertificateMetadata, CertificateError>` - Certificate metadata if found, Error otherwise
    fn verify_certificate(
        env: Env,
        certificate_id: BytesN<32>,
    ) -> Result<CertificateMetadata, CertificateError>;

    /// Revoke a certificate
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or certificate not found
    ///
    /// # Authentication
    /// * Requires authorization from a user with Revoke permission
    fn revoke_certificate(
        env: Env,
        revoker: Address,
        certificate_id: BytesN<32>,
    ) -> Result<(), CertificateError>;

    /// Get all certificates for a user
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user_address` - Address of the user
    ///
    /// # Returns
    /// * `Vec<BytesN<32>>` - Collection of certificate IDs, empty if none found
    fn track_certificates(env: Env, user_address: Address) -> Vec<BytesN<32>>;

    /// Add a certificate to a user's tracked certificates
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user_address` - Address of the user
    /// * `certificate_id` - Certificate ID to add
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if certificate not found
    fn add_user_certificate(
        env: Env,
        user_address: Address,
        certificate_id: BytesN<32>,
    ) -> Result<(), CertificateError>;
}
