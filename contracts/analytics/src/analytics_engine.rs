use crate::{
    errors::AnalyticsError,
    events::AnalyticsEvents,
    storage::AnalyticsStorage,
    types::{
        Achievement, AchievementType, AggregatedMetrics, AnalyticsConfig, CourseAnalytics,
        DifficultyRating, LeaderboardEntry, LeaderboardMetric, LearningSession, ModuleAnalytics,
        PerformanceTrend, ProgressAnalytics, SessionType,
    },
};
use soroban_sdk::{Address, BytesN, Env, IntoVal, String, Symbol, Vec};

/// Core analytics calculation engine
pub struct AnalyticsEngine;

impl AnalyticsEngine {
    /// Calculate comprehensive progress analytics for a student
    pub fn calculate_progress_analytics(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
    ) -> Result<ProgressAnalytics, AnalyticsError> {
        let sessions = AnalyticsStorage::get_student_sessions(env, student, course_id);

        if sessions.is_empty() {
            return Err(AnalyticsError::InsufficientData);
        }

        let mut total_time_spent = 0u64;
        let mut total_sessions = 0u32;
        let mut completed_modules = 0u32;
        let mut total_modules = 0u32;
        let mut scores: Vec<u32> = Vec::new(env);
        let mut first_activity = u64::MAX;
        let mut last_activity = 0u64;
        let mut module_completions: Vec<Symbol> = Vec::new(env);

        // Process all sessions
        for i in 0..sessions.len() {
            let session_id = sessions.get(i).unwrap();
            if let Some(session) = AnalyticsStorage::get_session(env, &session_id) {
                total_time_spent += session.time_spent;
                total_sessions += 1;

                if session.start_time < first_activity {
                    first_activity = session.start_time;
                }
                if session.end_time > last_activity {
                    last_activity = session.end_time;
                }

                // Track module completions
                if session.completion_percentage == 100 {
                    let mut already_completed = false;
                    for j in 0..module_completions.len() {
                        if module_completions.get(j).unwrap() == session.module_id {
                            already_completed = true;
                            break;
                        }
                    }
                    if !already_completed {
                        module_completions.push_back(session.module_id.clone());
                        completed_modules += 1;
                    }
                }

                // Collect scores
                if let Some(score) = session.score {
                    scores.push_back(score);
                }
            }
        }

        // Calculate average session time
        let average_session_time = if total_sessions > 0 {
            total_time_spent / total_sessions as u64
        } else {
            0
        };

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

        // Estimate total modules (this would ideally come from course metadata)
        total_modules = Self::estimate_total_modules(env, course_id, &module_completions);

        // Calculate completion percentage
        let completion_percentage = if total_modules > 0 {
            (completed_modules * 100) / total_modules
        } else {
            0
        };

        // Calculate streak days
        let streak_days = Self::calculate_streak_days(env, student, course_id);

        // Determine performance trend
        let performance_trend = Self::calculate_performance_trend(env, student, course_id, &scores);

        let analytics = ProgressAnalytics {
            student: student.clone(),
            course_id: course_id.clone(),
            total_modules,
            completed_modules,
            completion_percentage,
            total_time_spent,
            average_session_time,
            total_sessions,
            last_activity,
            first_activity: if first_activity == u64::MAX {
                0
            } else {
                first_activity
            },
            average_score,
            streak_days,
            performance_trend: performance_trend.clone(),
        };

        // Store updated analytics
        AnalyticsStorage::set_progress_analytics(env, student, course_id, &analytics);

        // Emit event
        AnalyticsEvents::emit_progress_updated(
            env,
            student,
            course_id,
            completion_percentage,
            total_time_spent,
            performance_trend,
        );

        Ok(analytics)
    }

