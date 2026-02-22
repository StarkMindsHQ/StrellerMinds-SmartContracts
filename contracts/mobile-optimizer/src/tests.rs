use crate::types::*;
use crate::{MobileOptimizerContract, MobileOptimizerContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Map, String, Vec};

fn setup_contract() -> (
    Env,
    MobileOptimizerContractClient<'static>,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin, user)
}

fn default_preferences() -> MobilePreferences {
    MobilePreferences {
        auto_batch_operations: true,
        max_batch_size: 10,
        prefer_low_gas: true,
        enable_offline_mode: true,
        auto_retry_failed: true,
        notification_preferences: NotificationPreferences {
            transaction_complete: true,
            transaction_failed: true,
            batch_ready: true,
            network_issues: true,
            gas_price_alerts: false,
            offline_sync_complete: true,
            learning_reminders: true,
            streak_alerts: true,
            course_updates: true,
        },
        data_usage_mode: DataUsageMode::Conservative,
        battery_optimization: true,
    }
}

// ============================================================================
// Initialization Tests
// ============================================================================

#[test]
fn test_initialization() {
    let (_env, client, admin, _) = setup_contract();
    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.max_batch_size, 10);
    assert_eq!(config.session_timeout_seconds, 3600);
}

#[test]
#[should_panic]
fn test_double_initialization_fails() {
    let (env, client, _admin, _) = setup_contract();
    let admin2 = Address::generate(&env);
    client.initialize(&admin2);
}

#[test]
fn test_update_config() {
    let (_env, client, admin, _) = setup_contract();
    let mut new_config = client.get_config();
    new_config.max_batch_size = 20;
    new_config.retry_attempts = 10;
    client.update_config(&admin, &new_config);

    let updated = client.get_config();
    assert_eq!(updated.max_batch_size, 20);
    assert_eq!(updated.retry_attempts, 10);
}

// ============================================================================
// Session Management Tests
// ============================================================================

#[test]
fn test_create_and_get_session() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "device_001");
    let prefs = default_preferences();

    let session_id = client.create_session(&user, &device_id, &prefs);
    let session = client.get_session(&user, &session_id);
    assert_eq!(session.user, user);
    assert_eq!(session.session_state, SessionState::Active);
    assert_eq!(session.preferences.max_batch_size, 10);
}

#[test]
fn test_session_lifecycle() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "device_002");
    let prefs = default_preferences();

    let session_id = client.create_session(&user, &device_id, &prefs);

    client.suspend_session(&user, &session_id);
    let session = client.get_session(&user, &session_id);
    assert_eq!(session.session_state, SessionState::Suspended);

    client.resume_session(&user, &session_id, &NetworkQuality::Good);
    let session = client.get_session(&user, &session_id);
    assert_eq!(session.session_state, SessionState::Active);

    client.end_session(&user, &session_id);
    let session = client.get_session(&user, &session_id);
    assert_eq!(session.session_state, SessionState::Expired);
}

#[test]
fn test_update_preferences() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "device_003");
    let prefs = default_preferences();
    let session_id = client.create_session(&user, &device_id, &prefs);

    let mut new_prefs = default_preferences();
    new_prefs.max_batch_size = 5;
    new_prefs.battery_optimization = false;
    client.update_mobile_preferences(&user, &session_id, &new_prefs);

    let session = client.get_session(&user, &session_id);
    assert_eq!(session.preferences.max_batch_size, 5);
    assert!(!session.preferences.battery_optimization);
}

#[test]
fn test_session_stats() {
    let (env, client, _, user) = setup_contract();
    let prefs = default_preferences();
    let _ = client.create_session(&user, &String::from_str(&env, "d1"), &prefs);
    let _ = client.create_session(&user, &String::from_str(&env, "d2"), &prefs);

    let stats = client.get_session_stats(&user);
    assert_eq!(stats.total_sessions, 2);
    assert_eq!(stats.active_sessions, 2);
}

#[test]
fn test_session_optimization() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "device_opt");
    let prefs = default_preferences();
    let session_id = client.create_session(&user, &device_id, &prefs);

    let opt = client.optimize_session(&user, &session_id);
    assert!(opt.performance_score > 0);
}

