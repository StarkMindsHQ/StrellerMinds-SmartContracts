#![no_std]

pub mod errors;

use crate::errors::GdprError;
use shared::gdpr_types::{
    GdprAnalyticsExport, GdprCommunityExport, GdprExportData, GdprExportRecord,
    GdprGamificationExport,
};
use shared::monitoring::{ContractHealthReport, Monitor};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, Vec};

const AUDIT_RETENTION_SECONDS: u64 = 365 * 86_400;
const EXPORT_EXPIRATION_SECONDS: u64 = 24 * 86_400;
const ENCRYPTION_VERSION: u32 = 1;

#[contracttype]
#[derive(Clone)]
enum GdprKey {
    Admin,
    Initialized,
    RequestCounter,
    ExportRequest(u64),
    AuditTrail(Address),
    ProgressContract,
    CertificateContract,
    AssessmentContract,
    AnalyticsContract,
    CommunityContract,
    GamificationContract,
    EncryptionKey,
}

#[contracttype]
#[derive(Clone, PartialEq)]
enum ExportStatus {
    Pending,
    Processing,
    Ready,
    Delivered,
    Expired,
}

#[contracttype]
#[derive(Clone)]
struct EncryptedExport {
    salt: BytesN<32>,
    key_verification_hash: BytesN<32>,
    data: Bytes,
    data_hash: BytesN<32>,
    encryption_version: u32,
}

#[contracttype]
#[derive(Clone)]
struct ExportRequest {
    request_id: u64,
    user: Address,
    requested_at: u64,
    status: ExportStatus,
    data: GdprExportData,
    has_data: bool,
    data_hash: BytesN<32>,
    has_hash: bool,
    encrypted_export: EncryptedExport,
}

fn create_default_analytics(env: &Env) -> GdprAnalyticsExport {
    GdprAnalyticsExport {
        total_sessions: 0,
        total_time_spent: 0,
        average_session_time: 0,
        completed_modules: 0,
        total_modules: 0,
        completion_percentage: 0,
        average_score: 0,
        has_average_score: false,
        streak_days: 0,
        performance_trend: BytesN::from_array(env, &[0u8; 32]),
    }
}

fn create_default_community(env: &Env) -> GdprCommunityExport {
    GdprCommunityExport {
        posts_created: 0,
        replies_given: 0,
        solutions_provided: 0,
        contributions_made: 0,
        events_attended: 0,
        mentorship_sessions: 0,
        helpful_votes_received: 0,
        reputation_score: 0,
        joined_at: 0,
    }
}

fn create_default_gamification(env: &Env) -> GdprGamificationExport {
    GdprGamificationExport {
        xp_total: 0,
        level: 0,
        achievements_count: 0,
        guild_id: 0,
        has_guild: false,
        current_streak: 0,
    }
}

fn create_empty_encrypted_export(env: &Env) -> EncryptedExport {
    EncryptedExport {
        salt: BytesN::from_array(env, &[0u8; 32]),
        key_verification_hash: BytesN::from_array(env, &[0u8; 32]),
        data: Bytes::new(env),
        data_hash: BytesN::from_array(env, &[0u8; 32]),
        encryption_version: 0,
    }
}

#[contract]
pub struct GdprContract;

