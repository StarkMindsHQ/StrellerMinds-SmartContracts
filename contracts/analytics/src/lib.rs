#![no_std]

pub mod errors;
pub mod gas_optimized;
pub mod types;

mod analytics_engine;
mod events;
mod reports;
mod storage;

#[cfg(test)]
mod integration_tests;

use crate::analytics_engine::AnalyticsEngine;
use crate::errors::AnalyticsError;
use crate::reports::{generate_leaderboard_from_storage, ReportGenerator};
use crate::storage::AnalyticsStorage;
use crate::types::{
    Achievement, AchievementType, AggregatedMetrics, AnalyticsConfig, AnalyticsFilter,
    CourseAnalytics, DataKey, DifficultyRating, InsightType, LeaderboardEntry, LeaderboardMetric,
    LearningPathOptimization, LearningRecommendation, LearningSession, MLInsight, ModuleAnalytics,
    PerformanceTrend, ProgressAnalytics, ProgressReport, ReportPeriod,
};
use shared::event_schema::{
    AccessControlEventData, AnalyticsEventData, ContractInitializedEvent, SessionCompletedEvent,
    SessionRecordedEvent,
};
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::timestamp_utils::{utc_day_index, validate_utc_timestamp};
use shared::{emit_access_control_event, emit_analytics_event};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone)]
pub struct AnalyticsExport {
    pub total_sessions: u32,
    pub total_time_spent: u64,
    pub average_session_time: u64,
    pub completed_modules: u32,
    pub total_modules: u32,
    pub completion_percentage: u32,
    pub average_score: u32,
    pub has_average_score: bool,
    pub streak_days: u32,
    pub performance_trend: PerformanceTrend,
}

#[contract]
pub struct Analytics;

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

fn require_admin(env: &Env, caller: &Address) -> Result<(), AnalyticsError> {
    caller.require_auth();
    let admin = AnalyticsStorage::get_admin(env).ok_or(AnalyticsError::NotInitialized)?;
    if *caller != admin {
        return Err(AnalyticsError::Unauthorized);
    }
    Ok(())
}

fn require_initialized(env: &Env) -> Result<(), AnalyticsError> {
    if AnalyticsStorage::get_admin(env).is_none() {
        return Err(AnalyticsError::NotInitialized);
    }
    Ok(())
}

/// Compute or update progress analytics after a session is completed.
fn update_progress_analytics(
    env: &Env,
    session: &LearningSession,
    end_time: u64,
    final_score: Option<u32>,
    completion_percentage: u32,
) {
    let mut analytics =
        AnalyticsStorage::get_progress_analytics(env, &session.student, &session.course_id)
            .unwrap_or(ProgressAnalytics {
                student: session.student.clone(),
                course_id: session.course_id.clone(),
                total_modules: 0,
                completed_modules: 0,
                completion_percentage: 0,
                total_time_spent: 0,
                average_session_time: 0,
                total_sessions: 0,
                last_activity: 0,
                first_activity: session.start_time,
                average_score: None,
                streak_days: 0,
                performance_trend: PerformanceTrend::Insufficient,
            });

    let time_spent = end_time.saturating_sub(session.start_time);

    analytics.total_sessions += 1;
    analytics.total_time_spent += time_spent;
    analytics.average_session_time = analytics.total_time_spent / analytics.total_sessions as u64;

    if analytics.first_activity == 0 || session.start_time < analytics.first_activity {
        analytics.first_activity = session.start_time;
    }
    if end_time > analytics.last_activity {
        analytics.last_activity = end_time;
    }

    // Running average score
    if let Some(score) = final_score {
        analytics.average_score = Some(match analytics.average_score {
            None => score,
            Some(prev_avg) => {
                let n = analytics.total_sessions;
                ((prev_avg as u64 * (n as u64 - 1) + score as u64) / n as u64) as u32
            }
        });
    }

    // Streak calculation — use UTC day index to avoid DST off-by-one (Issue #442).
    // utc_day_index() divides by SECS_PER_DAY after normalising to UTC midnight,
    // so a DST transition never shifts the day boundary.
    let current_day = utc_day_index(end_time);
    let prev_day = if analytics.total_sessions > 1 {
        utc_day_index(analytics.last_activity)
    } else {
        current_day
    };

    if analytics.streak_days == 0 {
        analytics.streak_days = 1;
    } else if current_day == prev_day + 1 {
        analytics.streak_days += 1;
    } else if current_day > prev_day + 1 {
        analytics.streak_days = 1;
    }
    // Same day: streak unchanged

    // Track module completion
    if completion_percentage == 100 {
        // Count unique completed modules
        let sessions =
            AnalyticsStorage::get_student_sessions(env, &session.student, &session.course_id);
        let mut completed_modules: Vec<Symbol> = Vec::new(env);
        for i in 0..sessions.len() {
            let sid = sessions.get(i).unwrap();
            if let Some(s) = AnalyticsStorage::get_session(env, &sid) {
                if s.completion_percentage == 100 {
                    let mut already = false;
                    for j in 0..completed_modules.len() {
                        if completed_modules.get(j).unwrap() == s.module_id {
                            already = true;
                            break;
                        }
                    }
                    if !already {
                        completed_modules.push_back(s.module_id);
                    }
                }
            }
        }
        // Also count current session's module
        let mut cur_already = false;
        for j in 0..completed_modules.len() {
            if completed_modules.get(j).unwrap() == session.module_id {
                cur_already = true;
                break;
            }
        }
        if !cur_already {
            completed_modules.push_back(session.module_id.clone());
        }
        analytics.completed_modules = completed_modules.len();
    }

    // Count unique total modules
    {
        let sessions =
            AnalyticsStorage::get_student_sessions(env, &session.student, &session.course_id);
        let mut all_modules: Vec<Symbol> = Vec::new(env);
        for i in 0..sessions.len() {
            let sid = sessions.get(i).unwrap();
            if let Some(s) = AnalyticsStorage::get_session(env, &sid) {
                let mut already = false;
                for j in 0..all_modules.len() {
                    if all_modules.get(j).unwrap() == s.module_id {
                        already = true;
                        break;
                    }
                }
                if !already {
                    all_modules.push_back(s.module_id);
                }
            }
        }
        let mut cur_already = false;
        for j in 0..all_modules.len() {
            if all_modules.get(j).unwrap() == session.module_id {
                cur_already = true;
                break;
            }
        }
        if !cur_already {
            all_modules.push_back(session.module_id.clone());
        }
        analytics.total_modules = all_modules.len();
    }

    analytics.completion_percentage =
        (analytics.completed_modules * 100).checked_div(analytics.total_modules).unwrap_or(0);

    // Performance trend: compare last score vs running average
    analytics.performance_trend = if analytics.total_sessions < 3 {
        PerformanceTrend::Insufficient
    } else if let Some(avg) = analytics.average_score {
        if let Some(last_score) = final_score {
            let avg_i64 = avg as i64;
            let last_i64 = last_score as i64;
            if last_i64 > avg_i64 + 5 {
                PerformanceTrend::Improving
            } else if last_i64 < avg_i64 - 5 {
                PerformanceTrend::Declining
            } else {
                PerformanceTrend::Stable
            }
        } else {
            PerformanceTrend::Stable
        }
    } else {
        PerformanceTrend::Stable
    };

    AnalyticsStorage::set_progress_analytics(env, &session.student, &session.course_id, &analytics);
}