#[test]
fn test_cross_device_sync() {
    let (env, client, _, user) = setup_contract();
    let d1 = String::from_str(&env, "phone");
    let d2 = String::from_str(&env, "tablet");
    let prefs = default_preferences();

    let sid1 = client.create_session(&user, &d1, &prefs);
    let sid2 = client.sync_session_state(&user, &sid1, &d2);

    let synced = client.get_session(&user, &sid2);
    assert_eq!(synced.session_state, SessionState::Active);
    assert_eq!(synced.preferences.max_batch_size, 10);
}

// ============================================================================
// Offline Operations Tests
// ============================================================================

#[test]
fn test_offline_queue_and_sync() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "offline_device");

    let op = QueuedOperation {
        operation_id: String::from_str(&env, "op_1"),
        operation_type: OperationType::ProgressUpdate,
        parameters: Vec::new(&env),
        created_at: 1000,
        priority: BatchPriority::Normal,
        local_state_hash: BytesN::from_array(&env, &[0u8; 32]),
        retry_count: 0,
        status: QueuedOperationStatus::Queued,
        estimated_gas: 25000,
    };

    client.queue_offline_operation(&user, &device_id, &op);
    let status = client.get_offline_queue_status(&user, &device_id);
    assert_eq!(status.total_operations, 1);
    assert_eq!(status.pending_operations, 1);

    let sync_result = client.sync_offline_operations(&user, &device_id);
    assert_eq!(sync_result.total_operations, 1);
    assert_eq!(sync_result.successful_syncs, 1);
}

#[test]
fn test_offline_cleanup() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "cleanup_device");

    let op = QueuedOperation {
        operation_id: String::from_str(&env, "cleanup_op"),
        operation_type: OperationType::PreferenceUpdate,
        parameters: Vec::new(&env),
        created_at: 1000,
        priority: BatchPriority::Low,
        local_state_hash: BytesN::from_array(&env, &[0u8; 32]),
        retry_count: 0,
        status: QueuedOperationStatus::Queued,
        estimated_gas: 10000,
    };

    client.queue_offline_operation(&user, &device_id, &op);
    let _ = client.sync_offline_operations(&user, &device_id);
    let cleaned = client.cleanup_offline_operations(&user, &device_id);
    assert_eq!(cleaned, 1);
}

#[test]
fn test_offline_capabilities() {
    let (_, client, _, _) = setup_contract();
    let caps = client.get_offline_capabilities();
    assert!(!caps.supported_operations.is_empty());
    assert_eq!(caps.max_queue_size, 100);
    assert_eq!(caps.max_offline_duration_hours, 168);
}

// ============================================================================
// Content Cache Tests
// ============================================================================

#[test]
fn test_cache_content_and_retrieve() {
    let (env, client, _, user) = setup_contract();
    let key = String::from_str(&env, "course_materials_101");
    let hash = BytesN::from_array(&env, &[1u8; 32]);

    client.cache_content(
        &user,
        &key,
        &hash,
        &ContentType::CourseMaterial,
        &1024,
        &86400,
    );

    let entry = client.get_cached_content(&user, &key);
    assert_eq!(entry.content_type, ContentType::CourseMaterial);
    assert_eq!(entry.size_bytes, 1024);
}

#[test]
fn test_cache_invalidation() {
    let (env, client, _, user) = setup_contract();
    let key = String::from_str(&env, "old_data");
    let hash = BytesN::from_array(&env, &[2u8; 32]);

    client.cache_content(
        &user,
        &key,
        &hash,
        &ContentType::SearchResults,
        &512,
        &86400,
    );
    client.invalidate_cache(&user, &key);

    let stats = client.get_cache_stats(&user);
    assert_eq!(stats.total_entries, 0);
}

#[test]
fn test_cache_stats() {
    let (env, client, _, user) = setup_contract();
    let hash = BytesN::from_array(&env, &[3u8; 32]);

    client.cache_content(
        &user,
        &String::from_str(&env, "item1"),
        &hash,
        &ContentType::QuizData,
        &2048,
        &86400,
    );
    client.cache_content(
        &user,
        &String::from_str(&env, "item2"),
        &hash,
        &ContentType::ProgressData,
        &1024,
        &86400,
    );

    let stats = client.get_cache_stats(&user);
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.total_size_bytes, 3072);
}

#[test]
fn test_prefetch_execution() {
    let (_env, client, _, user) = setup_contract();
    let _count = client.execute_prefetch(&user, &PrefetchTrigger::OnCourseEnroll);
}