#[contractimpl]
impl GdprContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), GdprError> {
        if env.storage().instance().has(&GdprKey::Initialized) {
            return Err(GdprError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&GdprKey::Admin, &admin);
        env.storage().instance().set(&GdprKey::Initialized, &true);
        env.storage().instance().set(&GdprKey::RequestCounter, &0u64);
        let default_key = BytesN::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&GdprKey::EncryptionKey, &default_key);
        Ok(())
    }

    pub fn set_encryption_key(env: Env, admin: Address, key: BytesN<32>) -> Result<(), GdprError> {
        admin.require_auth();
        env.storage().instance().set(&GdprKey::EncryptionKey, &key);
        Ok(())
    }

    pub fn set_data_contracts(
        env: Env,
        admin: Address,
        progress: Address,
        certificate: Address,
        assessment: Address,
        analytics: Address,
        community: Address,
        gamification: Address,
    ) -> Result<(), GdprError> {
        admin.require_auth();
        env.storage().instance().set(&GdprKey::ProgressContract, &progress);
        env.storage().instance().set(&GdprKey::CertificateContract, &certificate);
        env.storage().instance().set(&GdprKey::AssessmentContract, &assessment);
        env.storage().instance().set(&GdprKey::AnalyticsContract, &analytics);
        env.storage().instance().set(&GdprKey::CommunityContract, &community);
        env.storage().instance().set(&GdprKey::GamificationContract, &gamification);
        Ok(())
    }

    pub fn request_export(env: Env, user: Address) -> Result<u64, GdprError> {
        user.require_auth();

        let counter: u64 = env.storage().instance().get(&GdprKey::RequestCounter).unwrap_or(0);
        let next_id = counter + 1;
        env.storage().instance().set(&GdprKey::RequestCounter, &next_id);

        let mut salt_bytes = [0u8; 32];
        salt_bytes[0..8].copy_from_slice(&next_id.to_be_bytes());
        salt_bytes[8..16].copy_from_slice(&env.ledger().timestamp().to_be_bytes());
        let salt = BytesN::from_array(&env, &salt_bytes);

        let request = ExportRequest {
            request_id: next_id,
            user: user.clone(),
            requested_at: env.ledger().timestamp(),
            status: ExportStatus::Processing,
            data: GdprExportData {
                exported_at: 0,
                progress_list: Vec::new(&env),
                certificate_list: Vec::new(&env),
                assessment_list: Vec::new(&env),
                has_analytics: false,
                analytics: create_default_analytics(&env),
                has_community: false,
                community: create_default_community(&env),
                has_gamification: false,
                gamification: create_default_gamification(&env),
            },
            has_data: false,
            data_hash: BytesN::from_array(&env, &[0u8; 32]),
            has_hash: false,
            encrypted_export: create_empty_encrypted_export(&env),
        };

        env.storage()
            .persistent()
            .set(&GdprKey::ExportRequest(next_id), &request);

        let mut audit_trail: Vec<GdprExportRecord> = env
            .storage()
            .persistent()
            .get(&GdprKey::AuditTrail(user.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        audit_trail.push_back(GdprExportRecord {
            request_id: next_id,
            requested_at: env.ledger().timestamp(),
            delivered_at: 0,
            data_hash: BytesN::from_array(&env, &[0u8; 32]),
        });

        env.storage()
            .persistent()
            .set(&GdprKey::AuditTrail(user), &audit_trail);

        Ok(next_id)
    }

    pub fn get_export_status(
        env: Env,
        user: Address,
        request_id: u64,
    ) -> Result<ExportStatus, GdprError> {
        let request: ExportRequest = env
            .storage()
            .persistent()
            .get(&GdprKey::ExportRequest(request_id))
            .ok_or(GdprError::ExportRequestNotFound)?;

        if request.user != user {
            return Err(GdprError::Unauthorized);
        }

        if request.status == ExportStatus::Processing {
            let expiration = request.requested_at + EXPORT_EXPIRATION_SECONDS;
            if env.ledger().timestamp() > expiration {
                return Ok(ExportStatus::Expired);
            }
        }

        Ok(request.status)
    }

    pub fn get_encrypted_export(
        env: Env,
        user: Address,
        request_id: u64,
    ) -> Result<EncryptedExport, GdprError> {
        let request: ExportRequest = env
            .storage()
            .persistent()
            .get(&GdprKey::ExportRequest(request_id))
            .ok_or(GdprError::ExportRequestNotFound)?;

        if request.user != user {
            return Err(GdprError::Unauthorized);
        }

        if request.status == ExportStatus::Processing {
            let expiration = request.requested_at + EXPORT_EXPIRATION_SECONDS;
            if env.ledger().timestamp() > expiration {
                return Err(GdprError::ExportExpired);
            }
        }

        if request.status != ExportStatus::Ready {
            return Err(GdprError::ExportNotReady);
        }

        if request.encrypted_export.data.is_empty() {
            return Err(GdprError::DataRetrievalFailed);
        }

        Ok(request.encrypted_export)
    }

    pub fn store_encrypted_export(
        env: Env,
        operator: Address,
        request_id: u64,
        salt: BytesN<32>,
        key_verification_hash: BytesN<32>,
        encrypted_data: Bytes,
        data_hash: BytesN<32>,
    ) -> Result<(), GdprError> {
        operator.require_auth();

        let mut request: ExportRequest = env
            .storage()
            .persistent()
            .get(&GdprKey::ExportRequest(request_id))
            .ok_or(GdprError::ExportRequestNotFound)?;

        if request.status != ExportStatus::Processing {
            return Err(GdprError::ExportNotReady);
        }

        let expiration = request.requested_at + EXPORT_EXPIRATION_SECONDS;
        if env.ledger().timestamp() > expiration {
            request.status = ExportStatus::Expired;
            env.storage()
                .persistent()
                .set(&GdprKey::ExportRequest(request_id), &request);
            return Err(GdprError::ExportExpired);
        }

        let stored_hash = data_hash.clone();
        request.encrypted_export = EncryptedExport {
            salt,
            key_verification_hash,
            data: encrypted_data,
            data_hash,
            encryption_version: ENCRYPTION_VERSION,
        };
        request.has_hash = true;
        request.status = ExportStatus::Ready;

        env.storage()
            .persistent()
            .set(&GdprKey::ExportRequest(request_id), &request);

        update_audit_trail_with_hash(&env, &request.user, request_id, stored_hash);

        Ok(())
    }

    pub fn verify_export_integrity(
        env: Env,
        user: Address,
        request_id: u64,
    ) -> Result<bool, GdprError> {
        let request: ExportRequest = env
            .storage()
            .persistent()
            .get(&GdprKey::ExportRequest(request_id))
            .ok_or(GdprError::ExportRequestNotFound)?;

        if request.user != user {
            return Err(GdprError::Unauthorized);
        }

        if !request.has_hash || request.encrypted_export.data.is_empty() {
            return Err(GdprError::DataRetrievalFailed);
        }

        let computed = env.crypto().sha256(&request.encrypted_export.data.clone().into());
        let expected: BytesN<32> = computed.into();
        Ok(request.encrypted_export.data_hash == expected)
    }

    pub fn get_export_salt(
        env: Env,
        user: Address,
        request_id: u64,
    ) -> Result<BytesN<32>, GdprError> {
        let request: ExportRequest = env
            .storage()
            .persistent()
            .get(&GdprKey::ExportRequest(request_id))
            .ok_or(GdprError::ExportRequestNotFound)?;

        if request.user != user {
            return Err(GdprError::Unauthorized);
        }

        Ok(request.encrypted_export.salt)
    }

    pub fn get_audit_trail(
        env: Env,
        user: Address,
    ) -> Result<Vec<GdprExportRecord>, GdprError> {
        let now = env.ledger().timestamp();
        let cutoff = now.saturating_sub(AUDIT_RETENTION_SECONDS);

        let all_records: Vec<GdprExportRecord> = env
            .storage()
            .persistent()
            .get(&GdprKey::AuditTrail(user))
            .unwrap_or_else(|| Vec::new(&env));

        let mut filtered: Vec<GdprExportRecord> = Vec::new(&env);
        for i in 0..all_records.len() {
            if let Some(record) = all_records.get(i) {
                if record.requested_at >= cutoff {
                    filtered.push_back(record);
                }
            }
        }

        Ok(filtered)
    }

    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&GdprKey::Initialized);
        let report = Monitor::build_health_report(&env, symbol_short!("gdpr"), initialized);
        Monitor::emit_health_check(&env, &report);
        report
    }
}

