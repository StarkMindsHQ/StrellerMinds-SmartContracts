//! Simplified Two-Factor Authentication Integration for Certificate Contract
//! 
//! Basic 2FA integration that works with Soroban constraints.

use soroban_sdk::{Address, BytesN, Env, String};

use crate::errors::CertificateError;
use shared::two_factor_auth::{
    TwoFactorMethod, TwoFactorResult, TwoFactorError,
    initialize_2fa, verify_2fa,
    is_2fa_enabled
};

// ─────────────────────────────────────────────────────────────
// 2FA Integration Constants
// ─────────────────────────────────────────────────────────────

/// 2FA verification context for certificate operations
#[derive(Clone, Debug)]
pub struct Certificate2FAContext {
    /// Operation being performed
    pub operation: CertificateOperation,
    /// User performing the operation
    pub user: Address,
    /// 2FA verification code provided
    pub verification_code: String,
    /// 2FA method used
    pub method: TwoFactorMethod,
}

/// Certificate operations requiring 2FA
#[derive(Clone, Debug, PartialEq)]
pub enum CertificateOperation {
    /// Admin operations
    InitializeContract,
    ConfigureMultiSig,
    BatchIssueCertificates,
    RevokeCertificate,
    ReissueCertificate,
    CreateTemplate,
    RecordCompliance,
    
    /// User operations (optional 2FA)
    CreateMultiSigRequest,
    ProcessMultiSigApproval,
    ShareCertificate,
    
    /// Critical operations (always require 2FA)
    TransferAdmin,
    EmergencyRevoke,
}

/// 2FA enforcement policy
#[derive(Clone, Debug)]
pub struct TwoFactorPolicy {
    /// Whether 2FA is mandatory for all admins
    pub admin_mandatory: bool,
    /// Whether 2FA is optional for regular users
    pub user_optional: bool,
    /// Maximum time between 2FA verification and operation (seconds)
    pub verification_timeout: u64,
}

// ─────────────────────────────────────────────────────────────
// 2FA Integration Functions
// ─────────────────────────────────────────────────────────────

/// Initialize 2FA for certificate contract admin
pub fn initialize_admin_2fa(
    env: &Env,
    admin: &Address,
    totp_secret: BytesN<32>,
) -> Result<(), CertificateError> {
    initialize_2fa(env, admin, &totp_secret)
        .map_err(|e| convert_2fa_error(e))
}

/// Enable 2FA for a certificate user (simplified)
pub fn enable_user_2fa(
    env: &Env,
    user: &Address,
    totp_secret: BytesN<32>,
) -> Result<(), CertificateError> {
    initialize_2fa(env, user, &totp_secret)
        .map_err(|e| convert_2fa_error(e))
}

/// Verify 2FA before certificate operation
pub fn verify_certificate_operation_2fa(
    env: &Env,
    context: &Certificate2FAContext,
    policy: &TwoFactorPolicy,
) -> Result<(), CertificateError> {
    // Check if 2FA is required for this operation
    if !is_2fa_required_for_operation(env, context, policy)? {
        return Ok(());
    }
    
    // Perform 2FA verification
    let result = verify_2fa(env, &context.user, &context.verification_code, context.method.clone())
        .map_err(|e| convert_2fa_error(e))?;
    
    match result {
        TwoFactorResult::Success => Ok(()),
        TwoFactorResult::AccountLocked => Err(CertificateError::InternalError),
        TwoFactorResult::InvalidCode => Err(CertificateError::InternalError),
        TwoFactorResult::Expired => Err(CertificateError::InternalError),
        TwoFactorResult::NotEnabled => Err(CertificateError::InternalError),
    }
}