    /// Calculate course-wide analytics
    pub fn calculate_course_analytics(
        env: &Env,
        course_id: &Symbol,
    ) -> Result<CourseAnalytics, AnalyticsError> {
        let students = AnalyticsStorage::get_course_students(env, course_id);

        if students.is_empty() {
            return Err(AnalyticsError::InsufficientData);
        }

        let mut total_students = students.len();
        let mut active_students = 0u32;
        let mut completed_students = 0u32;
        let mut total_completion_time = 0u64;
        let mut completion_times: Vec<u64> = Vec::new(env);
        let mut all_scores: Vec<u32> = Vec::new(env);
        let mut total_time_invested = 0u64;
        let mut module_difficulty_data: Vec<(Symbol, u32, u64)> = Vec::new(env); // (module_id, attempts, total_time)

        let current_time = env.ledger().timestamp();
        let active_threshold = 30 * 24 * 3600; // 30 days in seconds

        // Process each student
        for i in 0..students.len() {
            let student = students.get(i).unwrap();

            if let Some(analytics) =
                AnalyticsStorage::get_progress_analytics(env, &student, course_id)
            {
                total_time_invested += analytics.total_time_spent;

                // Check if student is active
                if current_time - analytics.last_activity <= active_threshold {
                    active_students += 1;
                }

                // Check if student completed the course
                if analytics.completion_percentage == 100 {
                    completed_students += 1;
                    let completion_time = analytics.last_activity - analytics.first_activity;
                    completion_times.push_back(completion_time);
                    total_completion_time += completion_time;
                }

                // Collect scores
                if let Some(score) = analytics.average_score {
                    all_scores.push_back(score);
                }
            }
        }

        // Calculate completion rate
        let completion_rate = if total_students > 0 {
            (completed_students * 100) / total_students
        } else {
            0
        };

        // Calculate dropout rate
        let dropout_rate = 100 - completion_rate;

        // Calculate average completion time
        let average_completion_time = if !completion_times.is_empty() {
            total_completion_time / completion_times.len() as u64
        } else {
            0
        };

        // Calculate average score
        let average_score = if !all_scores.is_empty() {
            let mut sum = 0u32;
            for i in 0..all_scores.len() {
                sum += all_scores.get(i).unwrap();
            }
            Some(sum / all_scores.len())
        } else {
            None
        };

        // Determine most difficult and easiest modules (placeholder logic)
        let (most_difficult_module, easiest_module) =
            Self::analyze_module_difficulty(env, course_id);

        let analytics = CourseAnalytics {
            course_id: course_id.clone(),
            total_students,
            active_students,
            completion_rate,
            average_completion_time,
            average_score,
            dropout_rate,
            most_difficult_module,
            easiest_module,
            total_time_invested,
        };

        // Store analytics
        AnalyticsStorage::set_course_analytics(env, course_id, &analytics);

        // Emit event
        AnalyticsEvents::emit_course_analytics_updated(
            env,
            course_id,
            total_students,
            completion_rate,
            average_score,
        );

        Ok(analytics)
    }

    /// Calculate module-specific analytics
    pub fn calculate_module_analytics(
        env: &Env,
        course_id: &Symbol,
        module_id: &Symbol,
    ) -> Result<ModuleAnalytics, AnalyticsError> {
        let students = AnalyticsStorage::get_course_students(env, course_id);

        if students.is_empty() {
            return Err(AnalyticsError::InsufficientData);
        }

        let mut total_attempts = 0u32;
        let mut completions = 0u32;
        let mut total_time = 0u64;
        let mut completion_times: Vec<u64> = Vec::new(env);
        let mut scores: Vec<u32> = Vec::new(env);

        // Analyze sessions for this specific module
        for i in 0..students.len() {
            let student = students.get(i).unwrap();
            let sessions = AnalyticsStorage::get_student_sessions(env, &student, course_id);

            for j in 0..sessions.len() {
                let session_id = sessions.get(j).unwrap();
                if let Some(session) = AnalyticsStorage::get_session(env, &session_id) {
                    if session.module_id == *module_id {
                        total_attempts += 1;
                        total_time += session.time_spent;

                        if session.completion_percentage == 100 {
                            completions += 1;
                            completion_times.push_back(session.time_spent);
                        }

                        if let Some(score) = session.score {
                            scores.push_back(score);
                        }
                    }
                }
            }
        }

        // Calculate completion rate
        let completion_rate = if total_attempts > 0 {
            (completions * 100) / total_attempts
        } else {
            0
        };

        // Calculate average time to complete
        let average_time_to_complete = if !completion_times.is_empty() {
            let mut sum = 0u64;
            for i in 0..completion_times.len() {
                sum += completion_times.get(i).unwrap();
            }
            sum / completion_times.len() as u64
        } else {
            0
        };

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

        // Determine difficulty rating
        let difficulty_rating =
            Self::calculate_difficulty_rating(env, completion_rate, average_time_to_complete);

        let analytics = ModuleAnalytics {
            course_id: course_id.clone(),
            module_id: module_id.clone(),
            total_attempts,
            completion_rate,
            average_time_to_complete,
            average_score,
            difficulty_rating: difficulty_rating.clone(),
            student_feedback_score: None, // Would be populated from feedback system
        };

        // Store analytics
        AnalyticsStorage::set_module_analytics(env, course_id, module_id, &analytics);

        // Emit event
        let difficulty_str = match difficulty_rating {
            DifficultyRating::Easy => "Easy",
            DifficultyRating::Medium => "Medium",
            DifficultyRating::Hard => "Hard",
            DifficultyRating::VeryHard => "VeryHard",
        };

        AnalyticsEvents::emit_module_analytics_updated(
            env,
            course_id,
            module_id,
            completion_rate,
            average_time_to_complete,
            difficulty_str,
        );

        Ok(analytics)
    }