// ============================================================================
// Battery Optimization Tests
// ============================================================================

#[test]
fn test_battery_profile_update() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "phone_batt");

    let profile = client.update_battery_profile(&user, &device_id, &75, &false);
    assert_eq!(profile.battery_level, 75);
    assert!(!profile.is_charging);
}

#[test]
fn test_battery_low_power_mode() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "low_batt");

    let _ = client.update_battery_profile(&user, &device_id, &10, &false);
    let settings = client.get_battery_optimized_settings(&user, &device_id);
    assert!(!settings.prefetch_enabled);
    assert!(!settings.animation_enabled);
}

#[test]
fn test_battery_charging_normal_mode() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "charging");

    let profile = client.update_battery_profile(&user, &device_id, &30, &true);
    assert_eq!(profile.power_mode, PowerMode::Normal);
}

#[test]
fn test_battery_impact_report() {
    let (env, client, _, _) = setup_contract();
    let sid = String::from_str(&env, "session_batt");

    let report = client.estimate_battery_impact(&sid, &30, &15, &60);
    assert!(report.estimated_drain_percent > 0);
    assert!(!report.recommendations.is_empty());
}

// ============================================================================
// Notification Tests
// ============================================================================

#[test]
fn test_create_learning_reminder() {
    let (env, client, _, user) = setup_contract();

    let reminder = client.create_learning_reminder(
        &user,
        &ReminderType::DailyStudy,
        &String::from_str(&env, "Study Time!"),
        &String::from_str(&env, "Continue your learning journey"),
        &1000,
        &RepeatInterval::Daily,
        &String::from_str(&env, "course_101"),
    );

    assert_eq!(reminder.reminder_type, ReminderType::DailyStudy);
    assert!(reminder.is_active);
}

#[test]
fn test_cancel_reminder() {
    let (env, client, _, user) = setup_contract();

    let reminder = client.create_learning_reminder(
        &user,
        &ReminderType::CourseDeadline,
        &String::from_str(&env, "Deadline approaching"),
        &String::from_str(&env, "Submit before deadline"),
        &2000,
        &RepeatInterval::Once,
        &String::from_str(&env, "course_202"),
    );

    client.cancel_reminder(&user, &reminder.reminder_id);
}

#[test]
fn test_streak_reminder() {
    let (_env, client, _, user) = setup_contract();
    let reminder = client.create_streak_reminder(&user, &7);
    assert_eq!(reminder.reminder_type, ReminderType::StreakMaintenance);
    assert!(reminder.is_active);
}

#[test]
fn test_notification_config() {
    let (env, client, _, user) = setup_contract();

    let mut channel_prefs = Map::new(&env);
    channel_prefs.set(String::from_str(&env, "push"), true);
    channel_prefs.set(String::from_str(&env, "email"), false);
    channel_prefs.set(String::from_str(&env, "in_app"), true);

    let config = NotificationConfig {
        user: user.clone(),
        enabled: true,
        quiet_hours_start: 23,
        quiet_hours_end: 7,
        max_daily_notifications: 5,
        channel_preferences: channel_prefs,
        priority_threshold: NotificationPriorityLevel::High,
        language_preference: String::from_str(&env, "es"),
        marketing_consent: true,
    };

    client.update_notification_config(&user, &config);
    let retrieved = client.get_notification_config(&user);
    assert_eq!(retrieved.max_daily_notifications, 5);
    assert_eq!(retrieved.quiet_hours_start, 23);
    assert_eq!(retrieved.language_preference, String::from_str(&env, "es"));
}

#[test]
fn test_notification_templates_and_campaigns() {
    let (env, client, admin, _user) = setup_contract();

    // Create Template
    let mut localized = Map::new(&env);
    localized.set(String::from_str(&env, "es"), String::from_str(&env, "Hola"));
    
    let mut channels = Vec::new(&env);
    channels.push_back(String::from_str(&env, "push"));

    let template = client.create_notification_template(
        &admin,
        &String::from_str(&env, "tpl_001"),
        &ReminderType::DailyStudy,
        &String::from_str(&env, "Hello"),
        &localized,
        &channels,
    );
    assert_eq!(template.template_id, String::from_str(&env, "tpl_001"));

    // Create Campaign
    let mut variants = Vec::new(&env);
    variants.push_back(ABTestVariant {
        variant_id: String::from_str(&env, "v1"),
        template_id: String::from_str(&env, "tpl_001"),
        weight: 100,
    });

    let campaign = client.create_notification_campaign(
        &admin,
        &String::from_str(&env, "camp_001"),
        &String::from_str(&env, "Summer Learning"),
        &variants,
        &1000,
        &2000,
    );
    assert_eq!(campaign.campaign_id, String::from_str(&env, "camp_001"));
    assert_eq!(campaign.variants.len(), 1);
}

