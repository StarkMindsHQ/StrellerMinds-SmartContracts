#![no_std]

mod errors;
mod events;
mod interface;
mod storage;
mod types;

#[cfg(test)]
mod test;

use errors::CertificateError;
use events::CertificateEvents;
use interface::CertificateTrait;
use storage::CertificateStorage;
use types::{CertificateMetadata, CertificateStatus, MetadataUpdateEntry, MintCertificateParams};

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

// Import the shared RBAC system
use shared::{
    access_control::AccessControl,
    roles::{Permission, RoleLevel},
    errors::AccessControlError,
};

use shared::reentrancy_guard::ReentrancyLock;

#[contract]
pub struct Certificate;

#[contractimpl]
impl CertificateTrait for Certificate {
    fn initialize(env: Env, admin: Address) -> Result<(), CertificateError> {
        let _guard = ReentrancyLock::new(&env);
        // Check if already initialized
        if CertificateStorage::is_initialized(&env) {
            return Err(CertificateError::AlreadyInitialized);
        }

        // Require authorization from the admin
        admin.require_auth();

        // Initialize the RBAC system
        AccessControl::initialize(&env, &admin)
            .map_err(|_| CertificateError::InitializationFailed)?;

        // Store admin address and mark as initialized
        CertificateStorage::set_admin(&env, &admin);
        CertificateStorage::set_initialized(&env);

        // Emit initialization event
        CertificateEvents::emit_contract_initialized(&env, &admin);

        Ok(())
    }

    fn get_admin(env: Env) -> Result<Address, CertificateError> {
        if !CertificateStorage::is_initialized(&env) {
            return Err(CertificateError::NotInitialized);
        }

        Ok(CertificateStorage::get_admin(&env))
    }

    fn grant_role(env: Env, user: Address, role_level: u32) -> Result<(), CertificateError> {
        let _guard = ReentrancyLock::new(&env);
        // Get the caller's address
        let caller = env.current_contract_address();
        
        // Convert role level to enum
        let role_level = RoleLevel::from_u32(role_level)
            .ok_or(CertificateError::InvalidRole)?;

        // Grant role using RBAC system
        AccessControl::grant_role(&env, &caller, &user, role_level)
            .map_err(|_| CertificateError::Unauthorized)?;

        Ok(())
    }

    fn revoke_role(env: Env, user: Address) -> Result<(), CertificateError> {
        let _guard = ReentrancyLock::new(&env);
        // Get the caller's address
        let caller = env.current_contract_address();

        // Revoke role using RBAC system
        AccessControl::revoke_role(&env, &caller, &user)
            .map_err(|_| CertificateError::Unauthorized)?;

        Ok(())
    }

    fn get_role(env: Env, user: Address) -> Option<shared::roles::Role> {
        AccessControl::get_role(&env, &user)
    }

    fn has_permission(env: Env, user: Address, permission: u32) -> bool {
        // Convert permission to enum
        let permission = match permission {
            0 => Permission::IssueCertificate,
            1 => Permission::RevokeCertificate,
            2 => Permission::TransferCertificate,
            3 => Permission::UpdateCertificateMetadata,
            _ => return false,
        };

        AccessControl::has_permission(&env, &user, &permission)
    }

    fn mint_certificate(
        env: Env,
        issuer: Address,
        params: MintCertificateParams,
    ) -> Result<(), CertificateError> {
        let _guard = ReentrancyLock::new(&env);
        // Require authorization from issuer
        issuer.require_auth();

        // Check if issuer has permission to issue certificates
        AccessControl::require_permission(&env, &issuer, &Permission::IssueCertificate)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Validate certificate metadata
        if params.course_id.is_empty() || params.title.is_empty() || params.description.is_empty() {
            return Err(CertificateError::InvalidMetadata);
        }

        // Check if certificate already exists
        if CertificateStorage::has_certificate(&env, &params.certificate_id) {
            return Err(CertificateError::CertificateAlreadyExists);
        }

        // Create certificate metadata
        let metadata = CertificateMetadata {
            course_id: params.course_id,
            student_id: params.student,
            instructor_id: issuer,
            issue_date: env.ledger().timestamp(),
            metadata_uri: params.metadata_uri,
            token_id: params.certificate_id,
            title: params.title,
            description: params.description,
            status: CertificateStatus::Active,
            expiry_date: params.expiry_date,
        };

        // Store certificate
        CertificateStorage::set_certificate(&env, &params.certificate_id, &metadata);

        // Track certificate ownership
        CertificateStorage::add_user_certificate(&env, &params.student, &params.certificate_id);
        CertificateStorage::add_instructor_certificate(&env, &issuer, &params.certificate_id);

        // Emit certificate minted event
        CertificateEvents::emit_certificate_minted(&env, &issuer, &params.student, &metadata);

        Ok(())
    }

