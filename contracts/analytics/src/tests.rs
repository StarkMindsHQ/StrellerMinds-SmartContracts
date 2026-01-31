#[cfg(test)]
extern crate std;

mod analytics_tests {
    use super::*;
    use crate::{
        errors::AnalyticsError,
        types::{
            DifficultyThresholds, InsightType, LeaderboardMetric, LearningSession, MLInsight,
            SessionType,
        },
        Analytics, AnalyticsClient,
    };
    use soroban_sdk::{
        testutils::Address as _,
        Address, BytesN, Env, String, Symbol, Vec,
    };
    use std::format;

    fn create_test_env() -> (Env, Address, Address, Address) {
        let env = Env::default();
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
        for (student, completion) in [(&student1, 100), (&student2, 30)] {
            let mut session = create_test_session(&env, student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[completion as u8; 32]);
            client.record_session(&session);

            let end_time = session.start_time + 1800;
            client.complete_session(
                &session.session_id,
                &end_time,
                &Some(completion),
                &completion,
            );
        }

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

        // Request insight
        let result =
            client.try_request_ml_insight(&student, &course_id, &InsightType::PatternRecognition);
        assert!(result.is_ok());
    }

    #[test]
    fn test_callback_ml_insight_authorized() {
        let (env, admin, _, student) = create_test_env();
        let oracle = Address::generate(&env);

        let contract_id = env.register_contract(None, Analytics);
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
        let mut session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);
        let end_time = session.start_time + 3600; // 1 hour
        client.complete_session(&session.session_id, &end_time, &Some(90), &50); // 50% complete

        // Request prediction
        client.request_ml_insight(&student, &course_id, &InsightType::CompletionPrediction);

        let insight =
            client.get_ml_insight(&student, &course_id, &InsightType::CompletionPrediction);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::CompletionPrediction);
        let data_str = insight.data.to_utf8().unwrap_or_default();
        assert!(data_str.contains("Predicted"));
    }

    #[test]
    fn test_generate_recommendations() {
        let (env, admin, _, student) = create_test_env();
        let client = setup_analytics_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");

        // Request recommendation
        client.request_ml_insight(&student, &course_id, &InsightType::Recommendation);

        let insight = client.get_ml_insight(&student, &course_id, &InsightType::Recommendation);
        assert!(insight.is_some());
        let insight = insight.unwrap();
        assert_eq!(insight.insight_type, InsightType::Recommendation);
        let data_str = insight.data.to_utf8().unwrap_or_default();
        assert!(
            data_str.contains("Review")
                || data_str.contains("Consider")
        );
    }

    // Note: prepare_ml_data is not exposed on AnalyticsClient public interface
    // #[test]
    // fn test_prepare_ml_data() {
    //     let (env, admin, _, student) = create_test_env();
    //     let client = setup_analytics_contract(&env, &admin);
    //     let course_id = Symbol::new(&env, "RUST101");
    //
    //     // Arrange: create and complete a session so there is data to summarize
    //     let mut session = create_test_session(&env, &student, "RUST101", "module_1");
    //     client.record_session(&session);
    //
    //     let end_time = session.start_time + 1800; // 30 minutes
    //     client.complete_session(&session.session_id, &end_time, &Some(85), &100);
    //
    //     // Act: prepare ML data
    //     let ml_data = client.prepare_ml_data(&course_id);
    //
    //     // Assert: should return a non-empty, masked summary string
    //     let ml_data_str = ml_data.to_utf8().unwrap_or_default();
    //
    //     assert!(!ml_data_str.is_empty());
    //
    //     // Course-level info is allowed
    //     assert!(ml_data_str.contains("RUST101"));
    //
    //     // PII must not be present
    //     assert!(!ml_data_str.contains(&student.to_string()));
    // }
}
