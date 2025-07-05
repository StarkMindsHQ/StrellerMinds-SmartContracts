use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::errors::CertificateError;
use crate::types::{CertificateMetadata, MetadataUpdateEntry, MintCertificateParams};

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
    /// * `role_level` - The role level to grant (1=Student, 2=Moderator, 3=Instructor, 4=Admin, 5=SuperAdmin)
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized
    ///
    /// # Authentication
    /// * Requires authorization from user with GrantRole permission
    fn grant_role(env: Env, user: Address, role_level: u32) -> Result<(), CertificateError>;

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
    /// * Requires authorization from user with RevokeRole permission
    fn revoke_role(env: Env, user: Address) -> Result<(), CertificateError>;

    /// Get a user's role
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - Address to check
    ///
    /// # Returns
    /// * `Option<Role>` - The user's role if found, None otherwise
    fn get_role(env: Env, user: Address) -> Option<shared::roles::Role>;

    /// Check if a user has a permission
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - Address to check
    /// * `permission` - Permission to check (0=IssueCertificate, 1=RevokeCertificate, 2=TransferCertificate, 3=UpdateCertificateMetadata)
    ///
    /// # Returns
    /// * `bool` - True if user has permission, false otherwise
    fn has_permission(env: Env, user: Address, permission: u32) -> bool;

    /// Mint a new certificate
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `issuer` - Address of the issuer
    /// * `params` - Parameters for minting the certificate
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or invalid input
    ///
    /// # Authentication
    /// * Requires authorization from a user with IssueCertificate permission
    #[allow(clippy::too_many_arguments)]
    fn mint_certificate(
        env: Env,
        issuer: Address,
        params: MintCertificateParams,
    ) -> Result<(), CertificateError>;

    /// Revoke a certificate
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `revoker` - Address of the revoker
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or certificate not found
    ///
    /// # Authentication
    /// * Requires authorization from a user with RevokeCertificate permission
    fn revoke_certificate(
        env: Env,
        revoker: Address,
        certificate_id: BytesN<32>,
    ) -> Result<(), CertificateError>;

    /// Transfer a certificate from one user to another
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `from` - Address of the current owner
    /// * `to` - Address of the new owner
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or certificate not found
    ///
    /// # Authentication
    /// * Requires authorization from the current owner with TransferCertificate permission
    fn transfer_certificate(
        env: Env,
        from: Address,
        to: Address,
        certificate_id: BytesN<32>,
    ) -> Result<(), CertificateError>;

    /// Update certificate metadata URI
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `updater` - Address of the updater
    /// * `certificate_id` - Unique identifier for the certificate
    /// * `new_uri` - New metadata URI
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or certificate not found
    ///
    /// # Authentication
    /// * Requires authorization from a user with UpdateCertificateMetadata permission
    fn update_certificate_uri(
        env: Env,
        updater: Address,
        certificate_id: BytesN<32>,
        new_uri: String,
    ) -> Result<(), CertificateError>;

    /// Get certificate metadata
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `Option<CertificateMetadata>` - Certificate metadata if found
    fn get_certificate(env: Env, certificate_id: BytesN<32>) -> Option<CertificateMetadata>;

    /// Get all certificates owned by a user
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - Address of the user
    ///
    /// # Returns
    /// * `Vec<BytesN<32>>` - List of certificate IDs owned by the user
    fn get_user_certificates(env: Env, user: Address) -> Vec<BytesN<32>>;

    /// Get all certificates issued by an instructor
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `instructor` - Address of the instructor
    ///
    /// # Returns
    /// * `Vec<BytesN<32>>` - List of certificate IDs issued by the instructor
    fn get_instructor_certificates(env: Env, instructor: Address) -> Vec<BytesN<32>>;

    /// Get metadata update history for a certificate
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `Vec<MetadataUpdateEntry>` - List of metadata update entries
    fn get_metadata_history(env: Env, certificate_id: BytesN<32>) -> Vec<MetadataUpdateEntry>;

    /// Check if a certificate is expired
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `bool` - True if certificate is expired, false otherwise
    fn is_certificate_expired(env: Env, certificate_id: BytesN<32>) -> bool;

    /// Check if a certificate is valid (active and not expired)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `bool` - True if certificate is valid, false otherwise
    fn is_valid_certificate(env: Env, certificate_id: BytesN<32>) -> bool;
}
