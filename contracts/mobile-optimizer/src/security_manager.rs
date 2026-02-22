use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::types::*;

pub struct SecurityManager;

impl SecurityManager {
    pub fn initialize_security_profile(env: &Env, user: &Address) -> SecurityProfile {
        let profile = SecurityProfile {
            user: user.clone(),
            biometric_enabled: false,
            biometric_type: BiometricType::None,
            session_lock_timeout: 300, // 5 minutes
            failed_attempts: 0,
            max_failed_attempts: 5,
            lockout_until: 0,
            trusted_devices: Vec::new(env),
            last_security_check: env.ledger().timestamp(),
            two_factor_enabled: false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::SecurityProfile(user.clone()), &profile);
        profile
    }

    pub fn enable_biometric_auth(
        env: &Env,
        user: &Address,
        biometric_type: BiometricType,
    ) -> Result<(), MobileOptimizerError> {
        let mut profile = Self::get_or_create_profile(env, user);
        profile.biometric_enabled = true;
        profile.biometric_type = biometric_type;
        profile.last_security_check = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::SecurityProfile(user.clone()), &profile);
        Ok(())
    }

    pub fn authenticate(
        env: &Env,
        user: &Address,
        device_id: String,
        auth_method: AuthMethod,
        ip_hash: BytesN<32>,
    ) -> Result<AuthenticationEvent, MobileOptimizerError> {
        let mut profile = Self::get_or_create_profile(env, user);
        let now = env.ledger().timestamp();

        if profile.lockout_until > now {
            return Err(MobileOptimizerError::AccountLocked);
        }

        let is_trusted = Self::is_trusted_device(&profile, &device_id);
        let risk_score = Self::calculate_risk_score(&profile, &device_id, &auth_method, is_trusted);

        let success = match auth_method {
            AuthMethod::Biometric => profile.biometric_enabled,
            AuthMethod::DeviceToken => is_trusted,
            AuthMethod::SessionResume => is_trusted && risk_score < 50,
            AuthMethod::Password | AuthMethod::TwoFactor => true,
        };

        let event = AuthenticationEvent {
            event_id: String::from_str(env, "auth_event"),
            user: user.clone(),
            device_id: device_id.clone(),
            auth_method: auth_method.clone(),
            timestamp: now,
            success,
            ip_hash,
            risk_score,
        };

        if success {
            profile.failed_attempts = 0;
            profile.last_security_check = now;
        } else {
            profile.failed_attempts += 1;
            if profile.failed_attempts >= profile.max_failed_attempts {
                profile.lockout_until = now + 1800; // 30-minute lockout

                let alert = SecurityAlert {
                    alert_id: String::from_str(env, "lockout_alert"),
                    user: user.clone(),
                    alert_type: SecurityAlertType::MultipleFailedAttempts,
                    severity: AlertSeverity::Warning,
                    message: String::from_str(
                        env,
                        "Account locked due to multiple failed login attempts",
                    ),
                    timestamp: now,
                    resolved: false,
                };
                Self::record_security_alert(env, user, alert);
            }
        }

        env.storage()
            .persistent()
            .set(&DataKey::SecurityProfile(user.clone()), &profile);
        Self::record_auth_event(env, user, &event);

        if success {
            Ok(event)
        } else {
            Err(MobileOptimizerError::BiometricAuthFailed)
        }
    }

    pub fn register_trusted_device(
        env: &Env,
        user: &Address,
        device_id: String,
    ) -> Result<(), MobileOptimizerError> {
        let mut profile = Self::get_or_create_profile(env, user);

        for existing in profile.trusted_devices.iter() {
            if existing == device_id {
                return Ok(());
            }
        }

        profile.trusted_devices.push_back(device_id);
        env.storage()
            .persistent()
            .set(&DataKey::SecurityProfile(user.clone()), &profile);
        Ok(())
    }

    pub fn revoke_trusted_device(
        env: &Env,
        user: &Address,
        device_id: String,
    ) -> Result<(), MobileOptimizerError> {
        let mut profile = Self::get_or_create_profile(env, user);
        let mut updated = Vec::new(env);

        for d in profile.trusted_devices.iter() {
            if d != device_id {
                updated.push_back(d.clone());
            }
        }
        profile.trusted_devices = updated;

        env.storage()
            .persistent()
            .set(&DataKey::SecurityProfile(user.clone()), &profile);
        Ok(())
    }

    pub fn get_security_profile(
        env: &Env,
        user: &Address,
    ) -> Result<SecurityProfile, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::SecurityProfile(user.clone()))
            .ok_or(MobileOptimizerError::SecurityViolation)
    }

    pub fn get_security_alerts(env: &Env, user: &Address) -> Vec<SecurityAlert> {
        env.storage()
            .persistent()
            .get(&DataKey::SecurityAlerts(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    pub fn resolve_security_alert(
        env: &Env,
        user: &Address,
        alert_id: String,
    ) -> Result<(), MobileOptimizerError> {
        let alerts: Vec<SecurityAlert> = env
            .storage()
            .persistent()
            .get(&DataKey::SecurityAlerts(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut updated = Vec::new(env);
        for alert in alerts.iter() {
            let mut a = alert.clone();
            if a.alert_id == alert_id {
                a.resolved = true;
            }
            updated.push_back(a);
        }

        env.storage()
            .persistent()
            .set(&DataKey::SecurityAlerts(user.clone()), &updated);
        Ok(())
    }

    pub fn update_session_lock_timeout(
        env: &Env,
        user: &Address,
        timeout_seconds: u64,
    ) -> Result<(), MobileOptimizerError> {
        let mut profile = Self::get_or_create_profile(env, user);
        profile.session_lock_timeout = timeout_seconds;
        env.storage()
            .persistent()
            .set(&DataKey::SecurityProfile(user.clone()), &profile);
        Ok(())
    }

    pub fn enable_two_factor(env: &Env, user: &Address) -> Result<(), MobileOptimizerError> {
        let mut profile = Self::get_or_create_profile(env, user);
        profile.two_factor_enabled = true;
        profile.last_security_check = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::SecurityProfile(user.clone()), &profile);
        Ok(())
    }

    pub fn check_session_validity(
        env: &Env,
        user: &Address,
        last_activity: u64,
    ) -> Result<bool, MobileOptimizerError> {
        let profile = Self::get_or_create_profile(env, user);
        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(last_activity);
        Ok(elapsed < profile.session_lock_timeout)
    }

    pub fn get_auth_history(env: &Env, user: &Address) -> Vec<AuthenticationEvent> {
        env.storage()
            .persistent()
            .get(&DataKey::AuthEvents(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    fn get_or_create_profile(env: &Env, user: &Address) -> SecurityProfile {
        env.storage()
            .persistent()
            .get(&DataKey::SecurityProfile(user.clone()))
            .unwrap_or_else(|| Self::initialize_security_profile(env, user))
    }

    fn is_trusted_device(profile: &SecurityProfile, device_id: &String) -> bool {
        for d in profile.trusted_devices.iter() {
            if d == *device_id {
                return true;
            }
        }
        false
    }

    fn calculate_risk_score(
        profile: &SecurityProfile,
        _device_id: &String,
        auth_method: &AuthMethod,
        is_trusted: bool,
    ) -> u32 {
        let mut score = 0u32;

        if !is_trusted {
            score += 30;
        }
        if profile.failed_attempts > 0 {
            score += profile.failed_attempts * 10;
        }
        match auth_method {
            AuthMethod::Biometric => {}
            AuthMethod::TwoFactor => score += 5,
            AuthMethod::Password => score += 15,
            AuthMethod::DeviceToken => score += 10,
            AuthMethod::SessionResume => score += 20,
        }
        if score > 100 {
            score = 100;
        }
        score
    }

    fn record_auth_event(env: &Env, user: &Address, event: &AuthenticationEvent) {
        let mut events: Vec<AuthenticationEvent> = env
            .storage()
            .persistent()
            .get(&DataKey::AuthEvents(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        events.push_back(event.clone());

        if events.len() > 100 {
            let mut trimmed = Vec::new(env);
            for i in (events.len() - 50)..events.len() {
                if let Some(e) = events.get(i) {
                    trimmed.push_back(e);
                }
            }
            events = trimmed;
        }

        env.storage()
            .persistent()
            .set(&DataKey::AuthEvents(user.clone()), &events);
    }

    fn record_security_alert(env: &Env, user: &Address, alert: SecurityAlert) {
        let mut alerts: Vec<SecurityAlert> = env
            .storage()
            .persistent()
            .get(&DataKey::SecurityAlerts(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        alerts.push_back(alert);
        env.storage()
            .persistent()
            .set(&DataKey::SecurityAlerts(user.clone()), &alerts);
    }
}
