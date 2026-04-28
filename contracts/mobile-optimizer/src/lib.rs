#![no_std]
#![allow(clippy::too_many_arguments)]

pub mod errors;

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Vec};

pub mod analytics_monitor;
pub mod batch_manager;
pub mod battery_optimizer;
pub mod collaboration_manager;
pub mod content_cache;
pub mod content_manager;
pub mod gas_optimizer;
pub mod interaction_flows;
pub mod network_manager;
pub mod notification_manager;
pub mod offline_manager;
pub mod pwa_manager;
pub mod security_manager;
pub mod session_manager;
pub mod types;
pub mod user_experience_manager;

#[cfg(test)]
mod tests;

use crate::errors::MobileOptimizerError;
use analytics_monitor::AnalyticsMonitor;
use batch_manager::{BatchExecutionResult, BatchManager};
use battery_optimizer::{BatteryOptimizedSettings, BatteryOptimizer};
use collaboration_manager::CollaborationManager;
use content_cache::ContentCacheManager;
use content_manager::ContentManager;
use gas_optimizer::GasOptimizer;
use interaction_flows::{InteractionFlows, MobileInteractionResult};
use network_manager::{
    BandwidthOptimization, ConnectionSettings, NetworkAdaptation, NetworkManager, NetworkStatistics,
};
use notification_manager::NotificationManager;
use offline_manager::{OfflineCapabilities, OfflineManager, OfflineQueueStatus, OfflineSyncResult};
use pwa_manager::{OfflineCapabilityReport, PwaManager};
use security_manager::SecurityManager;
use session_manager::{SessionManager, SessionOptimization, SessionStats};
use shared::config::DeploymentEnv;
use types::*;
use user_experience_manager::UserExperienceManager;

#[contract]
pub struct MobileOptimizerContract;

#[allow(clippy::too_many_arguments)]
#[contractimpl]
impl MobileOptimizerContract {
    // ========================================================================
    // Initialization & Admin
    // ========================================================================

    /// Initialize the mobile optimizer contract and set the admin address.
    ///
    /// Must be called once before any other function. Stores the default configuration
    /// and sets all global counters to zero. Requires authorization from `admin`.
    ///
    /// # Arguments
    /// * `admin` - Address that will hold administrative privileges.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::AlreadyInitialized`] if the contract has already been initialized.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin);
    /// ```
    pub fn initialize(env: Env, admin: Address) -> Result<(), MobileOptimizerError> {
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Initialized) {
            return Err(MobileOptimizerError::AlreadyInitialized);
        }

        let config = MobileOptimizerConfig::for_env(admin.clone(), DeploymentEnv::Production);
        config.validate()?;

