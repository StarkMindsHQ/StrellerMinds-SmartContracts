use crate::types::{AchievementType, LeaderboardMetric, PerformanceTrend, SessionType};
use shared::event_schema::{AnalyticsEventData, EventData, StandardEvent};
use soroban_sdk::{Address, BytesN, Env, String, Symbol};

/// Analytics contract events for tracking and auditing
pub struct AnalyticsEvents;

impl AnalyticsEvents {
    /// Emit event when a new learning session is recorded
    #[allow(clippy::too_many_arguments)]
    pub fn emit_session_recorded(
        env: &Env,
        session_id: &BytesN<32>,
        student: &Address,
        course_id: &Symbol,
        module_id: &Symbol,
        session_type: SessionType,
        time_spent: u64,
        completion_percentage: u32,
    ) {
        let session_type_str = match session_type {
            SessionType::Study => "study",
            SessionType::Assessment => "assessment",
            SessionType::Practice => "practice",
            SessionType::Review => "review",
        };

        let event_data = AnalyticsEventData::SessionRecorded(
            session_id.clone(),
            student.clone(),
            course_id.clone(),
            module_id.clone(),
            String::from_str(env, session_type_str),
            time_spent,
            completion_percentage,
        );

        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when a learning session is completed
    pub fn emit_session_completed(
        env: &Env,
        session_id: &BytesN<32>,
        student: &Address,
        course_id: &Symbol,
        module_id: &Symbol,
        final_score: Option<u32>,
        total_time: u64,
    ) {
        let event_data = AnalyticsEventData::SessionCompleted(
            session_id.clone(),
            student.clone(),
            course_id.clone(),
            module_id.clone(),
            final_score,
            total_time,
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when progress analytics are updated
    pub fn emit_progress_updated(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        completion_percentage: u32,
        total_time_spent: u64,
        performance_trend: PerformanceTrend,
    ) {
        let trend_str = match performance_trend {
            PerformanceTrend::Improving => "improving",
            PerformanceTrend::Stable => "stable",
            PerformanceTrend::Declining => "declining",
            PerformanceTrend::Insufficient => "insufficient",
        };
        let event_data = AnalyticsEventData::ProgressUpdated(
            student.clone(),
            course_id.clone(),
            completion_percentage,
            total_time_spent,
            String::from_str(env, trend_str),
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    // ... Implement other methods similarly if needed, or stick to emit_insight methods ...

    /// Emit event when course analytics are recalculated
    pub fn emit_course_analytics_updated(
        env: &Env,
        course_id: &Symbol,
        total_students: u32,
        completion_rate: u32,
        average_score: Option<u32>,
    ) {
        let event_data = AnalyticsEventData::CourseAnalyticsUpdated(
            course_id.clone(),
            total_students,
            completion_rate,
            average_score,
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            env.current_contract_address(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when module analytics are updated
    pub fn emit_module_analytics_updated(
        env: &Env,
        course_id: &Symbol,
        module_id: &Symbol,
        completion_rate: u32,
        average_time: u64,
        difficulty_rating: &str,
    ) {
        let event_data = AnalyticsEventData::ModuleAnalyticsUpdated(
            course_id.clone(),
            module_id.clone(),
            completion_rate,
            average_time,
            String::from_str(env, difficulty_rating),
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            env.current_contract_address(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when a student earns an achievement
    pub fn emit_achievement_earned(
        env: &Env,
        student: &Address,
        achievement_id: &Symbol,
        achievement_type: AchievementType,
        course_id: &Symbol,
        earned_date: u64,
    ) {
        let type_str = match achievement_type {
            AchievementType::Completion => "completion",
            AchievementType::Streak => "streak",
            AchievementType::Speed => "speed",
            AchievementType::Excellence => "excellence",
            AchievementType::Consistency => "consistency",
        };
        let event_data = AnalyticsEventData::AchievementEarned(
            student.clone(),
            achievement_id.clone(),
            String::from_str(env, type_str),
            course_id.clone(),
            earned_date,
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when leaderboard is updated
    pub fn emit_leaderboard_updated(
        env: &Env,
        course_id: &Symbol,
        metric_type: LeaderboardMetric,
        top_student: &Address,
        top_score: u32,
        total_entries: u32,
    ) {
        let metric_str = match metric_type {
            LeaderboardMetric::CompletionSpeed => "completion_speed",
            LeaderboardMetric::TimeSpent => "total_time",
            LeaderboardMetric::TotalScore => "score",
            LeaderboardMetric::ConsistencyScore => "consistency",
        };
        let event_data = AnalyticsEventData::LeaderboardUpdated(
            course_id.clone(),
            String::from_str(env, metric_str),
            top_student.clone(),
            top_score,
            total_entries,
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            env.current_contract_address(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when a progress report is generated
    pub fn emit_report_generated(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        report_period: &str,
        start_date: u64,
        end_date: u64,
        sessions_count: u32,
    ) {
        let event_data = AnalyticsEventData::ReportGenerated(
            student.clone(),
            course_id.clone(),
            String::from_str(env, report_period),
            start_date,
            end_date,
            sessions_count,
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when batch analytics processing is completed
    pub fn emit_batch_processed(
        env: &Env,
        batch_size: u32,
        processing_time: u64,
        updated_analytics: u32,
    ) {
        let event_data =
            AnalyticsEventData::BatchProcessed(batch_size, processing_time, updated_analytics);
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            env.current_contract_address(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when analytics configuration is updated
    pub fn emit_config_updated(env: &Env, admin: &Address, config_type: &str) {
        let event_data =
            AnalyticsEventData::ConfigUpdated(admin.clone(), String::from_str(env, config_type));
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            admin.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when data aggregation is performed
    pub fn emit_data_aggregated(
        env: &Env,
        course_id: &Symbol,
        date: u64,
        active_students: u32,
        total_sessions: u32,
    ) {
        let event_data = AnalyticsEventData::DataAggregated(
            course_id.clone(),
            date,
            active_students,
            total_sessions,
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            env.current_contract_address(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when performance trend changes
    #[allow(dead_code)]
    pub fn emit_trend_change(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        old_trend: PerformanceTrend,
        new_trend: PerformanceTrend,
    ) {
        let old_str = match old_trend {
            PerformanceTrend::Improving => "improving",
            PerformanceTrend::Stable => "stable",
            PerformanceTrend::Declining => "declining",
            PerformanceTrend::Insufficient => "insufficient",
        };
        let new_str = match new_trend {
            PerformanceTrend::Improving => "improving",
            PerformanceTrend::Stable => "stable",
            PerformanceTrend::Declining => "declining",
            PerformanceTrend::Insufficient => "insufficient",
        };
        let event_data = AnalyticsEventData::TrendChange(
            student.clone(),
            course_id.clone(),
            String::from_str(env, old_str),
            String::from_str(env, new_str),
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when streak milestone is reached
    #[allow(dead_code)]
    pub fn emit_streak_milestone(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        streak_days: u32,
        milestone_type: &str,
    ) {
        let event_data = AnalyticsEventData::StreakMilestone(
            student.clone(),
            course_id.clone(),
            streak_days,
            String::from_str(env, milestone_type),
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when an insight is requested
    pub fn emit_insight_requested(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        insight_type: &str,
    ) {
        let event_data = AnalyticsEventData::InsightRequested(
            student.clone(),
            course_id.clone(),
            String::from_str(env, insight_type),
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }

    /// Emit event when an insight is received
    pub fn emit_insight_received(
        env: &Env,
        student: &Address,
        insight_id: &BytesN<32>,
        insight_type: &str,
        content: &str,
        timestamp: u64,
    ) {
        let event_data = AnalyticsEventData::InsightReceived(
            student.clone(),
            insight_id.clone(),
            String::from_str(env, insight_type),
            String::from_str(env, content),
            timestamp,
        );
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(event_data),
        )
        .emit(env);
    }
}