// ============================================================================
// Security Tests
// ============================================================================

#[test]
fn test_biometric_auth_enable() {
    let (_env, client, _, user) = setup_contract();
    client.enable_biometric_auth(&user, &BiometricType::Fingerprint);

    let profile = client.get_security_profile(&user);
    assert!(profile.biometric_enabled);
    assert_eq!(profile.biometric_type, BiometricType::Fingerprint);
}

#[test]
fn test_trusted_device_registration() {
    let (env, client, _, user) = setup_contract();
    let device = String::from_str(&env, "my_phone");

    client.register_trusted_device(&user, &device);
    let profile = client.get_security_profile(&user);
    assert_eq!(profile.trusted_devices.len(), 1);

    client.revoke_trusted_device(&user, &device);
    let profile = client.get_security_profile(&user);
    assert_eq!(profile.trusted_devices.len(), 0);
}

#[test]
fn test_authentication_success() {
    let (env, client, _, user) = setup_contract();
    let device = String::from_str(&env, "auth_device");
    let ip = BytesN::from_array(&env, &[5u8; 32]);

    client.register_trusted_device(&user, &device);
    let event = client.authenticate(&user, &device, &AuthMethod::DeviceToken, &ip);
    assert!(event.success);
}

#[test]
fn test_two_factor_enable() {
    let (_env, client, _, user) = setup_contract();
    client.enable_two_factor(&user);
    let profile = client.get_security_profile(&user);
    assert!(profile.two_factor_enabled);
}

#[test]
fn test_security_alerts() {
    let (_env, client, _, user) = setup_contract();
    let alerts = client.get_security_alerts(&user);
    assert_eq!(alerts.len(), 0);
}

// ============================================================================
// PWA Tests
// ============================================================================

#[test]
fn test_pwa_config_initialization() {
    let (_env, client, _, user) = setup_contract();
    client.update_pwa_install_status(&user, &PwaInstallStatus::NotInstalled);
    let config = client.get_pwa_config(&user);
    assert_eq!(config.install_status, PwaInstallStatus::NotInstalled);
    assert!(!config.cached_routes.is_empty());
}

#[test]
fn test_pwa_install_status_update() {
    let (_env, client, _, user) = setup_contract();
    client.update_pwa_install_status(&user, &PwaInstallStatus::Installed);
    let config = client.get_pwa_config(&user);
    assert_eq!(config.install_status, PwaInstallStatus::Installed);
}

#[test]
fn test_service_worker() {
    let (env, client, _, user) = setup_contract();
    let version = String::from_str(&env, "2.0.0");
    let status = client.update_service_worker(&user, &version);
    assert_eq!(status.state, SwState::Activated);

    let retrieved = client.get_service_worker_status(&user);
    assert_eq!(retrieved.state, SwState::Activated);
}

#[test]
fn test_cached_route_registration() {
    let (env, client, _, user) = setup_contract();
    let route = String::from_str(&env, "/certificates");
    client.register_cached_route(&user, &route);

    let config = client.get_pwa_config(&user);
    assert!(config.cached_routes.len() >= 5);
}

#[test]
fn test_offline_capability_report() {
    let (_env, client, _, user) = setup_contract();
    let report = client.get_offline_capability_report(&user);
    assert!(!report.is_installed);
    assert!(report.cached_routes_count > 0);
}

#[test]
fn test_pwa_manifest() {
    let (_, client, _, _) = setup_contract();
    let manifest = client.get_pwa_manifest();
    assert_eq!(manifest.display_mode, DisplayMode::Standalone);
}

// ============================================================================
// Analytics & Monitoring Tests
// ============================================================================

#[test]
fn test_track_analytics_event() {
    let (env, client, _, user) = setup_contract();
    let props = Map::new(&env);
    let session = String::from_str(&env, "analytics_session");

    let event = client.track_analytics_event(
        &user,
        &AnalyticsEventType::SessionStart,
        &props,
        &session,
        &DeviceType::MobilePhone,
    );
    assert_eq!(event.event_type, AnalyticsEventType::SessionStart);
}

