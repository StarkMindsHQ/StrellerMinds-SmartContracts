use soroban_sdk::{Address, BytesN, Env, String, Vec};
use crate::errors::CertificateError;
use crate::types::{
    CertificateMetadata, CertificateStatus, RenewalRequest, RenewalStatus, 
    ExpiryNotification, NotificationType, BulkExpiryOperation, ExtensionParams,
    PackedCertificateData, DataKey
};
use crate::storage::CertificateStorage;
use crate::events::CertificateEvents;

/// Configuration for expiry management
pub struct ExpiryConfig;

impl ExpiryConfig {
    // Notification thresholds (in seconds)
    pub const WARNING_30_DAYS: u64 = 30 * 24 * 60 * 60;
    pub const WARNING_7_DAYS: u64 = 7 * 24 * 60 * 60;
    pub const WARNING_1_DAY: u64 = 24 * 60 * 60;
    
    // Renewal limits
    pub const MAX_RENEWAL_COUNT: u32 = 5;
    pub const MIN_EXTENSION_PERIOD: u64 = 30 * 24 * 60 * 60; // 30 days
    pub const MAX_EXTENSION_PERIOD: u64 = 5 * 365 * 24 * 60 * 60; // 5 years
    
    // Bulk operation limits
    pub const MAX_BULK_OPERATION_SIZE: usize = 50;
    
    // Request expiry
    pub const RENEWAL_REQUEST_EXPIRY: u64 = 30 * 24 * 60 * 60; // 30 days
}

/// Advanced expiry management functionality
pub struct ExpiryManager;

