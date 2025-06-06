use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::errors::CertificateError;
use crate::types::{CertificateMetadata, MetadataUpdateEntry, Permission, Role, MintCertificateParams};

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
    /// * `issuer` - Address of the issuer
    /// * `params` - Parameters for minting the certificate
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or invalid input
    ///
    /// # Authentication
    /// * Requires authorization from a user with Issue permission
    #[allow(clippy::too_many_arguments)]
    fn mint_certificate(
        env: Env,
        issuer: Address,
        params: MintCertificateParams,
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

    /// Check if a certificate is valid
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `Result<(bool, CertificateMetadata), CertificateError>` - Tuple containing validity status and certificate metadata
    ///   - First element is true if certificate is valid (active and not expired), false otherwise
    ///   - Second element contains the certificate metadata
    fn is_valid_certificate(
        env: Env,
        certificate_id: BytesN<32>,
    ) -> Result<(bool, CertificateMetadata), CertificateError>;

    /// Update the metadata URI of a certificate
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `updater` - Address of the user requesting the update
    /// * `certificate_id` - Unique identifier for the certificate
    /// * `new_uri` - New metadata URI
    ///
    /// # Returns
    /// * `Result<(), CertificateError>` - Ok if successful, Error if unauthorized or invalid input
    ///
    /// # Authentication
    /// * Requires authorization from the original issuer or admin
    fn update_certificate_uri(
        env: Env,
        updater: Address,
        certificate_id: BytesN<32>,
        new_uri: String,
    ) -> Result<(), CertificateError>;

    /// Get metadata update history for a certificate
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `certificate_id` - Unique identifier for the certificate
    ///
    /// # Returns
    /// * `Vec<MetadataUpdateEntry>` - Collection of metadata update entries, empty if none found
    fn get_metadata_history(env: Env, certificate_id: BytesN<32>) -> Vec<MetadataUpdateEntry>;
}
