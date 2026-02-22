#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Vec};

pub mod analytics_monitor;
pub mod batch_manager;
pub mod battery_optimizer;
pub mod content_cache;
pub mod gas_optimizer;
pub mod interaction_flows;
pub mod network_manager;
pub mod notification_manager;
pub mod offline_manager;
pub mod pwa_manager;
pub mod security_manager;
pub mod session_manager;
pub mod types;

#[cfg(test)]
mod tests;

use analytics_monitor::AnalyticsMonitor;
use batch_manager::{BatchExecutionResult, BatchManager};
use battery_optimizer::{BatteryOptimizedSettings, BatteryOptimizer};
use content_cache::ContentCacheManager;
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
use types::*;

#[contract]
pub struct MobileOptimizerContract;

#[contractimpl]
#[allow(clippy::too_many_arguments)]
impl MobileOptimizerContract {
    // ========================================================================
    // Initialization & Admin
    // ========================================================================

    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Initialized) {
            panic!("already initialized");
        }

        let config = MobileOptimizerConfig {
            admin: admin.clone(),
            max_batch_size: 10,
            default_gas_limit: 1_000_000,
            session_timeout_seconds: 3600,
            offline_queue_limit: 100,
            network_timeout_ms: 30000,
            retry_attempts: 5,
            cache_ttl_seconds: 86400,
            max_devices_per_user: 5,
            analytics_retention_days: 90,
        };

        env.storage().persistent().set(&DataKey::Config, &config);
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage()
            .persistent()
            .set(&DataKey::TotalSessions, &0u64);
        env.storage()
            .persistent()
            .set(&DataKey::TotalBatches, &0u64);
        env.storage()
            .persistent()
            .set(&DataKey::TotalOfflineOps, &0u64);
    }

    pub fn get_config(env: Env) -> Result<MobileOptimizerConfig, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::Config)
            .ok_or(MobileOptimizerError::ConfigNotFound)
    }

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

    pub fn get_session(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<MobileSession, MobileOptimizerError> {
        user.require_auth();
        SessionManager::get_session(&env, &session_id)
    }

    pub fn update_session(
        env: Env,
        user: Address,
        session_id: String,
        network_quality: NetworkQuality,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::update_session(&env, session_id, Some(network_quality), None)
    }

    pub fn update_mobile_preferences(
        env: Env,
        user: Address,
        session_id: String,
        preferences: MobilePreferences,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::update_preferences(&env, session_id, preferences)
    }

    pub fn suspend_session(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::suspend_session(&env, session_id)
    }

    pub fn resume_session(
        env: Env,
        user: Address,
        session_id: String,
        network_quality: NetworkQuality,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::resume_session(&env, session_id, network_quality)
    }

    pub fn end_session(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SessionManager::end_session(&env, session_id)
    }

    pub fn get_session_stats(env: Env, user: Address) -> SessionStats {
        user.require_auth();
        SessionManager::get_session_stats(&env, &user)
    }

    pub fn optimize_session(
        env: Env,
        user: Address,
        session_id: String,
    ) -> Result<SessionOptimization, MobileOptimizerError> {
        user.require_auth();
        SessionManager::optimize_session_performance(&env, session_id)
    }

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

    pub fn get_gas_tips(env: Env) -> Vec<String> {
        GasOptimizer::get_mobile_gas_tips(&env)
    }

    // ========================================================================
    // Quick Interaction Flows
    // ========================================================================

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

    pub fn sync_offline_operations(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<OfflineSyncResult, MobileOptimizerError> {
        user.require_auth();
        let nq = NetworkManager::detect_network_quality(&env);
        OfflineManager::sync_offline_operations(&env, user, device_id, nq)
    }

    pub fn get_offline_queue_status(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<OfflineQueueStatus, MobileOptimizerError> {
        user.require_auth();
        OfflineManager::get_queue_status(&env, &user, &device_id)
    }

    pub fn resolve_offline_conflicts(
        env: Env,
        user: Address,
        device_id: String,
        strategy: ConflictResolution,
    ) -> Result<u32, MobileOptimizerError> {
        user.require_auth();
        OfflineManager::resolve_conflicts(&env, user, device_id, strategy)
    }

    pub fn cleanup_offline_operations(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<u32, MobileOptimizerError> {
        user.require_auth();
        OfflineManager::cleanup_completed_operations(&env, user, device_id)
    }

    pub fn get_offline_capabilities(env: Env) -> OfflineCapabilities {
        OfflineManager::get_offline_capabilities(&env)
    }

    // ========================================================================
    // Network Management
    // ========================================================================

    pub fn get_network_quality(env: Env) -> NetworkQuality {
        NetworkManager::detect_network_quality(&env)
    }

    pub fn get_connection_settings(
        _env: Env,
        network_quality: NetworkQuality,
    ) -> ConnectionSettings {
        NetworkManager::optimize_connection_settings(&network_quality)
    }

    pub fn get_bandwidth_optimization(
        env: Env,
        network_quality: NetworkQuality,
        data_usage_mode: DataUsageMode,
    ) -> BandwidthOptimization {
        NetworkManager::get_bandwidth_optimization(&env, &network_quality, &data_usage_mode)
    }

    pub fn get_network_statistics(
        env: Env,
        user: Address,
        session_id: String,
    ) -> NetworkStatistics {
        user.require_auth();
        NetworkManager::get_network_statistics(&env, session_id)
    }

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

    pub fn get_cached_content(
        env: Env,
        user: Address,
        cache_key: String,
    ) -> Result<CacheEntry, MobileOptimizerError> {
        user.require_auth();
        ContentCacheManager::get_cached_content(&env, &user, cache_key)
    }

    pub fn invalidate_cache(
        env: Env,
        user: Address,
        cache_key: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        ContentCacheManager::invalidate_cache(&env, &user, cache_key)
    }

    pub fn setup_prefetch_rules(
        env: Env,
        user: Address,
        rules: Vec<PrefetchRule>,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        ContentCacheManager::setup_prefetch_rules(&env, &user, rules)
    }

    pub fn execute_prefetch(
        env: Env,
        user: Address,
        trigger: PrefetchTrigger,
    ) -> Result<u32, MobileOptimizerError> {
        user.require_auth();
        let nq = NetworkManager::detect_network_quality(&env);
        ContentCacheManager::execute_prefetch(&env, &user, trigger, &nq)
    }

    pub fn get_cache_stats(env: Env, user: Address) -> Result<CacheStats, MobileOptimizerError> {
        user.require_auth();
        ContentCacheManager::get_cache_stats(&env, &user)
    }

    // ========================================================================
    // Battery Optimization (NEW)
    // ========================================================================

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

    pub fn get_battery_optimized_settings(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<BatteryOptimizedSettings, MobileOptimizerError> {
        user.require_auth();
        BatteryOptimizer::get_optimized_settings(&env, &user, &device_id)
    }

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

    pub fn get_pending_notifications(
        env: Env,
        user: Address,
    ) -> Result<Vec<LearningReminder>, MobileOptimizerError> {
        user.require_auth();
        NotificationManager::get_pending_notifications(&env, &user)
    }

    pub fn mark_notification_sent(
        env: Env,
        user: Address,
        reminder_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        NotificationManager::mark_notification_sent(&env, &user, reminder_id)
    }

    pub fn cancel_reminder(
        env: Env,
        user: Address,
        reminder_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        NotificationManager::cancel_reminder(&env, &user, reminder_id)
    }

    pub fn update_notification_config(
        env: Env,
        user: Address,
        config: NotificationConfig,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        NotificationManager::update_notification_config(&env, &user, config)
    }

    pub fn get_notification_config(
        env: Env,
        user: Address,
    ) -> Result<NotificationConfig, MobileOptimizerError> {
        user.require_auth();
        NotificationManager::get_notification_config(&env, &user)
    }

    pub fn create_streak_reminder(
        env: Env,
        user: Address,
        streak_days: u32,
    ) -> Result<LearningReminder, MobileOptimizerError> {
        user.require_auth();
        NotificationManager::create_streak_reminder(&env, &user, streak_days)
    }

    // ========================================================================
    // Mobile Security & Biometric Auth (NEW)
    // ========================================================================

    pub fn enable_biometric_auth(
        env: Env,
        user: Address,
        biometric_type: BiometricType,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::enable_biometric_auth(&env, &user, biometric_type)
    }

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

    pub fn register_trusted_device(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::register_trusted_device(&env, &user, device_id)
    }

    pub fn revoke_trusted_device(
        env: Env,
        user: Address,
        device_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::revoke_trusted_device(&env, &user, device_id)
    }

    pub fn get_security_profile(
        env: Env,
        user: Address,
    ) -> Result<SecurityProfile, MobileOptimizerError> {
        user.require_auth();
        SecurityManager::get_security_profile(&env, &user)
    }

    pub fn get_security_alerts(env: Env, user: Address) -> Vec<SecurityAlert> {
        user.require_auth();
        SecurityManager::get_security_alerts(&env, &user)
    }

    pub fn resolve_security_alert(
        env: Env,
        user: Address,
        alert_id: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::resolve_security_alert(&env, &user, alert_id)
    }

    pub fn enable_two_factor(env: Env, user: Address) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        SecurityManager::enable_two_factor(&env, &user)
    }

    // ========================================================================
    // PWA Capabilities (NEW)
    // ========================================================================

    pub fn get_pwa_config(env: Env, user: Address) -> Result<PwaConfig, MobileOptimizerError> {
        user.require_auth();
        PwaManager::get_pwa_config(&env, &user)
    }

    pub fn update_pwa_install_status(
        env: Env,
        user: Address,
        status: PwaInstallStatus,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        PwaManager::update_install_status(&env, &user, status)
    }

    pub fn get_pwa_manifest(env: Env) -> PwaManifest {
        PwaManager::get_pwa_manifest(&env)
    }

    pub fn update_service_worker(
        env: Env,
        user: Address,
        version: String,
    ) -> Result<ServiceWorkerStatus, MobileOptimizerError> {
        user.require_auth();
        PwaManager::update_service_worker(&env, &user, version)
    }

    pub fn get_service_worker_status(
        env: Env,
        user: Address,
    ) -> Result<ServiceWorkerStatus, MobileOptimizerError> {
        user.require_auth();
        PwaManager::get_service_worker_status(&env, &user)
    }

    pub fn register_cached_route(
        env: Env,
        user: Address,
        route: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        PwaManager::register_cached_route(&env, &user, route)
    }

    pub fn register_offline_page(
        env: Env,
        user: Address,
        page: String,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        PwaManager::register_offline_page(&env, &user, page)
    }

    pub fn toggle_background_sync(
        env: Env,
        user: Address,
        enabled: bool,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        PwaManager::toggle_background_sync(&env, &user, enabled)
    }

    pub fn get_offline_capability_report(env: Env, user: Address) -> OfflineCapabilityReport {
        user.require_auth();
        PwaManager::get_offline_capability_report(&env, &user)
    }

    // ========================================================================
    // Analytics & Performance Monitoring (NEW)
    // ========================================================================

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

    pub fn record_performance_metrics(
        env: Env,
        user: Address,
        metrics: PerformanceMetrics,
    ) -> Result<(), MobileOptimizerError> {
        user.require_auth();
        AnalyticsMonitor::record_performance_metrics(&env, &user, metrics)
    }

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

    pub fn get_user_engagement(
        env: Env,
        user: Address,
    ) -> Result<UserEngagement, MobileOptimizerError> {
        user.require_auth();
        AnalyticsMonitor::get_user_engagement(&env, &user)
    }

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

    pub fn get_contract_statistics(
        env: Env,
        admin: Address,
    ) -> Result<ContractStatistics, MobileOptimizerError> {
        Self::require_admin(&env, &admin)?;

        let total_sessions: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalSessions)
            .unwrap_or(0);
        let total_batches: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalBatches)
            .unwrap_or(0);
        let total_offline_ops: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalOfflineOps)
            .unwrap_or(0);

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