        env.storage().persistent().set(&DataKey::Config, &config);
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage().persistent().set(&DataKey::TotalSessions, &0u64);
        env.storage().persistent().set(&DataKey::TotalBatches, &0u64);
        env.storage().persistent().set(&DataKey::TotalOfflineOps, &0u64);
        Ok(())
    }

    /// Retrieve the current optimizer configuration from persistent storage.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::ConfigNotFound`] if no configuration has been stored.
    ///
    /// # Example
    /// ```ignore
    /// let config = client.get_config();
    /// ```
    pub fn get_config(env: Env) -> Result<MobileOptimizerConfig, MobileOptimizerError> {
        env.storage().persistent().get(&DataKey::Config).ok_or(MobileOptimizerError::ConfigNotFound)
    }

    /// Replace the optimizer configuration with new values.
    ///
    /// Requires admin authorization. All subsequent operations will use the updated config.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    /// * `config` - New configuration to persist.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::UnauthorizedAdmin`] if `admin` is not the stored admin.
    ///
    /// # Example
    /// ```ignore
    /// client.update_config(&admin, &new_config);
    /// ```
    pub fn update_config(
        env: Env,
        admin: Address,
        config: MobileOptimizerConfig,
    ) -> Result<(), MobileOptimizerError> {
        Self::require_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::Config, &config);
        Ok(())
    }

    // ========================================================================
    // Session Management
    // ========================================================================

    /// Create a new mobile session for a user on a specific device.
    ///
    /// Requires authorization from `user`. Increments the global session counter on success.
    ///
    /// # Arguments
    /// * `user` - Address of the user creating the session.
    /// * `device_id` - Identifier of the device initiating the session.
    /// * `preferences` - Initial mobile preferences for this session.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SessionCreationFailed`] if session creation fails.
    ///
    /// # Example
    /// ```ignore
    /// let session_id = client.create_session(&user, &device_id, &preferences);
    /// ```
    pub fn create_session(
        env: Env,
        user: Address,
        device_id: String,
        preferences: MobilePreferences,
    ) -> Result<String, MobileOptimizerError> {
        user.require_auth();
        let session_id = SessionManager::create_session(&env, user, device_id, preferences)?;
        Self::increment_counter(&env, &DataKey::TotalSessions);
        Ok(session_id)
    }

    /// Retrieve an existing mobile session by its ID.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the session owner.
    /// * `session_id` - Unique identifier of the session to retrieve.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SessionNotFound`] if no session matches the ID.
    ///
    /// # Example
    /// ```ignore
    /// let session = client.get_session(&user, &session_id);
    /// ```
    pub fn get_session(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<MobileSession, MobileOptimizerError> {
        user.require_auth();
        SessionManager::get_session(&env, &session_id)
    }

    /// Update the network quality recorded for an active session.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the session owner.
    /// * `session_id` - Unique identifier of the session to update.
    /// * `network_quality` - Current network quality to record on the session.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SessionNotFound`] if the session does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.update_session(&user, &session_id, &NetworkQuality::Good);
    /// ```
    pub fn update_session(
        env: Env,
        user: Address,
        session_id: String,
        network_quality: NetworkQuality,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::update_session(&env, session_id, Some(network_quality), None)
    }

    /// Update the mobile preferences attached to an existing session.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the session owner.
    /// * `session_id` - Unique identifier of the session to update.
    /// * `preferences` - New mobile preferences to associate with the session.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SessionNotFound`] if the session does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.update_mobile_preferences(&user, &session_id, &preferences);
    /// ```
    pub fn update_mobile_preferences(
        env: Env,
        user: Address,
        session_id: String,
        preferences: MobilePreferences,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::update_preferences(&env, session_id, preferences)
    }

    /// Suspend an active session, preserving its state for later resumption.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the session owner.
    /// * `session_id` - Unique identifier of the session to suspend.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SessionNotFound`] if the session does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.suspend_session(&user, &session_id);
    /// ```
    pub fn suspend_session(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::suspend_session(&env, session_id)
    }

    /// Resume a previously suspended session with the current network quality.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the session owner.
    /// * `session_id` - Unique identifier of the suspended session.
    /// * `network_quality` - Current network quality at the time of resumption.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SessionNotFound`] if the session does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.resume_session(&user, &session_id, &NetworkQuality::Good);
    /// ```
    pub fn resume_session(
        env: Env,
        user: Address,
        session_id: String,
        network_quality: NetworkQuality,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::resume_session(&env, session_id, network_quality)
    }

    /// Terminate an active session and persist its final state.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the session owner.
    /// * `session_id` - Unique identifier of the session to end.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SessionNotFound`] if the session does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.end_session(&user, &session_id);
    /// ```
    pub fn end_session(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::end_session(&env, session_id)
    }

    /// Return aggregate session statistics for the given user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose session stats are requested.
    ///
    /// # Example
    /// ```ignore
    /// let stats = client.get_session_stats(&user);
    /// ```
    pub fn get_session_stats(env: Env, user: Address) -> SessionStats {
        user.require_auth();
        SessionManager::get_session_stats(&env, &user)
    }

    /// Analyze a session and return performance optimization recommendations.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the session owner.
    /// * `session_id` - Unique identifier of the session to optimize.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SessionNotFound`] if the session does not exist.
    ///
    /// # Example
    /// ```ignore
    /// let optimization = client.optimize_session(&user, &session_id);
    /// ```
    pub fn optimize_session(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<SessionOptimization, MobileOptimizerError> {
        user.require_auth();
        SessionManager::optimize_session_performance(&env, session_id)
    }

    /// Clone the state of an existing session onto a new target device, returning the new session ID.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the session owner.
    /// * `source_session_id` - ID of the session whose state will be copied.
    /// * `target_device_id` - Device identifier on which the new session will be created.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SessionNotFound`] if the source session does not exist.
    ///
    /// # Example
    /// ```ignore
    /// let new_session_id = client.sync_session_state(&user, &source_session_id, &target_device_id);
    /// ```
    pub fn sync_session_state(
        env: Env,
        user: Address,
        source_session_id: String,
        target_device_id: String,
    ) -> Result<String, MobileOptimizerError> {
        user.require_auth();
        SessionManager::sync_session_state(&env, &user, source_session_id, target_device_id)
    }

    // ========================================================================
    // Batch Execution
    // ========================================================================

    /// Create a batch of operations to be executed together, returning the batch ID.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user creating the batch.
    /// * `operations` - List of operations to include in the batch.
    /// * `priority` - Execution priority for the batch.
    /// * `strategy` - Strategy controlling how operations within the batch are executed.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::BatchExecutionFailed`] if batch creation fails.
    ///
    /// # Example
    /// ```ignore
    /// let batch_id = client.create_batch(&user, &operations, &BatchPriority::High, &ExecutionStrategy::Sequential);
    /// ```
    pub fn create_batch(
        env: Env,
        user: Address,
        operations: Vec<BatchOperation>,
        priority: BatchPriority,
        strategy: ExecutionStrategy,
    ) -> Result<String, MobileOptimizerError> {
        user.require_auth();
        BatchManager::create_batch(&env, user, operations, priority, strategy)
    }

    /// Execute a previously created batch and return the execution result.
    ///
    /// Requires authorization from `user`. Increments the global batch counter on success.
    ///
    /// # Arguments
    /// * `user` - Address of the batch owner.
    /// * `batch_id` - Unique identifier of the batch to execute.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::BatchNotFound`] if no batch matches the ID.
    ///
    /// # Example
    /// ```ignore
    /// let result = client.execute_batch(&user, &batch_id);
    /// ```
    pub fn execute_batch(
        env: Env,
        user: Address,
        batch_id: String,
    ) -> Result<BatchExecutionResult, MobileOptimizerError> {
        user.require_auth();
        let result = BatchManager::execute_batch(&env, batch_id, user)?;
        Self::increment_counter(&env, &DataKey::TotalBatches);
        Ok(result)
    }

    /// Cancel a pending batch before it is executed.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the batch owner.
    /// * `batch_id` - Unique identifier of the batch to cancel.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::BatchNotFound`] if no batch matches the ID.
    ///
    /// # Example
    /// ```ignore
    /// client.cancel_batch(&user, &batch_id);
    /// ```
    pub fn cancel_batch(
        env: Env,
        user: Address,
        batch_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        BatchManager::cancel_batch(&env, batch_id, user)
    }

    // ========================================================================
    // Gas Optimization
    // ========================================================================

    /// Estimate the gas cost for a list of operations given the current network quality.
    ///
    /// Returns a gas estimate for each operation in the same order as the input list.
    ///
    /// # Arguments
    /// * `operations` - List of operations to estimate gas for.
    /// * `network_quality` - Current network quality used to adjust estimates.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::GasEstimationFailed`] if estimation fails for any operation.
    ///
    /// # Example
    /// ```ignore
    /// let estimates = client.estimate_gas(&operations, &NetworkQuality::Good);
    /// ```
    pub fn estimate_gas(
        env: Env,
        operations: Vec<BatchOperation>,
        network_quality: NetworkQuality,
    ) -> Result<Vec<GasEstimate>, MobileOptimizerError> {
        let mut estimates = Vec::new(&env);
        for op in operations.iter() {
            let estimate = GasOptimizer::estimate_operation_gas(&env, &op, &network_quality)?;
            estimates.push_back(estimate);
        }
        Ok(estimates)
    }

    /// Return a list of mobile-specific gas optimization tips.
    ///
    /// # Example
    /// ```ignore
    /// let tips = client.get_gas_tips();
    /// ```
    pub fn get_gas_tips(env: Env) -> Vec<String> {
        GasOptimizer::get_mobile_gas_tips(&env)
    }

    // ========================================================================
    // Quick Interaction Flows
    // ========================================================================

    /// Enroll a user in a course via a mobile-optimized quick interaction flow.
    ///
    /// Requires authorization from `user`. Automatically detects network quality and
    /// adapts the interaction accordingly.
    ///
    /// # Arguments
    /// * `user` - Address of the user enrolling in the course.
    /// * `course_id` - Unique identifier of the course.
    /// * `session_id` - Active session ID associated with this interaction.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::InteractionFailed`] if the enrollment flow fails.
    ///
    /// # Example
    /// ```ignore
    /// let result = client.quick_enroll_course(&user, &course_id, &session_id);
    /// ```
    pub fn quick_enroll_course(
        env: Env,
        user: Address,
        course_id: String,
        session_id: String,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        user.require_auth();
        let nq = NetworkManager::detect_network_quality(&env);
        InteractionFlows::quick_enroll_course(&env, &user, &course_id, &session_id, &nq)
    }

    /// Record a user's progress on a course module via a mobile-optimized quick interaction flow.
    ///
    /// Requires authorization from `user`. Automatically detects network quality.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose progress is being updated.
    /// * `course_id` - Unique identifier of the course.
    /// * `module_id` - Unique identifier of the module within the course.
    /// * `progress_percentage` - Completion percentage (0–100) to record for the module.
    /// * `session_id` - Active session ID associated with this interaction.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::InteractionFailed`] if the progress update fails.
    ///
    /// # Example
    /// ```ignore
    /// let result = client.quick_update_progress(&user, &course_id, &module_id, &50u32, &session_id);
    /// ```
    pub fn quick_update_progress(
        env: Env,
        user: Address,
        course_id: String,
        module_id: String,
        progress_percentage: u32,
        session_id: String,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        user.require_auth();
        let nq = NetworkManager::detect_network_quality(&env);
        InteractionFlows::quick_update_progress(
            &env,
            &user,
            &course_id,
            &module_id,
            progress_percentage,
            &session_id,
            &nq,
        )
    }

    /// Claim a course completion certificate via a mobile-optimized quick interaction flow.
    ///
    /// Requires authorization from `user`. Automatically detects network quality.
    ///
    /// # Arguments
    /// * `user` - Address of the user claiming the certificate.
    /// * `course_id` - Unique identifier of the completed course.
    /// * `session_id` - Active session ID associated with this interaction.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::InteractionFailed`] if the certificate claim fails.
    ///
    /// # Example
    /// ```ignore
    /// let result = client.quick_claim_certificate(&user, &course_id, &session_id);
    /// ```
    pub fn quick_claim_certificate(
        env: Env,
        user: Address,
        course_id: String,
        session_id: String,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        user.require_auth();
        let nq = NetworkManager::detect_network_quality(&env);
        InteractionFlows::quick_claim_certificate(&env, &user, &course_id, &session_id, &nq)
    }

    // ========================================================================
    // Offline Operations
    // ========================================================================

    /// Queue an operation for later execution when the device is offline.
    ///
    /// Requires authorization from `user`. Increments the global offline operations counter.
    ///
    /// # Arguments
    /// * `user` - Address of the user queuing the operation.
    /// * `device_id` - Identifier of the device on which the operation is queued.
    /// * `operation` - The operation to queue for deferred execution.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::OfflineQueueFull`] if the queue has reached its limit.
    ///
    /// # Example
    /// ```ignore
    /// client.queue_offline_operation(&user, &device_id, &operation);
    /// ```
    pub fn queue_offline_operation(
        env: Env,
        user: Address,
        device_id: String,
        operation: QueuedOperation,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        OfflineManager::queue_operation(&env, user, device_id, operation)?;
        Self::increment_counter(&env, &DataKey::TotalOfflineOps);
        Ok(())
    }

    /// Sync all pending offline operations for a device to the network.
    ///
    /// Requires authorization from `user`. Detects current network quality before syncing.
    ///
    /// # Arguments
    /// * `user` - Address of the device owner.
    /// * `device_id` - Identifier of the device whose queue will be synced.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::OfflineSyncFailed`] if the sync operation fails.
    ///
    /// # Example
    /// ```ignore
    /// let result = client.sync_offline_operations(&user, &device_id);
    /// ```
    pub fn sync_offline_operations(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<OfflineSyncResult, MobileOptimizerError> {
        user.require_auth();
        let nq = NetworkManager::detect_network_quality(&env);
        OfflineManager::sync_offline_operations(&env, user, device_id, nq)
    }

    /// Return the current status of the offline operation queue for a device.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the device owner.
    /// * `device_id` - Identifier of the device whose queue status is requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::DeviceNotRegistered`] if the device is unknown.
    ///
    /// # Example
    /// ```ignore
    /// let status = client.get_offline_queue_status(&user, &device_id);
    /// ```
    pub fn get_offline_queue_status(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<OfflineQueueStatus, MobileOptimizerError> {
        user.require_auth();
        OfflineManager::get_queue_status(&env, &user, &device_id)
    }

    /// Resolve conflicting offline operations using the given strategy, returning the number of conflicts resolved.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the device owner.
    /// * `device_id` - Identifier of the device whose conflicts should be resolved.
    /// * `strategy` - Strategy to apply when two conflicting operations are found.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::ConflictResolutionFailed`] if resolution fails.
    ///
    /// # Example
    /// ```ignore
    /// let resolved = client.resolve_offline_conflicts(&user, &device_id, &ConflictResolution::LastWriteWins);
    /// ```
    pub fn resolve_offline_conflicts(
        env: Env,
        user: Address,
        device_id: String,
        strategy: ConflictResolution,
    ) -> Result<u32, MobileOptimizerError> {
        user.require_auth();
        OfflineManager::resolve_conflicts(&env, user, device_id, strategy)
    }

    /// Remove completed or expired offline operations from the queue, returning the number of entries removed.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the device owner.
    /// * `device_id` - Identifier of the device whose queue should be cleaned up.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::OfflineOperationFailed`] if cleanup fails.
    ///
    /// # Example
    /// ```ignore
    /// let removed = client.cleanup_offline_operations(&user, &device_id);
    /// ```
    pub fn cleanup_offline_operations(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<u32, MobileOptimizerError> {
        user.require_auth();
        OfflineManager::cleanup_completed_operations(&env, user, device_id)
    }

    /// Return the offline capabilities supported by this contract deployment.
    ///
    /// # Example
    /// ```ignore
    /// let caps = client.get_offline_capabilities();
    /// ```
    pub fn get_offline_capabilities(env: Env) -> OfflineCapabilities {
        OfflineManager::get_offline_capabilities(&env)
    }

    // ========================================================================
    // Network Management
    // ========================================================================

    /// Detect and return the current network quality experienced by the contract environment.
    ///
    /// # Example
    /// ```ignore
    /// let quality = client.get_network_quality();
    /// ```
    pub fn get_network_quality(env: Env) -> NetworkQuality {
        NetworkManager::detect_network_quality(&env)
    }

    /// Return optimized connection settings appropriate for the given network quality.
    ///
    /// # Arguments
    /// * `network_quality` - The current network quality level.
    ///
    /// # Example
    /// ```ignore
    /// let settings = client.get_connection_settings(&NetworkQuality::Poor);
    /// ```
    pub fn get_connection_settings(
        _env: Env,
        network_quality: NetworkQuality,
    ) -> ConnectionSettings {
        NetworkManager::optimize_connection_settings(&network_quality)
    }

    /// Return bandwidth optimization settings for the given network quality and data usage mode.
    ///
    /// # Arguments
    /// * `network_quality` - The current network quality level.
    /// * `data_usage_mode` - The user's preferred data usage mode (e.g. low-data, unrestricted).
    ///
    /// # Example
    /// ```ignore
    /// let opt = client.get_bandwidth_optimization(&NetworkQuality::Good, &DataUsageMode::LowData);
    /// ```
    pub fn get_bandwidth_optimization(
        env: Env,
        network_quality: NetworkQuality,
        data_usage_mode: DataUsageMode,
    ) -> BandwidthOptimization {
        NetworkManager::get_bandwidth_optimization(&env, &network_quality, &data_usage_mode)
    }

    /// Return network usage statistics collected during the specified session.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the session owner.
    /// * `session_id` - Unique identifier of the session to retrieve stats for.
    ///
    /// # Example
    /// ```ignore
    /// let stats = client.get_network_statistics(&user, &session_id);
    /// ```
    pub fn get_network_statistics(
        env: Env,
        user: Address,
        session_id: String,
    ) -> NetworkStatistics {
        user.require_auth();
        NetworkManager::get_network_statistics(&env, session_id)
    }

    /// Generate network adaptation recommendations when network quality changes.
    ///
    /// # Arguments
    /// * `previous_quality` - The network quality level before the change.
    /// * `current_quality` - The network quality level after the change.
    ///
    /// # Example
    /// ```ignore
    /// let adaptation = client.adapt_network(&NetworkQuality::Good, &NetworkQuality::Poor);
    /// ```
    pub fn adapt_network(
        env: Env,
        previous_quality: NetworkQuality,
        current_quality: NetworkQuality,
    ) -> NetworkAdaptation {
        NetworkManager::adapt_to_network_change(&env, &previous_quality, &current_quality)
    }

    // ========================================================================
    // Content Caching & Prefetching (NEW)
    // ========================================================================

    /// Store content in the mobile cache with a specified TTL.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user caching the content.
    /// * `cache_key` - Unique string key used to retrieve the cached content later.
    /// * `content_hash` - 32-byte hash of the content for integrity verification.
    /// * `content_type` - Type classification of the content being cached.
    /// * `size_bytes` - Size of the content in bytes.
    /// * `ttl_seconds` - Time-to-live in seconds before the cache entry expires.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CacheFull`] if the cache has no remaining capacity.
    ///
    /// # Example
    /// ```ignore
    /// client.cache_content(&user, &cache_key, &content_hash, &ContentType::Video, &1024u64, &86400u64);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn cache_content(
        env: Env,
        user: Address,
        cache_key: String,
        content_hash: BytesN<32>,
        content_type: ContentType,
        size_bytes: u64,
        ttl_seconds: u64,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        ContentCacheManager::cache_content(
            &env,
            &user,
            cache_key,
            content_hash,
            content_type,
            size_bytes,
            ttl_seconds,
        )
    }

    /// Retrieve a cached content entry by its cache key.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user who owns the cache entry.
    /// * `cache_key` - Key used when the content was stored.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CacheError`] if the entry is not found or has expired.
    ///
    /// # Example
    /// ```ignore
    /// let entry = client.get_cached_content(&user, &cache_key);
    /// ```
    pub fn get_cached_content(
        env: Env,
        user: Address,
        cache_key: String,
    ) -> Result<CacheEntry, MobileOptimizerError> {
        user.require_auth();
        ContentCacheManager::get_cached_content(&env, &user, cache_key)
    }

    /// Immediately invalidate a cached entry, forcing it to be refreshed on the next access.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user who owns the cache entry.
    /// * `cache_key` - Key of the cache entry to invalidate.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CacheError`] if the entry does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.invalidate_cache(&user, &cache_key);
    /// ```
    pub fn invalidate_cache(
        env: Env,
        user: Address,
        cache_key: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        ContentCacheManager::invalidate_cache(&env, &user, cache_key)
    }

    /// Configure prefetch rules that determine which content should be proactively cached.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user setting up prefetch rules.
    /// * `rules` - List of prefetch rules to store and apply on trigger events.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CacheError`] if the rules cannot be persisted.
    ///
    /// # Example
    /// ```ignore
    /// client.setup_prefetch_rules(&user, &rules);
    /// ```
    pub fn setup_prefetch_rules(
        env: Env,
        user: Address,
        rules: Vec<PrefetchRule>,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        ContentCacheManager::setup_prefetch_rules(&env, &user, rules)
    }

    /// Execute prefetch rules matching the given trigger and return the number of items prefetched.
    ///
    /// Requires authorization from `user`. Only prefetches when network quality is sufficient.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose prefetch rules should be evaluated.
    /// * `trigger` - Event that activated the prefetch (e.g. session start, course open).
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CacheError`] if prefetching fails.
    ///
    /// # Example
    /// ```ignore
    /// let count = client.execute_prefetch(&user, &PrefetchTrigger::SessionStart);
    /// ```
    pub fn execute_prefetch(
        env: Env,
        user: Address,
        trigger: PrefetchTrigger,
    ) -> Result<u32, MobileOptimizerError> {
        user.require_auth();
        let nq = NetworkManager::detect_network_quality(&env);
        ContentCacheManager::execute_prefetch(&env, &user, trigger, &nq)
    }

    /// Return cache usage statistics (hit rate, size, entry count) for the user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose cache stats are requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CacheError`] if stats cannot be computed.
    ///
    /// # Example
    /// ```ignore
    /// let stats = client.get_cache_stats(&user);
    /// ```
    pub fn get_cache_stats(env: Env, user: Address) -> Result<CacheStats, MobileOptimizerError> {
        user.require_auth();
        ContentCacheManager::get_cache_stats(&env, &user)
    }

    // ========================================================================
    // Battery Optimization (NEW)
    // ========================================================================

    /// Update the battery profile for a device and return the updated profile.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the device owner.
    /// * `device_id` - Identifier of the device to update.
    /// * `battery_level` - Current battery charge level as a percentage (0–100).
    /// * `is_charging` - Whether the device is currently charging.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::DeviceNotRegistered`] if the device is not registered.
    ///
    /// # Example
    /// ```ignore
    /// let profile = client.update_battery_profile(&user, &device_id, &42u32, &false);
    /// ```
    pub fn update_battery_profile(
        env: Env,
        user: Address,
        device_id: String,
        battery_level: u32,
        is_charging: bool,
    ) -> Result<BatteryProfile, MobileOptimizerError> {
        user.require_auth();
        BatteryOptimizer::update_battery_profile(&env, &user, device_id, battery_level, is_charging)
    }

    /// Return battery-optimized operation settings based on the device's current battery profile.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the device owner.
    /// * `device_id` - Identifier of the device to generate settings for.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::DeviceNotRegistered`] if no battery profile exists for the device.
    ///
    /// # Example
    /// ```ignore
    /// let settings = client.get_battery_optimized_settings(&user, &device_id);
    /// ```
    pub fn get_battery_optimized_settings(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<BatteryOptimizedSettings, MobileOptimizerError> {
        user.require_auth();
        BatteryOptimizer::get_optimized_settings(&env, &user, &device_id)
    }

    /// Estimate the battery impact of a session's planned operations and return a detailed report.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session to estimate battery impact for.
    /// * `operations_count` - Number of contract operations planned for the session.
    /// * `sync_count` - Number of offline sync operations planned.
    /// * `cache_operations` - Number of cache read/write operations planned.
    ///
    /// # Example
    /// ```ignore
    /// let report = client.estimate_battery_impact(&session_id, &10u32, &2u32, &5u32);
    /// ```
    pub fn estimate_battery_impact(
        env: Env,
        session_id: String,
        operations_count: u32,
        sync_count: u32,
        cache_operations: u32,
    ) -> BatteryImpactReport {
        BatteryOptimizer::estimate_session_battery_impact(
            &env,
            &session_id,
            operations_count,
            sync_count,
            cache_operations,
        )
    }

    /// Update the battery optimization configuration for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user updating the configuration.
    /// * `config` - New battery optimization configuration to apply.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::InternalError`] if the configuration cannot be stored.
    ///
    /// # Example
    /// ```ignore
    /// client.update_battery_config(&user, &battery_config);
    /// ```
    pub fn update_battery_config(
        env: Env,
        user: Address,
        config: BatteryOptimizationConfig,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        BatteryOptimizer::update_battery_config(&env, &user, config)
    }

    // ========================================================================
    // Push Notifications & Reminders (NEW)
    // ========================================================================

    /// Create a new push notification reminder for a learning activity.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user who will receive the reminder.
    /// * `reminder_type` - Category of the reminder (e.g. deadline, streak, achievement).
    /// * `title` - Short title text for the notification.
    /// * `message` - Full body message for the notification.
    /// * `scheduled_at` - Unix timestamp at which the reminder should be delivered.
    /// * `repeat_interval` - How often the reminder should repeat after first delivery.
    /// * `course_id` - Identifier of the course the reminder relates to.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::NotificationError`] if the reminder cannot be created.
    ///
    /// # Example
    /// ```ignore
    /// let reminder = client.create_learning_reminder(&user, &ReminderType::Deadline, &title, &message, &1234567890u64, &RepeatInterval::Daily, &course_id);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn create_learning_reminder(
        env: Env,
        user: Address,
        reminder_type: ReminderType,
        title: String,
        message: String,
        scheduled_at: u64,
        repeat_interval: RepeatInterval,
        course_id: String,
    ) -> Result<LearningReminder, MobileOptimizerError> {
        user.require_auth();
        NotificationManager::create_learning_reminder(
            &env,
            &user,
            reminder_type,
            title,
            message,
            scheduled_at,
            repeat_interval,
            course_id,
        )
    }

    /// Return all pending (unsent) notifications scheduled for the user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose pending notifications are requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::NotificationError`] if the query fails.
    ///
    /// # Example
    /// ```ignore
    /// let notifications = client.get_pending_notifications(&user);
    /// ```
    pub fn get_pending_notifications(
        env: Env,
        user: Address,
    ) -> Result<Vec<LearningReminder>, MobileOptimizerError> {
        user.require_auth();
        NotificationManager::get_pending_notifications(&env, &user)
    }

    /// Mark a notification as successfully sent so it is not delivered again.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the notification owner.
    /// * `reminder_id` - Unique identifier of the reminder to mark as sent.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::NotificationError`] if the reminder is not found.
    ///
    /// # Example
    /// ```ignore
    /// client.mark_notification_sent(&user, &reminder_id);
    /// ```
    pub fn mark_notification_sent(
        env: Env,
        user: Address,
        reminder_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        NotificationManager::mark_notification_sent(&env, &user, reminder_id)
    }

    /// Cancel a scheduled reminder so it will not be delivered.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the notification owner.
    /// * `reminder_id` - Unique identifier of the reminder to cancel.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::NotificationError`] if the reminder is not found.
    ///
    /// # Example
    /// ```ignore
    /// client.cancel_reminder(&user, &reminder_id);
    /// ```
    pub fn cancel_reminder(
        env: Env,
        user: Address,
        reminder_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        NotificationManager::cancel_reminder(&env, &user, reminder_id)
    }

    /// Update the push notification preferences for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user updating their notification config.
    /// * `config` - New notification configuration to persist.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::NotificationError`] if the config cannot be stored.
    ///
    /// # Example
    /// ```ignore
    /// client.update_notification_config(&user, &notification_config);
    /// ```
    pub fn update_notification_config(
        env: Env,
        user: Address,
        config: NotificationConfig,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        NotificationManager::update_notification_config(&env, &user, config)
    }

    /// Retrieve the current push notification configuration for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose notification config is requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::NotificationError`] if no config record exists.
    ///
    /// # Example
    /// ```ignore
    /// let config = client.get_notification_config(&user);
    /// ```
    pub fn get_notification_config(
        env: Env,
        user: Address,
    ) -> Result<NotificationConfig, MobileOptimizerError> {
        user.require_auth();
        NotificationManager::get_notification_config(&env, &user)
    }

    /// Create a streak-maintenance reminder celebrating a user's current learning streak.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user to remind.
    /// * `streak_days` - Number of consecutive days the user has maintained their learning streak.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::NotificationError`] if the reminder cannot be created.
    ///
    /// # Example
    /// ```ignore
    /// let reminder = client.create_streak_reminder(&user, &7u32);
    /// ```
    pub fn create_streak_reminder(
        env: Env,
        user: Address,
        streak_days: u32,
    ) -> Result<LearningReminder, MobileOptimizerError> {
        user.require_auth();
        NotificationManager::create_streak_reminder(&env, &user, streak_days)
    }

    /// Create a reusable notification template for admin-managed campaigns.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    /// * `template_id` - Unique identifier for the new template.
    /// * `category` - Reminder type category this template applies to.
    /// * `default_content` - Default notification body text used when no localized variant matches.
    /// * `localized_content` - Map of locale codes to localized notification body text.
    /// * `supported_channels` - Delivery channels (e.g. push, email) this template supports.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::UnauthorizedAdmin`] if `admin` is not the stored admin.
    ///
    /// # Example
    /// ```ignore
    /// let template = client.create_notification_template(&admin, &template_id, &ReminderType::Streak, &default_content, &localized_content, &channels);
    /// ```
    pub fn create_notification_template(
        env: Env,
        admin: Address,
        template_id: String,
        category: ReminderType,
        default_content: String,
        localized_content: Map<String, String>,
        supported_channels: Vec<String>,
    ) -> Result<NotificationTemplate, MobileOptimizerError> {
        Self::require_admin(&env, &admin)?;
        NotificationManager::create_notification_template(
            &env,
            template_id,
            category,
            default_content,
            localized_content,
            supported_channels,
        )
    }

    /// Create an A/B-tested notification campaign managed by the admin.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    /// * `campaign_id` - Unique identifier for the campaign.
    /// * `name` - Human-readable name for the campaign.
    /// * `variants` - A/B test variants to use in the campaign.
    /// * `start_date` - Unix timestamp when the campaign starts.
    /// * `end_date` - Unix timestamp when the campaign ends.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::UnauthorizedAdmin`] if `admin` is not the stored admin.
    ///
    /// # Example
    /// ```ignore
    /// let campaign = client.create_notification_campaign(&admin, &campaign_id, &name, &variants, &start_date, &end_date);
    /// ```
    pub fn create_notification_campaign(
        env: Env,
        admin: Address,
        campaign_id: String,
        name: String,
        variants: Vec<ABTestVariant>,
        start_date: u64,
        end_date: u64,
    ) -> Result<NotificationCampaign, MobileOptimizerError> {
        Self::require_admin(&env, &admin)?;
        NotificationManager::create_campaign(
            &env,
            campaign_id,
            name,
            variants,
            start_date,
            end_date,
        )
    }

    /// Record that a user interacted with (opened or tapped) a notification.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user who engaged with the notification.
    /// * `notification_id` - Unique identifier of the notification that was engaged with.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::NotificationError`] if tracking fails.
    ///
    /// # Example
    /// ```ignore
    /// client.track_notification_engagement(&user, &notification_id);
    /// ```
    pub fn track_notification_engagement(
        env: Env,
        user: Address,
        notification_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        NotificationManager::track_engagement(&env, &user, notification_id)
    }

    // ========================================================================
    // Content Management (NEW)
    // ========================================================================

    /// Publish new content to the platform and return its metadata record.
    ///
    /// Requires authorization from `author`.
    ///
    /// # Arguments
    /// * `author` - Address of the content creator.
    /// * `content_id` - Unique identifier for the new content piece.
    /// * `content_type` - Type classification of the content (video, document, etc.).
    /// * `title` - Human-readable title for the content.
    /// * `uri` - URI where the content is hosted or stored.
    /// * `access_rule` - Access control rule governing who may retrieve the content.
    /// * `delivery_config` - Configuration for how the content should be delivered to mobile clients.
    /// * `content_hash` - 32-byte hash of the content for integrity verification.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::ContentError`] if the content cannot be published.
    ///
    /// # Example
    /// ```ignore
    /// let metadata = client.publish_content(&author, &content_id, &ContentType::Video, &title, &uri, &access_rule, &delivery_config, &content_hash);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn publish_content(
        env: Env,
        author: Address,
        content_id: String,
        content_type: ContentType,
        title: String,
        uri: String,
        access_rule: ContentAccessRule,
        delivery_config: ContentDeliveryConfig,
        content_hash: BytesN<32>,
    ) -> Result<ContentMetadata, MobileOptimizerError> {
        author.require_auth();
        ContentManager::publish_content(
            &env,
            &author,
            content_id,
            content_type,
            title,
            uri,
            access_rule,
            delivery_config,
            content_hash,
        )
    }

    /// Publish a new version of existing content and return the version record.
    ///
    /// Requires authorization from `author`.
    ///
    /// # Arguments
    /// * `author` - Address of the content creator (must match the original author).
    /// * `content_id` - Identifier of the content to update.
    /// * `new_uri` - URI pointing to the new version of the content.
    /// * `content_hash` - 32-byte hash of the new content version.
    /// * `changelog` - Human-readable description of what changed in this version.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::ContentError`] if the content is not found or the update fails.
    ///
    /// # Example
    /// ```ignore
    /// let version = client.update_content_version(&author, &content_id, &new_uri, &content_hash, &changelog);
    /// ```
    pub fn update_content_version(
        env: Env,
        author: Address,
        content_id: String,
        new_uri: String,
        content_hash: BytesN<32>,
        changelog: String,
    ) -> Result<ContentVersion, MobileOptimizerError> {
        author.require_auth();
        ContentManager::update_content_version(
            &env,
            &author,
            content_id,
            new_uri,
            content_hash,
            changelog,
        )
    }

    /// Retrieve the metadata record for a previously published content item.
    ///
    /// # Arguments
    /// * `content_id` - Unique identifier of the content to look up.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::ContentError`] if no content matches the ID.
    ///
    /// # Example
    /// ```ignore
    /// let metadata = client.get_content_metadata(&content_id);
    /// ```
    pub fn get_content_metadata(
        env: Env,
        content_id: String,
    ) -> Result<ContentMetadata, MobileOptimizerError> {
        ContentManager::get_content(&env, content_id)
    }

    /// Retrieve the full version history for a content item.
    ///
    /// # Arguments
    /// * `content_id` - Unique identifier of the content whose history is requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::ContentError`] if no content matches the ID.
    ///
    /// # Example
    /// ```ignore
    /// let history = client.get_content_history(&content_id);
    /// ```
    pub fn get_content_history(
        env: Env,
        content_id: String,
    ) -> Result<Vec<ContentVersion>, MobileOptimizerError> {
        ContentManager::get_version_history(&env, content_id)
    }

    // ========================================================================
    // Collaboration & Social (NEW)
    // ========================================================================

    /// Create a new study group and return its record.
    ///
    /// Requires authorization from `creator`.
    ///
    /// # Arguments
    /// * `creator` - Address of the user creating the group.
    /// * `group_id` - Unique identifier for the new study group.
    /// * `name` - Human-readable name for the group.
    /// * `topic` - Subject or topic the group focuses on.
    /// * `max_members` - Maximum number of members allowed in the group.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CollaborationError`] if creation fails.
    ///
    /// # Example
    /// ```ignore
    /// let group = client.create_study_group(&creator, &group_id, &name, &topic, &20u32);
    /// ```
    pub fn create_study_group(
        env: Env,
        creator: Address,
        group_id: String,
        name: String,
        topic: String,
        max_members: u32,
    ) -> Result<StudyGroup, MobileOptimizerError> {
        creator.require_auth();
        CollaborationManager::create_study_group(&env, &creator, group_id, name, topic, max_members)
    }

    /// Add a user as a member of an existing study group.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user joining the group.
    /// * `group_id` - Unique identifier of the study group to join.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CollaborationError`] if the group is full or not found.
    ///
    /// # Example
    /// ```ignore
    /// client.join_study_group(&user, &group_id);
    /// ```
    pub fn join_study_group(
        env: Env,
        user: Address,
        group_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        CollaborationManager::join_study_group(&env, &user, group_id)
    }

    /// Create a forum post (or reply) within a study group and return the post record.
    ///
    /// Requires authorization from `author`.
    ///
    /// # Arguments
    /// * `author` - Address of the post author.
    /// * `post_id` - Unique identifier for the new post.
    /// * `group_id` - Study group the post belongs to.
    /// * `content` - Text body of the post.
    /// * `parent_id` - Optional ID of the parent post if this is a reply.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CollaborationError`] if the post cannot be created.
    ///
    /// # Example
    /// ```ignore
    /// let post = client.create_forum_post(&author, &post_id, &group_id, &content, &None);
    /// ```
    pub fn create_forum_post(
        env: Env,
        author: Address,
        post_id: String,
        group_id: String,
        content: String,
        parent_id: Option<String>,
    ) -> Result<ForumPost, MobileOptimizerError> {
        author.require_auth();
        CollaborationManager::create_post(&env, &author, post_id, group_id, content, parent_id)
    }

    /// Submit a peer review for another user's work and return the review record.
    ///
    /// Requires authorization from `reviewer`.
    ///
    /// # Arguments
    /// * `reviewer` - Address of the user submitting the review.
    /// * `target_user` - Address of the user whose work is being reviewed.
    /// * `review_id` - Unique identifier for this review.
    /// * `context_id` - Identifier of the assignment or submission being reviewed.
    /// * `score` - Numeric score awarded by the reviewer.
    /// * `comments` - Written feedback accompanying the score.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CollaborationError`] if the review cannot be submitted.
    ///
    /// # Example
    /// ```ignore
    /// let review = client.submit_peer_review(&reviewer, &target_user, &review_id, &context_id, &85u32, &comments);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn submit_peer_review(
        env: Env,
        reviewer: Address,
        target_user: Address,
        review_id: String,
        context_id: String,
        score: u32,
        comments: String,
    ) -> Result<PeerReview, MobileOptimizerError> {
        reviewer.require_auth();
        CollaborationManager::submit_review(
            &env,
            &reviewer,
            &target_user,
            review_id,
            context_id,
            score,
            comments,
        )
    }

    /// Request a mentorship session with a specified mentor and return the session record.
    ///
    /// Requires authorization from `mentee`.
    ///
    /// # Arguments
    /// * `mentee` - Address of the user requesting mentorship.
    /// * `mentor` - Address of the mentor being requested.
    /// * `session_id` - Unique identifier for the mentorship session.
    /// * `topic` - Subject or focus area for the mentorship session.
    /// * `scheduled_at` - Unix timestamp when the session is scheduled to begin.
    /// * `duration_minutes` - Expected duration of the session in minutes.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CollaborationError`] if the request cannot be created.
    ///
    /// # Example
    /// ```ignore
    /// let session = client.request_mentorship(&mentee, &mentor, &session_id, &topic, &1234567890u64, &60u32);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn request_mentorship(
        env: Env,
        mentee: Address,
        mentor: Address,
        session_id: String,
        topic: String,
        scheduled_at: u64,
        duration_minutes: u32,
    ) -> Result<MentorshipSession, MobileOptimizerError> {
        mentee.require_auth();
        CollaborationManager::request_mentorship(
            &env,
            &mentee,
            &mentor,
            session_id,
            topic,
            scheduled_at,
            duration_minutes,
        )
    }

    /// Update the status of an existing mentorship session (e.g. accept, decline, complete).
    ///
    /// Requires authorization from `caller` (either the mentor or mentee).
    ///
    /// # Arguments
    /// * `caller` - Address of the mentor or mentee updating the status.
    /// * `session_id` - Unique identifier of the mentorship session.
    /// * `new_status` - The new status to assign to the session.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::CollaborationError`] if the session is not found or the transition is invalid.
    ///
    /// # Example
    /// ```ignore
    /// client.update_mentorship_status(&mentor, &session_id, &MentorshipStatus::Accepted);
    /// ```
    pub fn update_mentorship_status(
        env: Env,
        caller: Address,
        session_id: String,
        new_status: MentorshipStatus,
    ) -> Result<(), MobileOptimizerError> {
        caller.require_auth();
        CollaborationManager::update_mentorship_status(&env, &caller, session_id, new_status)
    }

    /// Retrieve the collaboration profile for a user, including group memberships and review history.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose collaboration profile is requested.
    ///
    /// # Example
    /// ```ignore
    /// let profile = client.get_collaboration_profile(&user);
    /// ```
    pub fn get_collaboration_profile(env: Env, user: Address) -> CollaborationProfile {
        CollaborationManager::get_profile(&env, &user)
    }

    // ========================================================================
    // User Experience & UI (NEW)
    // ========================================================================

    /// Set or replace the UI preferences for a user and return the stored preferences.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user updating their UI preferences.
    /// * `theme_id` - Identifier of the visual theme to apply.
    /// * `language` - Locale/language code (e.g. `"en"`, `"fr"`).
    /// * `font_scale` - Font size multiplier as a percentage (e.g. 100 = default).
    /// * `high_contrast` - Whether high-contrast mode should be enabled.
    /// * `reduce_motion` - Whether reduced-motion mode should be enabled.
    /// * `layout_mode` - Preferred layout mode (e.g. list, grid).
    /// * `accessibility_settings` - Map of accessibility feature flags.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::UserExperienceError`] if the preferences cannot be stored.
    ///
    /// # Example
    /// ```ignore
    /// let prefs = client.set_ui_preferences(&user, &theme_id, &"en", &100u32, &false, &false, &LayoutMode::List, &settings);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn set_ui_preferences(
        env: Env,
        user: Address,
        theme_id: String,
        language: String,
        font_scale: u32,
        high_contrast: bool,
        reduce_motion: bool,
        layout_mode: LayoutMode,
        accessibility_settings: Map<String, bool>,
    ) -> Result<UiPreferences, MobileOptimizerError> {
        user.require_auth();
        UserExperienceManager::set_ui_preferences(
            &env,
            &user,
            theme_id,
            language,
            font_scale,
            high_contrast,
            reduce_motion,
            layout_mode,
            accessibility_settings,
        )
    }

    /// Retrieve the stored UI preferences for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose preferences are requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::UserExperienceError`] if no preferences are found.
    ///
    /// # Example
    /// ```ignore
    /// let prefs = client.get_ui_preferences(&user);
    /// ```
    pub fn get_ui_preferences(
        env: Env,
        user: Address,
    ) -> Result<UiPreferences, MobileOptimizerError> {
        user.require_auth();
        UserExperienceManager::get_ui_preferences(&env, &user)
    }

    /// Mark an onboarding step as completed or skipped and return the updated onboarding state.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user completing the onboarding step.
    /// * `step_id` - Unique identifier of the onboarding step.
    /// * `is_skipped` - Whether the step was skipped rather than completed normally.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::UserExperienceError`] if the state cannot be updated.
    ///
    /// # Example
    /// ```ignore
    /// let state = client.update_onboarding_progress(&user, &step_id, &false);
    /// ```
    pub fn update_onboarding_progress(
        env: Env,
        user: Address,
        step_id: String,
        is_skipped: bool,
    ) -> Result<OnboardingState, MobileOptimizerError> {
        user.require_auth();
        UserExperienceManager::update_onboarding_progress(&env, &user, step_id, is_skipped)
    }

    /// Submit user feedback for a specific category and return the recorded feedback entry.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user submitting feedback.
    /// * `category` - Category label for the feedback (e.g. `"usability"`, `"performance"`).
    /// * `rating` - Numeric rating score provided by the user.
    /// * `comment` - Written feedback comment.
    /// * `context_data` - Additional key-value context attached to the feedback entry.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::UserExperienceError`] if the feedback cannot be stored.
    ///
    /// # Example
    /// ```ignore
    /// let feedback = client.submit_user_feedback(&user, &"usability", &4u32, &comment, &context_data);
    /// ```
    pub fn submit_user_feedback(
        env: Env,
        user: Address,
        category: String,
        rating: u32,
        comment: String,
        context_data: Map<String, String>,
    ) -> Result<UserFeedback, MobileOptimizerError> {
        user.require_auth();
        UserExperienceManager::submit_feedback(&env, &user, category, rating, comment, context_data)
    }

    // ========================================================================
    // Mobile Security & Biometric Auth (NEW)
    // ========================================================================

    /// Enable biometric authentication for a user using the specified biometric type.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user enabling biometric auth.
    /// * `biometric_type` - The type of biometric factor to enable (e.g. fingerprint, face).
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::BiometricAuthFailed`] if setup fails.
    ///
    /// # Example
    /// ```ignore
    /// client.enable_biometric_auth(&user, &BiometricType::Fingerprint);
    /// ```
    pub fn enable_biometric_auth(
        env: Env,
        user: Address,
        biometric_type: BiometricType,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::enable_biometric_auth(&env, &user, biometric_type)
    }

    /// Record an authentication attempt for a user and return the resulting authentication event.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user authenticating.
    /// * `device_id` - Identifier of the device used for authentication.
    /// * `auth_method` - Authentication method used (e.g. password, biometric, 2FA).
    /// * `ip_hash` - 32-byte hash of the originating IP address for audit purposes.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SecurityViolation`] if the authentication fails or the account is locked.
    ///
    /// # Example
    /// ```ignore
    /// let event = client.authenticate(&user, &device_id, &AuthMethod::Biometric, &ip_hash);
    /// ```
    pub fn authenticate(
        env: Env,
        user: Address,
        device_id: String,
        auth_method: AuthMethod,
        ip_hash: BytesN<32>,
    ) -> Result<AuthenticationEvent, MobileOptimizerError> {
        user.require_auth();
        SecurityManager::authenticate(&env, &user, device_id, auth_method, ip_hash)
    }

    /// Register a device as trusted for a user so it is not challenged on future logins.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user adding the trusted device.
    /// * `device_id` - Identifier of the device to trust.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::MaxDevicesReached`] if the user has reached the device limit.
    ///
    /// # Example
    /// ```ignore
    /// client.register_trusted_device(&user, &device_id);
    /// ```
    pub fn register_trusted_device(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::register_trusted_device(&env, &user, device_id)
    }

    /// Remove a previously trusted device from the user's trusted-device list.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user revoking the device.
    /// * `device_id` - Identifier of the device to revoke.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::DeviceNotRegistered`] if the device is not found.
    ///
    /// # Example
    /// ```ignore
    /// client.revoke_trusted_device(&user, &device_id);
    /// ```
    pub fn revoke_trusted_device(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::revoke_trusted_device(&env, &user, device_id)
    }

    /// Retrieve the security profile for a user, including auth settings and trusted devices.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose security profile is requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SecurityViolation`] if the profile cannot be retrieved.
    ///
    /// # Example
    /// ```ignore
    /// let profile = client.get_security_profile(&user);
    /// ```
    pub fn get_security_profile(
        env: Env,
        user: Address,
    ) -> Result<SecurityProfile, MobileOptimizerError> {
        user.require_auth();
        SecurityManager::get_security_profile(&env, &user)
    }

    /// Return the list of active security alerts for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose security alerts are requested.
    ///
    /// # Example
    /// ```ignore
    /// let alerts = client.get_security_alerts(&user);
    /// ```
    pub fn get_security_alerts(env: Env, user: Address) -> Vec<SecurityAlert> {
        user.require_auth();
        SecurityManager::get_security_alerts(&env, &user)
    }

    /// Mark a security alert as resolved so it no longer appears in the active alerts list.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the alert owner.
    /// * `alert_id` - Unique identifier of the alert to resolve.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SecurityViolation`] if the alert is not found.
    ///
    /// # Example
    /// ```ignore
    /// client.resolve_security_alert(&user, &alert_id);
    /// ```
    pub fn resolve_security_alert(
        env: Env,
        user: Address,
        alert_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::resolve_security_alert(&env, &user, alert_id)
    }

    /// Enable two-factor authentication for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user enabling 2FA.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::SecurityViolation`] if 2FA cannot be enabled.
    ///
    /// # Example
    /// ```ignore
    /// client.enable_two_factor(&user);
    /// ```
    pub fn enable_two_factor(env: Env, user: Address) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::enable_two_factor(&env, &user)
    }

    // ========================================================================
    // PWA Capabilities (NEW)
    // ========================================================================

    /// Retrieve the Progressive Web App configuration for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose PWA config is requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::PwaError`] if no config is found.
    ///
    /// # Example
    /// ```ignore
    /// let config = client.get_pwa_config(&user);
    /// ```
    pub fn get_pwa_config(env: Env, user: Address) -> Result<PwaConfig, MobileOptimizerError> {
        user.require_auth();
        PwaManager::get_pwa_config(&env, &user)
    }

    /// Update the PWA installation status for a user (e.g. installed, uninstalled).
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user updating their PWA install status.
    /// * `status` - The new PWA installation status.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::PwaError`] if the status cannot be persisted.
    ///
    /// # Example
    /// ```ignore
    /// client.update_pwa_install_status(&user, &PwaInstallStatus::Installed);
    /// ```
    pub fn update_pwa_install_status(
        env: Env,
        user: Address,
        status: PwaInstallStatus,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        PwaManager::update_install_status(&env, &user, status)
    }

    /// Return the PWA manifest describing the application's metadata and capabilities.
    ///
    /// # Example
    /// ```ignore
    /// let manifest = client.get_pwa_manifest();
    /// ```
    pub fn get_pwa_manifest(env: Env) -> PwaManifest {
        PwaManager::get_pwa_manifest(&env)
    }

    /// Register a new service worker version for a user and return its status.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user updating their service worker.
    /// * `version` - Version string of the new service worker (e.g. `"1.2.3"`).
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::PwaError`] if the update fails.
    ///
    /// # Example
    /// ```ignore
    /// let status = client.update_service_worker(&user, &"1.2.3");
    /// ```
    pub fn update_service_worker(
        env: Env,
        user: Address,
        version: String,
    ) -> Result<ServiceWorkerStatus, MobileOptimizerError> {
        user.require_auth();
        PwaManager::update_service_worker(&env, &user, version)
    }

    /// Retrieve the current service worker status for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose service worker status is requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::PwaError`] if no service worker status is found.
    ///
    /// # Example
    /// ```ignore
    /// let status = client.get_service_worker_status(&user);
    /// ```
    pub fn get_service_worker_status(
        env: Env,
        user: Address,
    ) -> Result<ServiceWorkerStatus, MobileOptimizerError> {
        user.require_auth();
        PwaManager::get_service_worker_status(&env, &user)
    }

    /// Register a URL route to be cached by the service worker for offline access.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user registering the route.
    /// * `route` - URL route string to cache.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::PwaError`] if the route cannot be registered.
    ///
    /// # Example
    /// ```ignore
    /// client.register_cached_route(&user, &"/courses");
    /// ```
    pub fn register_cached_route(
        env: Env,
        user: Address,
        route: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        PwaManager::register_cached_route(&env, &user, route)
    }

    /// Register an HTML page to be served when the user is offline.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user registering the offline page.
    /// * `page` - URL path of the page to serve offline.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::PwaError`] if the page cannot be registered.
    ///
    /// # Example
    /// ```ignore
    /// client.register_offline_page(&user, &"/offline.html");
    /// ```
    pub fn register_offline_page(
        env: Env,
        user: Address,
        page: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        PwaManager::register_offline_page(&env, &user, page)
    }

    /// Enable or disable background sync for a user's PWA installation.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user toggling background sync.
    /// * `enabled` - `true` to enable background sync, `false` to disable it.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::PwaError`] if the setting cannot be persisted.
    ///
    /// # Example
    /// ```ignore
    /// client.toggle_background_sync(&user, &true);
    /// ```
    pub fn toggle_background_sync(
        env: Env,
        user: Address,
        enabled: bool,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        PwaManager::toggle_background_sync(&env, &user, enabled)
    }

    /// Return a report summarising the offline capabilities available to a user's PWA.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose offline capability report is requested.
    ///
    /// # Example
    /// ```ignore
    /// let report = client.get_offline_capability_report(&user);
    /// ```
    pub fn get_offline_capability_report(env: Env, user: Address) -> OfflineCapabilityReport {
        user.require_auth();
        PwaManager::get_offline_capability_report(&env, &user)
    }

    // ========================================================================
    // Analytics & Performance Monitoring (NEW)
    // ========================================================================

    /// Record an analytics event for a user session and return the stored event record.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user associated with the event.
    /// * `event_type` - Classification of the analytics event.
    /// * `properties` - Key-value map of additional properties for the event.
    /// * `session_id` - Active session ID during which the event occurred.
    /// * `device_type` - Type of device that generated the event.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::AnalyticsNotAvailable`] if tracking fails.
    ///
    /// # Example
    /// ```ignore
    /// let event = client.track_analytics_event(&user, &AnalyticsEventType::PageView, &props, &session_id, &DeviceType::Mobile);
    /// ```
    pub fn track_analytics_event(
        env: Env,
        user: Address,
        event_type: AnalyticsEventType,
        properties: Map<String, String>,
        session_id: String,
        device_type: DeviceType,
    ) -> Result<AnalyticsEvent, MobileOptimizerError> {
        user.require_auth();
        AnalyticsMonitor::track_event(&env, &user, event_type, properties, session_id, device_type)
    }

    /// Record mobile performance metrics for a user session.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose metrics are being recorded.
    /// * `metrics` - Performance metrics data to persist.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::AnalyticsNotAvailable`] if the metrics cannot be stored.
    ///
    /// # Example
    /// ```ignore
    /// client.record_performance_metrics(&user, &metrics);
    /// ```
    pub fn record_performance_metrics(
        env: Env,
        user: Address,
        metrics: PerformanceMetrics,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        AnalyticsMonitor::record_performance_metrics(&env, &user, metrics)
    }

    /// Update engagement counters for a user and return the updated engagement record.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose engagement is being updated.
    /// * `session_duration_seconds` - Duration of the latest session in seconds.
    /// * `courses_accessed` - Number of courses accessed during the session.
    /// * `modules_completed` - Number of modules completed during the session.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::AnalyticsNotAvailable`] if the engagement record cannot be updated.
    ///
    /// # Example
    /// ```ignore
    /// let engagement = client.update_user_engagement(&user, &3600u64, &2u32, &5u32);
    /// ```
    pub fn update_user_engagement(
        env: Env,
        user: Address,
        session_duration_seconds: u64,
        courses_accessed: u32,
        modules_completed: u32,
    ) -> Result<UserEngagement, MobileOptimizerError> {
        user.require_auth();
        AnalyticsMonitor::update_user_engagement(
            &env,
            &user,
            session_duration_seconds,
            courses_accessed,
            modules_completed,
        )
    }

    /// Retrieve the stored engagement record for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose engagement data is requested.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::AnalyticsNotAvailable`] if no engagement record exists.
    ///
    /// # Example
    /// ```ignore
    /// let engagement = client.get_user_engagement(&user);
    /// ```
    pub fn get_user_engagement(
        env: Env,
        user: Address,
    ) -> Result<UserEngagement, MobileOptimizerError> {
        user.require_auth();
        AnalyticsMonitor::get_user_engagement(&env, &user)
    }

    /// Retrieve aggregated mobile analytics for a user and device within a time range.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose analytics are requested.
    /// * `device_id` - Identifier of the device to filter analytics for.
    /// * `period_start` - Unix timestamp marking the start of the analysis window.
    /// * `period_end` - Unix timestamp marking the end of the analysis window.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::AnalyticsNotAvailable`] if data cannot be retrieved.
    ///
    /// # Example
    /// ```ignore
    /// let analytics = client.get_mobile_analytics(&user, &device_id, &start, &end);
    /// ```
    pub fn get_mobile_analytics(
        env: Env,
        user: Address,
        device_id: String,
        period_start: u64,
        period_end: u64,
    ) -> Result<MobileAnalytics, MobileOptimizerError> {
        user.require_auth();
        AnalyticsMonitor::get_mobile_analytics(&env, &user, device_id, period_start, period_end)
    }

    /// Return the platform-wide analytics dashboard summary, accessible to admins only.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::UnauthorizedAdmin`] if `admin` is not the stored admin.
    ///
    /// # Example
    /// ```ignore
    /// let dashboard = client.get_analytics_dashboard(&admin);
    /// ```
    pub fn get_analytics_dashboard(
        env: Env,
        admin: Address,
    ) -> Result<AnalyticsDashboard, MobileOptimizerError> {
        Self::require_admin(&env, &admin)?;
        Ok(AnalyticsMonitor::get_analytics_dashboard(&env))
    }

    // ========================================================================
    // Contract Statistics (Admin)
    // ========================================================================

    /// Return aggregate contract-level statistics (sessions, batches, offline ops), accessible to admins only.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    ///
    /// # Errors
    /// Returns [`MobileOptimizerError::UnauthorizedAdmin`] if `admin` is not the stored admin.
    ///
    /// # Example
    /// ```ignore
    /// let stats = client.get_contract_statistics(&admin);
    /// ```
    pub fn get_contract_statistics(
        env: Env,
        admin: Address,
    ) -> Result<ContractStatistics, MobileOptimizerError> {
        Self::require_admin(&env, &admin)?;

        let total_sessions: u64 =
            env.storage().persistent().get(&DataKey::TotalSessions).unwrap_or(0);
        let total_batches: u64 =
            env.storage().persistent().get(&DataKey::TotalBatches).unwrap_or(0);
        let total_offline_ops: u64 =
            env.storage().persistent().get(&DataKey::TotalOfflineOps).unwrap_or(0);

        Ok(ContractStatistics {
            total_sessions,
            total_batches_executed: total_batches,
            total_offline_operations: total_offline_ops,
        })
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn require_admin(env: &Env, admin: &Address) -> Result<(), MobileOptimizerError> {
        admin.require_auth();
        let stored: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(MobileOptimizerError::AdminNotSet)?;
        if *admin != stored {
            return Err(MobileOptimizerError::UnauthorizedAdmin);
        }
        Ok(())
    }

    fn increment_counter(env: &Env, key: &DataKey) {
        let current: u64 = env.storage().persistent().get(key).unwrap_or(0);
        env.storage().persistent().set(key, &(current + 1));
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractStatistics {
    pub total_sessions: u64,
    pub total_batches_executed: u64,
    pub total_offline_operations: u64,
}
