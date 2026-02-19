#[cfg(test)]
extern crate std;

mod analytics_tests {
    use super::*;
    use crate::{
        errors::AnalyticsError,
        types::{
            AnalyticsConfig, BatchSessionUpdate, DifficultyThresholds, InsightType,
            LeaderboardMetric, LearningSession, MLInsight, ReportPeriod, SessionType,
        },
        Analytics, AnalyticsClient,
    };
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, BytesN, Env, String, Symbol, Vec,
    };
    use std::format;
    use std::string::ToString;

    fn create_test_env() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.ledger().with_mut(|li| li.timestamp = 10000);
        let admin = Address::generate(&env);
        let instructor = Address::generate(&env);
        let student = Address::generate(&env);

        env.mock_all_auths();

        (env, admin, instructor, student)
    }

    fn setup_analytics_contract<'a>(env: &Env, admin: &Address) -> AnalyticsClient<'a> {
        let contract_id = env.register(Analytics, ());
        let client = AnalyticsClient::new(env, &contract_id);

        let config = AnalyticsConfig {
            min_session_time: 60,      // 1 minute
            max_session_time: 14400,   // 4 hours
            streak_threshold: 86400,   // 24 hours
            active_threshold: 2592000, // 30 days
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

    fn create_test_session(
        env: &Env,
        student: &Address,
        course_id: &str,
        module_id: &str,
    ) -> LearningSession {
        let session_id = BytesN::from_array(env, &[1u8; 32]);
        let current_time = env.ledger().timestamp();

        LearningSession {
            session_id,
            student: student.clone(),
            course_id: Symbol::new(env, course_id),
            module_id: Symbol::new(env, module_id),
            start_time: current_time,
            end_time: 0, // Will be set when completing session
            completion_percentage: 0,
            time_spent: 0,
            interactions: 0,
            score: None,
            session_type: SessionType::Study,
        }
    }

    #[test]
    fn test_initialize_analytics_contract() {
        let (env, admin, _, _) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        // Verify initialization
        let stored_admin = client.get_admin();
        assert_eq!(stored_admin, Some(admin));

        let config = client.get_config();
        assert!(config.is_some());
        assert_eq!(config.unwrap().min_session_time, 60);
    }

    #[test]
    fn test_initialize_already_initialized() {
        let (env, admin, _, _) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let config = AnalyticsConfig {
            min_session_time: 120,
            max_session_time: 7200,
            streak_threshold: 86400,
            active_threshold: 2592000,
            difficulty_thresholds: DifficultyThresholds {
                easy_completion_rate: 80,
                medium_completion_rate: 60,
                hard_completion_rate: 40,
            },
            oracle_address: None,
        };

        // Try to initialize again
        let result = client.try_initialize(&admin, &config);
        assert_eq!(result, Err(Ok(AnalyticsError::AlreadyInitialized)));
    }

    #[test]
    fn test_record_learning_session() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let session = create_test_session(&env, &student, "RUST101", "module_1");

        // Record session
        let result = client.try_record_session(&session);
        assert!(result.is_ok());

        // Verify session was stored
        let stored_session = client.get_session(&session.session_id);
        assert!(stored_session.is_some());
        assert_eq!(stored_session.unwrap().student, student);
    }

    #[test]
    fn test_record_duplicate_session() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let session = create_test_session(&env, &student, "RUST101", "module_1");

        // Record session first time
        client.record_session(&session);

        // Try to record same session again
        let result = client.try_record_session(&session);
        assert_eq!(result, Err(Ok(AnalyticsError::SessionAlreadyExists)));
    }

    #[test]
    fn test_complete_session() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);

        // Complete the session
        let end_time = env.ledger().timestamp() + 1800; // 30 minutes later
        let final_score = Some(85u32);
        let completion_percentage = 100u32;

        let result = client.try_complete_session(
            &session.session_id,
            &end_time,
            &final_score,
            &completion_percentage,
        );
        assert!(result.is_ok());

        // Verify session was updated
        let updated_session = client.get_session(&session.session_id).unwrap();
        assert_eq!(updated_session.end_time, end_time);
        assert_eq!(updated_session.score, final_score);
        assert_eq!(updated_session.completion_percentage, completion_percentage);
        assert_eq!(updated_session.time_spent, 1800);
    }

    #[test]
    fn test_complete_session_invalid_data() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);

        // Try to complete with invalid percentage
        let end_time = env.ledger().timestamp() + 1800;
        let result = client.try_complete_session(
            &session.session_id,
            &end_time,
            &Some(85u32),
            &150u32, // Invalid percentage > 100
        );
        assert_eq!(result, Err(Ok(AnalyticsError::InvalidPercentage)));

        // Try to complete with invalid score
        let result = client.try_complete_session(
            &session.session_id,
            &end_time,
            &Some(150u32), // Invalid score > 100
            &100u32,
        );
        assert_eq!(result, Err(Ok(AnalyticsError::InvalidScore)));

        // Try to complete with invalid time range
        let result = client.try_complete_session(
            &session.session_id,
            &(session.start_time - 100), // End time before start time
            &Some(85u32),
            &100u32,
        );
        assert_eq!(result, Err(Ok(AnalyticsError::InvalidTimeRange)));
    }

    #[test]
    fn test_session_too_short() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);

        // Try to complete with session too short (< 1 minute)
        let end_time = session.start_time + 30; // 30 seconds
        let result =
            client.try_complete_session(&session.session_id, &end_time, &Some(85u32), &100u32);
        assert_eq!(result, Err(Ok(AnalyticsError::SessionTooShort)));
    }

    #[test]
    fn test_session_too_long() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);

        // Try to complete with session too long (> 4 hours)
        let end_time = session.start_time + 15000; // > 4 hours
        let result =
            client.try_complete_session(&session.session_id, &end_time, &Some(85u32), &100u32);
        assert_eq!(result, Err(Ok(AnalyticsError::SessionTooLong)));
    }

    #[test]
    fn test_get_student_sessions() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let course_id = Symbol::new(&env, "RUST101");

        // Create and record multiple sessions
        for i in 0..3 {
            let mut session =
                create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);
        }

        // Get student sessions
        let sessions = client.get_student_sessions(&student, &course_id);
        assert_eq!(sessions.len(), 3);
    }

    #[test]
    fn test_progress_analytics_calculation() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let course_id = Symbol::new(&env, "RUST101");

        // Create and complete multiple sessions
        for i in 0..3 {
            let mut session =
                create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);

            // Complete each session
            let end_time = session.start_time + 1800; // 30 minutes
            client.complete_session(&session.session_id, &end_time, &Some(80 + i * 5), &100);
        }

        // Get progress analytics
        let result = client.try_get_progress_analytics(&student, &course_id);
        assert!(result.is_ok());

        let analytics = result.unwrap().unwrap();
        assert_eq!(analytics.student, student);
        assert_eq!(analytics.course_id, course_id);
        assert_eq!(analytics.completed_modules, 3);
        assert_eq!(analytics.total_sessions, 3);
        assert!(analytics.total_time_spent > 0);
        assert!(analytics.average_score.is_some());
    }

    // #[test]
    // fn test_course_analytics_calculation() {
    //     let (env, admin, _, student1) = create_test_env();
    //     let student2 = Address::generate(&env);
    //     let client = setup_analytics_contract(&env, &admin);

    //     let course_id = Symbol::new(&env, "RUST101");

    //     // Create sessions for multiple students
    //     for student in [&student1, &student2] {
    //         let prefix = String::from_str(&env, "module_");
    //         for i in 0..2 {
    //             // let mut module_name = prefix;
    //             // let num_str = String::from_str(&env, &i.to_string());
    //             let mut session = create_test_session(&env, student, "RUST101", &format!("module_{}", i + 1));
    //             session.session_id = BytesN::from_array(&env, &[(*student).to_string().as_bytes()[0] + i as u8; 32]);

    //             client.record_session(&session);

    //             let end_time = session.start_time + 1800;
    //             client.complete_session(&session.session_id, &end_time, &Some(85), &100);
    //         }
    //     }

    //     // Get course analytics
    //     let result = client.try_get_course_analytics(&course_id);
    //     assert!(result.is_ok());

    //     let analytics = result.unwrap();
    //     assert_eq!(analytics.course_id, course_id);
    //     assert_eq!(analytics.total_students, 2);
    //     assert!(analytics.total_time_invested > 0);
    // }

    #[test]
    fn test_module_analytics_calculation() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let course_id = Symbol::new(&env, "RUST101");
        let module_id = Symbol::new(&env, "module_1");

        // Create multiple attempts at the same module
        for i in 0..5 {
            let mut session = create_test_session(&env, &student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);

            let end_time = session.start_time + 1200; // 20 minutes
            let completion = if i < 4 { 100 } else { 75 }; // 4 out of 5 completed
            client.complete_session(&session.session_id, &end_time, &Some(80), &completion);
        }

        // Get module analytics
        let result = client.try_get_module_analytics(&course_id, &module_id);
        assert!(result.is_ok());

        let analytics = result.unwrap().unwrap();
        assert_eq!(analytics.course_id, course_id);
        assert_eq!(analytics.module_id, module_id);
        assert_eq!(analytics.total_attempts, 5);
        assert_eq!(analytics.completion_rate, 80); // 4 out of 5 = 80%
        assert!(analytics.average_time_to_complete > 0);
    }

    #[test]
    fn test_generate_progress_report() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let course_id = Symbol::new(&env, "RUST101");
        let start_time = env.ledger().timestamp();

        // Create sessions over time
        for i in 0..3 {
            let mut session =
                create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.start_time = start_time + (i as u64 * 86400); // One per day
            client.record_session(&session);

            let end_time = session.start_time + 1800;
            client.complete_session(&session.session_id, &end_time, &Some(85), &100);
        }

        // Generate weekly report
        let end_time = start_time + (7 * 86400);
        let result = client.try_generate_progress_report(
            &student,
            &course_id,
            &ReportPeriod::Weekly,
            &start_time,
            &end_time,
        );

        assert!(result.is_ok());
        let report = result.unwrap().unwrap();
        assert_eq!(report.student, student);
        assert_eq!(report.course_id, course_id);
        assert_eq!(report.sessions_count, 3);
        assert_eq!(report.modules_completed, 3);
    }

    #[test]
    fn test_generate_leaderboard() {
        let (env, admin, _, student1) = create_test_env();
        let student2 = Address::generate(&env);
        let student3 = Address::generate(&env);
        let client = setup_analytics_contract(&env, &admin);

        let course_id = Symbol::new(&env, "RUST101");

        // Create sessions with different scores for each student
        let students_scores = [(&student1, 95), (&student2, 85), (&student3, 75)];

        for (student, score) in students_scores {
            let mut session = create_test_session(&env, student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[score as u8; 32]);
            client.record_session(&session);

            let end_time = session.start_time + 1800;
            client.complete_session(&session.session_id, &end_time, &Some(score), &100);
        }

        // Generate leaderboard
        let result =
            client.try_generate_leaderboard(&course_id, &LeaderboardMetric::TotalScore, &10);
        assert!(result.is_ok());

        let leaderboard = result.unwrap().unwrap();
        assert_eq!(leaderboard.len(), 3);

        // Verify ordering (highest score first)
        let top_entry = leaderboard.get(0).unwrap();
        assert_eq!(top_entry.student, student1);
        assert_eq!(top_entry.rank, 1);
        assert_eq!(top_entry.score, 95);
    }

    #[test]
    fn test_update_config() {
        let (env, admin, _, _) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let new_config = AnalyticsConfig {
            min_session_time: 120,     // 2 minutes
            max_session_time: 7200,    // 2 hours
            streak_threshold: 43200,   // 12 hours
            active_threshold: 1296000, // 15 days
            difficulty_thresholds: DifficultyThresholds {
                easy_completion_rate: 85,
                medium_completion_rate: 65,
                hard_completion_rate: 45,
            },
            oracle_address: None,
        };

        // Update configuration
        let result = client.try_update_config(&admin, &new_config);
        assert!(result.is_ok());

        // Verify configuration was updated
        let stored_config = client.get_config().unwrap();
        assert_eq!(stored_config.min_session_time, 120);
        assert_eq!(stored_config.max_session_time, 7200);
    }

    #[test]
    fn test_unauthorized_config_update() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

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

        // Try to update configuration as non-admin
        let result = client.try_update_config(&student, &new_config);
        assert_eq!(result, Err(Ok(AnalyticsError::Unauthorized)));
    }

    #[test]
    fn test_transfer_admin() {
        let (env, admin, _, new_admin) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        // Transfer admin role
        let result = client.try_transfer_admin(&admin, &new_admin);
        assert!(result.is_ok());

        // Verify new admin
        let stored_admin = client.get_admin().unwrap();
        assert_eq!(stored_admin, new_admin);
    }

    #[test]
    fn test_get_struggling_students() {
        let (env, admin, _, student1) = create_test_env();
        let student2 = Address::generate(&env);
        let client = setup_analytics_contract(&env, &admin);

        let course_id = Symbol::new(&env, "RUST101");

        // Create sessions - student1 performs well, student2 struggles
        // Student 1: Performs well (5 modules)
        for i in 0..5 {
            let mut session =
                create_test_session(&env, &student1, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);
            let end_time = session.start_time + 1800;
            client.complete_session(&session.session_id, &end_time, &Some(100), &100);
        }

        // Student 2: Struggles (1 module, low score)
        let mut session = create_test_session(&env, &student2, "RUST101", "module_1");
        session.session_id = BytesN::from_array(&env, &[10u8; 32]);
        client.record_session(&session);
        let end_time = session.start_time + 1800;
        client.complete_session(&session.session_id, &end_time, &Some(30), &30);

        // Get struggling students (threshold 50%)
        let struggling = client.get_struggling_students(&course_id, &50);

        // Should only include student2
        assert_eq!(struggling.len(), 1);
        assert_eq!(struggling.get(0).unwrap(), student2);
    }

    #[test]
    fn test_batch_session_update() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let mut sessions: Vec<LearningSession> = Vec::new(&env);

        // Create batch of sessions
        for i in 0..5 {
            let mut session =
                create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.end_time = session.start_time + 1800;
            session.time_spent = 1800;
            session.completion_percentage = 100;
            session.score = Some(80 + i * 2);
            sessions.push_back(session);
        }

        let batch = BatchSessionUpdate {
            sessions,
            update_analytics: true,
            update_leaderboards: false,
        };

        // Process batch
        let result = client.try_batch_update_sessions(&batch);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.unwrap(), 5);

        // Verify sessions were stored
        let student_sessions = client.get_student_sessions(&student, &Symbol::new(&env, "RUST101"));
        assert_eq!(student_sessions.len(), 5);
    }

    #[test]
    fn test_batch_size_limit() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let mut sessions: Vec<LearningSession> = Vec::new(&env);

        // Create batch exceeding limit (>50 sessions)
        for i in 0..60 {
            let mut session = create_test_session(&env, &student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            sessions.push_back(session);
        }

        let batch = BatchSessionUpdate {
            sessions,
            update_analytics: false,
            update_leaderboards: false,
        };

        // Should fail due to batch size limit
        let result = client.try_batch_update_sessions(&batch);
        assert_eq!(result, Err(Ok(AnalyticsError::InvalidBatchSize)));
    }

    #[test]
    fn test_request_ml_insight() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);

        let course_id = Symbol::new(&env, "RUST101");

        // Create sufficient data for pattern recognition (needs >= 5 sessions)
        for i in 0..5 {
            let mut session = create_test_session(&env, &student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);
            let end_time = session.start_time + 1800; // 30 minutes
            client.complete_session(&session.session_id, &end_time, &Some(85), &100);
        }

        // Request insight
        let result =
            client.try_request_ml_insight(&student, &course_id, &InsightType::PatternRecognition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_callback_ml_insight_authorized() {
        let (env, admin, _, student) = create_test_env();
        let oracle = Address::generate(&env);

        let contract_id = env.register(Analytics, ());
        let client = AnalyticsClient::new(&env, &contract_id);

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
            oracle_address: Some(oracle.clone()),
        };
        client.initialize(&admin, &config);

        let course_id = Symbol::new(&env, "RUST101");
        let insight = MLInsight {
            insight_id: BytesN::from_array(&env, &[1u8; 32]),
            student: student.clone(),
            course_id: course_id.clone(),
            insight_type: InsightType::PatternRecognition,
            data: String::from_str(&env, "Study intensive pattern"),
            confidence: 90,
            timestamp: env.ledger().timestamp(),
        };

        // Callback from authorized oracle
        let result = client.try_callback_ml_insight(&oracle, &insight);
        assert!(result.is_ok());

        // Verify stored insight
        let stored = client.get_ml_insight(&student, &course_id, &InsightType::PatternRecognition);
        assert!(stored.is_some());
        assert_eq!(stored.unwrap().confidence, 90);
    }

    #[test]
    fn test_predict_completion_rates() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Set up some progress data
        let session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);
        let end_time = session.start_time + 3600; // 1 hour
        client.complete_session(&session.session_id, &end_time, &Some(90), &100); // 100% complete

        // Request prediction
        client.request_ml_insight(&student, &course_id, &InsightType::CompletionPrediction);

        let insight =
            client.get_ml_insight(&student, &course_id, &InsightType::CompletionPrediction);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::CompletionPrediction);
        let data_str = insight.data.to_string();
        assert!(
            data_str.contains("Less than")
                || data_str.contains("Approximately")
                || data_str.contains("Over")
        );
    }

    #[test]
    fn test_generate_recommendations() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create and record a session to generate some data
        let session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);
        client.complete_session(
            &session.session_id,
            &(session.start_time + 1000),
            &Some(80),
            &100,
        );

        // Request recommendation
        client.request_ml_insight(&student, &course_id, &InsightType::Recommendation);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::Recommendation);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::Recommendation);
        let data = insight.data.to_string();
        assert!(
            data.contains("Review")
                || data.contains("Consider")
                || data.contains("Suggested")
                || data.contains("Focus")
        );
    }

    #[test]
    fn test_advanced_pattern_recognition() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create diverse learning sessions for pattern analysis
        let session_types = [
            SessionType::Study,
            SessionType::Practice,
            SessionType::Assessment,
            SessionType::Review,
        ];

        for i in 0..10 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i % 3 + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.session_type = session_types[i % 4].clone();
            session.interactions = 15 + i * 2;
            client.record_session(&session);

            let end_time = session.start_time + 1800 + (i as u64 * 300); // Varying session times
            let score = Some(75 + i * 2);
            client.complete_session(&session.session_id, &end_time, &score, &100);
        }

        // Request pattern recognition insight
        client.request_ml_insight(&student, &course_id, &InsightType::PatternRecognition);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::PatternRecognition);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::PatternRecognition);
        assert!(insight.confidence >= 60);
        assert!(insight.model_version >= 2);
    }

    #[test]
    fn test_engagement_prediction() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create sessions with varying engagement patterns
        for i in 0..8 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.interactions = 20 + i * 3;
            client.record_session(&session);

            let end_time = session.start_time + 2400; // 40 minutes
            let score = Some(70 + i * 3);
            client.complete_session(&session.session_id, &end_time, &score, &100);
        }

        // Request engagement prediction
        client.request_ml_insight(&student, &course_id, &InsightType::EngagementPrediction);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::EngagementPrediction);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::EngagementPrediction);
        assert!(insight.confidence >= 70);
    }

    #[test]
    fn test_knowledge_gap_analysis() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create sessions with varying performance to simulate knowledge gaps
        let modules_scores = [("module_1", 85), ("module_2", 45), ("module_3", 90), ("module_4", 55)];
        
        for (i, (module, score)) in modules_scores.iter().enumerate() {
            let mut session = create_test_session(&env, &student, "RUST101", module);
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);

            let end_time = session.start_time + 1800;
            let final_score = Some(*score);
            client.complete_session(&session.session_id, &end_time, &final_score, &100);
        }

        // Request knowledge gap analysis
        client.request_ml_insight(&student, &course_id, &InsightType::KnowledgeGapAnalysis);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::KnowledgeGapAnalysis);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::KnowledgeGapAnalysis);
        assert!(insight.confidence >= 75);
    }

    #[test]
    fn test_collaborative_learning_analysis() {
        let (env, admin, _, student1) = create_test_env();
        let student2 = Address::generate(&env);
        let student3 = Address::generate(&env);
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create sessions for multiple students with different performance levels
        let students_data = [
            (&student1, 95, "high_performer"),
            (&student2, 75, "average_performer"),
            (&student3, 55, "struggling_student"),
        ];

        for (student, score, _label) in students_data {
            for i in 0..3 {
                let mut session = create_test_session(&env, student, "RUST101", &format!("module_{}", i + 1));
                session.session_id = BytesN::from_array(&env, &[score as u8 + i as u8; 32]);
                client.record_session(&session);

                let end_time = session.start_time + 1800;
                let final_score = Some(score);
                client.complete_session(&session.session_id, &end_time, &final_score, &100);
            }
        }

        // Request collaborative learning insight for student1
        client.request_ml_insight(&student1, &course_id, &InsightType::CollaborativeInsight);

        let insight = client.get_ml_insight(&student1, &course_id, &InsightType::CollaborativeInsight);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::CollaborativeInsight);
        assert!(insight.confidence >= 70);
    }

    #[test]
    fn test_advanced_anomaly_detection() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create normal sessions first
        for i in 0..5 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.interactions = 20;
            client.record_session(&session);

            let end_time = session.start_time + 1800; // Consistent 30-minute sessions
            let score = Some(80);
            client.complete_session(&session.session_id, &end_time, &score, &100);
        }

        // Create anomalous sessions (very short and very long)
        let mut short_session = create_test_session(&env, &student, "RUST101", "module_6");
        short_session.session_id = BytesN::from_array(&env, &[10u8; 32]);
        short_session.interactions = 5;
        client.record_session(&short_session);
        client.complete_session(&short_session.session_id, &(short_session.start_time + 300), &Some(40), &50);

        let mut long_session = create_test_session(&env, &student, "RUST101", "module_7");
        long_session.session_id = BytesN::from_array(&env, &[11u8; 32]);
        long_session.interactions = 50;
        client.record_session(&long_session);
        client.complete_session(&long_session.session_id, &(long_session.start_time + 7200), &Some(85), &100);

        // Request anomaly detection
        client.request_ml_insight(&student, &course_id, &InsightType::AnomalyDetection);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::AnomalyDetection);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::AnomalyDetection);
        assert!(insight.confidence >= 75);
    }

    #[test]
    fn test_learning_path_optimization() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create sessions with mixed performance to trigger optimization
        let modules_performance = [
            ("module_1", 90, true),   // Completed successfully
            ("module_2", 55, false),  // Struggling
            ("module_3", 85, true),   // Completed successfully
            ("module_4", 45, false),  // Struggling
            ("module_5", 75, true),   // Completed successfully
        ];

        for (i, (module, score, completed)) in modules_performance.iter().enumerate() {
            let mut session = create_test_session(&env, &student, "RUST101", module);
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);

            let end_time = session.start_time + 1800;
            let final_score = Some(*score);
            let completion = if *completed { 100 } else { 60 };
            client.complete_session(&session.session_id, &end_time, &final_score, &completion);
        }

        // Request learning path optimization
        client.request_ml_insight(&student, &course_id, &InsightType::LearningPathOptimization);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::LearningPathOptimization);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::LearningPathOptimization);
        assert!(insight.confidence >= 70);
    }

    #[test]
    fn test_effectiveness_metrics() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create diverse sessions for effectiveness analysis
        let session_types_data = [
            (SessionType::Study, 85, 1800),
            (SessionType::Practice, 80, 2400),
            (SessionType::Assessment, 90, 1500),
            (SessionType::Review, 75, 1200),
            (SessionType::Study, 88, 2000),
            (SessionType::Practice, 82, 2200),
            (SessionType::Assessment, 92, 1600),
        ];

        for (i, (session_type, score, duration)) in session_types_data.iter().enumerate() {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.session_type = session_type.clone();
            session.interactions = 15 + i * 2;
            client.record_session(&session);

            let end_time = session.start_time + *duration;
            let final_score = Some(*score);
            client.complete_session(&session.session_id, &end_time, &final_score, &100);
        }

        // Request effectiveness metrics
        client.request_ml_insight(&student, &course_id, &InsightType::EffectivenessMetrics);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::EffectivenessMetrics);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::EffectivenessMetrics);
        assert!(insight.confidence >= 75);
    }

    #[test]
    fn test_adaptive_recommendations() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create sessions showing different progress levels
        let progress_data = [
            ("module_1", 95, true),   // Strong start
            ("module_2", 70, true),   // Decent progress
            ("module_3", 45, false),  // Struggling
        ];

        for (i, (module, score, completed)) in progress_data.iter().enumerate() {
            let mut session = create_test_session(&env, &student, "RUST101", module);
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);

            let end_time = session.start_time + 1800;
            let final_score = Some(*score);
            let completion = if *completed { 100 } else { 50 };
            client.complete_session(&session.session_id, &end_time, &final_score, &completion);
        }

        // Request adaptive recommendations
        client.request_ml_insight(&student, &course_id, &InsightType::AdaptiveRecommendation);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::AdaptiveRecommendation);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::AdaptiveRecommendation);
        assert!(insight.confidence >= 80);
    }

    #[test]
    fn test_insufficient_data_handling() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create only 2 sessions (insufficient for most ML analyses)
        for i in 0..2 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);

            let end_time = session.start_time + 1800;
            let score = Some(80);
            client.complete_session(&session.session_id, &end_time, &score, &100);
        }

        // Request insights that should fail due to insufficient data
        let insight_types = [
            InsightType::PatternRecognition,
            InsightType::EngagementPrediction,
            InsightType::KnowledgeGapAnalysis,
            InsightType::CollaborativeInsight,
            InsightType::LearningPathOptimization,
            InsightType::EffectivenessMetrics,
        ];

        for insight_type in insight_types {
            client.request_ml_insight(&student, &course_id, &insight_type);
            
            // These should either not generate insights or generate low-confidence ones
            let insight = client.get_ml_insight(&student, &course_id, &insight_type);
            if let Some(insight) = insight {
                // If insight exists, it should have low confidence due to insufficient data
                assert!(insight.confidence <= 60);
            }
        }
    }

    #[test]
    fn test_ml_insight_metadata() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create sufficient data for pattern recognition
        for i in 0..6 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.interactions = 20 + i * 5;
            client.record_session(&session);

            let end_time = session.start_time + 1800 + (i as u64 * 300);
            let score = Some(75 + i * 3);
            client.complete_session(&session.session_id, &end_time, &score, &100);
        }

        // Request pattern recognition
        client.request_ml_insight(&student, &course_id, &InsightType::PatternRecognition);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::PatternRecognition);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        
        // Verify metadata structure
        assert!(!insight.metadata.is_empty());
        assert_eq!(insight.model_version, 2);
        assert!(insight.timestamp > 0);
        
        // Check for expected metadata keys
        let mut found_study_ratio = false;
        let mut found_consistency = false;
        
        for i in 0..insight.metadata.len() {
            let (key, _value) = insight.metadata.get(i).unwrap();
            let key_str = key.to_string();
            if key_str.contains("study_ratio") {
                found_study_ratio = true;
            }
            if key_str.contains("consistency") {
                found_consistency = true;
            }
        }
        
        assert!(found_study_ratio);
        assert!(found_consistency);
    }

    #[test]
    fn test_multiple_insight_types_for_same_student() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Create comprehensive data
        for i in 0..8 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.interactions = 15 + i * 3;
            client.record_session(&session);

            let end_time = session.start_time + 1800;
            let score = Some(70 + i * 3);
            client.complete_session(&session.session_id, &end_time, &score, &100);
        }

        // Request multiple types of insights
        let insight_types = [
            InsightType::PatternRecognition,
            InsightType::CompletionPrediction,
            InsightType::Recommendation,
            InsightType::EngagementPrediction,
        ];

        for insight_type in insight_types {
            client.request_ml_insight(&student, &course_id, &insight_type);
            
            let insight = client.get_ml_insight(&student, &course_id, &insight_type);
            assert!(insight.is_some());
            let insight = insight.unwrap();
            assert_eq!(insight.insight_type, insight_type);
            assert!(insight.confidence >= 60);
        }
    }
}
