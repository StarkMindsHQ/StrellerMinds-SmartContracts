//! 2FA Integration for Certificate Contract
//! 
//! Integrates two-factor authentication with certificate operations,
//! enforcing 2FA for admin functions and providing optional 2FA for users.

use soroban_sdk::{Address, BytesN, Env, String, Vec, Symbol};

use crate::errors::CertificateError;
use shared::two_factor_auth::{
    TwoFactorConfig, TwoFactorMethod, TwoFactorResult, TwoFactorError,
    initialize_2fa, enable_2fa, disable_2fa, verify_2fa, send_sms_code,
    is_2fa_required, get_2fa_config, has_2fa_config
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
    /// Additional context data
    pub context_data: Vec<String>,
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
    /// Operations that always require 2FA regardless of user role
    pub critical_operations: Vec<CertificateOperation>,
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
    phone_number: Option<String>,
) -> Result<(), CertificateError> {
    initialize_2fa(env, admin, true, Some(totp_secret), phone_number)
        .map_err(|e| convert_2fa_error(e))
}

/// Enable 2FA for a certificate user
pub fn enable_user_2fa(
    env: &Env,
    user: &Address,
    totp_secret: BytesN<32>,
    phone_number: Option<String>,
) -> Result<(), CertificateError> {
    enable_2fa(env, user, totp_secret, phone_number)
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
        TwoFactorResult::Success => {
            // Log successful 2FA verification
            log_certificate_2fa_event(env, context, true);
            Ok(())
        }
        TwoFactorResult::AccountLocked => {
            log_certificate_2fa_event(env, context, false);
            Err(CertificateError::TwoFactorAccountLocked)
        }
        TwoFactorResult::InvalidCode => {
            log_certificate_2fa_event(env, context, false);
            Err(CertificateError::TwoFactorInvalidCode)
        }
        TwoFactorResult::Expired => {
            log_certificate_2fa_event(env, context, false);
            Err(CertificateError::TwoFactorCodeExpired)
        }
        TwoFactorResult::NotEnabled => {
            log_certificate_2fa_event(env, context, false);
            Err(CertificateError::TwoFactorNotEnabled)
        }
        TwoFactorResult::InvalidMethod => {
            log_certificate_2fa_event(env, context, false);
            Err(CertificateError::TwoFactorInvalidMethod)
        }
    }
}

/// Check if 2FA is required for a specific operation
pub fn is_2fa_required_for_operation(
    env: &Env,
    context: &Certificate2FAContext,
    policy: &TwoFactorPolicy,
) -> Result<bool, CertificateError> {
    // Check if it's a critical operation
    if policy.critical_operations.contains(&context.operation) {
        return Ok(true);
    }
    
    // Check if user has 2FA configured
    if !has_2fa_config(env, &context.user) {
        return Ok(false);
    }
    
    let config = get_2fa_config(env, &context.user)
        .map_err(|e| convert_2fa_error(e))?;
    
    // If 2FA is mandatory for this user, require it
    if config.mandatory {
        return Ok(true);
    }
    
    // Check if this is an admin operation
    if is_admin_operation(&context.operation) && policy.admin_mandatory {
        return Ok(true);
    }
    
    // Check if user has opted into 2FA for this type of operation
    if config.enabled && policy.user_optional {
        return Ok(true);
    }
    
    Ok(false)
}

/// Send SMS code for certificate operation verification
pub fn send_certificate_operation_sms(
    env: &Env,
    user: &Address,
    operation: &CertificateOperation,
) -> Result<(), CertificateError> {
    send_sms_code(env, user)
        .map_err(|e| convert_2fa_error(e))?;
    
    // Log SMS code generation
    log_certificate_sms_event(env, user, operation);
    
    Ok(())
}

/// Create 2FA verification context for certificate operation
pub fn create_certificate_2fa_context(
    operation: CertificateOperation,
    user: Address,
    verification_code: String,
    method: TwoFactorMethod,
    context_data: Vec<String>,
) -> Certificate2FAContext {
    Certificate2FAContext {
        operation,
        user,
        verification_code,
        method,
        context_data,
    }
}

