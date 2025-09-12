use soroban_sdk::{Address, Env, String, Vec, Map};
use crate::types::*;

/// Mobile session management for persistent interactions
pub struct SessionManager;

impl SessionManager {
    /// Create a new mobile session
    pub fn create_session(
        env: &Env,
        user: Address,
        device_id: String,
        network_quality: NetworkQuality,
    ) -> Result<String, SessionError> {
        let session_id = Self::generate_session_id(env, &user, &device_id);
        
        let session = MobileSession {
            session_id: session_id.clone(),
            user: user.clone(),
            device_id,
            created_at: env.ledger().timestamp(),
            last_activity: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + 86400, // 24 hours
            network_quality,
            cached_data: Map::new(env),
            pending_operations: Vec::new(env),
            preferences: Self::get_default_mobile_preferences(env),
            session_state: SessionState::Active,
        };

        // Store the session
        env.storage().persistent().set(&DataKey::MobileSession(session_id.clone()), &session);
        
        // Add to user's session list
        Self::add_to_user_sessions(env, &user, &session_id)?;

        Ok(session_id)
    }

    /// Update session activity and network quality
    pub fn update_session(
        env: &Env,
        session_id: String,
        network_quality: Option<NetworkQuality>,
        state: Option<SessionState>,
    ) -> Result<(), SessionError> {
        let mut session = env.storage().persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(SessionError::SessionNotFound)?;

        session.last_activity = env.ledger().timestamp();
        
        if let Some(quality) = network_quality {
            session.network_quality = quality;
        }
        
        if let Some(new_state) = state {
            session.session_state = new_state;
        }

        env.storage().persistent().set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    /// Get active session for user and device
    pub fn get_active_session(
        env: &Env,
        user: &Address,
        device_id: &String,
    ) -> Result<MobileSession, SessionError> {
        let user_sessions = env.storage().persistent()
            .get(&DataKey::UserSessions(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        for session_id in user_sessions {
            if let Some(session) = env.storage().persistent().get(&DataKey::MobileSession(session_id)) {
                if session.device_id == *device_id && 
                   session.session_state == SessionState::Active &&
                   env.ledger().timestamp() < session.expires_at {
                    return Ok(session);
                }
            }
        }

        Err(SessionError::NoActiveSession)
    }

    /// Cache data in mobile session
    pub fn cache_data(
        env: &Env,
        session_id: String,
        key: String,
        value: String,
    ) -> Result<(), SessionError> {
        let mut session = env.storage().persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(SessionError::SessionNotFound)?;

        session.cached_data.set(key, value);
        session.last_activity = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    /// Retrieve cached data from mobile session
    pub fn get_cached_data(
        env: &Env,
        session_id: String,
        key: String,
    ) -> Result<Option<String>, SessionError> {
        let session = env.storage().persistent()
            .get(&DataKey::MobileSession(session_id))
            .ok_or(SessionError::SessionNotFound)?;

        Ok(session.cached_data.get(key))
    }

    /// Add pending operation to session
    pub fn add_pending_operation(
        env: &Env,
        session_id: String,
        batch_id: String,
    ) -> Result<(), SessionError> {
        let mut session = env.storage().persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(SessionError::SessionNotFound)?;

        session.pending_operations.push_back(batch_id);
        session.last_activity = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    /// Remove completed operation from session
    pub fn remove_pending_operation(
        env: &Env,
        session_id: String,
        batch_id: String,
    ) -> Result<(), SessionError> {
        let mut session = env.storage().persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(SessionError::SessionNotFound)?;

        // Find and remove the batch_id
        let mut new_pending = Vec::new(env);
        for pending_id in &session.pending_operations {
            if pending_id != &batch_id {
                new_pending.push_back(pending_id.clone());
            }
        }
        session.pending_operations = new_pending;
        session.last_activity = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    /// Update mobile preferences for session
    pub fn update_preferences(
        env: &Env,
        session_id: String,
        preferences: MobilePreferences,
    ) -> Result<(), SessionError> {
        let mut session = env.storage().persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(SessionError::SessionNotFound)?;

        session.preferences = preferences;
        session.last_activity = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    /// Suspend session (app goes to background)
    pub fn suspend_session(env: &Env, session_id: String) -> Result<(), SessionError> {
        Self::update_session(env, session_id, None, Some(SessionState::Suspended))
    }

    /// Resume session (app comes to foreground)
    pub fn resume_session(
        env: &Env,
        session_id: String,
        network_quality: NetworkQuality,
    ) -> Result<(), SessionError> {
        Self::update_session(env, session_id, Some(network_quality), Some(SessionState::Active))
    }

    /// End session
    pub fn end_session(env: &Env, session_id: String) -> Result<(), SessionError> {
        let mut session = env.storage().persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(SessionError::SessionNotFound)?;

        session.session_state = SessionState::Expired;
        env.storage().persistent().set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(env: &Env) -> Result<u32, SessionError> {
        let current_time = env.ledger().timestamp();
        let cleanup_key = DataKey::SessionCleanup(current_time / 3600); // Hourly cleanup
        
        // This would iterate through sessions and clean up expired ones
        // For now, return a count of cleaned sessions
        Ok(0)
    }

    /// Get session statistics
    pub fn get_session_stats(
        env: &Env,
        user: &Address,
    ) -> Result<SessionStats, SessionError> {
        let user_sessions = env.storage().persistent()
            .get(&DataKey::UserSessions(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut active_sessions = 0u32;
        let mut total_sessions = user_sessions.len() as u32;
        let mut network_quality_distribution = Map::new(env);

        for session_id in user_sessions {
            if let Some(session) = env.storage().persistent().get(&DataKey::MobileSession(session_id)) {
                if session.session_state == SessionState::Active {
                    active_sessions += 1;
                }

                // Update network quality distribution
                let quality_key = Self::network_quality_to_string(&session.network_quality);
                let current_count = network_quality_distribution.get(quality_key.clone()).unwrap_or(0);
                network_quality_distribution.set(quality_key, current_count + 1);
            }
        }

        Ok(SessionStats {
            user: user.clone(),
            total_sessions,
            active_sessions,
            network_quality_distribution,
            average_session_duration: 3600, // Would calculate from actual data
            most_common_device: String::from_str(env, "mobile_device"),
        })
    }

    /// Sync session state across devices
    pub fn sync_session_state(
        env: &Env,
        user: &Address,
        source_session_id: String,
        target_device_id: String,
    ) -> Result<String, SessionError> {
        let source_session = env.storage().persistent()
            .get(&DataKey::MobileSession(source_session_id))
            .ok_or(SessionError::SessionNotFound)?;

        if source_session.user != *user {
            return Err(SessionError::Unauthorized);
        }

        // Create new session for target device with synced state
        let target_session_id = Self::generate_session_id(env, user, &target_device_id);
        
        let target_session = MobileSession {
            session_id: target_session_id.clone(),
            user: user.clone(),
            device_id: target_device_id,
            created_at: env.ledger().timestamp(),
            last_activity: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + 86400,
            network_quality: NetworkQuality::Good, // Default for new device
            cached_data: source_session.cached_data.clone(),
            pending_operations: source_session.pending_operations.clone(),
            preferences: source_session.preferences.clone(),
            session_state: SessionState::Active,
        };

        env.storage().persistent().set(&DataKey::MobileSession(target_session_id.clone()), &target_session);
        Self::add_to_user_sessions(env, user, &target_session_id)?;

        Ok(target_session_id)
    }

    /// Generate unique session ID
    fn generate_session_id(env: &Env, user: &Address, device_id: &String) -> String {
        let timestamp = env.ledger().timestamp();
        let user_str = user.to_string();
        String::from_str(env, &format!("session_{}_{}_{}", timestamp, user_str.len(), device_id.len()))
    }

    /// Add session to user's session list
    fn add_to_user_sessions(env: &Env, user: &Address, session_id: &String) -> Result<(), SessionError> {
        let mut user_sessions = env.storage().persistent()
            .get(&DataKey::UserSessions(user.clone()))
            .unwrap_or_else(|| Vec::new(env));
        
        user_sessions.push_back(session_id.clone());
        env.storage().persistent().set(&DataKey::UserSessions(user.clone()), &user_sessions);
        
        Ok(())
    }

    /// Get default mobile preferences
    fn get_default_mobile_preferences(env: &Env) -> MobilePreferences {
        MobilePreferences {
            auto_batch_operations: true,
            max_batch_size: 5,
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
            },
            data_usage_mode: DataUsageMode::Conservative,
            battery_optimization: true,
        }
    }

    /// Convert network quality to string for storage
    fn network_quality_to_string(quality: &NetworkQuality) -> String {
        match quality {
            NetworkQuality::Excellent => String::from_str(&quality.env(), "excellent"),
            NetworkQuality::Good => String::from_str(&quality.env(), "good"),
            NetworkQuality::Fair => String::from_str(&quality.env(), "fair"),
            NetworkQuality::Poor => String::from_str(&quality.env(), "poor"),
            NetworkQuality::Offline => String::from_str(&quality.env(), "offline"),
        }
    }

    /// Optimize session for mobile performance
    pub fn optimize_session_performance(
        env: &Env,
        session_id: String,
    ) -> Result<SessionOptimization, SessionError> {
        let session = env.storage().persistent()
            .get(&DataKey::MobileSession(session_id))
            .ok_or(SessionError::SessionNotFound)?;

        let mut optimizations = Vec::new(env);
        let mut performance_score = 100u32;

        // Check cache size
        if session.cached_data.len() > 50 {
            optimizations.push_back(String::from_str(env, "Clear old cached data to improve performance"));
            performance_score -= 10;
        }

        // Check pending operations
        if session.pending_operations.len() > 10 {
            optimizations.push_back(String::from_str(env, "Execute or cancel old pending operations"));
            performance_score -= 15;
        }

        // Check network quality
        match session.network_quality {
            NetworkQuality::Poor | NetworkQuality::Offline => {
                optimizations.push_back(String::from_str(env, "Switch to WiFi for better performance"));
                performance_score -= 20;
            },
            NetworkQuality::Fair => {
                optimizations.push_back(String::from_str(env, "Consider batching operations for better efficiency"));
                performance_score -= 10;
            },
            _ => {}
        }

        // Check session age
        let session_age = env.ledger().timestamp() - session.created_at;
        if session_age > 86400 { // 24 hours
            optimizations.push_back(String::from_str(env, "Consider refreshing session for optimal performance"));
            performance_score -= 5;
        }

        Ok(SessionOptimization {
            session_id: session.session_id,
            performance_score,
            optimization_suggestions: optimizations,
            recommended_actions: Self::get_recommended_actions(env, &session),
        })
    }

    /// Get recommended actions for session optimization
    fn get_recommended_actions(env: &Env, session: &MobileSession) -> Vec<String> {
        let mut actions = Vec::new(env);

        if session.preferences.data_usage_mode == DataUsageMode::Unlimited {
            actions.push_back(String::from_str(env, "Enable data usage optimization for mobile networks"));
        }

        if !session.preferences.auto_batch_operations {
            actions.push_back(String::from_str(env, "Enable automatic operation batching"));
        }

        if !session.preferences.battery_optimization {
            actions.push_back(String::from_str(env, "Enable battery optimization features"));
        }

        actions
    }
}

/// Session statistics
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStats {
    pub user: Address,
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub network_quality_distribution: Map<String, u32>,
    pub average_session_duration: u64,
    pub most_common_device: String,
}

/// Session optimization result
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionOptimization {
    pub session_id: String,
    pub performance_score: u32, // 0-100
    pub optimization_suggestions: Vec<String>,
    pub recommended_actions: Vec<String>,
}

/// Session management errors
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionError {
    SessionNotFound,
    NoActiveSession,
    Unauthorized,
    SessionExpired,
    InvalidDevice,
    SyncFailed,
}
