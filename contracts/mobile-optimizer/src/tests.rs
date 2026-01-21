#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Address, Env};

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.max_batch_size, 10);
}

#[test]
fn test_session_management() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    let device_id = String::from_str(&env, "mobile_device_123");
    let preferences = MobilePreferences::default();
    
    let session_id = client.create_session(&user, &device_id, &preferences);
    assert!(session_id.len() > 0);
    
    let session = client.get_session(&user, &session_id);
    assert_eq!(session.user, user);
    assert_eq!(session.device_id, device_id);
}

#[test]
fn test_batch_execution() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    // Create session first
    let device_id = String::from_str(&env, "mobile_device_123");
    let preferences = MobilePreferences::default();
    let session_id = client.create_session(&user, &device_id, &preferences);
    
    // Create batch operations
    let mut operations = Vec::new(&env);
    operations.push_back(BatchOperation {
        operation_id: String::from_str(&env, "test_op_1"),
        operation_type: OperationType::ProgressUpdate,
        contract_address: Address::generate(&env),
        function_name: String::from_str(&env, "update_progress"),
        parameters: Vec::new(&env),
        estimated_gas: 50000,
        priority: OperationPriority::High,
        retry_config: RetryConfig {
            max_retries: 3,
            base_delay_ms: 500,
            max_delay_ms: 5000,
            backoff_multiplier: 200,
            timeout_ms: 10000,
        },
        dependencies: Vec::new(&env),
    });
    
    let result = client.execute_batch(
        &user,
        &operations,
        &ExecutionStrategy::Sequential,
        &session_id,
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_gas_estimation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    let mut operations = Vec::new(&env);
    operations.push_back(BatchOperation {
        operation_id: String::from_str(&env, "test_op_1"),
        operation_type: OperationType::CourseEnrollment,
        contract_address: Address::generate(&env),
        function_name: String::from_str(&env, "enroll_student"),
        parameters: Vec::new(&env),
        estimated_gas: 75000,
        priority: OperationPriority::High,
        retry_config: RetryConfig {
            max_retries: 3,
            base_delay_ms: 500,
            max_delay_ms: 5000,
            backoff_multiplier: 200,
            timeout_ms: 10000,
        },
        dependencies: Vec::new(&env),
    });
    
    let estimates = client.estimate_gas(
        &operations,
        &NetworkQuality::Good,
        &GasEstimationMode::Conservative,
    );
    
    assert!(estimates.is_ok());
    let estimates = estimates.unwrap();
    assert_eq!(estimates.len(), 1);
}

#[test]
fn test_quick_interactions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    // Create session
    let device_id = String::from_str(&env, "mobile_device_123");
    let preferences = MobilePreferences::default();
    let session_id = client.create_session(&user, &device_id, &preferences);
    
    // Test quick enrollment
    let course_id = String::from_str(&env, "course_123");
    let enroll_result = client.quick_enroll_course(&user, &course_id, &session_id);
    assert!(enroll_result.is_ok());
    
    // Test quick progress update
    let module_id = String::from_str(&env, "module_456");
    let progress_result = client.quick_update_progress(
        &user,
        &course_id,
        &module_id,
        75,
        &session_id,
    );
    assert!(progress_result.is_ok());
}

#[test]
fn test_offline_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    let device_id = String::from_str(&env, "mobile_device_123");
    
    // Queue offline operation
    let queued_op = QueuedOperation {
        operation_id: String::from_str(&env, "offline_op_1"),
        operation_type: OperationType::ProgressUpdate,
        parameters: Vec::new(&env),
        estimated_gas: 25000,
        created_at: env.ledger().timestamp(),
        status: QueuedOperationStatus::Queued,
        retry_count: 0,
        local_state_hash: BytesN::from_array(&env, &[0u8; 32]),
    };
    
    let queue_result = client.queue_offline_operation(&user, &device_id, &queued_op);
    assert!(queue_result.is_ok());
    
    // Check queue status
    let status = client.get_offline_queue_status(&user, &device_id);
    assert!(status.is_ok());
    let status = status.unwrap();
    assert_eq!(status.total_operations, 1);
}