    /// Calculate student's learning streak
    fn calculate_streak_days(env: &Env, student: &Address, course_id: &Symbol) -> u32 {
        let sessions = AnalyticsStorage::get_student_sessions(env, student, course_id);

        if sessions.is_empty() {
            return 0;
        }

        let config =
            AnalyticsStorage::get_config(env).unwrap_or(AnalyticsStorage::get_default_config(env));

        let mut activity_days: Vec<u64> = Vec::new(env);

        // Collect unique activity days
        for i in 0..sessions.len() {
            let session_id = sessions.get(i).unwrap();
            if let Some(session) = AnalyticsStorage::get_session(env, &session_id) {
                let day = session.start_time / 86400; // Convert to day number

                // Check if this day is already recorded
                let mut day_exists = false;
                for j in 0..activity_days.len() {
                    if activity_days.get(j).unwrap() == day {
                        day_exists = true;
                        break;
                    }
                }

                if !day_exists {
                    activity_days.push_back(day);
                }
            }
        }

        // Sort activity days (simple bubble sort for small datasets)
        for i in 0..activity_days.len() {
            for j in 0..activity_days.len() - 1 {
                if activity_days.get(j).unwrap() > activity_days.get(j + 1).unwrap() {
                    let temp = activity_days.get(j).unwrap();
                    activity_days.set(j, activity_days.get(j + 1).unwrap());
                    activity_days.set(j + 1, temp);
                }
            }
        }

        // Calculate current streak from the end
        let current_day = env.ledger().timestamp() / 86400;
        let mut streak = 0u32;

        for i in (0..activity_days.len()).rev() {
            let day = activity_days.get(i).unwrap();
            let expected_day = current_day - streak as u64;

            if day == expected_day || (streak == 0 && current_day - day <= 1) {
                streak += 1;
            } else {
                break;
            }
        }

        streak
    }

    /// Calculate performance trend based on recent scores
    fn calculate_performance_trend(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        scores: &Vec<u32>,
    ) -> PerformanceTrend {
        if scores.len() < 3 {
            return PerformanceTrend::Insufficient;
        }

        let len = scores.len();
        let recent_count = if len >= 5 { 3 } else { len / 2 };

        // Calculate average of recent scores vs earlier scores
        let mut recent_sum = 0u32;
        let mut earlier_sum = 0u32;
        let mut earlier_count = 0u32;

        // Recent scores
        for i in (len - recent_count)..len {
            recent_sum += scores.get(i).unwrap();
        }
        let recent_avg = recent_sum / recent_count;

        // Earlier scores
        for i in 0..(len - recent_count) {
            earlier_sum += scores.get(i).unwrap();
            earlier_count += 1;
        }

        if earlier_count == 0 {
            return PerformanceTrend::Insufficient;
        }

        let earlier_avg = earlier_sum / earlier_count;

        // Determine trend
        if recent_avg > earlier_avg + 5 {
            PerformanceTrend::Improving
        } else if recent_avg + 5 < earlier_avg {
            PerformanceTrend::Declining
        } else {
            PerformanceTrend::Stable
        }
    }

