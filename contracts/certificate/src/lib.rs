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
use types::{CertificateMetadata, CertificateStatus, Permission, Role};

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

#[contract]
pub struct Certificate;

#[contractimpl]
impl CertificateTrait for Certificate {
    fn initialize(env: Env, admin: Address) -> Result<(), CertificateError> {
        // Check if already initialized
        if CertificateStorage::is_initialized(&env) {
            return Err(CertificateError::AlreadyInitialized);
        }

        // Require authorization from the admin
        admin.require_auth();

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

    fn grant_role(env: Env, user: Address, role: Role) -> Result<(), CertificateError> {
        // Check if contract is initialized and get admin
        if !CertificateStorage::is_initialized(&env) {
            return Err(CertificateError::NotInitialized);
        }

        let admin = CertificateStorage::get_admin(&env);
        admin.require_auth();

        // Set the role
        CertificateStorage::set_role(&env, &user, &role);

        // Emit role added event
        CertificateEvents::emit_role_added(&env, &user, &role);

        Ok(())
    }

    fn update_role(env: Env, user: Address, new_role: Role) -> Result<(), CertificateError> {
        // Check if contract is initialized and get admin
        if !CertificateStorage::is_initialized(&env) {
            return Err(CertificateError::NotInitialized);
        }

        let admin = CertificateStorage::get_admin(&env);
        admin.require_auth();

        // Ensure the role exists before updating
        if CertificateStorage::get_role(&env, &user).is_none() {
            return Err(CertificateError::RoleNotFound);
        }

        // Update the role
        CertificateStorage::set_role(&env, &user, &new_role);

        // Emit role updated event
        CertificateEvents::emit_role_updated(&env, &user, &new_role);

        Ok(())
    }

    fn revoke_role(env: Env, user: Address) -> Result<(), CertificateError> {
        // Check if contract is initialized and get admin
        if !CertificateStorage::is_initialized(&env) {
            return Err(CertificateError::NotInitialized);
        }

        let admin = CertificateStorage::get_admin(&env);
        admin.require_auth();

        // Ensure the role exists before revoking
        if CertificateStorage::get_role(&env, &user).is_none() {
            return Err(CertificateError::RoleNotFound);
        }

        // Remove the role
        CertificateStorage::remove_role(&env, &user);

        // Emit role removed event
        CertificateEvents::emit_role_removed(&env, &user);

        Ok(())
    }

    fn get_role(env: Env, user: Address) -> Option<Role> {
        CertificateStorage::get_role(&env, &user)
    }

    fn has_permission(env: Env, user: Address, permission: Permission) -> bool {
        if let Some(role) = CertificateStorage::get_role(&env, &user) {
            role.has(permission)
        } else {
            false
        }
    }

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
    ) -> Result<(), CertificateError> {
        // Check if initialized
        if !CertificateStorage::is_initialized(&env) {
            return Err(CertificateError::NotInitialized);
        }

        // Get caller and check permission
        if !Self::has_permission(env.clone(), issuer.clone(), Permission::Issue) {
            return Err(CertificateError::Unauthorized);
        }

        // Validate inputs
        if title.is_empty()
            || description.is_empty()
            || metadata_uri.is_empty()
            || course_id.is_empty()
        {
            return Err(CertificateError::InvalidMetadata);
        }

        // Check if certificate already exists
        if CertificateStorage::has_certificate(&env, &certificate_id) {
            return Err(CertificateError::CertificateAlreadyExists);
        }

        let token_id = certificate_id.clone();

        // Create certificate metadata
        let metadata = CertificateMetadata {
            course_id,
            student_id: student.clone(),
            instructor_id: issuer.clone(),
            issue_date: env.ledger().timestamp(),
            metadata_uri,
            token_id: token_id.clone(),
            title,
            description,
            status: CertificateStatus::Active,
            expiry_date,
        };

        // Store certificate metadata
        CertificateStorage::set_certificate(&env, &certificate_id, &metadata);

        // Add to user's certificates
        CertificateStorage::add_user_certificate(&env, &student, &certificate_id);

        // Emit certificate minted event
        CertificateEvents::emit_certificate_minted(
            &env,
            &certificate_id,
            &metadata,
            &student,
            &issuer,
            &token_id,
        );

        Ok(())
    }

    fn is_certificate_expired(env: Env, certificate_id: BytesN<32>) -> bool {
        if let Some(metadata) = CertificateStorage::get_certificate(&env, &certificate_id) {
            if metadata.expiry_date == 0 {
                return false;
            }

            metadata.expiry_date < env.ledger().timestamp()
        } else {
            true
        }
    }

    fn verify_certificate(
        env: Env,
        certificate_id: BytesN<32>,
    ) -> Result<CertificateMetadata, CertificateError> {
        // Check if certificate exists and get metadata
        let metadata = CertificateStorage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        // Check if certificate is revoked
        if metadata.status == CertificateStatus::Revoked {
            return Err(CertificateError::CertificateRevoked);
        }

        // Check if certificate is expired
        if Self::is_certificate_expired(env, certificate_id) {
            return Err(CertificateError::CertificateExpired);
        }

        Ok(metadata)
    }

    fn revoke_certificate(
        env: Env,
        revoker: Address,
        certificate_id: BytesN<32>,
    ) -> Result<(), CertificateError> {
        // Check if initialized
        if !CertificateStorage::is_initialized(&env) {
            return Err(CertificateError::NotInitialized);
        }

        // Get caller and check permission
        if !Self::has_permission(env.clone(), revoker.clone(), Permission::Revoke) {
            return Err(CertificateError::Unauthorized);
        }

        // Check if certificate exists and get metadata
        let mut metadata = CertificateStorage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        // Update certificate status
        metadata.status = CertificateStatus::Revoked;

        // Store updated metadata
        CertificateStorage::set_certificate(&env, &certificate_id, &metadata);

        // Get current timestamp
        let timestamp = env.ledger().timestamp();

        // Emit certificate revoked event
        CertificateEvents::emit_certificate_revoked(
            &env,
            &certificate_id,
            &metadata,
            &revoker,
            timestamp,
        );

        Ok(())
    }

    fn track_certificates(env: Env, user_address: Address) -> Vec<BytesN<32>> {
        CertificateStorage::get_user_certificates(&env, &user_address)
    }

    fn add_user_certificate(
        env: Env,
        user_address: Address,
        certificate_id: BytesN<32>,
    ) -> Result<(), CertificateError> {
        // Verify certificate exists
        if !CertificateStorage::has_certificate(&env, &certificate_id) {
            return Err(CertificateError::CertificateNotFound);
        }

        // Add certificate to user's list
        CertificateStorage::add_user_certificate(&env, &user_address, &certificate_id);

        Ok(())
    }
}