    fn revoke_certificate(
        env: Env,
        revoker: Address,
        certificate_id: BytesN<32>,
    ) -> Result<(), CertificateError> {
        let _guard = ReentrancyLock::new(&env);
        // Require authorization from revoker
        revoker.require_auth();

        // Check if revoker has permission to revoke certificates
        AccessControl::require_permission(&env, &revoker, &Permission::RevokeCertificate)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Get certificate metadata
        let mut metadata = CertificateStorage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        // Check if certificate is already revoked
        if metadata.status == CertificateStatus::Revoked {
            return Err(CertificateError::CertificateAlreadyRevoked);
        }

        // Update certificate status
        metadata.status = CertificateStatus::Revoked;
        CertificateStorage::set_certificate(&env, &certificate_id, &metadata);

        // Emit certificate revoked event
        CertificateEvents::emit_certificate_revoked(&env, &revoker, &certificate_id);

        Ok(())
    }

    fn transfer_certificate(
        env: Env,
        from: Address,
        to: Address,
        certificate_id: BytesN<32>,
    ) -> Result<(), CertificateError> {
        let _guard = ReentrancyLock::new(&env);
        // Require authorization from sender
        from.require_auth();

        // Check if sender has permission to transfer certificates
        AccessControl::require_permission(&env, &from, &Permission::TransferCertificate)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Get certificate metadata
        let metadata = CertificateStorage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        // Check if certificate is revoked
        if metadata.status == CertificateStatus::Revoked {
            return Err(CertificateError::CertificateRevoked);
        }

        // Check if sender owns the certificate
        if metadata.student_id != from {
            return Err(CertificateError::Unauthorized);
        }

        // Update certificate ownership
        CertificateStorage::remove_user_certificate(&env, &from, &certificate_id);
        CertificateStorage::add_user_certificate(&env, &to, &certificate_id);

        // Emit certificate transferred event
        CertificateEvents::emit_certificate_transferred(&env, &from, &to, &certificate_id);

        Ok(())
    }

    fn update_certificate_uri(
        env: Env,
        updater: Address,
        certificate_id: BytesN<32>,
        new_uri: String,
    ) -> Result<(), CertificateError> {
        let _guard = ReentrancyLock::new(&env);
        // Require authorization from updater
        updater.require_auth();

        // Check if updater has permission to update certificate metadata
        AccessControl::require_permission(&env, &updater, &Permission::UpdateCertificateMetadata)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Validate new URI
        if new_uri.is_empty() {
            return Err(CertificateError::InvalidMetadata);
        }

        // Get certificate metadata
        let mut metadata = CertificateStorage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        // Check if certificate is revoked
        if metadata.status == CertificateStatus::Revoked {
            return Err(CertificateError::CertificateRevoked);
        }

        // Store old URI for history
        let old_uri = metadata.metadata_uri.clone();

        // Update metadata URI
        metadata.metadata_uri = new_uri.clone();

        // Store updated certificate
        CertificateStorage::set_certificate(&env, &certificate_id, &metadata);

        // Add to metadata history
        let history_entry = MetadataUpdateEntry {
            updater,
            timestamp: env.ledger().timestamp(),
            old_uri,
            new_uri,
        };
        CertificateStorage::add_metadata_history(&env, &certificate_id, &history_entry);

        // Emit metadata updated event
        CertificateEvents::emit_metadata_updated(&env, &certificate_id, &old_uri, &new_uri);

        Ok(())
    }

    fn get_certificate(env: Env, certificate_id: BytesN<32>) -> Option<CertificateMetadata> {
        CertificateStorage::get_certificate(&env, &certificate_id)
    }

    fn get_user_certificates(env: Env, user: Address) -> Vec<BytesN<32>> {
        CertificateStorage::get_user_certificates(&env, &user)
    }

    fn get_instructor_certificates(env: Env, instructor: Address) -> Vec<BytesN<32>> {
        CertificateStorage::get_instructor_certificates(&env, &instructor)
    }

    fn get_metadata_history(env: Env, certificate_id: BytesN<32>) -> Vec<MetadataUpdateEntry> {
        CertificateStorage::get_metadata_history(&env, &certificate_id)
    }

    fn is_certificate_expired(env: Env, certificate_id: BytesN<32>) -> bool {
        if let Some(metadata) = CertificateStorage::get_certificate(&env, &certificate_id) {
            let current_time = env.ledger().timestamp();
            current_time > metadata.expiry_date
        } else {
            false
        }
    }

    fn is_valid_certificate(env: Env, certificate_id: BytesN<32>) -> bool {
        if let Some(metadata) = CertificateStorage::get_certificate(&env, &certificate_id) {
            let current_time = env.ledger().timestamp();
            metadata.status == CertificateStatus::Active && current_time <= metadata.expiry_date
        } else {
            false
        }
    }
}
