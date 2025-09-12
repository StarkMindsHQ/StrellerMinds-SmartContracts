#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec, Map, BytesN};

mod types;
mod batch_manager;
mod gas_optimizer;
mod session_manager;
mod offline_manager;
mod interaction_flows;
mod network_manager;

#[cfg(test)]
mod tests;

use types::*;
use batch_manager::BatchManager;
use gas_optimizer::GasOptimizer;
use session_manager::SessionManager;
use offline_manager::OfflineManager;
use interaction_flows::InteractionFlows;
use network_manager::NetworkManager;

#[contract]
pub struct MobileOptimizerContract;

#[contractimpl]
impl MobileOptimizerContract {
    /// Initialize the mobile optimizer contract
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        
        let config = MobileOptimizerConfig {
            admin: admin.clone(),
            max_batch_size: 10,
            default_gas_limit: 1000000,
            session_timeout_seconds: 3600, // 1 hour
            offline_queue_limit: 100,
            network_timeout_ms: 30000,
            retry_attempts: 5,
        };
        
        env.storage().persistent().set(&DataKey::Config, &config);
        env.storage().persistent().set(&DataKey::Admin, &admin);
    }

    /// Create a new mobile session
    pub fn create_session(
        env: Env,
        user: Address,
        device_id: String,
        preferences: MobilePreferences,
    ) -> Result<String, MobileOptimizerError> {
        user.require_auth();
        
        SessionManager::create_session(&env, user, device_id, preferences)
            .map_err(|_| MobileOptimizerError::SessionCreationFailed)
    }

    /// Update mobile session
    pub fn update_session(
        env: Env,
        user: Address,
        session_id: String,
        preferences: MobilePreferences,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        
        SessionManager::update_session(&env, user, session_id, preferences)
            .map_err(|_| MobileOptimizerError::SessionUpdateFailed)
    }

    /// Get mobile session information
    pub fn get_session(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<MobileSession, MobileOptimizerError> {
        user.require_auth();
        
        SessionManager::get_session(&env, &user, &session_id)
            .map_err(|_| MobileOptimizerError::SessionNotFound)
    }

    /// Create and execute a transaction batch
    pub fn execute_batch(
        env: Env,
        user: Address,
        operations: Vec<BatchOperation>,
        execution_strategy: ExecutionStrategy,
        session_id: String,
    ) -> Result<BatchExecutionResult, MobileOptimizerError> {
        user.require_auth();
        
        // Detect current network quality
        let network_quality = NetworkManager::detect_network_quality(&env);
        
        // Create batch
        let batch = TransactionBatch {
            batch_id: format!("batch_{}", env.ledger().timestamp()),
            user: user.clone(),
            operations,
            execution_strategy,
            created_at: env.ledger().timestamp(),
            status: BatchStatus::Pending,
            total_estimated_gas: 0, // Will be calculated
            retry_config: Self::create_default_retry_config(&network_quality),
            network_quality,
        };
        
        BatchManager::execute_batch(&env, batch, session_id)
            .map_err(|_| MobileOptimizerError::BatchExecutionFailed)
    }

    /// Estimate gas for operations
    pub fn estimate_gas(
        env: Env,
        operations: Vec<BatchOperation>,
        network_quality: NetworkQuality,
        estimation_mode: GasEstimationMode,
    ) -> Result<Vec<GasEstimate>, MobileOptimizerError> {
        let mut estimates = Vec::new(&env);
        
        for operation in operations {
            match GasOptimizer::estimate_operation_gas(&env, &operation, &network_quality, estimation_mode.clone()) {
                Ok(estimate) => estimates.push_back(estimate),
                Err(_) => return Err(MobileOptimizerError::GasEstimationFailed),
            }
        }
        
        Ok(estimates)
    }

    /// Get gas optimization suggestions
    pub fn get_gas_optimization_suggestions(
        env: Env,
        operations: Vec<BatchOperation>,
        network_quality: NetworkQuality,
    ) -> Result<Vec<GasOptimizationSuggestion>, MobileOptimizerError> {
        GasOptimizer::suggest_optimizations(&env, operations, network_quality)
            .map_err(|_| MobileOptimizerError::OptimizationFailed)
    }

    /// Quick course enrollment flow
    pub fn quick_enroll_course(
        env: Env,
        user: Address,
        course_id: String,
        session_id: String,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        user.require_auth();
        
        let network_quality = NetworkManager::detect_network_quality(&env);
        
        InteractionFlows::quick_enroll_course(&env, user, course_id, session_id, network_quality)
            .map_err(|_| MobileOptimizerError::InteractionFailed)
    }

    /// Quick progress update flow
    pub fn quick_update_progress(
        env: Env,
        user: Address,
        course_id: String,
        module_id: String,
        progress_percentage: u32,
        session_id: String,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        user.require_auth();
        
        let network_quality = NetworkManager::detect_network_quality(&env);
        
        InteractionFlows::quick_update_progress(
            &env,
            user,
            course_id,
            module_id,
            progress_percentage,
            session_id,
            network_quality,
        )
        .map_err(|_| MobileOptimizerError::InteractionFailed)
    }

    /// Quick certificate claim flow
    pub fn quick_claim_certificate(
        env: Env,
        user: Address,
        course_id: String,
        session_id: String,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        user.require_auth();
        
        let network_quality = NetworkManager::detect_network_quality(&env);
        
        InteractionFlows::quick_claim_certificate(&env, user, course_id, session_id, network_quality)
            .map_err(|_| MobileOptimizerError::InteractionFailed)
    }

    /// Queue operation for offline execution
    pub fn queue_offline_operation(
        env: Env,
        user: Address,
        device_id: String,
        operation: QueuedOperation,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        
        OfflineManager::queue_operation(&env, user, device_id, operation)
            .map_err(|_| MobileOptimizerError::OfflineOperationFailed)
    }

    /// Sync offline operations
    pub fn sync_offline_operations(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<OfflineSyncResult, MobileOptimizerError> {
        user.require_auth();
        
        let network_quality = NetworkManager::detect_network_quality(&env);
        
        OfflineManager::sync_offline_operations(&env, user, device_id, network_quality)
            .map_err(|_| MobileOptimizerError::OfflineSyncFailed)
    }

    /// Get offline queue status
    pub fn get_offline_queue_status(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<OfflineQueueStatus, MobileOptimizerError> {
        user.require_auth();
        
        OfflineManager::get_queue_status(&env, &user, &device_id)
            .map_err(|_| MobileOptimizerError::OfflineOperationFailed)
    }

    /// Resolve offline conflicts
    pub fn resolve_offline_conflicts(
        env: Env,
        user: Address,
        device_id: String,
        resolution_strategy: ConflictResolution,
        operation_resolutions: Vec<OperationResolution>,
    ) -> Result<ConflictResolutionResult, MobileOptimizerError> {
        user.require_auth();
        
        OfflineManager::resolve_conflicts(&env, user, device_id, resolution_strategy, operation_resolutions)
            .map_err(|_| MobileOptimizerError::ConflictResolutionFailed)
    }

    /// Get network statistics
    pub fn get_network_statistics(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<NetworkStatistics, MobileOptimizerError> {
        user.require_auth();
        
        Ok(NetworkManager::get_network_statistics(&env, session_id))
    }

    /// Get mobile capabilities
    pub fn get_mobile_capabilities(env: Env) -> MobileCapabilities {
        MobileCapabilities {
            max_batch_size: 10,
            supported_operations: Self::get_supported_operations(&env),
            offline_capabilities: OfflineManager::get_offline_capabilities(&env),
            gas_optimization_features: Self::get_gas_optimization_features(&env),
            network_adaptation_features: Self::get_network_adaptation_features(&env),
        }
    }

    /// Update mobile preferences
    pub fn update_mobile_preferences(
        env: Env,
        user: Address,
        session_id: String,
        preferences: MobilePreferences,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        
        SessionManager::update_preferences(&env, user, session_id, preferences)
            .map_err(|_| MobileOptimizerError::PreferenceUpdateFailed)
    }

    /// Get mobile analytics
    pub fn get_mobile_analytics(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<MobileAnalytics, MobileOptimizerError> {
        user.require_auth();
        
        SessionManager::get_session_analytics(&env, &user, &session_id)
            .map_err(|_| MobileOptimizerError::AnalyticsNotAvailable)
    }

    /// Clean up completed offline operations
    pub fn cleanup_offline_operations(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<u32, MobileOptimizerError> {
        user.require_auth();
        
        OfflineManager::cleanup_completed_operations(&env, user, device_id)
            .map_err(|_| MobileOptimizerError::OfflineOperationFailed)
    }

    // Admin functions

    /// Update contract configuration (admin only)
    pub fn update_config(
        env: Env,
        admin: Address,
        config: MobileOptimizerConfig,
    ) -> Result<(), MobileOptimizerError> {
        Self::require_admin(&env, &admin)?;
        
        env.storage().persistent().set(&DataKey::Config, &config);
        Ok(())
    }

    /// Get contract configuration
    pub fn get_config(env: Env) -> Result<MobileOptimizerConfig, MobileOptimizerError> {
        env.storage().persistent()
            .get(&DataKey::Config)
            .ok_or(MobileOptimizerError::ConfigNotFound)
    }

    /// Get contract statistics (admin only)
    pub fn get_contract_statistics(
        env: Env,
        admin: Address,
    ) -> Result<ContractStatistics, MobileOptimizerError> {
        Self::require_admin(&env, &admin)?;
        
        Ok(ContractStatistics {
            total_sessions: Self::count_total_sessions(&env),
            active_sessions: Self::count_active_sessions(&env),
            total_batches_executed: Self::count_total_batches(&env),
            total_offline_operations: Self::count_offline_operations(&env),
            average_gas_savings: Self::calculate_average_gas_savings(&env),
        })
    }

    // Helper functions

    fn require_admin(env: &Env, admin: &Address) -> Result<(), MobileOptimizerError> {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().persistent()
            .get(&DataKey::Admin)
            .ok_or(MobileOptimizerError::AdminNotSet)?;
        
        if *admin != stored_admin {
            return Err(MobileOptimizerError::UnauthorizedAdmin);
        }
        
        Ok(())
    }

    fn create_default_retry_config(network_quality: &NetworkQuality) -> RetryConfig {
        match network_quality {
            NetworkQuality::Excellent | NetworkQuality::Good => RetryConfig {
                max_retries: 3,
                base_delay_ms: 500,
                max_delay_ms: 5000,
                backoff_multiplier: 200,
                timeout_ms: 10000,
            },
            NetworkQuality::Fair => RetryConfig {
                max_retries: 5,
                base_delay_ms: 1000,
                max_delay_ms: 10000,
                backoff_multiplier: 150,
                timeout_ms: 15000,
            },
            NetworkQuality::Poor => RetryConfig {
                max_retries: 7,
                base_delay_ms: 2000,
                max_delay_ms: 20000,
                backoff_multiplier: 125,
                timeout_ms: 30000,
            },
            NetworkQuality::Offline => RetryConfig {
                max_retries: 0,
                base_delay_ms: 0,
                max_delay_ms: 0,
                backoff_multiplier: 100,
                timeout_ms: 1000,
            },
        }
    }

    fn get_supported_operations(env: &Env) -> Vec<OperationType> {
        let mut operations = Vec::new(env);
        operations.push_back(OperationType::CourseEnrollment);
        operations.push_back(OperationType::ProgressUpdate);
        operations.push_back(OperationType::CertificateGeneration);
        operations.push_back(OperationType::TokenReward);
        operations.push_back(OperationType::PreferenceUpdate);
        operations.push_back(OperationType::SearchQuery);
        operations
    }

    fn get_gas_optimization_features(env: &Env) -> Vec<String> {
        let mut features = Vec::new(env);
        features.push_back(String::from_str(env, "Dynamic gas estimation"));
        features.push_back(String::from_str(env, "Network-aware optimization"));
        features.push_back(String::from_str(env, "Batch gas optimization"));
        features.push_back(String::from_str(env, "Operation prioritization"));
        features
    }

    fn get_network_adaptation_features(env: &Env) -> Vec<String> {
        let mut features = Vec::new(env);
        features.push_back(String::from_str(env, "Automatic network quality detection"));
        features.push_back(String::from_str(env, "Adaptive retry strategies"));
        features.push_back(String::from_str(env, "Connection optimization"));
        features.push_back(String::from_str(env, "Performance monitoring"));
        features
    }

    // Statistics helper functions (simplified implementations)
    fn count_total_sessions(env: &Env) -> u32 {
        // Would count actual sessions from storage
        0
    }

    fn count_active_sessions(env: &Env) -> u32 {
        // Would count active sessions
        0
    }

    fn count_total_batches(env: &Env) -> u32 {
        // Would count executed batches
        0
    }

    fn count_offline_operations(env: &Env) -> u32 {
        // Would count offline operations
        0
    }

    fn calculate_average_gas_savings(env: &Env) -> u32 {
        // Would calculate actual gas savings
        0
    }

    fn format(template: &str, value: u64) -> String {
        // Simple format implementation for no_std
        String::from_str(&Env::default(), &template.replace("{}", &value.to_string()))
    }
}

// Additional contract types

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileOptimizerConfig {
    pub admin: Address,
    pub max_batch_size: u32,
    pub default_gas_limit: u64,
    pub session_timeout_seconds: u64,
    pub offline_queue_limit: u32,
    pub network_timeout_ms: u32,
    pub retry_attempts: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileCapabilities {
    pub max_batch_size: u32,
    pub supported_operations: Vec<OperationType>,
    pub offline_capabilities: OfflineCapabilities,
    pub gas_optimization_features: Vec<String>,
    pub network_adaptation_features: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractStatistics {
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub total_batches_executed: u32,
    pub total_offline_operations: u32,
    pub average_gas_savings: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MobileOptimizerError {
    SessionCreationFailed,
    SessionUpdateFailed,
    SessionNotFound,
    BatchExecutionFailed,
    GasEstimationFailed,
    OptimizationFailed,
    InteractionFailed,
    OfflineOperationFailed,
    OfflineSyncFailed,
    ConflictResolutionFailed,
    PreferenceUpdateFailed,
    AnalyticsNotAvailable,
    ConfigNotFound,
    AdminNotSet,
    UnauthorizedAdmin,
}