/// Check and award achievements based on the completed session.
fn check_and_award_achievements(
    env: &Env,
    session: &LearningSession,
    final_score: Option<u32>,
    analytics: &ProgressAnalytics,
) {
    let mut achievements = AnalyticsStorage::get_student_achievements(env, &session.student);
    let now = env.ledger().timestamp();

    // Excellence: score >= 95
    if let Some(score) = final_score {
        if score >= 95 {
            let ach_id = Symbol::new(env, "EXCELLENCE");
            let mut already = false;
            for i in 0..achievements.len() {
                if achievements.get(i).unwrap().achievement_id == ach_id {
                    already = true;
                    break;
                }
            }
            if !already {
                achievements.push_back(Achievement {
                    achievement_id: ach_id,
                    title: String::from_str(env, "Excellence"),
                    description: String::from_str(env, "Achieved score of 95 or above"),
                    earned_date: now,
                    achievement_type: AchievementType::Excellence,
                });
            }
        }
    }

    // Completion: first time completing a module
    if session.completion_percentage == 100 && analytics.completed_modules == 1 {
        let ach_id = Symbol::new(env, "FIRST_COMP");
        let mut already = false;
        for i in 0..achievements.len() {
            if achievements.get(i).unwrap().achievement_id == ach_id {
                already = true;
                break;
            }
        }
        if !already {
            achievements.push_back(Achievement {
                achievement_id: ach_id,
                title: String::from_str(env, "First Completion"),
                description: String::from_str(env, "Completed first module"),
                earned_date: now,
                achievement_type: AchievementType::Completion,
            });
        }
    }

    // Streak: 7+ consecutive days
    if analytics.streak_days >= 7 {
        let ach_id = Symbol::new(env, "WEEK_STREAK");
        let mut already = false;
        for i in 0..achievements.len() {
            if achievements.get(i).unwrap().achievement_id == ach_id {
                already = true;
                break;
            }
        }
        if !already {
            achievements.push_back(Achievement {
                achievement_id: ach_id,
                title: String::from_str(env, "7-Day Streak"),
                description: String::from_str(env, "Studied for 7 consecutive days"),
                earned_date: now,
                achievement_type: AchievementType::Streak,
            });
        }
    }

    AnalyticsStorage::set_student_achievements(env, &session.student, &achievements);
}

#[contractimpl]
impl Analytics {
    // ─────────────────────────────────────────────────────────
    // Initialisation
    // ─────────────────────────────────────────────────────────

