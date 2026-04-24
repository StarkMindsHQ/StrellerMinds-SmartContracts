use crate::{
    errors::AnalyticsError,
    events::AnalyticsEvents,
    storage::AnalyticsStorage,
    types::{
        Achievement, AggregatedMetrics, AnalyticsFilter, LeaderboardEntry, LeaderboardMetric,
        LearningSession, OptionalSessionType, ProgressReport, ReportPeriod,
    },
};
use soroban_sdk::{Address, Env, Symbol, Vec};

/// Report generation and time-based analytics
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate comprehensive progress report for a student
    pub fn generate_progress_report(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        period: &ReportPeriod,
        start_date: u64,
        end_date: u64,
    ) -> Result<ProgressReport, AnalyticsError> {
        if start_date >= end_date {
            return Err(AnalyticsError::InvalidTimeRange);
        }

        let sessions = AnalyticsStorage::get_student_sessions(env, student, course_id);
        let mut filtered_sessions: Vec<LearningSession> = Vec::new(env);

        // Filter sessions by date range
        for i in 0..sessions.len() {
            let session_id = sessions.get(i).unwrap();
            if let Some(session) = AnalyticsStorage::get_session(env, &session_id) {
                if session.start_time >= start_date && session.end_time <= end_date {
                    filtered_sessions.push_back(session);
                }
            }
        }

        if filtered_sessions.is_empty() {
            return Err(AnalyticsError::InsufficientData);
        }

        // Calculate report metrics
        let sessions_count = filtered_sessions.len();
        let mut total_time = 0u64;
        let mut modules_completed = 0u32;
        let mut completed_modules: Vec<Symbol> = Vec::new(env);

        for i in 0..filtered_sessions.len() {
            let session = filtered_sessions.get(i).unwrap();
            total_time += session.time_spent;

            if session.completion_percentage == 100 {
                let mut already_counted = false;
                for j in 0..completed_modules.len() {
                    if completed_modules.get(j).unwrap() == session.module_id {
                        already_counted = true;
                        break;
                    }
                }
                if !already_counted {
                    completed_modules.push_back(session.module_id.clone());
                    modules_completed += 1;
                }
            }
        }

        // Calculate average daily time
        let report_days = ((end_date - start_date) / 86400) + 1; // +1 to include both start and end days
        let average_daily_time = if report_days > 0 {
            total_time / report_days
        } else {
            0
        };

        // Calculate consistency score
        let consistency_score =
            Self::calculate_consistency_score(env, &filtered_sessions, start_date, end_date);

        // Get achievements earned during this period
        let achievements = Self::get_achievements_in_period(env, student, start_date, end_date);

        let report = ProgressReport {
            student: student.clone(),
            course_id: course_id.clone(),
            report_period: period.clone(),
            start_date,
            end_date,
            sessions_count,
            total_time,
            modules_completed,
            average_daily_time,
            consistency_score,
            achievements,
        };

        // Store the report
        let report_timestamp = env.ledger().timestamp();
        AnalyticsStorage::set_progress_report(env, student, course_id, report_timestamp, &report);

        // Emit event
        let period_str = match period {
            ReportPeriod::Daily => "Daily",
            ReportPeriod::Weekly => "Weekly",
            ReportPeriod::Monthly => "Monthly",
            ReportPeriod::Custom => "Custom",
        };

        AnalyticsEvents::emit_report_generated(
            env,
            student,
            course_id,
            period_str,
            start_date,
            end_date,
            sessions_count,
        );

        Ok(report)
    }

    /// Generate daily aggregated metrics for a course
    pub fn generate_daily_metrics(
        env: &Env,
        course_id: &Symbol,
        date: u64,
    ) -> Result<AggregatedMetrics, AnalyticsError> {
        let day_start = (date / 86400) * 86400; // Start of day
        let day_end = day_start + 86400; // End of day

        let students = AnalyticsStorage::get_course_students(env, course_id);
        let mut active_students = 0u32;
        let mut total_sessions = 0u32;
        let mut total_time = 0u64;
        let mut completions = 0u32;
        let mut scores: Vec<u32> = Vec::new(env);

        // Process each student's activity for this day
        for i in 0..students.len() {
            let student = students.get(i).unwrap();
            let sessions = AnalyticsStorage::get_student_sessions(env, &student, course_id);
            let mut student_active = false;

            for j in 0..sessions.len() {
                let session_id = sessions.get(j).unwrap();
                if let Some(session) = AnalyticsStorage::get_session(env, &session_id) {
                    if session.start_time >= day_start && session.start_time < day_end {
                        if !student_active {
                            active_students += 1;
                            student_active = true;
                        }

                        total_sessions += 1;
                        total_time += session.time_spent;

                        if session.completion_percentage == 100 {
                            completions += 1;
                        }

                        if let Some(score) = session.score {
                            scores.push_back(score);
                        }
                    }
                }
            }
        }

        // Calculate average score
        let average_score = if !scores.is_empty() {
            let mut sum = 0u32;
            for i in 0..scores.len() {
                sum += scores.get(i).unwrap();
            }
            Some(sum / scores.len())
        } else {
            None
        };

        let metrics = AggregatedMetrics {
            course_id: course_id.clone(),
            date: day_start,
            active_students,
            total_sessions,
            total_time,
            completions,
            average_score,
        };

        // Store metrics
        AnalyticsStorage::set_daily_metrics(env, course_id, day_start, &metrics);

        // Emit event
        AnalyticsEvents::emit_data_aggregated(
            env,
            course_id,
            day_start,
            active_students,
            total_sessions,
        );

        Ok(metrics)
    }

    /// Generate leaderboard for a course
    pub fn generate_leaderboard(
        env: &Env,
        course_id: &Symbol,
        metric: &LeaderboardMetric,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, AnalyticsError> {
        let students = AnalyticsStorage::get_course_students(env, course_id);

        if students.is_empty() {
            return Err(AnalyticsError::InsufficientData);
        }

        let mut entries: Vec<LeaderboardEntry> = Vec::new(env);

        // Calculate scores for each student based on metric type
        for i in 0..students.len() {
            let student = students.get(i).unwrap();

            if let Some(analytics) =
                AnalyticsStorage::get_progress_analytics(env, &student, course_id)
            {
                let score = match metric {
                    LeaderboardMetric::CompletionSpeed => {
                        if analytics.completion_percentage == 100 {
                            // Lower time is better, so invert the score
                            let completion_time =
                                analytics.last_activity - analytics.first_activity;
                            if completion_time > 0 {
                                1000000 / completion_time as u32 // Arbitrary scaling
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    }
                    LeaderboardMetric::TotalScore => analytics.average_score.unwrap_or(0),
                    LeaderboardMetric::ConsistencyScore => {
                        // Calculate consistency based on streak and regular activity
                        analytics.streak_days * 10 + (analytics.total_sessions / 10)
                    }
                    LeaderboardMetric::TimeSpent => {
                        (analytics.total_time_spent / 3600) as u32 // Convert to hours
                    }
                };

                let entry = LeaderboardEntry {
                    student: student.clone(),
                    score,
                    rank: 0, // Will be set after sorting
                    course_id: course_id.clone(),
                    metric_type: metric.clone(),
                };
                entries.push_back(entry);
            }
        }

        // Sort entries by score (descending)
        Self::sort_leaderboard_entries(env, &mut entries);

        // Assign ranks and limit results
        let mut final_entries: Vec<LeaderboardEntry> = Vec::new(env);
        let max_entries = if limit > 0 && limit < entries.len() {
            limit
        } else {
            entries.len()
        };

        for i in 0..max_entries {
            let mut entry = entries.get(i).unwrap();
            entry.rank = i + 1;
            final_entries.push_back(entry);
        }

        // Store leaderboard
        AnalyticsStorage::set_leaderboard(env, course_id, metric, &final_entries);

        // Emit event
        if !final_entries.is_empty() {
            let top_entry = final_entries.get(0).unwrap();
            AnalyticsEvents::emit_leaderboard_updated(
                env,
                course_id,
                metric.clone(),
                &top_entry.student,
                top_entry.score,
                final_entries.len(),
            );
        }

        Ok(final_entries)
    }

    /// Get filtered analytics data
    pub fn get_filtered_analytics(
        env: &Env,
        filter: &AnalyticsFilter,
    ) -> Result<Vec<LearningSession>, AnalyticsError> {
        let mut filtered_sessions: Vec<LearningSession> = Vec::new(env);

        // If course_id is specified, get sessions for that course
        if let Some(course_id) = &filter.course_id {
            if let Some(student) = &filter.student {
                // Get sessions for specific student and course
                let sessions = AnalyticsStorage::get_student_sessions(env, student, course_id);

                for i in 0..sessions.len() {
                    let session_id = sessions.get(i).unwrap();
                    if let Some(session) = AnalyticsStorage::get_session(env, &session_id) {
                        if Self::session_matches_filter(&session, filter) {
                            filtered_sessions.push_back(session);
                        }
                    }
                }
            } else {
                // Get sessions for all students in the course
                let students = AnalyticsStorage::get_course_students(env, course_id);

                for i in 0..students.len() {
                    let student = students.get(i).unwrap();
                    let sessions = AnalyticsStorage::get_student_sessions(env, &student, course_id);

                    for j in 0..sessions.len() {
                        let session_id = sessions.get(j).unwrap();
                        if let Some(session) = AnalyticsStorage::get_session(env, &session_id) {
                            if Self::session_matches_filter(&session, filter) {
                                filtered_sessions.push_back(session);
                            }
                        }
                    }
                }
            }
        }

        Ok(filtered_sessions)
    }

    /// Calculate consistency score based on regular activity
    fn calculate_consistency_score(
        env: &Env,
        sessions: &Vec<LearningSession>,
        start_date: u64,
        end_date: u64,
    ) -> u32 {
        if sessions.is_empty() {
            return 0;
        }

        let total_days = ((end_date - start_date) / 86400) + 1;
        let mut active_days: Vec<u64> = Vec::new(env);

        // Count unique active days
        for i in 0..sessions.len() {
            let session = sessions.get(i).unwrap();
            let day = session.start_time / 86400;

            let mut day_exists = false;
            for j in 0..active_days.len() {
                if active_days.get(j).unwrap() == day {
                    day_exists = true;
                    break;
                }
            }

            if !day_exists {
                active_days.push_back(day);
            }
        }

        // Calculate consistency as percentage of days with activity
        if total_days > 0 {
            (active_days.len() * 100) / total_days as u32
        } else {
            0
        }
    }

    /// Get achievements earned in a specific time period
    fn get_achievements_in_period(
        env: &Env,
        student: &Address,
        start_date: u64,
        end_date: u64,
    ) -> Vec<Achievement> {
        let all_achievements = AnalyticsStorage::get_student_achievements(env, student);
        let mut period_achievements: Vec<Achievement> = Vec::new(env);

        for i in 0..all_achievements.len() {
            let achievement = all_achievements.get(i).unwrap();
            if achievement.earned_date >= start_date && achievement.earned_date <= end_date {
                period_achievements.push_back(achievement);
            }
        }

        period_achievements
    }

    /// Check if a session matches the given filter
    fn session_matches_filter(session: &LearningSession, filter: &AnalyticsFilter) -> bool {
        // Check date range
        if let Some(start_date) = filter.start_date {
            if session.start_time < start_date {
                return false;
            }
        }

        if let Some(end_date) = filter.end_date {
            if session.end_time > end_date {
                return false;
            }
        }

        // Check session type
        // if let Some(session_type) = &filter.session_type {
        //     if session.session_type != *session_type {
        //         return false;
        //     }
        // }

        let sess = &filter.session_type;
        match sess {
            OptionalSessionType::None => {}
            OptionalSessionType::Some(session_type) => {
                if session.session_type != *session_type {
                    return false;
                }
            }
        };

        // Check minimum score
        if let Some(min_score) = filter.min_score {
            if let Some(score) = session.score {
                if score < min_score {
                    return false;
                }
            } else {
                return false; // No score available
            }
        }

        true
    }

    /// Sort leaderboard entries by score (descending)
    fn sort_leaderboard_entries(_env: &Env, entries: &mut Vec<LeaderboardEntry>) {
        // Simple bubble sort for small datasets
        for _i in 0..entries.len() {
            for j in 0..entries.len() - 1 {
                let entry_j = entries.get(j).unwrap();
                let entry_j_plus_1 = entries.get(j + 1).unwrap();

                if entry_j.score < entry_j_plus_1.score {
                    // Swap entries
                    entries.set(j, entry_j_plus_1);
                    entries.set(j + 1, entry_j);
                }
            }
        }
    }

    /// Generate weekly summary report
    pub fn generate_weekly_summary(
        env: &Env,
        course_id: &Symbol,
        week_start: u64,
    ) -> Result<Vec<AggregatedMetrics>, AnalyticsError> {
        let mut weekly_metrics: Vec<AggregatedMetrics> = Vec::new(env);

        // Generate metrics for each day of the week
        for day in 0..7 {
            let date = week_start + (day * 86400);
            if let Ok(metrics) = Self::generate_daily_metrics(env, course_id, date) {
                weekly_metrics.push_back(metrics);
            }
        }

        Ok(weekly_metrics)
    }

    /// Generate monthly summary report
    pub fn generate_monthly_summary(
        env: &Env,
        course_id: &Symbol,
        month_start: u64,
        days_in_month: u32,
    ) -> Result<Vec<AggregatedMetrics>, AnalyticsError> {
        let mut monthly_metrics: Vec<AggregatedMetrics> = Vec::new(env);

        // Generate metrics for each day of the month
        for day in 0..days_in_month {
            let date = month_start + (day as u64 * 86400);
            if let Ok(metrics) = Self::generate_daily_metrics(env, course_id, date) {
                monthly_metrics.push_back(metrics);
            }
        }

        Ok(monthly_metrics)
    }
}
