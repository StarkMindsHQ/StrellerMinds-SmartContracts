#[cfg(test)]
extern crate std;

mod integration_tests {
    use super::*;
    use crate::{
        errors::AnalyticsError,
        types::{
            AchievementType, AnalyticsConfig, AnalyticsFilter, DifficultyThresholds,
            LeaderboardMetric, LearningSession, OptionalSessionType, PerformanceTrend,
            ReportPeriod, SessionType,
        },
        Analytics, AnalyticsClient,
    };
    use soroban_sdk::{
        testutils::Address as _,
        Address, BytesN, Env, Symbol, Vec,
    };
    use std::format;

    /// Minimal setup for integration tests: Fresh Env, admin, and students
    fn setup_integration_env() -> (Env, Address, Vec<Address>) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let mut students = Vec::new(&env);
        for _ in 0..5 {
            students.push_back(Address::generate(&env));
        }
        (env, admin, students)
    }

    /// Helper to create and initialize contract
    fn init_contract<'a>(env: &'a Env, admin: &Address) -> AnalyticsClient<'a> {
        let contract_id = env.register(Analytics, ());
        let client = AnalyticsClient::new(env, &contract_id);
        let config = AnalyticsConfig {
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
        };
        client.initialize(admin, &config);
        client
    }

    /// Helper to create learning session (does not modify contract state)
    fn create_learning_session(
        env: &Env,
        student: &Address,
        course_id: &str,
        module_id: &str,
        session_num: u8,
        start_offset: u64,
    ) -> LearningSession {
        let session_id = BytesN::from_array(env, &[session_num; 32]);
        let base_time = env.ledger().timestamp();

        LearningSession {
            session_id,
            student: student.clone(),
            course_id: Symbol::new(env, course_id),
            module_id: Symbol::new(env, module_id),
            start_time: base_time + start_offset,
            end_time: 0,
            completion_percentage: 0,
            time_spent: 0,
            interactions: 10 + (session_num as u32 * 2),
            score: None,
            session_type: SessionType::Study,
        }
    }

    #[test]
    fn test_complete_learning_journey() {
        // Fresh setup per test - new Env, new contract, new students
        let (env, admin, students) = setup_integration_env();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = students.get(0).unwrap();
        let course_id = Symbol::new(&env, "BLOCKCHAIN_FUNDAMENTALS");

        // Simulate a complete learning journey over multiple days
        let mut session_counter = 0u8;

        // Week 1: Student starts strong
        for day in 0..7 {
            for module in 1..=2 {
                let session = create_learning_session(
                    &env,
                    &student,
                    "BLOCKCHAIN_FUNDAMENTALS",
                    &format!("module_{}", module),
                    session_counter,
                    day * 86400 + (module - 1) * 3600, // Spread sessions throughout day
                );
                session_counter += 1;

                client.record_session(&session);

                // Complete session with good performance
                let end_time = session.start_time + 1800; // 30 minutes
                let score = 85 + (day % 3) * 3; // Varying scores 85-93
                client.complete_session(&session.session_id, &end_time, &Some(score as u32), &100);
            }
        }

        // Week 2: Student struggles a bit
        for day in 7..14 {
            let session = create_learning_session(
                &env,
                &student,
                "BLOCKCHAIN_FUNDAMENTALS",
                "module_3",
                session_counter,
                day * 86400,
            );
            session_counter += 1;

            client.record_session(&session);

            // Complete session with declining performance
            let end_time = session.start_time + 2400; // 40 minutes (taking longer)
            let score = 75 - (day - 7) * 2; // Declining scores 75-61
            let completion = if day < 12 { 100 } else { 80 }; // Some incomplete sessions
            client.complete_session(
                &session.session_id,
                &end_time,
                &Some(score as u32),
                &completion,
            );
        }

        // Week 3: Student recovers
        for day in 14..21 {
            let session = create_learning_session(
                &env,
                &student,
                "BLOCKCHAIN_FUNDAMENTALS",
                "module_4",
                session_counter,
                day * 86400,
            );
            session_counter += 1;

            client.record_session(&session);

            // Complete session with improving performance
            let end_time = session.start_time + 1500; // 25 minutes (getting faster)
            let score = 80 + (day - 14) * 2; // Improving scores 80-92
            client.complete_session(&session.session_id, &end_time, &Some(score as u32), &100);
        }

        // Analyze the complete journey
        let progress_analytics = client.get_progress_analytics(&student, &course_id);

        // Verify comprehensive analytics
        assert_eq!(progress_analytics.student, student);
        assert_eq!(progress_analytics.course_id, course_id);
        assert!(progress_analytics.total_sessions > 20);
        assert!(progress_analytics.total_time_spent > 0);
        assert!(progress_analytics.average_session_time > 0);
        assert!(progress_analytics.streak_days > 0);
        assert!(progress_analytics.average_score.is_some());

        // Performance should show improvement trend due to recovery in week 3
        assert!(matches!(
            progress_analytics.performance_trend,
            PerformanceTrend::Improving | PerformanceTrend::Stable
        ));

        // Generate comprehensive report for the entire period
        let start_date = env.ledger().timestamp();
        let end_date = start_date + (21 * 86400);

        let report = client.generate_progress_report(
            &student,
            &course_id,
            &ReportPeriod::Custom,
            &start_date,
            &end_date,
        );

        assert_eq!(report.student, student);
        assert!(report.sessions_count > 20);
        assert!(report.total_time > 0);
        assert!(report.consistency_score > 0);
    }

    #[test]
    fn test_multi_student_course_analytics() {
        // Fresh setup per test - new Env, new contract, new students
        let (env, admin, students) = setup_integration_env();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let course_id = Symbol::new(&env, "DATA_STRUCTURES");

        // Create diverse student performance patterns
        let performance_patterns = [
            (90, 100, 5), // High performer: score 90, completion 100%, 5 modules
            (75, 80, 3),  // Average performer: score 75, completion 80%, 3 modules
            (60, 60, 2),  // Struggling student: score 60, completion 60%, 2 modules
            (85, 95, 4),  // Good performer: score 85, completion 95%, 4 modules
            (70, 70, 2),  // Below average: score 70, completion 70%, 2 modules
        ];

        for (i, (base_score, completion_rate, modules)) in performance_patterns.iter().enumerate() {
            let student = students.get(i as u32).unwrap();
            let mut session_counter = (i * 10) as u8;

            for module in 1..=*modules {
                let session = create_learning_session(
                    &env,
                    &student,
                    "DATA_STRUCTURES",
                    &format!("module_{}", module),
                    session_counter,
                    (i as u64 * 3600) + (module as u64 * 1800),
                );
                session_counter += 1;

                client.record_session(&session);

                let end_time = session.start_time + 1800;
                let score = base_score + (module as u32 % 3) * 2;
                client.complete_session(
                    &session.session_id,
                    &end_time,
                    &Some(score),
                    completion_rate,
                );
            }
        }

        // Analyze course-wide performance
        let course_analytics = client.get_course_analytics(&course_id);

        assert_eq!(course_analytics.course_id, course_id);
        assert_eq!(course_analytics.total_students, 5);
        assert!(course_analytics.completion_rate > 0);
        assert!(course_analytics.average_score.is_some());
        assert!(course_analytics.total_time_invested > 0);

        // Test leaderboard generation
        let leaderboard =
            client.generate_leaderboard(&course_id, &LeaderboardMetric::TotalScore, &10);

        assert!(leaderboard.len() <= 5);
        assert!(leaderboard.len() > 0);

        // Verify leaderboard is sorted correctly
        for i in 1..leaderboard.len() {
            let current = leaderboard.get(i).unwrap();
            let previous = leaderboard.get(i - 1).unwrap();
            assert!(current.score <= previous.score);
        }

        // Test struggling students identification
        let struggling = client.get_struggling_students(&course_id, &75);
        assert!(struggling.len() > 0); // Should identify students with < 75% performance
    }

    #[test]
    fn test_time_based_analytics_and_trends() {
        // Fresh setup per test - new Env, new contract, new students
        let (env, admin, students) = setup_integration_env();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = students.get(0).unwrap();
        let course_id = Symbol::new(&env, "MACHINE_LEARNING");

        let base_time = env.ledger().timestamp();
        let mut session_counter = 0u8;

        // Generate 30 days of learning data with varying patterns
        for day in 0..30 {
            // Simulate different activity levels throughout the month
            let sessions_per_day = match day {
                0..=7 => 3,   // Week 1: High activity
                8..=14 => 1,  // Week 2: Low activity
                15..=21 => 2, // Week 3: Medium activity
                _ => 4,       // Week 4: Very high activity
            };

            for session_num in 0..sessions_per_day {
                let session = create_learning_session(
                    &env,
                    &student,
                    "MACHINE_LEARNING",
                    &format!("module_{}", (day % 5) + 1),
                    session_counter,
                    day * 86400 + session_num * 3600,
                );
                session_counter += 1;

                client.record_session(&session);

                // Simulate performance improvement over time
                let end_time = session.start_time + 1800;
                let base_score = 60 + (day * 2) / 3; // Gradual improvement
                let score = base_score + (session_num * 5);
                client.complete_session(&session.session_id, &end_time, &Some(score as u32), &100);
            }

            // Generate daily metrics
            let daily_date = base_time + (day * 86400);
            let _ = client.generate_daily_metrics(&course_id, &daily_date);
        }

        // Test weekly summary generation
        let week_start = base_time;
        let weekly_summary = client.generate_weekly_summary(&course_id, &week_start);
        assert_eq!(weekly_summary.len(), 7);

        // Verify weekly summary shows activity patterns
        let first_day_metrics = weekly_summary.get(0).unwrap();
        assert!(first_day_metrics.total_sessions > 0);
        assert!(first_day_metrics.active_students > 0);

        // Test monthly summary generation
        let monthly_summary = client.generate_monthly_summary(&course_id, &base_time, &30);
        assert_eq!(monthly_summary.len(), 30);

        // Test completion trends
        let trends =
            client.get_completion_trends(&course_id, &base_time, &(base_time + 30 * 86400));
        assert!(trends.len() > 0);

        // Verify trend data shows progression
        let early_metrics = trends.get(0);
        let late_metrics = trends.get(trends.len() - 1);

        if early_metrics.is_some() && late_metrics.is_some() {
            // Later periods should show more completions due to improved performance
            assert!(late_metrics.unwrap().completions >= early_metrics.unwrap().completions);
        }
    }

    #[test]
    fn test_achievement_system_integration() {
        // Fresh setup per test - new Env, new contract, new students
        let (env, admin, students) = setup_integration_env();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = students.get(0).unwrap();
        let _course_id = Symbol::new(&env, "WEB_DEVELOPMENT");

        let mut session_counter = 0u8;

        // Create sessions that should trigger various achievements

        // 1. Excellence achievement (high score)
        let session = create_learning_session(
            &env,
            &student,
            "WEB_DEVELOPMENT",
            "module_1",
            session_counter,
            0,
        );
        session_counter += 1;

        client.record_session(&session);
        let end_time = session.start_time + 1800;
        client.complete_session(&session.session_id, &end_time, &Some(98), &100); // High score

        // 2. Multiple completions for streak
        for day in 1..=7 {
            let session = create_learning_session(
                &env,
                &student,
                "WEB_DEVELOPMENT",
                &format!("module_{}", day % 3 + 1),
                session_counter,
                day * 86400,
            );
            session_counter += 1;

            client.record_session(&session);
            let end_time = session.start_time + 1800;
            client.complete_session(&session.session_id, &end_time, &Some(85), &100);
        }

        // Check achievements were awarded
        let achievements = client.get_student_achievements(&student);
        assert!(achievements.len() > 0);

        // Verify achievement types
        let mut has_excellence = false;
        let mut _has_streak = false;
        let mut has_completion = false;

        for i in 0..achievements.len() {
            let achievement = achievements.get(i).unwrap();
            match achievement.achievement_type {
                AchievementType::Excellence => has_excellence = true,
                AchievementType::Streak => _has_streak = true,
                AchievementType::Completion => has_completion = true,
                _ => {}
            }
        }

        assert!(has_excellence || has_completion); // Should have at least one type
    }

    #[test]
    fn test_filtered_analytics_queries() {
        // Fresh setup per test - new Env, new contract, new students
        let (env, admin, students) = setup_integration_env();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = students.get(0).unwrap();
        let course_id = Symbol::new(&env, "CYBERSECURITY");

        let base_time = env.ledger().timestamp();
        let mut session_counter = 0u8;

        // Create sessions with different types and scores
        let session_types = [
            SessionType::Study,
            SessionType::Assessment,
            SessionType::Practice,
        ];
        let scores = [70, 85, 95];

        for (i, (session_type, score)) in session_types.iter().zip(scores.iter()).enumerate() {
            let mut session = create_learning_session(
                &env,
                &student,
                "CYBERSECURITY",
                &format!("module_{}", i + 1),
                session_counter,
                i as u64 * 3600,
            );
            session.session_type = session_type.clone();
            session_counter += 1;

            client.record_session(&session);
            let end_time = session.start_time + 1800;
            client.complete_session(&session.session_id, &end_time, &Some(*score), &100);
        }

        // Test filtering by session type
        let study_filter = AnalyticsFilter {
            course_id: Some(course_id.clone()),
            student: Some(student.clone()),
            start_date: None,
            end_date: None,
            session_type: OptionalSessionType::Some(SessionType::Study),
            min_score: None,
        };

        let study_sessions = client.get_filtered_sessions(&study_filter);
        assert_eq!(study_sessions.len(), 1);
        assert_eq!(
            study_sessions.get(0).unwrap().session_type,
            SessionType::Study
        );

        // Test filtering by minimum score
        let high_score_filter = AnalyticsFilter {
            course_id: Some(course_id.clone()),
            student: Some(student.clone()),
            start_date: None,
            end_date: None,
            session_type: OptionalSessionType::None,
            min_score: Some(90),
        };

        let high_score_sessions = client.get_filtered_sessions(&high_score_filter);
        assert_eq!(high_score_sessions.len(), 1);
        assert!(high_score_sessions.get(0).unwrap().score.unwrap() >= 90);

        // Test filtering by date range
        let date_filter = AnalyticsFilter {
            course_id: Some(course_id.clone()),
            student: Some(student.clone()),
            start_date: Some(base_time),
            end_date: Some(base_time + 7200), // First 2 hours
            session_type: OptionalSessionType::None,
            min_score: None,
        };

        let date_filtered_sessions = client.get_filtered_sessions(&date_filter);
        assert!(date_filtered_sessions.len() <= 2); // Should filter out later sessions
    }

    #[test]
    fn test_performance_comparison_and_insights() {
        // Fresh setup per test - new Env, new contract, new students
        let (env, admin, students) = setup_integration_env();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student1 = students.get(0).unwrap();
        let student2 = students.get(1).unwrap();
        let course_id = Symbol::new(&env, "ALGORITHMS");

        // Create contrasting performance patterns

        // Student 1: Consistent high performer
        for i in 0..5 {
            let session = create_learning_session(
                &env,
                &student1,
                "ALGORITHMS",
                &format!("module_{}", i + 1),
                i as u8,
                i as u64 * 3600,
            );

            client.record_session(&session);
            let end_time = session.start_time + 1500; // Fast completion
            client.complete_session(&session.session_id, &end_time, &Some(90 + i), &100);
        }

        // Student 2: Slower but improving
        for i in 0..5 {
            let session = create_learning_session(
                &env,
                &student2,
                "ALGORITHMS",
                &format!("module_{}", i + 1),
                (i + 10) as u8,
                i as u64 * 3600,
            );

            client.record_session(&session);
            let end_time = session.start_time + 2400; // Slower completion
            let score = 70 + (i * 4); // Improving scores
            client.complete_session(&session.session_id, &end_time, &Some(score), &100);
        }

        // Compare student performance
        let (analytics1, analytics2) =
            client.compare_student_performance(&student1, &student2, &course_id);

        // Verify comparison results
        assert_eq!(analytics1.student, student1);
        assert_eq!(analytics2.student, student2);

        // Student 1 should have better average score
        assert!(analytics1.average_score.unwrap() > analytics2.average_score.unwrap());

        // Student 1 should have faster average session time
        assert!(analytics1.average_session_time < analytics2.average_session_time);

        // Both should show positive trends (improving or stable)
        assert!(matches!(
            analytics1.performance_trend,
            PerformanceTrend::Improving | PerformanceTrend::Stable
        ));

        // Test top performers identification
        let top_performers =
            client.get_top_performers(&course_id, &LeaderboardMetric::TotalScore, &3);

        assert!(top_performers.len() <= 3);
        if !top_performers.is_empty() {
            // Student 1 should be ranked higher than Student 2
            let student1_rank = top_performers
                .iter()
                .position(|entry| entry.student == student1);
            let student2_rank = top_performers
                .iter()
                .position(|entry| entry.student == student2);

            if student1_rank.is_some() && student2_rank.is_some() {
                assert!(student1_rank.unwrap() < student2_rank.unwrap());
            }
        }
    }

    #[test]
    fn test_admin_operations_and_maintenance() {
        // Fresh setup per test - new Env, new contract, new students
        let (env, admin, students) = setup_integration_env();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let new_admin = Address::generate(&env);
        let _course_id = Symbol::new(&env, "DATABASE_SYSTEMS");

        // Create some test data
        let student = students.get(0).unwrap();
        let session =
            create_learning_session(&env, &student, "DATABASE_SYSTEMS", "module_1", 1, 0);

        client.record_session(&session);
        let end_time = session.start_time + 1800;
        client.complete_session(&session.session_id, &end_time, &Some(85), &100);

        // Test admin configuration update
        let new_config = AnalyticsConfig {
            min_session_time: 120,
            max_session_time: 7200,
            streak_threshold: 43200,
            active_threshold: 1296000,
            difficulty_thresholds: DifficultyThresholds {
                easy_completion_rate: 85,
                medium_completion_rate: 65,
                hard_completion_rate: 45,
            },
            oracle_address: None,
        };

        let _result = client.update_config(&admin, &new_config);
        // // assert!(_result.is_ok());

        // Test recalculation of analytics
        // let _result = client.recalculate_course_analytics(&admin, &_course_id);
        // // assert!(_result.is_ok());

        // Test admin transfer
        let _result = client.transfer_admin(&admin, &new_admin);
        // // assert!(_result.is_ok());

        // Verify new admin
        let current_admin = client.get_admin().unwrap();
        assert_eq!(current_admin, new_admin);

        // Test cleanup operation (should work with new admin)
        let old_date = env.ledger().timestamp() - 86400; // 1 day ago
        let _result = client.cleanup_old_data(&new_admin, &old_date);
        // assert!(_result);

        // Test unauthorized operations fail
        let unauthorized_user = Address::generate(&env);
        let result = client.try_update_config(&unauthorized_user, &new_config);
        assert_eq!(result, Err(Ok(AnalyticsError::Unauthorized)));
    }
}