#[test]
fn test_user_engagement() {
    let (_env, client, _, user) = setup_contract();

    let engagement = client.update_user_engagement(&user, &1800, &2, &1);
    assert!(engagement.engagement_score > 0);
    assert_eq!(engagement.sessions_today, 1);
    assert_eq!(engagement.courses_accessed, 2);
    assert_eq!(engagement.modules_completed, 1);
}

#[test]
fn test_get_mobile_analytics() {
    let (env, client, _, user) = setup_contract();
    let device = String::from_str(&env, "analytics_device");

    let analytics = client.get_mobile_analytics(&user, &device, &0, &999999);
    assert_eq!(analytics.user, user);
}

#[test]
fn test_performance_metrics_recording() {
    let (env, client, _, user) = setup_contract();
    let session = String::from_str(&env, "perf_session");

    let metrics = PerformanceMetrics {
        session_id: session.clone(),
        timestamp: 1000,
        page_load_time_ms: 250,
        api_response_time_ms: 100,
        render_time_ms: 50,
        memory_usage_bytes: 50_000_000,
        network_latency_ms: 30,
        frame_rate: 60,
        error_count: 0,
        crash_count: 0,
    };

    client.record_performance_metrics(&user, &metrics);
}

#[test]
fn test_analytics_dashboard() {
    let (_env, client, admin, _) = setup_contract();
    let dashboard = client.get_analytics_dashboard(&admin);
    assert_eq!(dashboard.total_users, 0);
}

// ============================================================================
// Gas Optimization Tests
// ============================================================================

#[test]
fn test_gas_estimation() {
    let (env, client, _, _) = setup_contract();

    let mut ops = Vec::new(&env);
    ops.push_back(BatchOperation {
        operation_id: String::from_str(&env, "gas_op"),
        operation_type: OperationType::CourseEnrollment,
        contract_address: Address::generate(&env),
        function_name: String::from_str(&env, "enroll"),
        parameters: Vec::new(&env),
        estimated_gas: 50000,
        priority: OperationPriority::High,
        retry_config: RetryConfig {
            max_retries: 3,
            retry_delay_ms: 500,
            backoff_multiplier: 2,
            max_delay_ms: 5000,
            retry_on_network_error: true,
            retry_on_gas_error: true,
            retry_on_timeout: true,
        },
        dependencies: Vec::new(&env),
    });

    let estimates = client.estimate_gas(&ops, &NetworkQuality::Good);
    assert_eq!(estimates.len(), 1);
    assert!(estimates.get(0).unwrap().estimated_gas > 0);
}

#[test]
fn test_gas_tips() {
    let (_, client, _, _) = setup_contract();
    let tips = client.get_gas_tips();
    assert!(tips.len() >= 5);
}

// ============================================================================
// Network Tests
// ============================================================================

#[test]
fn test_connection_settings() {
    let (_, client, _, _) = setup_contract();
    let settings = client.get_connection_settings(&NetworkQuality::Poor);
    assert_eq!(settings.timeout_ms, 30000);
    assert_eq!(settings.max_concurrent_operations, 1);
    assert!(settings.compression_enabled);
}

#[test]
fn test_bandwidth_optimization() {
    let (_, client, _, _) = setup_contract();
    let opt =
        client.get_bandwidth_optimization(&NetworkQuality::Fair, &DataUsageMode::Conservative);
    assert_eq!(opt.image_quality_percent, 60);
    assert!(!opt.prefetch_enabled);
}

#[test]
fn test_network_adaptation() {
    let (_, client, _, _) = setup_contract();
    let adaptation = client.adapt_network(&NetworkQuality::Good, &NetworkQuality::Poor);
    assert!(adaptation.degraded);
    assert!(!adaptation.actions.is_empty());
}

// ============================================================================
// Quick Interaction Tests
// ============================================================================

#[test]
fn test_quick_enroll() {
    let (env, client, _, user) = setup_contract();
    let prefs = default_preferences();
    let session_id = client.create_session(&user, &String::from_str(&env, "enroll_dev"), &prefs);
    let course = String::from_str(&env, "course_001");

    let result = client.quick_enroll_course(&user, &course, &session_id);
    assert!(result.success);
}

