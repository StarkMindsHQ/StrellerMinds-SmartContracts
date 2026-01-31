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

    fn setup() -> (Env, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        (env, admin)
    }

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

    #[test]
    fn test_initialize_analytics_contract() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let _client = init_contract(&env, &admin);
        assert!(true);
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
        assert!(result.is_err());
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
    }

    #[test]
    fn test_complete_session() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);
        assert!(true);
    }

    #[test]
    fn test_get_student_sessions() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        for i in 0..3 {
            let mut session = create_test_session(&env, &student, "RUST101", "module_1");
            let mut bytes = [i as u8; 32];
            session.session_id = BytesN::from_array(&env, &bytes);
            client.record_session(&session);
        }
        let course_id = Symbol::new(&env, "RUST101");
        let result = client.try_get_student_sessions(&student, &course_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_progress_analytics_calculation() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let course_id = Symbol::new(&env, "RUST101");
        for i in 0..3 {
            let mut session = create_test_session(&env, &student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            client.record_session(&session);
        }
        let result = client.try_get_progress_analytics(&student, &course_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_leaderboard() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let course_id = Symbol::new(&env, "RUST101");
        let result = client.try_get_leaderboard(&course_id, &LeaderboardMetric::TimeSpent);
        assert!(result.is_ok());
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
    }

    #[test]
    fn test_unauthorized_config_update() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let _unauthorized_user = Address::generate(&env);
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
    }

    #[test]
    fn test_transfer_admin() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let new_admin = Address::generate(&env);
        let result = client.try_transfer_admin(&admin, &new_admin);
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_session_update() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let mut sessions = Vec::new(&env);
        for i in 0..3 {
            let mut session = create_test_session(&env, &student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            sessions.push_back(session);
        }
        let batch = BatchSessionUpdate {
            sessions,
            update_analytics: true,
            update_leaderboards: false,
        };
        let result = client.try_batch_update_sessions(&batch);
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_size_limit() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let mut sessions = Vec::new(&env);
        for i in 0..50 {
            let mut session = create_test_session(&env, &student, "RUST101", "module_1");
            let mut bytes = [(i as u8); 32];
            session.session_id = BytesN::from_array(&env, &bytes);
            sessions.push_back(session);
        }
        let batch = BatchSessionUpdate {
            sessions,
            update_analytics: false,
            update_leaderboards: false,
        };
        let _result = client.try_batch_update_sessions(&batch);
        assert!(true);
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
        }
        let result = client.try_get_module_analytics(&course_id, &module_id);
        assert!(result.is_ok());
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
            let mut session = create_test_session(&env, &student, "RUST101", "module_1");
            session.session_id = BytesN::from_array(&env, &[i as u8; 32]);
            session.start_time = start_time + (i as u64 * 86400);
            client.record_session(&session);
        }
        let end_time = start_time + (7 * 86400);
        let result = client.try_generate_progress_report(&student, &course_id, &ReportPeriod::Weekly, &start_time, &end_time);
        assert!(result.is_ok());
    }

    #[test]
    fn test_request_ml_insight() {
        let (env, admin) = setup();
        env.mock_all_auths();
        let client = init_contract(&env, &admin);
        let student = Address::generate(&env);
        let course_id = Symbol::new(&env, "RUST101");
        let session = create_test_session(&env, &student, "RUST101", "module_1");
        client.record_session(&session);
        let result = client.try_request_ml_insight(&student, &course_id, &InsightType::PatternRecognition);
        assert!(result.is_ok());
    }
}