#[test]
fn test_mobile_capabilities() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let capabilities = client.get_mobile_capabilities();
    
    assert_eq!(capabilities.max_batch_size, 10);
    assert!(capabilities.supported_operations.len() > 0);
    assert!(capabilities.gas_optimization_features.len() > 0);
    assert!(capabilities.network_adaptation_features.len() > 0);
}

#[test]
fn test_admin_functions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    // Test config update by admin
    let new_config = MobileOptimizerConfig {
        admin: admin.clone(),
        max_batch_size: 15,
        default_gas_limit: 2000000,
        session_timeout_seconds: 7200,
        offline_queue_limit: 200,
        network_timeout_ms: 45000,
        retry_attempts: 7,
    };
    
    let update_result = client.update_config(&admin, &new_config);
    assert!(update_result.is_ok());
    
    let updated_config = client.get_config();
    assert_eq!(updated_config.max_batch_size, 15);
    
    // Test statistics access by admin
    let stats_result = client.get_contract_statistics(&admin);
    assert!(stats_result.is_ok());
}

#[test]
fn test_network_quality_adaptation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    // Create session
    let device_id = String::from_str(&env, "mobile_device_123");
    let preferences = MobilePreferences::default();
    let session_id = client.create_session(&user, &device_id, &preferences);
    
    // Test network statistics
    let stats_result = client.get_network_statistics(&user, &session_id);
    assert!(stats_result.is_ok());
}

#[test]
fn test_error_handling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    // Test accessing non-existent session
    let invalid_session_id = String::from_str(&env, "invalid_session");
    let session_result = client.get_session(&user, &invalid_session_id);
    assert!(session_result.is_err());
    
    // Test unauthorized admin access
    let non_admin = Address::generate(&env);
    let stats_result = client.get_contract_statistics(&non_admin);
    assert!(stats_result.is_err());
}

#[test]
fn test_preference_updates() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();
    
    client.initialize(&admin);
    
    // Create session
    let device_id = String::from_str(&env, "mobile_device_123");
    let preferences = MobilePreferences::default();
    let session_id = client.create_session(&user, &device_id, &preferences);
    
    // Update preferences
    let new_preferences = MobilePreferences {
        auto_batch_enabled: false,
        max_batch_size: 5,
        gas_optimization_level: GasOptimizationLevel::Aggressive,
        offline_mode_enabled: true,
        network_timeout_ms: 20000,
        retry_failed_operations: true,
        cache_search_results: true,
        compress_data: true,
    };
    
    let update_result = client.update_mobile_preferences(&user, &session_id, &new_preferences);
    assert!(update_result.is_ok());
    
    // Verify preferences were updated
    let session = client.get_session(&user, &session_id);
    assert_eq!(session.preferences.max_batch_size, 5);
}

// Integration tests for complex workflows
#[test]
fn test_complete_mobile_workflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MobileOptimizerContract);
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();
    
    // Initialize contract
    client.initialize(&admin);
    
    // Create mobile session
    let device_id = String::from_str(&env, "mobile_device_123");
    let preferences = MobilePreferences::default();
    let session_id = client.create_session(&user, &device_id, &preferences);
    
    // Enroll in course
    let course_id = String::from_str(&env, "course_123");
    let enroll_result = client.quick_enroll_course(&user, &course_id, &session_id);
    assert!(enroll_result.is_ok());
    
    // Update progress multiple times
    let module_id = String::from_str(&env, "module_456");
    for progress in [25, 50, 75, 100] {
        let progress_result = client.quick_update_progress(
            &user,
            &course_id,
            &module_id,
            progress,
            &session_id,
        );
        assert!(progress_result.is_ok());
    }
    
    // Claim certificate
    let cert_result = client.quick_claim_certificate(&user, &course_id, &session_id);
    assert!(cert_result.is_ok());
    
    // Get analytics
    let analytics_result = client.get_mobile_analytics(&user, &session_id);
    assert!(analytics_result.is_ok());
}

impl Default for MobilePreferences {
    fn default() -> Self {
        Self {
            auto_batch_enabled: true,
            max_batch_size: 10,
            gas_optimization_level: GasOptimizationLevel::Balanced,
            offline_mode_enabled: true,
            network_timeout_ms: 30000,
            retry_failed_operations: true,
            cache_search_results: true,
            compress_data: false,
        }
    }
}
