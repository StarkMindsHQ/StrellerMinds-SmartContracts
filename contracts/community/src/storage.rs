use soroban_sdk::{Address, Env};

use crate::errors::Error;
use crate::types::{CommunityConfig, CommunityKey, ModeratorRole};

pub struct CommunityStorage;

impl CommunityStorage {
    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&CommunityKey::Admin)
    }

    pub fn require_admin(env: &Env, addr: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&CommunityKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin != *addr {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    pub fn require_moderator(env: &Env, addr: &Address) -> Result<(), Error> {
        let role: Option<ModeratorRole> = env
            .storage()
            .persistent()
            .get(&CommunityKey::Moderator(addr.clone()));
        
        if role.is_none() {
            Self::require_admin(env, addr)?;
        }
        Ok(())
    }

    pub fn get_config(env: &Env) -> CommunityConfig {
        env.storage()
            .instance()
            .get(&CommunityKey::Config)
            .unwrap_or(CommunityConfig {
                post_xp_reward: 10,
                reply_xp_reward: 5,
                solution_xp_reward: 50,
                contribution_base_xp: 100,
                contribution_base_tokens: 1000,
                mentor_session_xp: 75,
                event_attendance_xp: 25,
                min_reputation_to_moderate: 500,
                max_reports_per_day: 10,
                vote_weight_threshold: 100,
            })
    }

    pub fn set_config(env: &Env, config: &CommunityConfig) {
        env.storage().instance().set(&CommunityKey::Config, config);
    }

    pub fn increment_counter(env: &Env, key: CommunityKey) -> u64 {
        let current: u64 = env.storage().persistent().get(&key).unwrap_or(0);
        let next = current + 1;
        env.storage().persistent().set(&key, &next);
        next
    }
}