/// Get default 2FA policy for certificate contract
pub fn get_default_2fa_policy() -> TwoFactorPolicy {
    TwoFactorPolicy {
        admin_mandatory: true,
        user_optional: true,
        critical_operations: vec![
            CertificateOperation::InitializeContract,
            CertificateOperation::TransferAdmin,
            CertificateOperation::EmergencyRevoke,
        ],
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

/// Convert 2FA error to certificate error
fn convert_2fa_error(error: TwoFactorError) -> CertificateError {
    match error {
        TwoFactorError::AlreadyConfigured => CertificateError::TwoFactorAlreadyConfigured,
        TwoFactorError::AlreadyEnabled => CertificateError::TwoFactorAlreadyEnabled,
        TwoFactorError::NotConfigured => CertificateError::TwoFactorNotConfigured,
        TwoFactorError::TOTPNotConfigured => CertificateError::TwoFactorTOTPNotConfigured,
        TwoFactorError::SMSNotConfigured => CertificateError::TwoFactorSMSNotConfigured,
        TwoFactorError::InvalidRecoveryCode => CertificateError::TwoFactorInvalidRecoveryCode,
        TwoFactorError::AccountLocked => CertificateError::TwoFactorAccountLocked,
        TwoFactorError::InvalidMethod => CertificateError::TwoFactorInvalidMethod,
        TwoFactorError::InternalError => CertificateError::InternalError,
    }
}

/// Log certificate 2FA event
fn log_certificate_2fa_event(
    env: &Env,
    context: &Certificate2FAContext,
    success: bool,
) {
    let event_type = if success {
        Symbol::from_str(env, "certificate_2fa_success")
    } else {
        Symbol::from_str(env, "certificate_2fa_failed")
    };
    
    let mut topics = Vec::new(env);
    topics.push_back(context.user.clone());
    topics.push_back(Symbol::from_str(env, &format!("{:?}", context.operation)));
    topics.push_back(Symbol::from_str(env, &format!("{:?}", context.method)));
    
    env.events().publish(event_type, topics);
}

/// Log certificate SMS event
fn log_certificate_sms_event(
    env: &Env,
    user: &Address,
    operation: &CertificateOperation,
) {
    let event_type = Symbol::from_str(env, "certificate_sms_sent");
    
    let mut topics = Vec::new(env);
    topics.push_back(user.clone());
    topics.push_back(Symbol::from_str(env, &format!("{:?}", operation)));
    
    env.events().publish(event_type, topics);
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
        Vec::new(env),
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
    authorized_approvers: &Vec<Address>,
    required_approvals: &u32,
    timeout_duration: &u64,
    priority: &String,
    verification_code: &String,
    method: TwoFactorMethod,
) -> Result<(), CertificateError> {
    // Create 2FA context
    let mut context_data = Vec::new(env);
    context_data.push_back(course_id.clone());
    context_data.push_back(String::from_str(env, &required_approvals.to_string()));
    
    let context = create_certificate_2fa_context(
        CertificateOperation::ConfigureMultiSig,
        admin.clone(),
        verification_code.clone(),
        method,
        context_data,
    );
    
    // Verify 2FA
    let policy = get_default_2fa_policy();
    verify_certificate_operation_2fa(env, &context, &policy)?;
    
    // Proceed with multi-sig configuration
    let config = crate::types::MultiSigConfig {
        course_id: course_id.clone(),
        required_approvals: *required_approvals,
        authorized_approvers: authorized_approvers.clone(),
        timeout_duration: *timeout_duration,
        priority: crate::types::CertificatePriority::Standard, // Convert from string
        auto_execute: true,
    };
    
    crate::storage::set_multisig_config(env, course_id, &config);
    
    Ok(())
}

/// Enhanced certificate revocation with 2FA
pub fn revoke_certificate_with_2fa(
    env: &Env,
    admin: &Address,
    certificate_id: &BytesN<32>,
    reason: &String,
    verification_code: &String,
    method: TwoFactorMethod,
) -> Result<(), CertificateError> {
    // Create 2FA context
    let mut context_data = Vec::new(env);
    context_data.push_back(String::from_str(env, &hex::encode(certificate_id)));
    context_data.push_back(reason.clone());
    
    let context = create_certificate_2fa_context(
        CertificateOperation::RevokeCertificate,
        admin.clone(),
        verification_code.clone(),
        method,
        context_data,
    );
    
    // Verify 2FA
    let policy = get_default_2fa_policy();
    verify_certificate_operation_2fa(env, &context, &policy)?;
    
    // Proceed with certificate revocation
    crate::storage::revoke_certificate(env, certificate_id, reason, false)
}

/// Enhanced multi-sig approval with optional 2FA
pub fn process_multisig_approval_with_2fa(
    env: &Env,
    approver: &Address,
    request_id: &BytesN<32>,
    approved: bool,
    comments: &String,
    verification_code: Option<String>,
    method: Option<TwoFactorMethod>,
) -> Result<(), CertificateError> {
    // Check if 2FA is required for this approver
    let policy = get_default_2fa_policy();
    
    if let (Some(code), Some(auth_method)) = (verification_code, method) {
        // Create 2FA context
        let mut context_data = Vec::new(env);
        context_data.push_back(String::from_str(env, &hex::encode(request_id)));
        context_data.push_back(String::from_str(env, &approved.to_string()));
        context_data.push_back(comments.clone());
        
        let context = create_certificate_2fa_context(
            CertificateOperation::ProcessMultiSigApproval,
            approver.clone(),
            code,
            auth_method,
            context_data,
        );
        
        // Verify 2FA
        verify_certificate_operation_2fa(env, &context, &policy)?;
    } else {
        // Check if 2FA is mandatory for this user
        if has_2fa_config(env, approver) {
            let config = get_2fa_config(env, approver)
                .map_err(|e| convert_2fa_error(e))?;
            
            if config.mandatory {
                return Err(CertificateError::TwoFactorRequired);
            }
        }
    }
    
    // Proceed with multi-sig approval
    crate::storage::process_multisig_approval(env, approver, request_id, approved, comments)
}

// ─────────────────────────────────────────────────────────────
// 2FA Status and Management Functions
// ─────────────────────────────────────────────────────────────

/// Get 2FA status for a certificate user
pub fn get_user_2fa_status(env: &Env, user: &Address) -> Result<TwoFactorConfig, CertificateError> {
    get_2fa_config(env, user)
        .map_err(|e| convert_2fa_error(e))
}

/// Check if user can perform certificate operations without 2FA
pub fn can_bypass_2fa(env: &Env, user: &Address, operation: &CertificateOperation) -> Result<bool, CertificateError> {
    if !has_2fa_config(env, user) {
        return Ok(true);
    }
    
    let config = get_2fa_config(env, user)
        .map_err(|e| convert_2fa_error(e))?;
    
    // Cannot bypass if 2FA is mandatory
    if config.mandatory {
        return Ok(false);
    }
    
    // Cannot bypass for critical operations
    let policy = get_default_2fa_policy();
    if policy.critical_operations.contains(operation) {
        return Ok(false);
    }
    
    // Can bypass if 2FA is disabled
    Ok(!config.enabled)
}

/// Generate 2FA setup instructions for certificate users
pub fn generate_2fa_setup_instructions(env: &Env, user: &Address) -> Result<String, CertificateError> {
    let mut instructions = String::from_str(env, "2FA Setup Instructions:\n");
    instructions = instructions.concat(&String::from_str(env, "========================\n\n"));
    
    if has_2fa_config(env, user) {
        instructions = instructions.concat(&String::from_str(env, "✅ 2FA is already configured for your account.\n"));
        
        let config = get_2fa_config(env, user)
            .map_err(|e| convert_2fa_error(e))?;
        
        if config.enabled {
            instructions = instructions.concat(&String::from_str(env, "✅ 2FA is currently enabled.\n"));
        } else {
            instructions = instructions.concat(&String::from_str(env, "⚠️  2FA is configured but not enabled.\n"));
        }
        
        if config.mandatory {
            instructions = instructions.concat(&String::from_str(env, "🔒 2FA is mandatory for your account.\n"));
        }
    } else {
        instructions = instructions.concat(&String::from_str(env, "❌ 2FA is not configured for your account.\n\n"));
        instructions = instructions.concat(&String::from_str(env, "To enable 2FA:\n"));
        instructions = instructions.concat(&String::from_str(env, "1. Generate a TOTP secret using your authenticator app\n"));
        instructions = instructions.concat(&String::from_str(env, "2. Call enable_user_2fa with your secret\n"));
        instructions = instructions.concat(&String::from_str(env, "3. Optionally add a phone number for SMS backup\n"));
        instructions = instructions.concat(&String::from_str(env, "4. Save your recovery codes in a secure location\n"));
    }
    
    Ok(instructions)
}
