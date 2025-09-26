#![no_std]

mod types;
mod errors;
mod events;
mod storage;
mod analytics_engine;
mod reports;
mod interface;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol, Vec};

// Import shared RBAC system
use shared::{
    access_control::AccessControl,
    roles::{Permission, RoleLevel},
    errors::AccessControlError,
    reentrancy_guard::ReentrancyLock,
};

use types::{
    LearningSession, ProgressAnalytics, CourseAnalytics, ModuleAnalytics,
    ProgressReport, ReportPeriod, Achievement, LeaderboardEntry, LeaderboardMetric,
    AggregatedMetrics, AnalyticsConfig, AnalyticsFilter, BatchSessionUpdate, SessionType
};
use errors::AnalyticsError;
use events::AnalyticsEvents;
use storage::AnalyticsStorage;
use analytics_engine::AnalyticsEngine;
use reports::ReportGenerator;
use interface::AnalyticsTrait;

#[contract]
pub struct Analytics;

#[contractimpl]
impl AnalyticsTrait for Analytics {
    fn initialize(env: Env, admin: Address, config: AnalyticsConfig) -> Result<(), AnalyticsError> {
        // Check if already initialized
        if AnalyticsStorage::get_admin(&env).is_some() {
            return Err(AnalyticsError::AlreadyInitialized);
        }

        admin.require_auth();

        // Validate configuration
        if config.min_session_time == 0 || config.max_session_time <= config.min_session_time {
            return Err(AnalyticsError::InvalidConfiguration);
        }

        // Store admin and configuration
        AnalyticsStorage::set_admin(&env, &admin);
        AnalyticsStorage::set_config(&env, &config);

        // Emit initialization event
        AnalyticsEvents::emit_config_updated(&env, &admin, "initialized");

        Ok(())
    }

    fn record_session(env: Env, session: LearningSession) -> Result<(), AnalyticsError> {
        let _guard = ReentrancyLock::new(&env);
        session.student.require_auth();

        // Validate session data
        Self::validate_session(&env, &session)?;

        // Check if session already exists
        if AnalyticsStorage::has_session(&env, &session.session_id) {
            return Err(AnalyticsError::SessionAlreadyExists);
        }

        // Store session
        AnalyticsStorage::set_session(&env, &session);

        // Add student to course if not already added
        AnalyticsStorage::add_course_student(&env, &session.course_id, &session.student);

        // Emit session recorded event
        AnalyticsEvents::emit_session_recorded(
            &env,
            &session.session_id,
            &session.student,
            &session.course_id,
            &session.module_id,
            &session.session_type,
            session.time_spent,
            session.completion_percentage,
        );

        // Check for achievements if session is completed
        if session.completion_percentage == 100 {
            let _ = AnalyticsEngine::check_achievements(&env, &session.student, &session.course_id, &session);
        }

        Ok(())
    }

    fn complete_session(
        env: Env,
        session_id: BytesN<32>,
        end_time: u64,
        final_score: Option<u32>,
        completion_percentage: u32,
    ) -> Result<(), AnalyticsError> {
        let _guard = ReentrancyLock::new(&env);
        let mut session = AnalyticsStorage::get_session(&env, &session_id)
            .ok_or(AnalyticsError::SessionNotFound)?;

        session.student.require_auth();

        // Validate completion data
        if completion_percentage > 100 {
            return Err(AnalyticsError::InvalidPercentage);
        }

        if let Some(score) = final_score {
            if score > 100 {
                return Err(AnalyticsError::InvalidScore);
            }
        }

        if end_time <= session.start_time {
            return Err(AnalyticsError::InvalidTimeRange);
        }

        // Update session
        session.end_time = end_time;
        session.time_spent = end_time - session.start_time;
        session.completion_percentage = completion_percentage;
        session.score = final_score;

        // Validate session duration
        let config = AnalyticsStorage::get_config(&env)
            .unwrap_or(AnalyticsStorage::get_default_config(&env));

        if session.time_spent < config.min_session_time {
            return Err(AnalyticsError::SessionTooShort);
        }

        if session.time_spent > config.max_session_time {
            return Err(AnalyticsError::SessionTooLong);
        }

        // Store updated session
        AnalyticsStorage::set_session(&env, &session);

        // Emit completion event
        AnalyticsEvents::emit_session_completed(
            &env,
            &session_id,
            &session.student,
            &session.course_id,
            &session.module_id,
            final_score,
            session.time_spent,
        );

        // Update analytics
        let _ = AnalyticsEngine::calculate_progress_analytics(&env, &session.student, &session.course_id);
        let _ = AnalyticsEngine::calculate_module_analytics(&env, &session.course_id, &session.module_id);

        // Check for achievements
        if completion_percentage == 100 {
            let _ = AnalyticsEngine::check_achievements(&env, &session.student, &session.course_id, &session);
        }

        Ok(())
    }