impl ExpiryManager {
    /// Request certificate renewal
    pub fn request_renewal(
        env: &Env,
        requester: &Address,
        certificate_id: &BytesN<32>,
        requested_extension: u64,
        reason: String,
    ) -> Result<(), CertificateError> {
        // Validate certificate exists and is owned by requester
        let packed = CertificateStorage::get_certificate(env, certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;
        
        if packed.owner != *requester {
            return Err(CertificateError::Unauthorized);
        }
        
        // Check if certificate is eligible for renewal
        Self::validate_renewal_eligibility(env, &packed)?;
        
        // Validate extension period
        if requested_extension < ExpiryConfig::MIN_EXTENSION_PERIOD 
            || requested_extension > ExpiryConfig::MAX_EXTENSION_PERIOD {
            return Err(CertificateError::InvalidInput);
        }
        
        // Check if there's already a pending request
        if let Some(existing_request) = Self::get_renewal_request(env, certificate_id) {
            if existing_request.status == RenewalStatus::Pending {
                return Err(CertificateError::InvalidInput); // Already has pending request
            }
        }
        
        // Create renewal request
        let renewal_request = RenewalRequest {
            certificate_id: certificate_id.clone(),
            requester: requester.clone(),
            requested_extension,
            reason,
            request_date: env.ledger().timestamp(),
            status: RenewalStatus::Pending,
            approver: None,
            approval_date: None,
        };
        
        // Store renewal request
        Self::set_renewal_request(env, certificate_id, &renewal_request);
        
        // Update certificate status
        let mut updated_packed = packed;
        updated_packed.metadata.status = CertificateStatus::PendingRenewal;
        CertificateStorage::set_certificate(env, certificate_id, &updated_packed);
        
        // Emit event
        CertificateEvents::emit_renewal_requested(env, certificate_id, requester, requested_extension);
        
        Ok(())
    }
    
    /// Approve or reject renewal request (admin only)
    pub fn process_renewal_request(
        env: &Env,
        approver: &Address,
        certificate_id: &BytesN<32>,
        approved: bool,
        admin_reason: Option<String>,
    ) -> Result<(), CertificateError> {
        // Get renewal request
        let mut renewal_request = Self::get_renewal_request(env, certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;
        
        if renewal_request.status != RenewalStatus::Pending {
            return Err(CertificateError::InvalidInput);
        }
        
        // Check if request has expired
        let current_time = env.ledger().timestamp();
        if current_time > renewal_request.request_date + ExpiryConfig::RENEWAL_REQUEST_EXPIRY {
            renewal_request.status = RenewalStatus::Expired;
            Self::set_renewal_request(env, certificate_id, &renewal_request);
            return Err(CertificateError::CertificateExpired);
        }
        
        // Get certificate
        let mut packed = CertificateStorage::get_certificate(env, certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;
        
        if approved {
            // Approve renewal
            renewal_request.status = RenewalStatus::Approved;
            renewal_request.approver = Some(approver.clone());
            renewal_request.approval_date = Some(current_time);
            
            // Update certificate
            packed.metadata.expiry_date += renewal_request.requested_extension;
            packed.metadata.renewal_count += 1;
            packed.metadata.last_renewed_date = current_time;
            packed.metadata.status = CertificateStatus::Renewed;
            
            // Emit approval event
            CertificateEvents::emit_renewal_approved(
                env, 
                certificate_id, 
                approver, 
                &renewal_request.requester,
                renewal_request.requested_extension
            );
        } else {
            // Reject renewal
            renewal_request.status = RenewalStatus::Rejected;
            renewal_request.approver = Some(approver.clone());
            renewal_request.approval_date = Some(current_time);
            
            // Revert certificate status
            packed.metadata.status = if current_time > packed.metadata.expiry_date {
                CertificateStatus::Expired
            } else {
                CertificateStatus::Active
            };
            
            // Emit rejection event
            CertificateEvents::emit_renewal_rejected(
                env, 
                certificate_id, 
                approver, 
                &renewal_request.requester,
                admin_reason.unwrap_or_else(|| String::from_str(env, "No reason provided"))
            );
        }
        
        // Update storage
        Self::set_renewal_request(env, certificate_id, &renewal_request);
        CertificateStorage::set_certificate(env, certificate_id, &packed);
        
        Ok(())
    }
    
    /// Extend certificate expiry (admin only)
    pub fn extend_certificate_expiry(
        env: &Env,
        admin: &Address,
        params: &ExtensionParams,
    ) -> Result<(), CertificateError> {
        // Validate extension parameters
        if params.extension_period < ExpiryConfig::MIN_EXTENSION_PERIOD 
            || params.extension_period > ExpiryConfig::MAX_EXTENSION_PERIOD {
            return Err(CertificateError::InvalidInput);
        }
        
        // Get certificate
        let mut packed = CertificateStorage::get_certificate(env, &params.certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;
        
        // Check renewal limits if specified
        if let Some(max_renewals) = params.max_renewals {
            if packed.metadata.renewal_count >= max_renewals {
                return Err(CertificateError::InvalidInput);
            }
        }
        
        // Check global renewal limit
        if packed.metadata.renewal_count >= ExpiryConfig::MAX_RENEWAL_COUNT {
            return Err(CertificateError::InvalidInput);
        }
        
        let current_time = env.ledger().timestamp();
        
        // Update certificate
        packed.metadata.expiry_date += params.extension_period;
        packed.metadata.renewal_count += 1;
        packed.metadata.last_renewed_date = current_time;
        
        // Update status based on current state
        packed.metadata.status = match packed.metadata.status {
            CertificateStatus::Expired => CertificateStatus::Renewed,
            CertificateStatus::PendingRenewal => CertificateStatus::Renewed,
            _ => CertificateStatus::Active,
        };
        
        // Store updated certificate
        CertificateStorage::set_certificate(env, &params.certificate_id, &packed);
        
        // Emit extension event
        CertificateEvents::emit_certificate_extended(
            env,
            &params.certificate_id,
            admin,
            &packed.owner,
            params.extension_period,
            params.reason.clone()
        );
        
        Ok(())
    }
    
    /// Bulk extend multiple certificates
    pub fn bulk_extend_certificates(
        env: &Env,
        admin: &Address,
        operation: &BulkExpiryOperation,
    ) -> Result<Vec<BytesN<32>>, CertificateError> {
        // Validate bulk operation size
        if operation.certificate_ids.len() > ExpiryConfig::MAX_BULK_OPERATION_SIZE {
            return Err(CertificateError::InvalidInput);
        }
        
        if operation.certificate_ids.is_empty() {
            return Err(CertificateError::InvalidInput);
        }
        
        let current_time = env.ledger().timestamp();
        let mut successful_updates = Vec::new(env);
        
        // Process each certificate
        for cert_id in operation.certificate_ids.iter() {
            if let Some(mut packed) = CertificateStorage::get_certificate(env, cert_id) {
                // Skip revoked certificates
                if packed.metadata.status == CertificateStatus::Revoked {
                    continue;
                }
                
                // Check renewal limits
                if packed.metadata.renewal_count >= ExpiryConfig::MAX_RENEWAL_COUNT {
                    continue;
                }
                
                // Calculate extension period
                let extension_period = if operation.new_expiry_date > packed.metadata.expiry_date {
                    operation.new_expiry_date - packed.metadata.expiry_date
                } else {
                    continue; // Skip if new date is not later
                };
                
                // Update certificate
                packed.metadata.expiry_date = operation.new_expiry_date;
                packed.metadata.renewal_count += 1;
                packed.metadata.last_renewed_date = current_time;
                
                // Update status
                packed.metadata.status = match packed.metadata.status {
                    CertificateStatus::Expired => CertificateStatus::Renewed,
                    CertificateStatus::PendingRenewal => CertificateStatus::Renewed,
                    _ => CertificateStatus::Active,
                };
                
                // Store updated certificate
                CertificateStorage::set_certificate(env, cert_id, &packed);
                successful_updates.push_back(cert_id.clone());
                
                // Emit individual extension event
                CertificateEvents::emit_certificate_extended(
                    env,
                    cert_id,
                    admin,
                    &packed.owner,
                    extension_period,
                    operation.reason.clone()
                );
            }
        }
        
        // Emit bulk operation event
        CertificateEvents::emit_bulk_extension_completed(
            env,
            admin,
            successful_updates.len() as u32,
            operation.reason.clone()
        );
        
        Ok(successful_updates)
    }
    
    /// Generate expiry notifications for certificates
    pub fn generate_expiry_notifications(env: &Env) -> Result<u32, CertificateError> {
        let current_time = env.ledger().timestamp();
        let mut notifications_created = 0u32;
        
        // Get all active certificates (this would need to be implemented with proper indexing)
        // For now, we'll assume we have a way to iterate through certificates
        
        // This is a placeholder - in a real implementation, you'd need proper indexing
        // to efficiently find certificates approaching expiry
        
        Ok(notifications_created)
    }
    
    /// Create expiry notification for a specific certificate
    pub fn create_expiry_notification(
        env: &Env,
        certificate_id: &BytesN<32>,
        notification_type: NotificationType,
    ) -> Result<(), CertificateError> {
        let packed = CertificateStorage::get_certificate(env, certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;
        
        let notification = ExpiryNotification {
            certificate_id: certificate_id.clone(),
            owner: packed.owner.clone(),
            expiry_date: packed.metadata.expiry_date,
            notification_type: notification_type.clone(),
            created_at: env.ledger().timestamp(),
            acknowledged: false,
        };
        
        // Store notification
        Self::add_expiry_notification(env, &packed.owner, &notification);
        
        // Emit notification event
        CertificateEvents::emit_expiry_notification(
            env,
            certificate_id,
            &packed.owner,
            &notification_type,
            packed.metadata.expiry_date
        );
        
        Ok(())
    }
    
    /// Get expiry notifications for a user
    pub fn get_user_notifications(env: &Env, user: &Address) -> Vec<ExpiryNotification> {
        env.storage()
            .persistent()
            .get(&DataKey::ExpiryNotifications(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }
    
    /// Acknowledge expiry notification
    pub fn acknowledge_notification(
        env: &Env,
        user: &Address,
        certificate_id: &BytesN<32>,
    ) -> Result<(), CertificateError> {
        let mut notifications = Self::get_user_notifications(env, user);
        let mut found = false;
        
        for notification in notifications.iter_mut() {
            if notification.certificate_id == *certificate_id && !notification.acknowledged {
                notification.acknowledged = true;
                found = true;
                break;
            }
        }
        
        if !found {
            return Err(CertificateError::CertificateNotFound);
        }
        
        // Update storage
        env.storage()
            .persistent()
            .set(&DataKey::ExpiryNotifications(user.clone()), &notifications);
        
        Ok(())
    }
    
    /// Update expired certificates status
    pub fn update_expired_certificates(env: &Env) -> Result<u32, CertificateError> {
        let current_time = env.ledger().timestamp();
        let mut updated_count = 0u32;
        
        // This would need proper indexing in a real implementation
        // For now, this is a placeholder for the concept
        
        Ok(updated_count)
    }
    
    /// Check if certificate is eligible for renewal
    fn validate_renewal_eligibility(
        env: &Env,
        packed: &PackedCertificateData,
    ) -> Result<(), CertificateError> {
        // Check if certificate is revoked
        if packed.metadata.status == CertificateStatus::Revoked {
            return Err(CertificateError::CertificateRevoked);
        }
        
        // Check renewal count limit
        if packed.metadata.renewal_count >= ExpiryConfig::MAX_RENEWAL_COUNT {
            return Err(CertificateError::InvalidInput);
        }
        
        // Check if certificate is close to expiry or expired
        let current_time = env.ledger().timestamp();
        let time_until_expiry = if packed.metadata.expiry_date > current_time {
            packed.metadata.expiry_date - current_time
        } else {
            0
        };
        
        // Allow renewal if certificate expires within 60 days or is already expired
        if time_until_expiry > 60 * 24 * 60 * 60 {
            return Err(CertificateError::InvalidInput);
        }
        
        Ok(())
    }
    
    /// Get renewal request for certificate
    fn get_renewal_request(env: &Env, certificate_id: &BytesN<32>) -> Option<RenewalRequest> {
        env.storage()
            .persistent()
            .get(&DataKey::RenewalRequest(certificate_id.clone()))
    }
    
    /// Set renewal request for certificate
    fn set_renewal_request(env: &Env, certificate_id: &BytesN<32>, request: &RenewalRequest) {
        env.storage()
            .persistent()
            .set(&DataKey::RenewalRequest(certificate_id.clone()), request);
    }
    
    /// Add expiry notification for user
    fn add_expiry_notification(env: &Env, user: &Address, notification: &ExpiryNotification) {
        let mut notifications = Self::get_user_notifications(env, user);
        notifications.push_back(notification.clone());
        
        env.storage()
            .persistent()
            .set(&DataKey::ExpiryNotifications(user.clone()), &notifications);
    }
    
    /// Get certificates expiring within a time period
    pub fn get_expiring_certificates(
        env: &Env,
        within_seconds: u64,
    ) -> Vec<BytesN<32>> {
        let current_time = env.ledger().timestamp();
        let expiry_threshold = current_time + within_seconds;
        
        // This would need proper indexing in a real implementation
        // Return empty vector for now
        Vec::new(env)
    }
    
    /// Clean up expired renewal requests
    pub fn cleanup_expired_requests(env: &Env) -> Result<u32, CertificateError> {
        let current_time = env.ledger().timestamp();
        let mut cleaned_count = 0u32;
        
        // This would need proper indexing to efficiently find expired requests
        // For now, this is a placeholder
        
        Ok(cleaned_count)
    }
}
