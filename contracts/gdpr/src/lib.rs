#![no_std]

pub mod errors;
pub mod types;

use crate::errors::GdprError;
use crate::types::*;
use shared::monitoring::{ContractHealthReport, Monitor};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Vec};

const AUDIT_RETENTION_SECONDS: u64 = 365 * 86_400;

#[contracttype]
#[derive(Clone)]
enum GdprKey {
    Admin,
    Initialized,
    RequestCounter,
    ExportRequest(u64),
    AuditTrail(Address),
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
        Ok(())
    }

    pub fn request_export(env: Env, user: Address) -> Result<u64, GdprError> {
        user.require_auth();

        let counter: u64 = env.storage().instance().get(&GdprKey::RequestCounter).unwrap_or(0);
        let next_id = counter + 1;
        env.storage().instance().set(&GdprKey::RequestCounter, &next_id);

        let request = ExportRequest {
            request_id: next_id,
            user: user.clone(),
            requested_at: env.ledger().timestamp(),
            status: ExportStatus::Ready,
            data: GdprExportData {
                exported_at: 0,
                certificates: Vec::new(&env),
                progress: Vec::new(&env),
                assessments: Vec::new(&env),
                has_analytics: false,
                analytics: AnalyticsExport {
                    total_sessions: 0,
                    total_time_spent: 0,
                    average_session_time: 0,
                    completed_modules: 0,
                    total_modules: 0,
                    completion_percentage: 0,
                    average_score: 0,
                    has_average_score: false,
                    streak_days: 0,
                    performance_trend: symbol_short!("NA"),
                },
                has_community: false,
                community: CommunityExport {
                    posts_count: 0,
                    replies_count: 0,
                    solutions_count: 0,
                    reputation_score: 0,
                },
                has_gamification: false,
                gamification: GamificationExport {
                    xp_total: 0,
                    level: 0,
                    achievements_count: 0,
                    guild_id: 0,
                    has_guild: false,
                    endorsements_count: 0,
                },
            },
            has_data: false,
            data_hash: BytesN::from_array(&env, &[0u8; 32]),
            has_hash: false,
        };

        env.storage()
            .persistent()
            .set(&GdprKey::ExportRequest(next_id), &request);

        let mut audit_trail: Vec<ExportRecord> = env
            .storage()
            .persistent()
            .get(&GdprKey::AuditTrail(user.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        audit_trail.push_back(ExportRecord {
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

        Ok(request.status)
    }

    pub fn get_export_data(
        env: Env,
        user: Address,
        request_id: u64,
    ) -> Result<GdprExportData, GdprError> {
        let mut request: ExportRequest = env
            .storage()
            .persistent()
            .get(&GdprKey::ExportRequest(request_id))
            .ok_or(GdprError::ExportRequestNotFound)?;

        if request.user != user {
            return Err(GdprError::Unauthorized);
        }

        if request.status != ExportStatus::Ready {
            return Err(GdprError::ExportNotReady);
        }

        if !request.has_data {
            return Err(GdprError::DataRetrievalFailed);
        }

        let data = request.data.clone();

        request.status = ExportStatus::Delivered;
        request.has_data = false;
        env.storage()
            .persistent()
            .set(&GdprKey::ExportRequest(request_id), &request);

        update_audit_trail_delivered(&env, &user, request_id);

        Ok(data)
    }

    pub fn get_audit_trail(
        env: Env,
        user: Address,
    ) -> Result<Vec<ExportRecord>, GdprError> {
        let now = env.ledger().timestamp();
        let cutoff = now.saturating_sub(AUDIT_RETENTION_SECONDS);

        let all_records: Vec<ExportRecord> = env
            .storage()
            .persistent()
            .get(&GdprKey::AuditTrail(user))
            .unwrap_or_else(|| Vec::new(&env));

        let mut filtered: Vec<ExportRecord> = Vec::new(&env);
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
    let mut audit_trail: Vec<ExportRecord> = env
        .storage()
        .persistent()
        .get(&GdprKey::AuditTrail(user.clone()))
        .unwrap_or_else(|| Vec::new(env));

    for i in 0..audit_trail.len() {
        if let Some(record) = audit_trail.get(i) {
            if record.request_id == request_id {
                audit_trail.set(i, ExportRecord {
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