    fn batch_update_sessions(env: Env, batch: BatchSessionUpdate) -> Result<u32, AnalyticsError> {
        let _guard = ReentrancyLock::new(&env);
        if batch.sessions.is_empty() {
            return Err(AnalyticsError::InvalidBatchSize);
        }

        // Limit batch size for gas optimization
        if batch.sessions.len() > 50 {
            return Err(AnalyticsError::InvalidBatchSize);
        }

        let start_time = env.ledger().timestamp();
        let mut processed = 0u32;

        // Process each session in the batch
        for i in 0..batch.sessions.len() {
            let session = batch.sessions.get(i).unwrap();
            
            // Validate and store session
            if Self::validate_session(&env, &session).is_ok() {
                AnalyticsStorage::set_session(&env, &session);
                AnalyticsStorage::add_course_student(&env, &session.course_id, &session.student);
                processed += 1;
            }
        }

        // Update analytics if requested
        if batch.update_analytics && processed > 0 {
            // Get unique course-student pairs for analytics updates
            let mut updates = 0u32;
            for i in 0..batch.sessions.len() {
                let session = batch.sessions.get(i).unwrap();
                if AnalyticsEngine::calculate_progress_analytics(&env, &session.student, &session.course_id).is_ok() {
                    updates += 1;
                }
            }
        }

        let processing_time = env.ledger().timestamp() - start_time;

        // Emit batch processing event
        AnalyticsEvents::emit_batch_processed(&env, batch.sessions.len(), processing_time, processed);

        Ok(processed)
    }

    fn get_session(env: Env, session_id: BytesN<32>) -> Option<LearningSession> {
        AnalyticsStorage::get_session(&env, &session_id)
    }

    fn get_student_sessions(env: Env, student: Address, course_id: Symbol) -> Vec<BytesN<32>> {
        AnalyticsStorage::get_student_sessions(&env, &student, &course_id)
    }

    fn get_progress_analytics(
        env: Env,
        student: Address,
        course_id: Symbol,
    ) -> Result<ProgressAnalytics, AnalyticsError> {
        // Try to get cached analytics first
        if let Some(analytics) = AnalyticsStorage::get_progress_analytics(&env, &student, &course_id) {
            return Ok(analytics);
        }

        // Calculate fresh analytics
        AnalyticsEngine::calculate_progress_analytics(&env, &student, &course_id)
    }

    fn get_course_analytics(env: Env, course_id: Symbol) -> Result<CourseAnalytics, AnalyticsError> {
        // Try to get cached analytics first
        if let Some(analytics) = AnalyticsStorage::get_course_analytics(&env, &course_id) {
            return Ok(analytics);
        }

        // Calculate fresh analytics
        AnalyticsEngine::calculate_course_analytics(&env, &course_id)
    }

    fn get_module_analytics(
        env: Env,
        course_id: Symbol,
        module_id: Symbol,
    ) -> Result<ModuleAnalytics, AnalyticsError> {
        // Try to get cached analytics first
        if let Some(analytics) = AnalyticsStorage::get_module_analytics(&env, &course_id, &module_id) {
            return Ok(analytics);
        }

        // Calculate fresh analytics
        AnalyticsEngine::calculate_module_analytics(&env, &course_id, &module_id)
    }

    fn generate_progress_report(
        env: Env,
        student: Address,
        course_id: Symbol,
        period: ReportPeriod,
        start_date: u64,
        end_date: u64,
    ) -> Result<ProgressReport, AnalyticsError> {
        ReportGenerator::generate_progress_report(&env, &student, &course_id, &period, start_date, end_date)
    }

    fn get_progress_report(
        env: Env,
        student: Address,
        course_id: Symbol,
        timestamp: u64,
    ) -> Option<ProgressReport> {
        AnalyticsStorage::get_progress_report(&env, &student, &course_id, timestamp)
    }

