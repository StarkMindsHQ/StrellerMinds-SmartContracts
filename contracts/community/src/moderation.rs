use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::Error;
use crate::events::CommunityEvents;
use crate::storage::CommunityStorage;
use crate::types::*;

pub struct ModerationManager;

impl ModerationManager {
    pub fn add_moderator(
        env: &Env,
        admin: &Address,
        moderator: &Address,
        role: ModeratorRole,
    ) -> Result<(), Error> {
        CommunityStorage::require_admin(env, admin)?;

        env.storage()
            .persistent()
            .set(&CommunityKey::Moderator(moderator.clone()), &role);

        Ok(())
    }

    pub fn report_content(
        env: &Env,
        reporter: &Address,
        content_type: String,
        content_id: u64,
        reason: ReportReason,
        description: String,
    ) -> Result<u64, Error> {
        // Check daily report limit
        let now = env.ledger().timestamp();
        let _day_bucket = now / 86_400;
        let _config = CommunityStorage::get_config(env);
        
        // Rate limiting would be implemented here
        
        let report_id = CommunityStorage::increment_counter(env, CommunityKey::ReportCounter);

        let report = ContentReport {
            id: report_id,
            reporter: reporter.clone(),
            content_type,
            content_id,
            reason,
            description,
            status: ReportStatus::Pending,
            created_at: now,
            resolved_at: 0,
            resolved_by: Address::from_string(&String::from_str(env, "")),
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::Report(report_id), &report);

        // Add to pending reports
        let mut pending: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::PendingReports)
            .unwrap_or_else(|| Vec::new(env));
        pending.push_back(report_id);
        env.storage()
            .persistent()
            .set(&CommunityKey::PendingReports, &pending);

        CommunityEvents::emit_content_reported(env, reporter, report_id);
        Ok(report_id)
    }

    pub fn resolve_report(
        env: &Env,
        moderator: &Address,
        report_id: u64,
        _action: String,
    ) -> Result<(), Error> {
        CommunityStorage::require_moderator(env, moderator)?;

        let mut report: ContentReport = env
            .storage()
            .persistent()
            .get(&CommunityKey::Report(report_id))
            .ok_or(Error::ReportNotFound)?;

        if report.status != ReportStatus::Pending
            && report.status != ReportStatus::UnderReview
        {
            return Err(Error::InvalidInput);
        }

        report.status = ReportStatus::Resolved;
        report.resolved_at = env.ledger().timestamp();
        report.resolved_by = moderator.clone();

        env.storage()
            .persistent()
            .set(&CommunityKey::Report(report_id), &report);

        // Remove from pending
        let pending: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::PendingReports)
            .unwrap_or_else(|| Vec::new(env));
        
        // Filter out the resolved report
        let mut new_pending = Vec::new(env);
        for id in pending.iter() {
            if id != report_id {
                new_pending.push_back(id);
            }
        }
        env.storage()
            .persistent()
            .set(&CommunityKey::PendingReports, &new_pending);

        Ok(())
    }

    pub fn take_action(
        env: &Env,
        moderator: &Address,
        target_user: &Address,
        action_type: String,
        reason: String,
        duration: u64,
    ) -> Result<u64, Error> {
        CommunityStorage::require_moderator(env, moderator)?;

        let action_id = CommunityStorage::increment_counter(env, CommunityKey::ReportCounter);

        let action = ModeratorAction {
            id: action_id,
            moderator: moderator.clone(),
            action_type,
            target_user: target_user.clone(),
            reason,
            duration,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::ModeratorAction(action_id), &action);

        // Add to user actions
        let mut actions: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserActions(target_user.clone()))
            .unwrap_or_else(|| Vec::new(env));
        actions.push_back(action_id);
        env.storage()
            .persistent()
            .set(&CommunityKey::UserActions(target_user.clone()), &actions);

        CommunityEvents::emit_moderator_action(env, moderator, action_id, target_user);
        Ok(action_id)
    }

    pub fn get_pending_reports(env: &Env) -> Vec<ContentReport> {
        let report_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::PendingReports)
            .unwrap_or_else(|| Vec::new(env));

        let mut reports = Vec::new(env);
        for id in report_ids.iter() {
            if let Some(report) = env.storage().persistent().get(&CommunityKey::Report(id)) {
                reports.push_back(report);
            }
        }
        reports
    }

    pub fn get_user_actions(env: &Env, user: &Address) -> Vec<ModeratorAction> {
        let action_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserActions(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut actions = Vec::new(env);
        for id in action_ids.iter() {
            if let Some(action) = env
                .storage()
                .persistent()
                .get(&CommunityKey::ModeratorAction(id))
            {
                actions.push_back(action);
            }
        }
        actions
    }
}
