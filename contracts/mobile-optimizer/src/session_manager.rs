use soroban_sdk::{contracttype, Address, Env, Map, String, Vec};

use crate::types::*;

pub struct SessionManager;

impl SessionManager {
    pub fn create_session(
        env: &Env,
        user: Address,
        device_id: String,
        preferences: MobilePreferences,
    ) -> Result<String, MobileOptimizerError> {
        let session_id = String::from_str(env, "session");

        let session = MobileSession {
            session_id: session_id.clone(),
            user: user.clone(),
            device_id,
            created_at: env.ledger().timestamp(),
            last_activity: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + 86400,
            network_quality: NetworkQuality::Good,
            cached_data: Map::new(env),
            pending_operations: Vec::new(env),
            preferences,
            session_state: SessionState::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::MobileSession(session_id.clone()), &session);
        Self::add_to_user_sessions(env, &user, &session_id);

        Ok(session_id)
    }

    pub fn get_session(
        env: &Env,
        session_id: &String,
    ) -> Result<MobileSession, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(MobileOptimizerError::SessionNotFound)
    }

    pub fn update_session(
        env: &Env,
        session_id: String,
        network_quality: Option<NetworkQuality>,
        state: Option<SessionState>,
    ) -> Result<(), MobileOptimizerError> {
        let mut session: MobileSession = env
            .storage()
            .persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(MobileOptimizerError::SessionNotFound)?;

        session.last_activity = env.ledger().timestamp();
        if let Some(quality) = network_quality {
            session.network_quality = quality;
        }
        if let Some(new_state) = state {
            session.session_state = new_state;
        }

        env.storage()
            .persistent()
            .set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    pub fn update_preferences(
        env: &Env,
        session_id: String,
        preferences: MobilePreferences,
    ) -> Result<(), MobileOptimizerError> {
        let mut session: MobileSession = env
            .storage()
            .persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(MobileOptimizerError::SessionNotFound)?;

        session.preferences = preferences;
        session.last_activity = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    pub fn cache_data(
        env: &Env,
        session_id: String,
        key: String,
        value: String,
    ) -> Result<(), MobileOptimizerError> {
        let mut session: MobileSession = env
            .storage()
            .persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(MobileOptimizerError::SessionNotFound)?;

        session.cached_data.set(key, value);
        session.last_activity = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    pub fn get_cached_data(
        env: &Env,
        session_id: &String,
        key: &String,
    ) -> Result<Option<String>, MobileOptimizerError> {
        let session: MobileSession = env
            .storage()
            .persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(MobileOptimizerError::SessionNotFound)?;
        Ok(session.cached_data.get(key.clone()))
    }

    pub fn add_pending_operation(
        env: &Env,
        session_id: String,
        batch_id: String,
    ) -> Result<(), MobileOptimizerError> {
        let mut session: MobileSession = env
            .storage()
            .persistent()
            .get(&DataKey::MobileSession(session_id.clone()))
            .ok_or(MobileOptimizerError::SessionNotFound)?;

        session.pending_operations.push_back(batch_id);
        session.last_activity = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::MobileSession(session_id), &session);
        Ok(())
    }

    pub fn suspend_session(env: &Env, session_id: String) -> Result<(), MobileOptimizerError> {
        Self::update_session(env, session_id, None, Some(SessionState::Suspended))
    }

    pub fn resume_session(
        env: &Env,
        session_id: String,
        network_quality: NetworkQuality,
    ) -> Result<(), MobileOptimizerError> {
        Self::update_session(
            env,
            session_id,
            Some(network_quality),
            Some(SessionState::Active),
        )
    }

    pub fn end_session(env: &Env, session_id: String) -> Result<(), MobileOptimizerError> {
        Self::update_session(env, session_id, None, Some(SessionState::Expired))
    }

    pub fn sync_session_state(
        env: &Env,
        user: &Address,
        source_session_id: String,
        target_device_id: String,
    ) -> Result<String, MobileOptimizerError> {
        let source: MobileSession = env
            .storage()
            .persistent()
            .get(&DataKey::MobileSession(source_session_id))
            .ok_or(MobileOptimizerError::SessionNotFound)?;

        if source.user != *user {
            return Err(MobileOptimizerError::Unauthorized);
        }

        let target_session_id = String::from_str(env, "synced_session");
        let target = MobileSession {
            session_id: target_session_id.clone(),
            user: user.clone(),
            device_id: target_device_id,
            created_at: env.ledger().timestamp(),
            last_activity: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + 86400,
            network_quality: NetworkQuality::Good,
            cached_data: source.cached_data.clone(),
            pending_operations: source.pending_operations.clone(),
            preferences: source.preferences.clone(),
            session_state: SessionState::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::MobileSession(target_session_id.clone()), &target);
        Self::add_to_user_sessions(env, user, &target_session_id);

        Ok(target_session_id)
    }

    pub fn get_session_stats(env: &Env, user: &Address) -> SessionStats {
        let sessions: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::UserSessions(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut active = 0u32;
        let total = sessions.len();

        for sid in sessions.iter() {
            if let Some(session) = env
                .storage()
                .persistent()
                .get::<DataKey, MobileSession>(&DataKey::MobileSession(sid))
            {
                if session.session_state == SessionState::Active {
                    active += 1;
                }
            }
        }

        SessionStats {
            total_sessions: total,
            active_sessions: active,
        }
    }

    pub fn optimize_session_performance(
        env: &Env,
        session_id: String,
    ) -> Result<SessionOptimization, MobileOptimizerError> {
        let session: MobileSession = env
            .storage()
            .persistent()
            .get(&DataKey::MobileSession(session_id))
            .ok_or(MobileOptimizerError::SessionNotFound)?;

        let mut suggestions = Vec::new(env);
        let mut score = 100u32;

        if session.cached_data.len() > 50 {
            suggestions.push_back(String::from_str(
                env,
                "Clear old cached data to improve performance",
            ));
            score = score.saturating_sub(10);
        }
        if session.pending_operations.len() > 10 {
            suggestions.push_back(String::from_str(
                env,
                "Execute or cancel old pending operations",
            ));
            score = score.saturating_sub(15);
        }
        match session.network_quality {
            NetworkQuality::Poor | NetworkQuality::Offline => {
                suggestions.push_back(String::from_str(
                    env,
                    "Switch to WiFi for better performance",
                ));
                score = score.saturating_sub(20);
            }
            NetworkQuality::Fair => {
                suggestions.push_back(String::from_str(
                    env,
                    "Consider batching operations for efficiency",
                ));
                score = score.saturating_sub(10);
            }
            _ => {}
        }

        Ok(SessionOptimization {
            session_id: session.session_id,
            performance_score: score,
            optimization_suggestions: suggestions,
        })
    }

    fn add_to_user_sessions(env: &Env, user: &Address, session_id: &String) {
        let mut sessions: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::UserSessions(user.clone()))
            .unwrap_or_else(|| Vec::new(env));
        sessions.push_back(session_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::UserSessions(user.clone()), &sessions);
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStats {
    pub total_sessions: u32,
    pub active_sessions: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionOptimization {
    pub session_id: String,
    pub performance_score: u32,
    pub optimization_suggestions: Vec<String>,
}