#[test]
fn test_quick_progress_update() {
    let (env, client, _, user) = setup_contract();
    let prefs = default_preferences();
    let session_id = client.create_session(&user, &String::from_str(&env, "prog_dev"), &prefs);

    let result = client.quick_update_progress(
        &user,
        &String::from_str(&env, "course_001"),
        &String::from_str(&env, "module_001"),
        &75,
        &session_id,
    );
    assert!(result.success);
}

// ============================================================================
// Contract Statistics Tests
// ============================================================================

#[test]
fn test_contract_statistics() {
    let (env, client, admin, user) = setup_contract();
    let prefs = default_preferences();

    let _ = client.create_session(&user, &String::from_str(&env, "stat_dev"), &prefs);

    let stats = client.get_contract_statistics(&admin);
    assert_eq!(stats.total_sessions, 1);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_complete_mobile_workflow() {
    let (env, client, admin, user) = setup_contract();
    let device_id = String::from_str(&env, "workflow_device");
    let prefs = default_preferences();

    // 1. Create session
    let session_id = client.create_session(&user, &device_id, &prefs);

    // 2. Register trusted device
    client.register_trusted_device(&user, &device_id);

    // 3. Authenticate
    let ip = BytesN::from_array(&env, &[10u8; 32]);
    let auth = client.authenticate(&user, &device_id, &AuthMethod::DeviceToken, &ip);
    assert!(auth.success);

    // 4. Cache content
    let hash = BytesN::from_array(&env, &[20u8; 32]);
    client.cache_content(
        &user,
        &String::from_str(&env, "course_material"),
        &hash,
        &ContentType::CourseMaterial,
        &5000,
        &86400,
    );

    // 5. Quick enroll
    let enroll =
        client.quick_enroll_course(&user, &String::from_str(&env, "course_001"), &session_id);
    assert!(enroll.success);

    // 6. Update progress
    let progress = client.quick_update_progress(
        &user,
        &String::from_str(&env, "course_001"),
        &String::from_str(&env, "module_001"),
        &50,
        &session_id,
    );
    assert!(progress.success);

    // 7. Track analytics
    let props = Map::new(&env);
    client.track_analytics_event(
        &user,
        &AnalyticsEventType::ModuleComplete,
        &props,
        &session_id,
        &DeviceType::MobilePhone,
    );

    // 8. Update engagement
    let engagement = client.update_user_engagement(&user, &3600, &1, &1);
    assert!(engagement.engagement_score > 0);

    // 9. Create reminder
    let reminder = client.create_learning_reminder(
        &user,
        &ReminderType::DailyStudy,
        &String::from_str(&env, "Keep learning!"),
        &String::from_str(&env, "Don't break your streak"),
        &99999,
        &RepeatInterval::Daily,
        &String::from_str(&env, "course_001"),
    );
    assert!(reminder.is_active);

    // 10. Update battery
    let battery = client.update_battery_profile(&user, &device_id, &85, &false);
    assert_eq!(battery.power_mode, PowerMode::Adaptive);

    // 11. Check stats
    let stats = client.get_contract_statistics(&admin);
    assert!(stats.total_sessions >= 1);
}

#[test]
fn test_offline_then_sync_workflow() {
    let (env, client, _, user) = setup_contract();
    let device_id = String::from_str(&env, "offline_sync_dev");

    for i in 0..3u32 {
        let op = QueuedOperation {
            operation_id: String::from_str(&env, "sync_op"),
            operation_type: OperationType::ProgressUpdate,
            parameters: Vec::new(&env),
            created_at: 1000 + i as u64,
            priority: BatchPriority::Normal,
            local_state_hash: BytesN::from_array(&env, &[0u8; 32]),
            retry_count: 0,
            status: QueuedOperationStatus::Queued,
            estimated_gas: 25000,
        };
        client.queue_offline_operation(&user, &device_id, &op);
    }

    let status = client.get_offline_queue_status(&user, &device_id);
    assert_eq!(status.total_operations, 3);

    let sync = client.sync_offline_operations(&user, &device_id);
    assert_eq!(sync.total_operations, 3);
    assert_eq!(sync.successful_syncs, 3);

    let cleaned = client.cleanup_offline_operations(&user, &device_id);
    assert_eq!(cleaned, 3);
}
