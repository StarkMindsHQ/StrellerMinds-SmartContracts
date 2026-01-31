#[cfg(test)]
extern crate std;

mod analytics_tests {
    use super::*;
    use crate::{
        errors::AnalyticsError,
        types::{
            AnalyticsConfig, BatchSessionUpdate, DifficultyThresholds, InsightType,
            LeaderboardMetric, LearningSession, ReportPeriod, SessionType,
        },
        Analytics, AnalyticsClient,
    };
    use soroban_sdk::{
        testutils::Address as _,
        Address, BytesN, Env, Symbol, Vec,
    };
    use std::format;

    /// Minimal setup: Creates fresh Env and admin only
    fn setup() -> (Env, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        (env, admin)
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

    /// Helper to create test session (does not modify contract state)
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
            end_time: 0,
            completion_percentage: 0,
            time_spent: 0,
            interactions: 0,
            score: None,
            session_type: SessionType::Study,
        }
    }

    // ============================================================================
    // UNIT TESTS - FRESH ENV PER TEST, env.mock_all_auths() BEFORE INITIALIZE()
    // ============================================================================

    #[test]
    fn test_initialize_analytics_contract() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let stored_admin = client.get_admin();
        assert_eq!(stored_admin, Some(admin));
        let config = client.get_config();
        assert!(config.is_some());
        let cfg = config.unwrap();
        assert_eq!(cfg.min_session_time, 60);
        assert_eq!(cfg.max_session_time, 14400);
    }

    #[test]
    fn test_initialize_already_initialized() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        
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
        let result = client.try_initialize(&admin, &new_config);
        assert_eq!(result, Err(Ok(AnalyticsError::AlreadyInitialized)));
    }

    #[test]
    fn test_record_learning_session() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let session = create_test_session(&env, &student, "RUST101", "module_1");
        let result = client.try_record_session(&session);
        assert!(result.is_ok());
        let stored_session = client.get_session(&session.session_id);
        assert!(stored_session.is_some());
        assert_eq!(stored_session.unwrap().student, student);
    }

    #[test]
    fn test_complete_session() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);
        let end_time = env.ledger().timestamp() + 1800;
        let result = client.try_complete_session(&session.session_id, &end_time, &Some(85u32), &100u32);
        assert!(result.is_ok());
        let updated_session = client.get_session(&session.session_id).unwrap();
        assert_eq!(updated_session.end_time, end_time);
        assert_eq!(updated_session.score, Some(85u32));
        assert_eq!(updated_session.completion_percentage, 100);
        assert_eq!(updated_session.time_spent, 1800);
    }

    #[test]
    fn test_get_student_sessions() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let course_id = Symbol::new(&env, "RUST101");
        for i in 0..3 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);
        }
        let sessions = client.get_student_sessions(&student, &course_id);
        assert_eq!(sessions.len(), 3);
    }

    #[test]
    fn test_progress_analytics_calculation() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let course_id = Symbol::new(&env, "RUST101");
        for i in 0..3 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);
            let end_time = session.start_time + 1800;
            client.complete_session(&session.session_id, &end_time, &Some(80 + i * 5), &100);
        }
        let result = client.try_get_progress_analytics(&student, &course_id);
        assert!(result.is_ok());
        let analytics = result.unwrap().unwrap();
        assert_eq!(analytics.student, student);
        assert_eq!(analytics.course_id, course_id);
        assert_eq!(analytics.total_sessions, 3);
        assert!(analytics.total_time_spent > 0);
    }

    #[test]
    fn test_generate_leaderboard() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student1 = Address::generate(&env);
        let student2 = Address::generate(&env);
        let student3 = Address::generate(&env);
        let course_id = Symbol::new(&env, "RUST101");
        let students_scores = [(&student1, 95), (&student2, 85), (&student3, 75)];
        for (student, score) in students_scores {
            let mut session = create_test_session(&env, student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[score as u8; 32]);
            client.record_session(&session);
            let end_time = session.start_time + 1800;
            client.complete_session(&session.session_id, &end_time, &Some(score), &100);
        }
        let result = client.try_generate_leaderboard(&course_id, &LeaderboardMetric::TotalScore, &10);
        assert!(result.is_ok());
        let leaderboard = result.unwrap().unwrap();
        assert_eq!(leaderboard.len(), 3);
        let top_entry = leaderboard.get(0).unwrap();
        assert_eq!(top_entry.student, student1);
        assert_eq!(top_entry.rank, 1);
        assert_eq!(top_entry.score, 95);
    }

    #[test]
    fn test_update_config() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
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
        let result = client.try_update_config(&admin, &new_config);
        assert!(result.is_ok());
        let stored_config = client.get_config().unwrap();
        assert_eq!(stored_config.min_session_time, 120);
        assert_eq!(stored_config.max_session_time, 7200);
    }

    #[test]
    fn test_unauthorized_config_update() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let unauthorized_user = Address::generate(&env);
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
        let result = client.try_update_config(&unauthorized_user, &new_config);
        assert_eq!(result, Err(Ok(AnalyticsError::Unauthorized)));
    }

    #[test]
    fn test_transfer_admin() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let new_admin = Address::generate(&env);
        let result = client.try_transfer_admin(&admin, &new_admin);
        assert!(result.is_ok());
        let stored_admin = client.get_admin().unwrap();
        assert_eq!(stored_admin, new_admin);
    }

    #[test]
    fn test_batch_session_update() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let mut sessions: Vec<LearningSession> = Vec::new(&env);
        for i in 0..5 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
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
        let result = client.try_batch_update_sessions(&batch);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.unwrap(), 5);
    }

    #[test]
    fn test_batch_size_limit() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let mut sessions: Vec<LearningSession> = Vec::new(&env);
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
        let result = client.try_batch_update_sessions(&batch);
        assert_eq!(result, Err(Ok(AnalyticsError::InvalidBatchSize)));
    }

    #[test]
    fn test_module_analytics_calculation() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let course_id = Symbol::new(&env, "RUST101");
        let module_id = Symbol::new(&env, "module_1");
        for i in 0..5 {
            let mut session = create_test_session(&env, &student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);
            let end_time = session.start_time + 1200;
            let completion = if i < 4 { 100 } else { 75 };
            client.complete_session(&session.session_id, &end_time, &Some(80), &completion);
        }
        let result = client.try_get_module_analytics(&course_id, &module_id);
        assert!(result.is_ok());
        let analytics = result.unwrap().unwrap();
        assert_eq!(analytics.course_id, course_id);
        assert_eq!(analytics.module_id, module_id);
        assert_eq!(analytics.total_attempts, 5);
        assert_eq!(analytics.completion_rate, 80);
    }

    #[test]
    fn test_generate_progress_report() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let course_id = Symbol::new(&env, "RUST101");
        let start_time = env.ledger().timestamp();
        for i in 0..3 {
            let mut session = create_test_session(&env, &student, "RUST101", &format!("module_{}", i + 1));
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.start_time = start_time + (i as u64 * 86400);
            client.record_session(&session);
            let end_time = session.start_time + 1800;
            client.complete_session(&session.session_id, &end_time, &Some(85), &100);
        }
        let end_time = start_time + (7 * 86400);
        let result = client.try_generate_progress_report(&student, &course_id, &ReportPeriod::Weekly, &start_time, &end_time);
        assert!(result.is_ok());
        let report = result.unwrap().unwrap();
        assert_eq!(report.student, student);
        assert_eq!(report.course_id, course_id);
        assert_eq!(report.sessions_count, 3);
    }

    #[test]
    fn test_request_ml_insight() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let course_id = Symbol::new(&env, "RUST101");
        let result = client.try_request_ml_insight(&student, &course_id, &InsightType::PatternRecognition);
        assert!(result.is_ok());
    }
}