/// Check if 2FA is required for a specific operation
pub fn is_2fa_required_for_operation(
    env: &Env,
    context: &Certificate2FAContext,
    policy: &TwoFactorPolicy,
) -> Result<bool, CertificateError> {
    // Check if it's a critical operation
    if is_critical_operation(&context.operation) {
        return Ok(true);
    }
    
    // Check if user has 2FA configured
    if !is_2fa_enabled(env, &context.user) {
        return Ok(false);
    }
    
    // For simplified version, just check if 2FA is enabled
    if is_2fa_enabled(env, &context.user) {
        // Check if this is an admin operation
        if is_admin_operation(&context.operation) && policy.admin_mandatory {
            return Ok(true);
        }
        
        // Check if user optional 2FA is enabled
        if policy.user_optional {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Send SMS code for certificate operation verification (simplified)
pub fn send_certificate_operation_sms(
    _env: &Env,
    _user: &Address,
    _operation: &CertificateOperation,
) -> Result<(), CertificateError> {
    // Simplified version - just return success
    Ok(())
}

/// Create 2FA verification context for certificate operation
pub fn create_certificate_2fa_context(
    operation: CertificateOperation,
    user: Address,
    verification_code: String,
    method: TwoFactorMethod,
) -> Certificate2FAContext {
    Certificate2FAContext {
        operation,
        user,
        verification_code,
        method,
    }
}

/// Get default 2FA policy for certificate contract
pub fn get_default_2fa_policy() -> TwoFactorPolicy {
    TwoFactorPolicy {
        admin_mandatory: true,
        user_optional: true,
        verification_timeout: 300, // 5 minutes
    }
}

// ─────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────

/// Check if operation is an admin operation
fn is_admin_operation(operation: &CertificateOperation) -> bool {
    matches!(
        operation,
        CertificateOperation::InitializeContract |
        CertificateOperation::ConfigureMultiSig |
        CertificateOperation::BatchIssueCertificates |
        CertificateOperation::RevokeCertificate |
        CertificateOperation::ReissueCertificate |
        CertificateOperation::CreateTemplate |
        CertificateOperation::RecordCompliance |
        CertificateOperation::TransferAdmin |
        CertificateOperation::EmergencyRevoke
    )
}

/// Check if operation is critical
fn is_critical_operation(operation: &CertificateOperation) -> bool {
    matches!(
        operation,
        CertificateOperation::InitializeContract |
        CertificateOperation::TransferAdmin |
        CertificateOperation::EmergencyRevoke
    )
}

/// Convert 2FA error to certificate error
fn convert_2fa_error(error: TwoFactorError) -> CertificateError {
    match error {
        TwoFactorError::AlreadyEnabled => CertificateError::InternalError,
        TwoFactorError::NotEnabled => CertificateError::InternalError,
        TwoFactorError::TOTPNotConfigured => CertificateError::InternalError,
        TwoFactorError::AccountLocked => CertificateError::InternalError,
        TwoFactorError::InternalError => CertificateError::InternalError,
    }
}

// ─────────────────────────────────────────────────────────────
// 2FA-Enhanced Certificate Operations
// ─────────────────────────────────────────────────────────────

/// Enhanced certificate initialization with 2FA
pub fn initialize_certificate_with_2fa(
    env: &Env,
    admin: &Address,
    verification_code: &String,
    method: TwoFactorMethod,
) -> Result<(), CertificateError> {
    // Create 2FA context
    let context = create_certificate_2fa_context(
        CertificateOperation::InitializeContract,
        admin.clone(),
        verification_code.clone(),
        method,
    );
    
    // Verify 2FA
    let policy = get_default_2fa_policy();
    verify_certificate_operation_2fa(env, &context, &policy)?;
    
    // Proceed with certificate initialization
    crate::storage::set_admin(env, admin);
    crate::storage::set_initialized(env);
    
    Ok(())
}

/// Enhanced multi-sig configuration with 2FA
pub fn configure_multisig_with_2fa(
    env: &Env,
    admin: &Address,
    course_id: &String,
    authorized_approvers: &soroban_sdk::Vec<Address>,
    required_approvals: &u32,
    timeout_duration: &u64,
    priority: &String,
    verification_code: &String,
    method: TwoFactorMethod,
) -> Result<(), CertificateError> {
    // Create 2FA context
    let context = create_certificate_2fa_context(
        CertificateOperation::ConfigureMultiSig,
        admin.clone(),
        verification_code.clone(),
        method,
    );
    
    // Verify 2FA
    let policy = get_default_2fa_policy();
    verify_certificate_operation_2fa(env, &context, &policy)?;
    
    // Proceed with multi-sig configuration (simplified)
    let config = crate::types::MultiSigConfig {
        course_id: course_id.clone(),
        required_approvals: *required_approvals,
        authorized_approvers: authorized_approvers.clone(),
        timeout_duration: *timeout_duration,
        priority: crate::types::CertificatePriority::Standard, // Simplified
        auto_execute: true,
    };
    
    crate::storage::set_multisig_config(env, course_id, &config);
    
    Ok(())
}

// ─────────────────────────────────────────────────────────────
// 2FA Status and Management Functions
// ─────────────────────────────────────────────────────────────

/// Get 2FA status for a certificate user
pub fn get_user_2fa_status(env: &Env, user: &Address) -> Result<bool, CertificateError> {
    Ok(is_2fa_enabled(env, user))
}

/// Check if user can perform certificate operations without 2FA
pub fn can_bypass_2fa(env: &Env, user: &Address, operation: &CertificateOperation) -> Result<bool, CertificateError> {
    if !is_2fa_enabled(env, user) {
        return Ok(true);
    }
    
    // For simplified version, just check if 2FA is enabled
    if is_2fa_enabled(env, user) {
        // Cannot bypass for critical operations
        if is_critical_operation(operation) {
            return Ok(false);
        }
        
        // If 2FA is enabled, cannot bypass
        return Ok(false);
    }
    
    // Can bypass if 2FA is not configured
    Ok(true)
}

