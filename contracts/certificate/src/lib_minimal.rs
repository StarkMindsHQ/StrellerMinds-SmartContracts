#![no_std]

mod errors;
mod storage;
mod types;

#[cfg(test)]
mod gas_regression_tests_simple;

use errors::CertificateError;
use storage::CertificateStorage;
use types::{CertificateMetadata, CertificateStatus, MintCertificateParams, PackedCertificateData, MetadataUpdateEntry};

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

// Import the shared RBAC system
use shared::{
    access_control::AccessControl,
    roles::{Permission, RoleLevel},
    errors::AccessControlError,
};

use shared::reentrancy_guard::ReentrancyLock;

/// Interface for basic certificate operations needed for gas testing
pub trait SimpleCertificateTrait {
    fn initialize(env: Env, admin: Address) -> Result<(), CertificateError>;
    fn mint_certificate(
        env: Env,
        issuer: Address,
        params: MintCertificateParams,
    ) -> Result<(), CertificateError>;
    fn get_certificate(env: Env, certificate_id: BytesN<32>) -> Option<CertificateMetadata>;
    fn get_user_certificates(env: Env, user: Address) -> Vec<BytesN<32>>;
    fn mint_certificates_batch(
        env: Env,
        issuer: Address,
        params_list: Vec<MintCertificateParams>,
    ) -> Result<(), CertificateError>;
}

#[contract]
pub struct Certificate;

#[contractimpl]
impl SimpleCertificateTrait for Certificate {
    fn initialize(env: Env, admin: Address) -> Result<(), CertificateError> {
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

        Ok(())
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

        // Check if certificate already exists
        if CertificateStorage::has_certificate(&env, &params.certificate_id) {
            return Err(CertificateError::CertificateAlreadyExists);
        }

        // Create certificate metadata
        let metadata = CertificateMetadata {
            course_id: params.course_id,
            student_id: params.student.clone(),
            instructor_id: issuer.clone(),
            issue_date: env.ledger().timestamp(),
            metadata_uri: params.metadata_uri,
            token_id: params.certificate_id.clone(),
            title: params.title,
            description: params.description,
            status: CertificateStatus::Active,
            expiry_date: params.expiry_date,
            original_expiry_date: params.expiry_date,
            renewal_count: 0,
            last_renewed_date: 0,
        };
        
        let packed = PackedCertificateData {
            metadata: metadata.clone(),
            owner: params.student.clone(),
            history: Vec::new(&env),
        };
        
        // Store packed certificate
        CertificateStorage::set_certificate(&env, &params.certificate_id, &packed);

        // Track certificate ownership
        CertificateStorage::add_user_certificate(&env, &params.student, &params.certificate_id);
        CertificateStorage::add_instructor_certificate(&env, &issuer, &params.certificate_id);

        Ok(())
    }

    fn get_certificate(env: Env, certificate_id: BytesN<32>) -> Option<CertificateMetadata> {
        CertificateStorage::get_certificate(&env, &certificate_id).map(|packed| packed.metadata)
    }

    fn get_user_certificates(env: Env, user: Address) -> Vec<BytesN<32>> {
        CertificateStorage::get_user_certificates(&env, &user)
    }

    fn mint_certificates_batch(
        env: Env,
        issuer: Address,
        params_list: Vec<MintCertificateParams>,
    ) -> Result<(), CertificateError> {
        let _guard = ReentrancyLock::new(&env);
        
        // Require authorization from issuer
        issuer.require_auth();

        // Check if issuer has permission to issue certificates
        AccessControl::require_permission(&env, &issuer, &Permission::IssueCertificate)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Mint all certificates
        for params in params_list.iter() {
            // Check if certificate already exists
            if CertificateStorage::has_certificate(&env, &params.certificate_id) {
                return Err(CertificateError::CertificateAlreadyExists);
            }

            // Create certificate metadata
            let metadata = CertificateMetadata {
                course_id: params.course_id.clone(),
                student_id: params.student.clone(),
                instructor_id: issuer.clone(),
                issue_date: env.ledger().timestamp(),
                metadata_uri: params.metadata_uri.clone(),
                token_id: params.certificate_id.clone(),
                title: params.title.clone(),
                description: params.description.clone(),
                status: CertificateStatus::Active,
                expiry_date: params.expiry_date,
                original_expiry_date: params.expiry_date,
                renewal_count: 0,
                last_renewed_date: 0,
            };
            
            let packed = PackedCertificateData {
                metadata: metadata.clone(),
                owner: params.student.clone(),
                history: Vec::new(&env),
            };
            
            // Store packed certificate
            CertificateStorage::set_certificate(&env, &params.certificate_id, &packed);

            // Track certificate ownership
            CertificateStorage::add_user_certificate(&env, &params.student, &params.certificate_id);
            CertificateStorage::add_instructor_certificate(&env, &issuer, &params.certificate_id);
        }

        Ok(())
    }
}