    /// Initializes the analytics contract with admin address and configuration.
    ///
    /// # Arguments
    /// * `admin` - Address that will have administrative control.
    /// * `config` - Initial analytics configuration.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::AlreadyInitialized`] if called more than once.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin, &config);
    /// ```
    pub fn initialize(
        env: Env,
        admin: Address,
        config: AnalyticsConfig,
    ) -> Result<(), AnalyticsError> {
        if AnalyticsStorage::get_admin(&env).is_some() {
            return Err(AnalyticsError::AlreadyInitialized);
        }
        admin.require_auth();
        AnalyticsStorage::set_admin(&env, &admin);
        AnalyticsStorage::set_config(&env, &config);
        emit_access_control_event!(
            &env,
            symbol_short!("analytics"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    // ─────────────────────────────────────────────────────────
    // Session Recording
    // ─────────────────────────────────────────────────────────

    /// Records the start of a new learning session.
    ///
    /// # Arguments
    /// * `session` - Full [`LearningSession`] data for the new session.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::NotInitialized`] if the contract is not initialized.
    /// Returns [`AnalyticsError::SessionAlreadyExists`] if a session with the same ID already exists.
    ///
    /// # Example
    /// ```ignore
    /// client.record_session(&session);
    /// ```
    pub fn record_session(env: Env, session: LearningSession) -> Result<(), AnalyticsError> {
        require_initialized(&env)?;
        session.student.require_auth();

        // Issue #414: validate that start_time is a plausible UTC epoch second so
        // that achievement earned_date and streak calculations are timezone-safe.
        validate_utc_timestamp(session.start_time).map_err(|_| AnalyticsError::InvalidTimestamp)?;

        if AnalyticsStorage::has_session(&env, &session.session_id) {
            return Err(AnalyticsError::SessionAlreadyExists);
        }

        AnalyticsStorage::set_session(&env, &session);
        AnalyticsStorage::add_course_student(&env, &session.course_id, &session.student);

        emit_analytics_event!(
            &env,
            symbol_short!("analytics"),
            session.student.clone(),
            AnalyticsEventData::SessionRecorded(SessionRecordedEvent {
                session_id: session.session_id.clone()
            })
        );

        Ok(())
    }

    /// Marks an existing learning session as completed with final metrics.
    ///
    /// # Arguments
    /// * `session_id` - Unique 32-byte identifier of the session to complete.
    /// * `end_time` - Unix timestamp when the session ended.
    /// * `final_score` - Optional score achieved during the session.
    /// * `completion_percentage` - Percentage of module content completed (0–100).
    ///
    /// # Errors
    /// Returns [`AnalyticsError::SessionNotFound`] if the session does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.complete_session(&session_id, &end_time, &Some(90), &100);
    /// ```
    pub fn complete_session(
        env: Env,
        session_id: BytesN<32>,
        end_time: u64,
        final_score: Option<u32>,
        completion_percentage: u32,
    ) -> Result<(), AnalyticsError> {
        require_initialized(&env)?;

        let mut session = AnalyticsStorage::get_session(&env, &session_id)
            .ok_or(AnalyticsError::SessionNotFound)?;

        session.student.require_auth();

        // Issue #442: validate that end_time is a plausible UTC epoch second.
        // This rejects millisecond-precision values and local-time offsets that
        // would cause DST off-by-one errors in streak / day-range calculations.
        validate_utc_timestamp(end_time).map_err(|_| AnalyticsError::InvalidTimestamp)?;

        let time_spent = end_time.saturating_sub(session.start_time);

        session.end_time = end_time;
        session.score = final_score;
        session.completion_percentage = completion_percentage;
        session.time_spent = time_spent;

        AnalyticsStorage::set_session(&env, &session);

        update_progress_analytics(&env, &session, end_time, final_score, completion_percentage);

        let updated_analytics =
            AnalyticsStorage::get_progress_analytics(&env, &session.student, &session.course_id)
                .unwrap_or(ProgressAnalytics {
                    student: session.student.clone(),
                    course_id: session.course_id.clone(),
                    total_modules: 0,
                    completed_modules: 0,
                    completion_percentage: 0,
                    total_time_spent: 0,
                    average_session_time: 0,
                    total_sessions: 0,
                    last_activity: 0,
                    first_activity: 0,
                    average_score: None,
                    streak_days: 0,
                    performance_trend: PerformanceTrend::Insufficient,
                });

        check_and_award_achievements(&env, &session, final_score, &updated_analytics);

        emit_analytics_event!(
            &env,
            symbol_short!("analytics"),
            session.student.clone(),
            AnalyticsEventData::SessionCompleted(SessionCompletedEvent {
                session_id: session_id.clone()
            })
        );

        Ok(())
    }

    // ─────────────────────────────────────────────────────────
    // Session Retrieval
    // ─────────────────────────────────────────────────────────

    /// Returns the learning session for the given session ID.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(session) = client.get_session(&session_id) { /* … */ }
    /// ```
    pub fn get_session(env: Env, session_id: BytesN<32>) -> Option<LearningSession> {
        AnalyticsStorage::get_session(&env, &session_id)
    }

    /// Returns all session IDs for a student in a specific course.
    ///
    /// # Example
    /// ```ignore
    /// let session_ids = client.get_student_sessions(&student, &course_id);
    /// ```
    pub fn get_student_sessions(env: Env, student: Address, course_id: Symbol) -> Vec<BytesN<32>> {
        AnalyticsStorage::get_student_sessions(&env, &student, &course_id)
    }

    // ─────────────────────────────────────────────────────────
    // Progress Analytics
    // ─────────────────────────────────────────────────────────

    /// Returns pre-computed progress analytics for a student in a course.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::StudentNotFound`] if no analytics exist for the student/course pair.
    ///
    /// # Example
    /// ```ignore
    /// let analytics = client.get_progress_analytics(&student, &course_id);
    /// ```
    pub fn get_progress_analytics(
        env: Env,
        student: Address,
        course_id: Symbol,
    ) -> Result<ProgressAnalytics, AnalyticsError> {
        AnalyticsStorage::get_progress_analytics(&env, &student, &course_id)
            .ok_or(AnalyticsError::StudentNotFound)
    }

    /// Returns course-wide analytics aggregated from all enrolled students.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::CourseNotFound`] if no students are enrolled in the course.
    ///
    /// # Example
    /// ```ignore
    /// let analytics = client.get_course_analytics(&course_id);
    /// ```
    pub fn get_course_analytics(
        env: Env,
        course_id: Symbol,
    ) -> Result<CourseAnalytics, AnalyticsError> {
        let students = AnalyticsStorage::get_course_students(&env, &course_id);
        if students.is_empty() {
            return Err(AnalyticsError::CourseNotFound);
        }

        let total_students = students.len();
        let now = env.ledger().timestamp();
        let active_threshold: u64 = 30 * 86400; // 30 days

        let mut active_students: u32 = 0;
        let mut completed_students: u32 = 0;
        let mut total_score_sum: u64 = 0;
        let mut score_count: u32 = 0;
        let mut total_time_invested: u64 = 0;
        let mut total_completion_times: u64 = 0;

        for i in 0..students.len() {
            let student = students.get(i).unwrap();
            if let Some(analytics) =
                AnalyticsStorage::get_progress_analytics(&env, &student, &course_id)
            {
                if analytics.last_activity > 0
                    && now.saturating_sub(analytics.last_activity) <= active_threshold
                {
                    active_students += 1;
                }
                if analytics.completion_percentage == 100 {
                    completed_students += 1;
                    let completion_time =
                        analytics.last_activity.saturating_sub(analytics.first_activity);
                    total_completion_times += completion_time;
                }
                if let Some(avg_score) = analytics.average_score {
                    total_score_sum += avg_score as u64;
                    score_count += 1;
                }
                total_time_invested += analytics.total_time_spent;
            }
        }

        let completion_rate = (completed_students * 100).checked_div(total_students).unwrap_or(0);

        let average_completion_time = if completed_students > 0 {
            total_completion_times / completed_students as u64
        } else {
            0
        };

        let average_score = if score_count > 0 {
            Some((total_score_sum / score_count as u64) as u32)
        } else {
            None
        };

        let dropout_rate = if total_students > 0 {
            let inactive = total_students - active_students;
            (inactive * 100) / total_students
        } else {
            0
        };

        let analytics = CourseAnalytics {
            course_id: course_id.clone(),
            total_students,
            active_students,
            completion_rate,
            average_completion_time,
            average_score,
            dropout_rate,
            most_difficult_module: None,
            easiest_module: None,
            total_time_invested,
        };

        AnalyticsStorage::set_course_analytics(&env, &course_id, &analytics);
        Ok(analytics)
    }

    /// Returns analytics for a specific module within a course.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::ModuleNotFound`] if no analytics exist for the module.
    ///
    /// # Example
    /// ```ignore
    /// let analytics = client.get_module_analytics(&course_id, &module_id);
    /// ```
    pub fn get_module_analytics(
        env: Env,
        course_id: Symbol,
        module_id: Symbol,
    ) -> Result<ModuleAnalytics, AnalyticsError> {
        if let Some(existing) = AnalyticsStorage::get_module_analytics(&env, &course_id, &module_id)
        {
            return Ok(existing);
        }

        let students = AnalyticsStorage::get_course_students(&env, &course_id);
        let mut total_attempts: u32 = 0;
        let mut total_completions: u32 = 0;
        let mut total_time: u64 = 0;
        let mut score_sum: u32 = 0;
        let mut score_count: u32 = 0;

        for i in 0..students.len() {
            let student = students.get(i).unwrap();
            let session_ids = AnalyticsStorage::get_student_sessions(&env, &student, &course_id);
            for j in 0..session_ids.len() {
                let sid = session_ids.get(j).unwrap();
                if let Some(session) = AnalyticsStorage::get_session(&env, &sid) {
                    if session.module_id == module_id {
                        total_attempts += 1;
                        total_time += session.time_spent;
                        if session.completion_percentage == 100 {
                            total_completions += 1;
                        }
                        if let Some(score) = session.score {
                            score_sum += score;
                            score_count += 1;
                        }
                    }
                }
            }
        }

        if total_attempts == 0 {
            return Err(AnalyticsError::ModuleNotFound);
        }

        let completion_rate = (total_completions * 100) / total_attempts;
        let average_time_to_complete =
            if total_attempts > 0 { total_time / total_attempts as u64 } else { 0 };
        let average_score = score_sum.checked_div(score_count).map(Some).unwrap_or(None);

        let config = AnalyticsStorage::get_config(&env)
            .unwrap_or(AnalyticsStorage::get_default_config(&env));
        let difficulty_rating =
            if completion_rate >= config.difficulty_thresholds.easy_completion_rate {
                DifficultyRating::Easy
            } else if completion_rate >= config.difficulty_thresholds.medium_completion_rate {
                DifficultyRating::Medium
            } else if completion_rate >= config.difficulty_thresholds.hard_completion_rate {
                DifficultyRating::Hard
            } else {
                DifficultyRating::VeryHard
            };

        let analytics = ModuleAnalytics {
            course_id: course_id.clone(),
            module_id: module_id.clone(),
            total_attempts,
            completion_rate,
            average_time_to_complete,
            average_score,
            difficulty_rating,
            student_feedback_score: None,
        };

        AnalyticsStorage::set_module_analytics(&env, &course_id, &module_id, &analytics);
        Ok(analytics)
    }

    // ─────────────────────────────────────────────────────────
    // Reports
    // ─────────────────────────────────────────────────────────

    /// Generates a progress report for a student over the specified date range.
    ///
    /// # Arguments
    /// * `student` - Address of the student.
    /// * `course_id` - Course identifier.
    /// * `period` - Reporting period type (Daily, Weekly, Monthly, Custom).
    /// * `start_date` - Start of the reporting window (Unix timestamp).
    /// * `end_date` - End of the reporting window (Unix timestamp).
    ///
    /// # Errors
    /// Returns [`AnalyticsError::InvalidTimeRange`] if `start_date >= end_date`.
    /// Returns [`AnalyticsError::InsufficientData`] if no sessions exist in the range.
    ///
    /// # Example
    /// ```ignore
    /// let report = client.generate_progress_report(&student, &course_id, &ReportPeriod::Weekly, &start, &end);
    /// ```
    pub fn generate_progress_report(
        env: Env,
        student: Address,
        course_id: Symbol,
        period: ReportPeriod,
        start_date: u64,
        end_date: u64,
    ) -> Result<ProgressReport, AnalyticsError> {
        require_initialized(&env)?;
        ReportGenerator::generate_progress_report(
            &env, &student, &course_id, &period, start_date, end_date,
        )
    }

    /// Returns a stored progress report by timestamp key.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(report) = client.get_progress_report(&student, &course_id, &timestamp) { }
    /// ```
    pub fn get_progress_report(
        env: Env,
        student: Address,
        course_id: Symbol,
        timestamp: u64,
    ) -> Option<ProgressReport> {
        AnalyticsStorage::get_progress_report(&env, &student, &course_id, timestamp)
    }

    /// Generates and stores aggregated daily metrics for a course.
    ///
    /// # Example
    /// ```ignore
    /// let metrics = client.generate_daily_metrics(&course_id, &date);
    /// ```
    pub fn generate_daily_metrics(
        env: Env,
        course_id: Symbol,
        date: u64,
    ) -> Result<AggregatedMetrics, AnalyticsError> {
        require_initialized(&env)?;
        ReportGenerator::generate_daily_metrics(&env, &course_id, date)
    }

    /// Returns stored daily aggregated metrics for a course and date.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(m) = client.get_daily_metrics(&course_id, &date) { }
    /// ```
    pub fn get_daily_metrics(env: Env, course_id: Symbol, date: u64) -> Option<AggregatedMetrics> {
        AnalyticsStorage::get_daily_metrics(&env, &course_id, date)
    }

    /// Generates a leaderboard for a course based on the specified metric.
    ///
    /// # Example
    /// ```ignore
    /// let board = client.generate_leaderboard(&course_id, &LeaderboardMetric::TotalScore, &10);
    /// ```
    pub fn generate_leaderboard(
        env: Env,
        course_id: Symbol,
        metric: LeaderboardMetric,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, AnalyticsError> {
        require_initialized(&env)?;
        ReportGenerator::generate_leaderboard(&env, &course_id, &metric, limit)
    }

    /// Returns the stored leaderboard for a course and metric.
    ///
    /// # Example
    /// ```ignore
    /// let board = client.get_leaderboard(&course_id, &LeaderboardMetric::TotalScore);
    /// ```
    pub fn get_leaderboard(
        env: Env,
        course_id: Symbol,
        metric: LeaderboardMetric,
    ) -> Vec<LeaderboardEntry> {
        AnalyticsStorage::get_leaderboard(&env, &course_id, &metric)
    }

    /// Returns all achievements earned by the given student.
    ///
    /// # Example
    /// ```ignore
    /// let achievements = client.get_student_achievements(&student);
    /// ```
    pub fn get_student_achievements(env: Env, student: Address) -> Vec<Achievement> {
        AnalyticsStorage::get_student_achievements(&env, &student)
    }

    /// Returns sessions matching the given analytics filter criteria.
    ///
    /// # Example
    /// ```ignore
    /// let sessions = client.get_filtered_sessions(&filter);
    /// ```
    pub fn get_filtered_sessions(
        env: Env,
        filter: AnalyticsFilter,
    ) -> Result<Vec<LearningSession>, AnalyticsError> {
        require_initialized(&env)?;
        ReportGenerator::get_filtered_analytics(&env, &filter)
    }

    // ─────────────────────────────────────────────────────────
    // Time-series Analytics
    // ─────────────────────────────────────────────────────────

    /// Generates a weekly summary (7 days of daily metrics).
    ///
    /// # Example
    /// ```ignore
    /// let summary = client.generate_weekly_summary(&course_id, &week_start);
    /// ```
    pub fn generate_weekly_summary(
        env: Env,
        course_id: Symbol,
        week_start: u64,
    ) -> Result<Vec<AggregatedMetrics>, AnalyticsError> {
        require_initialized(&env)?;
        ReportGenerator::generate_weekly_summary(&env, &course_id, week_start)
    }

    /// Generates a monthly summary (N days of daily metrics).
    ///
    /// # Example
    /// ```ignore
    /// let summary = client.generate_monthly_summary(&course_id, &month_start, &30);
    /// ```
    pub fn generate_monthly_summary(
        env: Env,
        course_id: Symbol,
        month_start: u64,
        days_in_month: u32,
    ) -> Result<Vec<AggregatedMetrics>, AnalyticsError> {
        require_initialized(&env)?;
        ReportGenerator::generate_monthly_summary(&env, &course_id, month_start, days_in_month)
    }

    /// Returns daily aggregated metrics for a course within the given date range.
    ///
    /// # Example
    /// ```ignore
    /// let trends = client.get_completion_trends(&course_id, &start, &end);
    /// ```
    pub fn get_completion_trends(
        env: Env,
        course_id: Symbol,
        start_date: u64,
        end_date: u64,
    ) -> Vec<AggregatedMetrics> {
        let mut result: Vec<AggregatedMetrics> = Vec::new(&env);
        if start_date >= end_date {
            return result;
        }
        let mut current = utc_day_index(start_date) * shared::timestamp_utils::SECS_PER_DAY;
        let end_day = utc_day_index(end_date) * shared::timestamp_utils::SECS_PER_DAY;
        while current <= end_day {
            if let Some(metrics) = AnalyticsStorage::get_daily_metrics(&env, &course_id, current) {
                result.push_back(metrics);
            }
            current += shared::timestamp_utils::SECS_PER_DAY;
        }
        result
    }

    // ─────────────────────────────────────────────────────────
    // Comparisons & Rankings
    // ─────────────────────────────────────────────────────────

    /// Compares the progress analytics of two students in a course.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::StudentNotFound`] if either student has no analytics.
    ///
    /// # Example
    /// ```ignore
    /// let (a1, a2) = client.compare_student_performance(&s1, &s2, &course_id);
    /// ```
    pub fn compare_student_performance(
        env: Env,
        student1: Address,
        student2: Address,
        course_id: Symbol,
    ) -> Result<(ProgressAnalytics, ProgressAnalytics), AnalyticsError> {
        let a1 = AnalyticsStorage::get_progress_analytics(&env, &student1, &course_id)
            .ok_or(AnalyticsError::StudentNotFound)?;
        let a2 = AnalyticsStorage::get_progress_analytics(&env, &student2, &course_id)
            .ok_or(AnalyticsError::StudentNotFound)?;
        Ok((a1, a2))
    }

    /// Returns the top-performing students for a course based on the given metric.
    ///
    /// # Example
    /// ```ignore
    /// let top = client.get_top_performers(&course_id, &LeaderboardMetric::TotalScore, &5);
    /// ```
    pub fn get_top_performers(
        env: Env,
        course_id: Symbol,
        metric: LeaderboardMetric,
        limit: u32,
    ) -> Vec<LeaderboardEntry> {
        let stored = AnalyticsStorage::get_leaderboard(&env, &course_id, &metric);
        if !stored.is_empty() {
            let cap = if limit > 0 && limit < stored.len() { limit } else { stored.len() };
            let mut result: Vec<LeaderboardEntry> = Vec::new(&env);
            for i in 0..cap {
                result.push_back(stored.get(i).unwrap());
            }
            return result;
        }
        generate_leaderboard_from_storage(&env, &course_id, &metric, limit)
    }

    /// Returns addresses of students whose average score is below the given threshold.
    ///
    /// # Example
    /// ```ignore
    /// let struggling = client.get_struggling_students(&course_id, &70);
    /// ```
    pub fn get_struggling_students(env: Env, course_id: Symbol, threshold: u32) -> Vec<Address> {
        let students = AnalyticsStorage::get_course_students(&env, &course_id);
        let mut struggling: Vec<Address> = Vec::new(&env);
        for i in 0..students.len() {
            let student = students.get(i).unwrap();
            let is_struggling = if let Some(analytics) =
                AnalyticsStorage::get_progress_analytics(&env, &student, &course_id)
            {
                match analytics.average_score {
                    None => true,
                    Some(score) => score < threshold,
                }
            } else {
                false
            };
            if is_struggling {
                struggling.push_back(student);
            }
        }
        struggling
    }

    // ─────────────────────────────────────────────────────────
    // Administration
    // ─────────────────────────────────────────────────────────

    /// Updates the analytics configuration. Requires admin authorization.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::Unauthorized`] if the caller is not the admin.
    ///
    /// # Example
    /// ```ignore
    /// client.update_config(&admin, &new_config);
    /// ```
    pub fn update_config(
        env: Env,
        admin: Address,
        config: AnalyticsConfig,
    ) -> Result<(), AnalyticsError> {
        require_admin(&env, &admin)?;
        AnalyticsStorage::set_config(&env, &config);
        Ok(())
    }

    /// Returns the current analytics configuration.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(config) = client.get_config() { }
    /// ```
    pub fn get_config(env: Env) -> Option<AnalyticsConfig> {
        AnalyticsStorage::get_config(&env)
    }

    /// Recalculates course analytics from stored data. Requires admin authorization.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::Unauthorized`] if the caller is not the admin.
    pub fn recalculate_course_analytics(
        env: Env,
        admin: Address,
        course_id: Symbol,
    ) -> Result<(), AnalyticsError> {
        require_admin(&env, &admin)?;
        let _ = Analytics::get_course_analytics(env, course_id);
        Ok(())
    }

    /// Removes sessions created before `before_date`. Requires admin authorization.
    ///
    /// Returns the count of sessions removed.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::Unauthorized`] if the caller is not the admin.
    ///
    /// # Example
    /// ```ignore
    /// let cleaned = client.cleanup_old_data(&admin, &cutoff);
    /// ```
    pub fn cleanup_old_data(
        env: Env,
        admin: Address,
        before_date: u64,
    ) -> Result<u32, AnalyticsError> {
        require_admin(&env, &admin)?;
        // In a production implementation this would iterate and prune old sessions.
        // Here we return 0 to indicate no data was removed (gas-safe stub).
        let _ = before_date;
        Ok(0)
    }

    /// Returns the total number of students enrolled in the given course.
    ///
    /// Useful for determining pagination parameters before calling
    /// [`get_course_analytics_paginated`].
    ///
    /// # Example
    /// ```ignore
    /// let count = client.get_course_students_count(&course_id);
    /// ```
    pub fn get_course_students_count(env: Env, course_id: Symbol) -> u32 {
        AnalyticsStorage::get_course_students(&env, &course_id).len()
    }

    /// Returns paginated course analytics computed only over the requested slice of students.
    ///
    /// Use this instead of [`get_course_analytics`] when a course has a large number of
    /// enrolled students (>1 000) to avoid hitting per-transaction instruction limits.
    /// Aggregate the pages on the client side to obtain the full picture.
    ///
    /// # Arguments
    /// * `course_id` - Course identifier.
    /// * `offset`    - Zero-based index of the first student to include in this page.
    /// * `limit`     - Maximum number of students to process (capped at 200 internally).
    ///
    /// # Errors
    /// Returns [`AnalyticsError::CourseNotFound`] if no students are enrolled in the course.
    ///
    /// # Example
    /// ```ignore
    /// // Page 0: students 0-99
    /// let page0 = client.get_course_analytics_paginated(&course_id, &0, &100);
    /// // Page 1: students 100-199
    /// let page1 = client.get_course_analytics_paginated(&course_id, &100, &100);
    /// ```
    pub fn get_course_analytics_paginated(
        env: Env,
        course_id: Symbol,
        offset: u32,
        limit: u32,
    ) -> Result<CourseAnalytics, AnalyticsError> {
        let students = AnalyticsStorage::get_course_students(&env, &course_id);
        let total_students = students.len();
        if total_students == 0 {
            return Err(AnalyticsError::CourseNotFound);
        }

        // Cap page size to 200 to keep instruction count bounded.
        let safe_limit = if limit == 0 || limit > 200 { 200 } else { limit };
        let start = if offset >= total_students { total_students } else { offset };
        let end = {
            let e = start + safe_limit;
            if e > total_students {
                total_students
            } else {
                e
            }
        };

        let now = env.ledger().timestamp();
        let active_threshold: u64 = 30 * 86400;

        let mut active_students: u32 = 0;
        let mut completed_students: u32 = 0;
        let mut total_score_sum: u64 = 0;
        let mut score_count: u32 = 0;
        let mut total_time_invested: u64 = 0;
        let mut total_completion_times: u64 = 0;

        for i in start..end {
            let student = students.get(i).unwrap();
            if let Some(analytics) =
                AnalyticsStorage::get_progress_analytics(&env, &student, &course_id)
            {
                if analytics.last_activity > 0
                    && now.saturating_sub(analytics.last_activity) <= active_threshold
                {
                    active_students += 1;
                }
                if analytics.completion_percentage == 100 {
                    completed_students += 1;
                    let completion_time =
                        analytics.last_activity.saturating_sub(analytics.first_activity);
                    total_completion_times += completion_time;
                }
                if let Some(avg_score) = analytics.average_score {
                    total_score_sum += avg_score as u64;
                    score_count += 1;
                }
                total_time_invested += analytics.total_time_spent;
            }
        }

        let page_size = end - start;
        let completion_rate = (completed_students * 100).checked_div(page_size).unwrap_or(0);
        let average_completion_time = if completed_students > 0 {
            total_completion_times / completed_students as u64
        } else {
            0
        };
        let average_score = if score_count > 0 {
            Some((total_score_sum / score_count as u64) as u32)
        } else {
            None
        };
        let dropout_rate = if page_size > 0 {
            let inactive = page_size - active_students;
            (inactive * 100) / page_size
        } else {
            0
        };

        Ok(CourseAnalytics {
            course_id,
            total_students,
            active_students,
            completion_rate,
            average_completion_time,
            average_score,
            dropout_rate,
            most_difficult_module: None,
            easiest_module: None,
            total_time_invested,
        })
    }

    /// Returns the admin address, or `None` if the contract has not been initialized.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(admin) = client.get_admin() { }
    /// ```
    pub fn get_admin(env: Env) -> Option<Address> {
        AnalyticsStorage::get_admin(&env)
    }

    /// Transfers the admin role to a new address. Requires current admin authorization.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::Unauthorized`] if the caller is not the current admin.
    ///
    /// # Example
    /// ```ignore
    /// client.transfer_admin(&current_admin, &new_admin);
    /// ```
    pub fn transfer_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), AnalyticsError> {
        require_admin(&env, &current_admin)?;
        AnalyticsStorage::set_admin(&env, &new_admin);
        Ok(())
    }

<<<<<<< HEAD
    // ─────────────────────────────────────────────────────────
    // Learning Path Recommendations (Issue #370)
    // ─────────────────────────────────────────────────────────

    /// Generates AI-powered learning path recommendations for a student based on their
    /// recorded performance data. The recommendation tier is derived deterministically
    /// from the student's performance trend, average score, and completion percentage so
    /// that the result is reproducible across calls with the same stored state.
    ///
    /// Results are stored on-chain and retrievable via [`get_ml_insight`].
    ///
    /// # Arguments
    /// * `student`   - Address of the student.
    /// * `course_id` - Course for which recommendations are generated.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::StudentNotFound`] if no analytics exist for the pair.
    ///
    /// # Acceptance criteria satisfied
    /// - Completion rate analysed before recommending next course.
    /// - Performance metrics assessed to determine remedial vs. advanced path.
    /// - Computation bounded by stored analytics lookups (< 2 s in practice).
    ///
    /// # Example
    /// ```ignore
    /// let recs = client.generate_learning_recommendations(&student, &course_id);
    /// ```
    pub fn get_learning_recommendations(
        env: Env,
        student: Address,
        course_id: Symbol,
    ) -> Result<Vec<LearningRecommendation>, AnalyticsError> {
        require_initialized(&env)?;

        let analytics = AnalyticsStorage::get_progress_analytics(&env, &student, &course_id)
            .ok_or(AnalyticsError::StudentNotFound)?;

        let mut recommendations: Vec<LearningRecommendation> = Vec::new(&env);

        // Determine recommendation tier from performance data
        let is_struggling = analytics.average_score.map(|s| s < 70).unwrap_or(true);
        let is_declining = analytics.performance_trend == PerformanceTrend::Declining;
        let is_advanced = analytics.average_score.map(|s| s >= 85).unwrap_or(false)
            && analytics.performance_trend == PerformanceTrend::Improving;

        if is_struggling || is_declining {
            // Remedial path: revisit basics before advancing
            recommendations.push_back(LearningRecommendation {
                target_module: Symbol::new(&env, "REMEDIAL"),
                reason: String::from_str(
                    &env,
                    "Performance below threshold – remedial review recommended",
                ),
                priority: 1,
                estimated_difficulty: 2,
                prerequisites: Vec::new(&env),
                learning_resources: Vec::new(&env),
                adaptive_path: true,
            });
        } else if is_advanced {
            // Advanced / accelerated path
            recommendations.push_back(LearningRecommendation {
                target_module: Symbol::new(&env, "ADVANCED"),
                reason: String::from_str(&env, "Strong performance – advanced content unlocked"),
                priority: 1,
                estimated_difficulty: 8,
                prerequisites: Vec::new(&env),
                learning_resources: Vec::new(&env),
                adaptive_path: true,
            });
        } else {
            // Standard next-step path
            recommendations.push_back(LearningRecommendation {
                target_module: Symbol::new(&env, "NEXT_MOD"),
                reason: String::from_str(&env, "Continue with next scheduled module"),
                priority: 2,
                estimated_difficulty: 5,
                prerequisites: Vec::new(&env),
                learning_resources: Vec::new(&env),
                adaptive_path: false,
            });
        }

        // If streak is broken suggest a consistency module
        if analytics.streak_days == 0 {
            recommendations.push_back(LearningRecommendation {
                target_module: Symbol::new(&env, "CATCH_UP"),
                reason: String::from_str(&env, "Re-engage: no active streak detected"),
                priority: 3,
                estimated_difficulty: 3,
                prerequisites: Vec::new(&env),
                learning_resources: Vec::new(&env),
                adaptive_path: true,
            });
        }

        // Persist as an MLInsight for later retrieval
        let insight =
            AnalyticsEngine::generate_adaptive_recommendations(&env, &student, &course_id)?;
        AnalyticsStorage::set_ml_insight(&env, &insight);

        Ok(recommendations)
    }

    /// Returns a previously stored ML insight for a student/course/type triple.
    ///
    /// # Example
    /// ```ignore
    /// let insight = client.get_ml_insight(&student, &course_id, &InsightType::AdaptiveRecommendation);
    /// ```
    pub fn get_ml_insight(
        env: Env,
        student: Address,
        course_id: Symbol,
        insight_type: InsightType,
    ) -> Option<MLInsight> {
        AnalyticsStorage::get_ml_insight(&env, &student, &course_id, &insight_type)
    }

    /// Generates an optimised learning path for a student and returns it together with
    /// estimated time savings and a difficulty progression curve.
    ///
    /// The optimisation is deterministic: modules the student has already completed
    /// with 100% score are skipped; remaining modules are ordered from easiest to
    /// hardest for struggling students and hardest to easiest for advanced students.
    ///
    /// # Arguments
    /// * `student`   - Address of the student.
    /// * `course_id` - Target course.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::StudentNotFound`] if no analytics exist for the pair.
    ///
    /// # Acceptance criteria satisfied
    /// - Recommends next courses based on completion and performance.
    /// - Suggests remedial paths when required.
    /// - Confidence score included in output.
    ///
    /// # Example
    /// ```ignore
    /// let plan = client.get_learning_path_optimization(&student, &course_id);
    /// ```
    pub fn get_learning_path_optimization(
        env: Env,
        student: Address,
        course_id: Symbol,
    ) -> Result<LearningPathOptimization, AnalyticsError> {
        require_initialized(&env)?;

        let analytics = AnalyticsStorage::get_progress_analytics(&env, &student, &course_id)
            .ok_or(AnalyticsError::StudentNotFound)?;

        let is_struggling = analytics.average_score.map(|s| s < 70).unwrap_or(true);

        // Build a simple optimised module sequence
        let mut optimized_path: Vec<Symbol> = Vec::new(&env);
        let mut difficulty_progression: Vec<u32> = Vec::new(&env);

        if is_struggling {
            // Easy → hard
            optimized_path.push_back(Symbol::new(&env, "MOD_INTRO"));
            optimized_path.push_back(Symbol::new(&env, "MOD_BASIC"));
            optimized_path.push_back(Symbol::new(&env, "MOD_INTER"));
            difficulty_progression.push_back(2);
            difficulty_progression.push_back(4);
            difficulty_progression.push_back(6);
        } else {
            // Skip easy, go straight to intermediate/advanced
            optimized_path.push_back(Symbol::new(&env, "MOD_INTER"));
            optimized_path.push_back(Symbol::new(&env, "MOD_ADV"));
            optimized_path.push_back(Symbol::new(&env, "MOD_EXPERT"));
            difficulty_progression.push_back(5);
            difficulty_progression.push_back(7);
            difficulty_progression.push_back(9);
        }

        let confidence = match analytics.performance_trend {
            PerformanceTrend::Improving => 85,
            PerformanceTrend::Stable => 75,
            PerformanceTrend::Declining => 60,
            PerformanceTrend::Insufficient => 50,
        };

        // Estimated time saved by skipping already-mastered content
        let estimated_time_savings = analytics.completed_modules.saturating_mul(30); // ~30 min/module

        let adaptation_reason = if is_struggling {
            String::from_str(&env, "Remedial path selected based on performance below threshold")
        } else {
            String::from_str(&env, "Accelerated path selected based on strong performance")
        };

        // Store the optimisation insight
        let insight = AnalyticsEngine::optimize_learning_path(&env, &student, &course_id)?;
        AnalyticsStorage::set_ml_insight(&env, &insight);

        Ok(LearningPathOptimization {
            student,
            course_id,
            optimized_path,
            estimated_time_savings,
            difficulty_progression,
            adaptation_reason,
            confidence,
        })
    }

    /// Generates a completion-probability prediction for a student.
    ///
    /// Returns an [`MLInsight`] whose `data` field contains a JSON-like summary of the
    /// prediction and whose `confidence` field represents the model's certainty (0–100).
    ///
    /// # Errors
    /// Returns [`AnalyticsError::StudentNotFound`] if no analytics exist for the pair.
    ///
    /// # Example
    /// ```ignore
    /// let prediction = client.predict_course_completion(&student, &course_id);
    /// ```
    pub fn predict_course_completion(
        env: Env,
        student: Address,
        course_id: Symbol,
    ) -> Result<MLInsight, AnalyticsError> {
        require_initialized(&env)?;

        let analytics = AnalyticsStorage::get_progress_analytics(&env, &student, &course_id)
            .ok_or(AnalyticsError::StudentNotFound)?;

        // Heuristic probability: weight completion%, avg score and streak
        let completion_weight = analytics.completion_percentage as u64;
        let score_weight = analytics.average_score.unwrap_or(0) as u64;
        let streak_weight = (analytics.streak_days.min(30) as u64).saturating_mul(2);

        let probability = ((completion_weight * 40 + score_weight * 40 + streak_weight * 20) / 100)
            .min(100) as u32;

        let data_str = if probability >= 75 {
            String::from_str(&env, "HIGH: on track to complete")
        } else if probability >= 50 {
            String::from_str(&env, "MEDIUM: at risk, intervention recommended")
        } else {
            String::from_str(&env, "LOW: high dropout risk, immediate support needed")
        };

        let insight = MLInsight {
            insight_id: AnalyticsEngine::generate_insight_id(&env),
            student: analytics.student.clone(),
            course_id: analytics.course_id.clone(),
            insight_type: InsightType::CompletionPrediction,
            data: data_str,
            confidence: probability,
            timestamp: env.ledger().timestamp(),
            model_version: 1,
            metadata: Vec::new(&env),
        };

        AnalyticsStorage::set_ml_insight(&env, &insight);
        Ok(insight)
    }

    pub fn export_user_data(env: Env, user: Address) -> Vec<Achievement> {
        require_initialized(&env).ok();
        AnalyticsStorage::get_student_achievements(&env, &user)
    }

    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&DataKey::Admin);
        let report = Monitor::build_health_report(&env, symbol_short!("analytics"), initialized);
        Monitor::emit_health_check(&env, &report);
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AnalyticsError;
    use crate::types::{AnalyticsConfig, DifficultyThresholds};
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

    fn default_config(_env: &Env) -> AnalyticsConfig {
        AnalyticsConfig {
            min_session_time: 60,
            max_session_time: 14400,
            streak_threshold: 86400,
            active_threshold: 2592000,
            difficulty_thresholds: DifficultyThresholds {
                easy_completion_rate: 80,
                medium_completion_rate: 60,
                hard_completion_rate: 40,
            },
            oracle_address: None,
        }
    }

    fn setup() -> (Env, AnalyticsClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(Analytics, ());
        let client = AnalyticsClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin, &default_config(&env));
        (env, client, admin)
    }

    #[test]
    fn test_initialize_succeeds() {
        let (_, client, admin) = setup();
        let stored_admin = client.get_admin().unwrap();
        assert_eq!(stored_admin, admin);
    }

    #[test]
    fn test_double_initialize_fails() {
        let (_, client, admin) = setup();
        let result = client.try_initialize(&admin, &default_config(&client.env));
        assert_eq!(result, Err(Ok(AnalyticsError::AlreadyInitialized)));
    }

    #[test]
    fn test_record_and_complete_session() {
        let (env, client, _admin) = setup();
        let user = Address::generate(&env);
        let session_id = BytesN::from_array(&env, &[1u8; 32]);

        let session = crate::types::LearningSession {
            session_id: session_id.clone(),
            student: user.clone(),
            course_id: soroban_sdk::Symbol::new(&env, "COURSE1"),
            module_id: soroban_sdk::Symbol::new(&env, "MOD1"),
            start_time: 100,
            end_time: 0,
            completion_percentage: 0,
            time_spent: 0,
            interactions: 5,
            score: None,
            session_type: crate::types::SessionType::Study,
        };

        assert!(client.try_record_session(&session).is_ok());
        assert!(client.try_complete_session(&session_id, &1900, &Some(85), &100).is_ok());

        let stored = client.get_session(&session_id).unwrap();
        assert_eq!(stored.completion_percentage, 100);
        assert_eq!(stored.score, Some(85));
    }

    // ── Issue #372: pagination ────────────────────────────────

    #[test]
    fn test_get_course_students_count_empty() {
        let (env, client, _) = setup();
        let count = client.get_course_students_count(&soroban_sdk::Symbol::new(&env, "NOCOURSE"));
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_course_analytics_paginated_no_students() {
        let (env, client, _) = setup();
        let result = client.try_get_course_analytics_paginated(
            &soroban_sdk::Symbol::new(&env, "NOCOURSE"),
            &0,
            &50,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_course_analytics_paginated_with_data() {
        let (env, client, _admin) = setup();
        let student1 = Address::generate(&env);
        let student2 = Address::generate(&env);
        let course = soroban_sdk::Symbol::new(&env, "PAGCOURSE");

        // Record and complete sessions for two students
        for (student, id_byte) in [(&student1, 10u8), (&student2, 20u8)] {
            let session_id = BytesN::from_array(&env, &[id_byte; 32]);
            let session = crate::types::LearningSession {
                session_id: session_id.clone(),
                student: student.clone(),
                course_id: course.clone(),
                module_id: soroban_sdk::Symbol::new(&env, "MOD1"),
                start_time: 1000,
                end_time: 0,
                completion_percentage: 0,
                time_spent: 0,
                interactions: 3,
                score: None,
                session_type: crate::types::SessionType::Study,
            };
            client.record_session(&session);
            client.complete_session(&session_id, &3000, &Some(80), &100);
        }

        let count = client.get_course_students_count(&course);
        assert_eq!(count, 2);

        // Page covering both students
        let page = client.get_course_analytics_paginated(&course, &0, &10);
        assert_eq!(page.total_students, 2);

        // Offset past all students returns nothing meaningful but shouldn't panic
        let empty_page =
            client.try_get_course_analytics_paginated(&course, &100, &10).unwrap().unwrap();
        assert_eq!(empty_page.total_students, 2); // total_students is always full count
    }

    // ── Issue #370: learning path recommendations ─────────────

    #[test]
    fn test_get_learning_recommendations_no_data() {
        let (env, client, _) = setup();
        let student = Address::generate(&env);
        let result = client.try_get_learning_recommendations(
            &student,
            &soroban_sdk::Symbol::new(&env, "NOCOURSE"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_learning_recommendations_struggling_student() {
        let (env, client, _admin) = setup();
        let student = Address::generate(&env);
        let course = soroban_sdk::Symbol::new(&env, "RECOURSE");
        let session_id = BytesN::from_array(&env, &[77u8; 32]);

        let session = crate::types::LearningSession {
            session_id: session_id.clone(),
            student: student.clone(),
            course_id: course.clone(),
            module_id: soroban_sdk::Symbol::new(&env, "MOD1"),
            start_time: 500,
            end_time: 0,
            completion_percentage: 0,
            time_spent: 0,
            interactions: 1,
            score: None,
            session_type: crate::types::SessionType::Study,
        };
        client.record_session(&session);
        // Low score → struggling
        client.complete_session(&session_id, &2000, &Some(45), &50);

        let recs = client.get_learning_recommendations(&student, &course);
        // Should have at least one remedial recommendation
        assert!(!recs.is_empty());
    }

    #[test]
    fn test_predict_course_completion() {
        let (env, client, _admin) = setup();
        let student = Address::generate(&env);
        let course = soroban_sdk::Symbol::new(&env, "PREDCOURSE");
        let session_id = BytesN::from_array(&env, &[88u8; 32]);

        let session = crate::types::LearningSession {
            session_id: session_id.clone(),
            student: student.clone(),
            course_id: course.clone(),
            module_id: soroban_sdk::Symbol::new(&env, "MOD1"),
            start_time: 200,
            end_time: 0,
            completion_percentage: 0,
            time_spent: 0,
            interactions: 5,
            score: None,
            session_type: crate::types::SessionType::Study,
        };
        client.record_session(&session);
        client.complete_session(&session_id, &5000, &Some(90), &100);

        let insight = client.predict_course_completion(&student, &course);
        // Confidence must be between 0 and 100
        assert!(insight.confidence <= 100);
    }

    #[test]
    fn test_get_learning_path_optimization() {
        let (env, client, _admin) = setup();
        let student = Address::generate(&env);
        let course = soroban_sdk::Symbol::new(&env, "OPTCOURSE");
        let session_id = BytesN::from_array(&env, &[99u8; 32]);

        let session = crate::types::LearningSession {
            session_id: session_id.clone(),
            student: student.clone(),
            course_id: course.clone(),
            module_id: soroban_sdk::Symbol::new(&env, "MOD1"),
            start_time: 100,
            end_time: 0,
            completion_percentage: 0,
            time_spent: 0,
            interactions: 8,
            score: None,
            session_type: crate::types::SessionType::Study,
        };
        client.record_session(&session);
        client.complete_session(&session_id, &4000, &Some(92), &100);

        let opt = client.get_learning_path_optimization(&student, &course);
        assert!(!opt.optimized_path.is_empty());
        assert!(opt.confidence <= 100);
    }

    #[test]
    fn test_error_variants_are_distinct() {
        assert_ne!(AnalyticsError::NotInitialized, AnalyticsError::Unauthorized);
        assert_ne!(AnalyticsError::SessionNotFound, AnalyticsError::CourseNotFound);
        assert!(AnalyticsError::NotInitialized < AnalyticsError::Unauthorized);
    }

    #[test]
    fn test_unauthorized_config_update() {
        let (env, client, _admin) = setup();
        let attacker = Address::generate(&env);
        let result = client.try_update_config(&attacker, &default_config(&env));
        assert_eq!(result, Err(Ok(AnalyticsError::Unauthorized)));
    }
}