fn update_audit_trail_delivered(env: &Env, user: &Address, request_id: u64) {
    let mut audit_trail: Vec<GdprExportRecord> = env
        .storage()
        .persistent()
        .get(&GdprKey::AuditTrail(user.clone()))
        .unwrap_or_else(|| Vec::new(env));

    for i in 0..audit_trail.len() {
        if let Some(record) = audit_trail.get(i) {
            if record.request_id == request_id {
                audit_trail.set(i, GdprExportRecord {
                    request_id: record.request_id,
                    requested_at: record.requested_at,
                    delivered_at: env.ledger().timestamp(),
                    data_hash: record.data_hash,
                });
                break;
            }
        }
    }

    env.storage()
        .persistent()
        .set(&GdprKey::AuditTrail(user.clone()), &audit_trail);
}

fn update_audit_trail_with_hash(env: &Env, user: &Address, request_id: u64, data_hash: BytesN<32>) {
    let mut audit_trail: Vec<GdprExportRecord> = env
        .storage()
        .persistent()
        .get(&GdprKey::AuditTrail(user.clone()))
        .unwrap_or_else(|| Vec::new(env));

    for i in 0..audit_trail.len() {
        if let Some(record) = audit_trail.get(i) {
            if record.request_id == request_id {
                audit_trail.set(i, GdprExportRecord {
                    request_id: record.request_id,
                    requested_at: record.requested_at,
                    delivered_at: 0,
                    data_hash,
                });
                break;
            }
        }
    }

    env.storage()
        .persistent()
        .set(&GdprKey::AuditTrail(user.clone()), &audit_trail);
}