    fn generate_daily_metrics(
        env: Env,
        course_id: Symbol,
        date: u64,
    ) -> Result<AggregatedMetrics, AnalyticsError> {
        ReportGenerator::generate_daily_metrics(&env, &course_id, date)
    }

    fn get_daily_metrics(env: Env, course_id: Symbol, date: u64) -> Option<AggregatedMetrics> {
        AnalyticsStorage::get_daily_metrics(&env, &course_id, date)
    }

    fn generate_leaderboard(
        env: Env,
        course_id: Symbol,
        metric: LeaderboardMetric,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, AnalyticsError> {
        ReportGenerator::generate_leaderboard(&env, &course_id, &metric, limit)
    }

    fn get_leaderboard(env: Env, course_id: Symbol, metric: LeaderboardMetric) -> Vec<LeaderboardEntry> {
        AnalyticsStorage::get_leaderboard(&env, &course_id, &metric)
    }

    fn get_student_achievements(env: Env, student: Address) -> Vec<Achievement> {
        AnalyticsStorage::get_student_achievements(&env, &student)
    }

    fn get_filtered_sessions(
        env: Env,
        filter: AnalyticsFilter,
    ) -> Result<Vec<LearningSession>, AnalyticsError> {
        ReportGenerator::get_filtered_analytics(&env, &filter)
    }

    fn update_config(
        env: Env,
        admin: Address,
        config: AnalyticsConfig,
    ) -> Result<(), AnalyticsError> {
        admin.require_auth();

        // Verify admin
        let stored_admin = AnalyticsStorage::get_admin(&env)
            .ok_or(AnalyticsError::NotInitialized)?;

        if admin != stored_admin {
            return Err(AnalyticsError::Unauthorized);
        }

        // Validate configuration
        if config.min_session_time == 0 || config.max_session_time <= config.min_session_time {
            return Err(AnalyticsError::InvalidConfiguration);
        }

        // Store updated configuration
        AnalyticsStorage::set_config(&env, &config);

        // Emit event
        AnalyticsEvents::emit_config_updated(&env, &admin, "updated");

        Ok(())
    }

    fn get_config(env: Env) -> Option<AnalyticsConfig> {
        AnalyticsStorage::get_config(&env)
    }

    fn recalculate_course_analytics(
        env: Env,
        admin: Address,
        course_id: Symbol,
    ) -> Result<(), AnalyticsError> {
        admin.require_auth();

        // Verify admin
        let stored_admin = AnalyticsStorage::get_admin(&env)
            .ok_or(AnalyticsError::NotInitialized)?;

        if admin != stored_admin {
            return Err(AnalyticsError::Unauthorized);
        }

        // Recalculate course analytics
        AnalyticsEngine::calculate_course_analytics(&env, &course_id)?;

        // Recalculate all student analytics for this course
        let students = AnalyticsStorage::get_course_students(&env, &course_id);
        for i in 0..students.len() {
            let student = students.get(i).unwrap();
            let _ = AnalyticsEngine::calculate_progress_analytics(&env, &student, &course_id);
        }

        Ok(())
    }

    fn cleanup_old_data(
        env: Env,
        admin: Address,
        before_date: u64,
    ) -> Result<u32, AnalyticsError> {
        admin.require_auth();

        // Verify admin
        let stored_admin = AnalyticsStorage::get_admin(&env)
            .ok_or(AnalyticsError::NotInitialized)?;

        if admin != stored_admin {
            return Err(AnalyticsError::Unauthorized);
        }

        // This is a placeholder for cleanup logic
        // In a real implementation, you would iterate through old sessions and remove them
        // For now, return 0 as no cleanup is performed
        Ok(0)
    }

    fn get_completion_trends(
        env: Env,
        course_id: Symbol,
        start_date: u64,
        end_date: u64,
    ) -> Vec<AggregatedMetrics> {
        let mut trends: Vec<AggregatedMetrics> = Vec::new(&env);
        
        let mut current_date = (start_date / 86400) * 86400; // Start of day
        let end_day = (end_date / 86400) * 86400;

        while current_date <= end_day {
            if let Some(metrics) = AnalyticsStorage::get_daily_metrics(&env, &course_id, current_date) {
                trends.push_back(metrics);
            }
            current_date += 86400; // Next day
        }

        trends
    }

    fn compare_student_performance(
        env: Env,
        student1: Address,
        student2: Address,
        course_id: Symbol,
    ) -> Result<(ProgressAnalytics, ProgressAnalytics), AnalyticsError> {
        let analytics1 = Self::get_progress_analytics(env.clone(), student1, course_id.clone())?;
        let analytics2 = Self::get_progress_analytics(env, student2, course_id)?;
        
        Ok((analytics1, analytics2))
    }

    fn get_top_performers(
        env: Env,
        course_id: Symbol,
        metric: LeaderboardMetric,
        limit: u32,
    ) -> Vec<LeaderboardEntry> {
        let leaderboard = AnalyticsStorage::get_leaderboard(&env, &course_id, &metric);
        
        if limit == 0 || limit >= leaderboard.len() {
            return leaderboard;
        }

        let mut top_performers: Vec<LeaderboardEntry> = Vec::new(&env);
        for i in 0..limit {
            if i < leaderboard.len() {
                top_performers.push_back(leaderboard.get(i).unwrap());
            }
        }

        top_performers
    }

    fn get_struggling_students(
        env: Env,
        course_id: Symbol,
        threshold: u32,
    ) -> Vec<Address> {
        let students = AnalyticsStorage::get_course_students(&env, &course_id);
        let mut struggling: Vec<Address> = Vec::new(&env);

        for i in 0..students.len() {
            let student = students.get(i).unwrap();
            if let Some(analytics) = AnalyticsStorage::get_progress_analytics(&env, &student, &course_id) {
                // Consider students struggling if completion percentage is below threshold
                // or if they have declining performance trend
                if analytics.completion_percentage < threshold || 
                   analytics.performance_trend == types::PerformanceTrend::Declining {
                    struggling.push_back(student);
                }
            }
        }

        struggling
    }

    fn get_engagement_metrics(
        env: Env,
        course_id: Symbol,
        date_range: u64,
    ) -> Result<CourseAnalytics, AnalyticsError> {
        // This could be enhanced to calculate engagement over a specific date range
        // For now, return the general course analytics
        Self::get_course_analytics(env, course_id)
    }

    fn generate_weekly_summary(
        env: Env,
        course_id: Symbol,
        week_start: u64,
    ) -> Result<Vec<AggregatedMetrics>, AnalyticsError> {
        ReportGenerator::generate_weekly_summary(&env, &course_id, week_start)
    }

    fn generate_monthly_summary(
        env: Env,
        course_id: Symbol,
        month_start: u64,
        days_in_month: u32,
    ) -> Result<Vec<AggregatedMetrics>, AnalyticsError> {
        ReportGenerator::generate_monthly_summary(&env, &course_id, month_start, days_in_month)
    }

    fn get_admin(env: Env) -> Option<Address> {
        AnalyticsStorage::get_admin(&env)
    }

    fn transfer_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), AnalyticsError> {
        current_admin.require_auth();

        // Verify current admin
        let stored_admin = AnalyticsStorage::get_admin(&env)
            .ok_or(AnalyticsError::NotInitialized)?;

        if current_admin != stored_admin {
            return Err(AnalyticsError::Unauthorized);
        }

        // Transfer admin role
        AnalyticsStorage::set_admin(&env, &new_admin);

        // Emit event
        AnalyticsEvents::emit_config_updated(&env, &new_admin, "admin_transferred");

        Ok(())
    }
}

impl Analytics {
    /// Validate session data
    fn validate_session(env: &Env, session: &LearningSession) -> Result<(), AnalyticsError> {
        let config = AnalyticsStorage::get_config(env)
            .unwrap_or(AnalyticsStorage::get_default_config(env));

        // Validate time range
        if session.end_time > 0 && session.end_time <= session.start_time {
            return Err(AnalyticsError::InvalidTimeRange);
        }

        // Validate completion percentage
        if session.completion_percentage > 100 {
            return Err(AnalyticsError::InvalidPercentage);
        }

        // Validate score if present
        if let Some(score) = session.score {
            if score > 100 {
                return Err(AnalyticsError::InvalidScore);
            }
        }

        // Validate session duration if completed
        if session.end_time > 0 {
            let duration = session.end_time - session.start_time;
            if duration < config.min_session_time {
                return Err(AnalyticsError::SessionTooShort);
            }
            if duration > config.max_session_time {
                return Err(AnalyticsError::SessionTooLong);
            }
        }

        Ok(())
    }
}