    /// Estimate total modules in a course
    fn estimate_total_modules(
        env: &Env,
        course_id: &Symbol,
        completed_modules: &Vec<Symbol>,
    ) -> u32 {
        // This is a simplified estimation - in a real system, this would come from course metadata
        let mut max_module_num = 0u32;

        for i in 0..completed_modules.len() {
            // Try to extract module number from symbol (assuming format like "module_1", "module_2", etc.)
            // This is a placeholder - real implementation would have proper course structure
            max_module_num += 1;
        }

        // Assume at least 5 modules per course, or use the maximum seen + buffer
        if max_module_num < 5 {
            5
        } else {
            max_module_num + 2 // Add buffer for incomplete modules
        }
    }

    /// Analyze module difficulty across the course
    fn analyze_module_difficulty(
        env: &Env,
        course_id: &Symbol,
    ) -> (Option<Symbol>, Option<Symbol>) {
        // Placeholder implementation - would analyze all modules in the course
        // and return the most difficult and easiest based on completion rates and time
        (None, None)
    }

    /// Calculate difficulty rating for a module
    fn calculate_difficulty_rating(
        env: &Env,
        completion_rate: u32,
        avg_time: u64,
    ) -> DifficultyRating {
        let config =
            AnalyticsStorage::get_config(env).unwrap_or(AnalyticsStorage::get_default_config(env));

        if completion_rate >= config.difficulty_thresholds.easy_completion_rate {
            DifficultyRating::Easy
        } else if completion_rate >= config.difficulty_thresholds.medium_completion_rate {
            DifficultyRating::Medium
        } else if completion_rate >= config.difficulty_thresholds.hard_completion_rate {
            DifficultyRating::Hard
        } else {
            DifficultyRating::VeryHard
        }
    }

    /// Check and award achievements
    pub fn check_achievements(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        session: &LearningSession,
    ) -> Result<Vec<Achievement>, AnalyticsError> {
        let mut new_achievements: Vec<Achievement> = Vec::new(env);
        let current_time = env.ledger().timestamp();

        // Get current analytics
        if let Some(analytics) = AnalyticsStorage::get_progress_analytics(env, student, course_id) {
            // Check for completion achievements
            if session.completion_percentage == 100 {
                let achievement = Achievement {
                    achievement_id: Symbol::new(env, "module_complete"),
                    title: String::from_str(env, "Module Completed"),
                    description: String::from_str(env, "Successfully completed a learning module"),
                    earned_date: current_time,
                    achievement_type: AchievementType::Completion,
                };
                new_achievements.push_back(achievement);
            }

            // Check for streak achievements
            if analytics.streak_days >= 7 && analytics.streak_days % 7 == 0 {
                let achievement = Achievement {
                    achievement_id: Symbol::new(env, "week_streak"),
                    title: String::from_str(env, "Weekly Streak"),
                    description: String::from_str(
                        env,
                        "Maintained learning activity for a full week",
                    ),
                    earned_date: current_time,
                    achievement_type: AchievementType::Streak,
                };
                new_achievements.push_back(achievement);
            }

            // Check for excellence achievements
            if let Some(score) = session.score {
                if score >= 95 {
                    let achievement = Achievement {
                        achievement_id: Symbol::new(env, "excellence"),
                        title: "Excellence".into_val(env),
                        description: "Achieved exceptional score in assessment".into_val(env),
                        earned_date: current_time,
                        achievement_type: AchievementType::Excellence,
                    };
                    new_achievements.push_back(achievement);
                }
            }
        }

        // Store new achievements
        for i in 0..new_achievements.len() {
            let achievement = new_achievements.get(i).unwrap();
            AnalyticsStorage::add_student_achievement(env, student, &achievement);

            // Emit achievement event
            AnalyticsEvents::emit_achievement_earned(
                env,
                student,
                &achievement.achievement_id,
                achievement.achievement_type,
                course_id,
                achievement.earned_date,
            );
        }

        Ok(new_achievements)
    }
}
