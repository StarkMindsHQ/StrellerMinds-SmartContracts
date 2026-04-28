use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Vec};

use crate::types::*;

// ─── Storage keys for new PWA features ────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
enum PwaExtKey {
    /// Pending push notifications for a user.
    PushQueue(Address),
    /// Install prompt history for a user.
    InstallHistory(Address),
    /// Global PWA metrics.
    Metrics,
}

pub struct PwaManager;

impl PwaManager {
    pub fn initialize_pwa_config(env: &Env, user: &Address) -> PwaConfig {
        let config = PwaConfig {
            user: user.clone(),
            install_status: PwaInstallStatus::NotInstalled,
            service_worker_version: String::from_str(env, "1.0.0"),
            cached_routes: Self::default_cached_routes(env),
            offline_pages: Self::default_offline_pages(env),
            background_sync_enabled: true,
            push_subscription_active: false,
            storage_quota_bytes: 100 * 1024 * 1024, // 100 MB
            storage_used_bytes: 0,
        };

        env.storage().persistent().set(&DataKey::PwaConfig(user.clone()), &config);
        config
    }

    pub fn get_pwa_config(env: &Env, user: &Address) -> Result<PwaConfig, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::PwaConfig(user.clone()))
            .ok_or(MobileOptimizerError::PwaError)
    }

    pub fn update_install_status(
        env: &Env,
        user: &Address,
        status: PwaInstallStatus,
    ) -> Result<(), MobileOptimizerError> {
        let mut config = Self::get_or_create_config(env, user);
        config.install_status = status;
        env.storage().persistent().set(&DataKey::PwaConfig(user.clone()), &config);
        Ok(())
    }

    pub fn get_pwa_manifest(env: &Env) -> PwaManifest {
        env.storage()
            .persistent()
            .get(&DataKey::PwaManifest)
            .unwrap_or_else(|| Self::default_manifest(env))
    }

    pub fn update_pwa_manifest(
        env: &Env,
        manifest: PwaManifest,
    ) -> Result<(), MobileOptimizerError> {
        env.storage().persistent().set(&DataKey::PwaManifest, &manifest);
        Ok(())
    }

    pub fn update_service_worker(
        env: &Env,
        user: &Address,
        version: String,
    ) -> Result<ServiceWorkerStatus, MobileOptimizerError> {
        let now = env.ledger().timestamp();
        let sw_status = ServiceWorkerStatus {
            version: version.clone(),
            state: SwState::Activated,
            last_updated: now,
            cached_assets_count: 0,
            cached_api_responses: 0,
            pending_sync_count: 0,
        };

        env.storage().persistent().set(&DataKey::SwStatus(user.clone()), &sw_status);

        let mut config = Self::get_or_create_config(env, user);
        config.service_worker_version = version;
        env.storage().persistent().set(&DataKey::PwaConfig(user.clone()), &config);

        Ok(sw_status)
    }

    pub fn get_service_worker_status(
        env: &Env,
        user: &Address,
    ) -> Result<ServiceWorkerStatus, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::SwStatus(user.clone()))
            .ok_or(MobileOptimizerError::PwaError)
    }

    pub fn register_cached_route(
        env: &Env,
        user: &Address,
        route: String,
    ) -> Result<(), MobileOptimizerError> {
        let mut config = Self::get_or_create_config(env, user);
        for existing in config.cached_routes.iter() {
            if existing == route {
                return Ok(());
            }
        }
        config.cached_routes.push_back(route);
        env.storage().persistent().set(&DataKey::PwaConfig(user.clone()), &config);
        Ok(())
    }

    pub fn register_offline_page(
        env: &Env,
        user: &Address,
        page: String,
    ) -> Result<(), MobileOptimizerError> {
        let mut config = Self::get_or_create_config(env, user);
        for existing in config.offline_pages.iter() {
            if existing == page {
                return Ok(());
            }
        }
        config.offline_pages.push_back(page);
        env.storage().persistent().set(&DataKey::PwaConfig(user.clone()), &config);
        Ok(())
    }

    pub fn toggle_background_sync(
        env: &Env,
        user: &Address,
        enabled: bool,
    ) -> Result<(), MobileOptimizerError> {
        let mut config = Self::get_or_create_config(env, user);
        config.background_sync_enabled = enabled;
        env.storage().persistent().set(&DataKey::PwaConfig(user.clone()), &config);
        Ok(())
    }

    pub fn toggle_push_subscription(
        env: &Env,
        user: &Address,
        active: bool,
    ) -> Result<(), MobileOptimizerError> {
        let mut config = Self::get_or_create_config(env, user);
        config.push_subscription_active = active;
        env.storage().persistent().set(&DataKey::PwaConfig(user.clone()), &config);
        Ok(())
    }

    pub fn update_storage_usage(
        env: &Env,
        user: &Address,
        used_bytes: u64,
    ) -> Result<(), MobileOptimizerError> {
        let mut config = Self::get_or_create_config(env, user);
        config.storage_used_bytes = used_bytes;
        env.storage().persistent().set(&DataKey::PwaConfig(user.clone()), &config);
        Ok(())
    }

    pub fn get_storage_usage_percent(
        env: &Env,
        user: &Address,
    ) -> Result<u32, MobileOptimizerError> {
        let config = Self::get_or_create_config(env, user);
        if config.storage_quota_bytes == 0 {
            return Ok(0);
        }
        let percent = ((config.storage_used_bytes * 100) / config.storage_quota_bytes) as u32;
        Ok(percent)
    }

    pub fn get_offline_capability_report(env: &Env, user: &Address) -> OfflineCapabilityReport {
        let config = Self::get_or_create_config(env, user);
        let sw_status = env
            .storage()
            .persistent()
            .get::<DataKey, ServiceWorkerStatus>(&DataKey::SwStatus(user.clone()));

        let sw_active = sw_status.as_ref().map(|s| s.state == SwState::Activated).unwrap_or(false);

        let storage_pct = if config.storage_quota_bytes > 0 {
            (config.storage_used_bytes * 100).checked_div(config.storage_quota_bytes).unwrap_or(0)
                as u32
        } else {
            0
        };

        OfflineCapabilityReport {
            is_installed: matches!(
                config.install_status,
                PwaInstallStatus::Installed | PwaInstallStatus::Standalone
            ),
            service_worker_active: sw_active,
            cached_routes_count: config.cached_routes.len(),
            offline_pages_count: config.offline_pages.len(),
            background_sync_enabled: config.background_sync_enabled,
            storage_usage_percent: storage_pct,
            push_enabled: config.push_subscription_active,
        }
    }

    fn get_or_create_config(env: &Env, user: &Address) -> PwaConfig {
        env.storage()
            .persistent()
            .get(&DataKey::PwaConfig(user.clone()))
            .unwrap_or_else(|| Self::initialize_pwa_config(env, user))
    }

    fn default_cached_routes(env: &Env) -> Vec<String> {
        let mut routes = Vec::new(env);
        routes.push_back(String::from_str(env, "/dashboard"));
        routes.push_back(String::from_str(env, "/courses"));
        routes.push_back(String::from_str(env, "/profile"));
        routes.push_back(String::from_str(env, "/progress"));
        routes
    }

    fn default_offline_pages(env: &Env) -> Vec<String> {
        let mut pages = Vec::new(env);
        pages.push_back(String::from_str(env, "/offline"));
        pages.push_back(String::from_str(env, "/dashboard"));
        pages
    }

    fn default_manifest(env: &Env) -> PwaManifest {
        PwaManifest {
            app_name: String::from_str(env, "StrellerMinds Learning"),
            short_name: String::from_str(env, "StrellerMinds"),
            version: String::from_str(env, "1.0.0"),
            theme_color: String::from_str(env, "#1a73e8"),
            background_color: String::from_str(env, "#ffffff"),
            display_mode: DisplayMode::Standalone,
            orientation: String::from_str(env, "portrait"),
            start_url: String::from_str(env, "/dashboard"),
        }
    }

    // ── Push Notifications (Issue #440) ───────────────────────────────────────

    /// Queue a push notification for `recipient`.
    ///
    /// The notification is stored on-chain so the off-chain push service can
    /// poll and deliver it.  Returns `Err` if the recipient has no active push
    /// subscription.
    pub fn send_push_notification(
        env: &Env,
        notification: PushNotification,
    ) -> Result<(), MobileOptimizerError> {
        // Require an active push subscription.
        let config = Self::get_or_create_config(env, &notification.recipient);
        if !config.push_subscription_active {
            return Err(MobileOptimizerError::PwaError);
        }

        let key = PwaExtKey::PushQueue(notification.recipient.clone());
        let mut queue: Vec<PushNotification> =
            env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));

        queue.push_back(notification.clone());
        env.storage().persistent().set(&key, &queue);

        // Update metrics.
        let mut metrics = Self::get_metrics(env);
        metrics.notifications_sent += 1;
        env.storage().persistent().set(&PwaExtKey::Metrics, &metrics);

        env.events().publish(
            (symbol_short!("push_sent"), notification.recipient.clone()),
            notification.notification_id,
        );

        Ok(())
    }

    /// Mark a notification as delivered (called by the push service).
    pub fn mark_notification_delivered(
        env: &Env,
        recipient: &Address,
        notification_id: &String,
    ) -> Result<(), MobileOptimizerError> {
        let key = PwaExtKey::PushQueue(recipient.clone());
        let mut queue: Vec<PushNotification> = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(MobileOptimizerError::PwaError)?;

        let mut found = false;
        let mut updated: Vec<PushNotification> = Vec::new(env);
        for mut n in queue.iter() {
            if n.notification_id == *notification_id {
                n.delivered = true;
                found = true;
            }
            updated.push_back(n);
        }
        if !found {
            return Err(MobileOptimizerError::PwaError);
        }
        env.storage().persistent().set(&key, &updated);

        let mut metrics = Self::get_metrics(env);
        metrics.notifications_delivered += 1;
        env.storage().persistent().set(&PwaExtKey::Metrics, &metrics);

        Ok(())
    }

    /// Mark a notification as read by the user.
    pub fn mark_notification_read(
        env: &Env,
        recipient: &Address,
        notification_id: &String,
    ) -> Result<(), MobileOptimizerError> {
        let key = PwaExtKey::PushQueue(recipient.clone());
        let mut queue: Vec<PushNotification> = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(MobileOptimizerError::PwaError)?;

        let mut found = false;
        let mut updated: Vec<PushNotification> = Vec::new(env);
        for mut n in queue.iter() {
            if n.notification_id == *notification_id {
                n.read = true;
                found = true;
            }
            updated.push_back(n);
        }
        if !found {
            return Err(MobileOptimizerError::PwaError);
        }
        env.storage().persistent().set(&key, &updated);

        let mut metrics = Self::get_metrics(env);
        metrics.notifications_read += 1;
        env.storage().persistent().set(&PwaExtKey::Metrics, &metrics);

        Ok(())
    }

    /// Return all pending (unread) notifications for `recipient`.
    pub fn get_pending_notifications(
        env: &Env,
        recipient: &Address,
    ) -> Vec<PushNotification> {
        let key = PwaExtKey::PushQueue(recipient.clone());
        let queue: Vec<PushNotification> =
            env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));

        let now = env.ledger().timestamp();
        let mut pending: Vec<PushNotification> = Vec::new(env);
        for n in queue.iter() {
            let not_expired = n.expires_at.map_or(true, |exp| now < exp);
            if !n.read && not_expired {
                pending.push_back(n);
            }
        }
        pending
    }

    // ── Install-to-Home-Screen (Issue #440) ───────────────────────────────────

    /// Record that the install prompt was shown to `user`.
    pub fn record_install_prompt_shown(
        env: &Env,
        user: &Address,
        platform: String,
    ) -> Result<(), MobileOptimizerError> {
        let record = InstallPromptRecord {
            user: user.clone(),
            shown_at: env.ledger().timestamp(),
            outcome: InstallPromptOutcome::Pending,
            platform,
        };

        let key = PwaExtKey::InstallHistory(user.clone());
        let mut history: Vec<InstallPromptRecord> =
            env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));
        history.push_back(record);
        env.storage().persistent().set(&key, &history);

        let mut metrics = Self::get_metrics(env);
        metrics.prompts_shown += 1;
        env.storage().persistent().set(&PwaExtKey::Metrics, &metrics);

        Ok(())
    }

    /// Record the user's response to the install prompt.
    pub fn record_install_prompt_outcome(
        env: &Env,
        user: &Address,
        outcome: InstallPromptOutcome,
    ) -> Result<(), MobileOptimizerError> {
        let key = PwaExtKey::InstallHistory(user.clone());
        let mut history: Vec<InstallPromptRecord> = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(MobileOptimizerError::PwaError)?;

        // Update the most recent pending record.
        let len = history.len();
        if len == 0 {
            return Err(MobileOptimizerError::PwaError);
        }
        let mut updated: Vec<InstallPromptRecord> = Vec::new(env);
        for (i, mut r) in history.iter().enumerate() {
            if i as u32 == len - 1 && r.outcome == InstallPromptOutcome::Pending {
                r.outcome = outcome.clone();
            }
            updated.push_back(r);
        }
        env.storage().persistent().set(&key, &updated);

        // Update metrics and install status.
        let mut metrics = Self::get_metrics(env);
        match outcome {
            InstallPromptOutcome::Accepted => {
                metrics.installs_accepted += 1;
                metrics.active_push_subscribers += 1;
                Self::update_install_status(env, user, PwaInstallStatus::Installed)?;
            }
            InstallPromptOutcome::Dismissed => {
                metrics.installs_dismissed += 1;
            }
            InstallPromptOutcome::Pending => {}
        }
        env.storage().persistent().set(&PwaExtKey::Metrics, &metrics);

        env.events().publish(
            (symbol_short!("pwa_install"), user.clone()),
            outcome == InstallPromptOutcome::Accepted,
        );

        Ok(())
    }

    /// Return the install prompt history for `user`.
    pub fn get_install_history(env: &Env, user: &Address) -> Vec<InstallPromptRecord> {
        env.storage()
            .persistent()
            .get(&PwaExtKey::InstallHistory(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Return global PWA metrics.
    pub fn get_metrics(env: &Env) -> PwaMetrics {
        env.storage()
            .persistent()
            .get(&PwaExtKey::Metrics)
            .unwrap_or(PwaMetrics {
                prompts_shown: 0,
                installs_accepted: 0,
                installs_dismissed: 0,
                notifications_sent: 0,
                notifications_delivered: 0,
                notifications_read: 0,
                active_push_subscribers: 0,
            })
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineCapabilityReport {
    pub is_installed: bool,
    pub service_worker_active: bool,
    pub cached_routes_count: u32,
    pub offline_pages_count: u32,
    pub background_sync_enabled: bool,
    pub storage_usage_percent: u32,
    pub push_enabled: bool,
}
