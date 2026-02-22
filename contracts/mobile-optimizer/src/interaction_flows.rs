use soroban_sdk::{contracttype, Address, Env, Map, String, Vec};

use crate::types::*;

pub struct InteractionFlows;

impl InteractionFlows {
    pub fn quick_enroll_course(
        env: &Env,
        user: &Address,
        course_id: &String,
        _session_id: &String,
        network_quality: &NetworkQuality,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        if *network_quality == NetworkQuality::Offline {
            return Self::handle_offline_enrollment(env, user, course_id);
        }

        Ok(MobileInteractionResult {
            success: true,
            operation_id: String::from_str(env, "quick_enroll"),
            gas_used: 50000,
            execution_time_ms: Self::estimate_time(network_quality),
            user_message: String::from_str(env, "Successfully enrolled in course!"),
            next_actions: Self::post_enrollment_actions(env),
            cached_data: Map::new(env),
        })
    }

    pub fn quick_update_progress(
        env: &Env,
        _user: &Address,
        course_id: &String,
        module_id: &String,
        progress_percentage: u32,
        _session_id: &String,
        network_quality: &NetworkQuality,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        if progress_percentage > 100 {
            return Err(MobileOptimizerError::InvalidInput);
        }

        if *network_quality == NetworkQuality::Offline {
            let mut cache = Map::new(env);
            cache.set(
                String::from_str(env, "course_id"),
                course_id.clone(),
            );
            cache.set(
                String::from_str(env, "module_id"),
                module_id.clone(),
            );
            return Ok(MobileInteractionResult {
                success: true,
                operation_id: String::from_str(env, "offline_progress"),
                gas_used: 0,
                execution_time_ms: 50,
                user_message: String::from_str(
                    env,
                    "Progress saved offline. Will sync when connected.",
                ),
                next_actions: Vec::new(env),
                cached_data: cache,
            });
        }

        Ok(MobileInteractionResult {
            success: true,
            operation_id: String::from_str(env, "progress_update"),
            gas_used: 30000,
            execution_time_ms: Self::estimate_time(network_quality),
            user_message: String::from_str(env, "Progress updated successfully!"),
            next_actions: Vec::new(env),
            cached_data: Map::new(env),
        })
    }

    pub fn quick_claim_certificate(
        env: &Env,
        _user: &Address,
        _course_id: &String,
        _session_id: &String,
        network_quality: &NetworkQuality,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        if *network_quality == NetworkQuality::Offline {
            return Err(MobileOptimizerError::InteractionFailed);
        }

        Ok(MobileInteractionResult {
            success: true,
            operation_id: String::from_str(env, "claim_certificate"),
            gas_used: 115000,
            execution_time_ms: Self::estimate_time(network_quality),
            user_message: String::from_str(env, "Certificate claimed successfully!"),
            next_actions: Self::post_certificate_actions(env),
            cached_data: Map::new(env),
        })
    }

    pub fn quick_search(
        env: &Env,
        _user: &Address,
        _query: &String,
        _session_id: &String,
        network_quality: &NetworkQuality,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        if *network_quality == NetworkQuality::Offline {
            return Ok(MobileInteractionResult {
                success: true,
                operation_id: String::from_str(env, "cached_search"),
                gas_used: 0,
                execution_time_ms: 10,
                user_message: String::from_str(
                    env,
                    "Showing cached results (offline)",
                ),
                next_actions: Vec::new(env),
                cached_data: Map::new(env),
            });
        }

        Ok(MobileInteractionResult {
            success: true,
            operation_id: String::from_str(env, "search"),
            gas_used: 15000,
            execution_time_ms: Self::estimate_time(network_quality),
            user_message: String::from_str(env, "Search completed"),
            next_actions: Vec::new(env),
            cached_data: Map::new(env),
        })
    }

    fn handle_offline_enrollment(
        env: &Env,
        _user: &Address,
        _course_id: &String,
    ) -> Result<MobileInteractionResult, MobileOptimizerError> {
        Ok(MobileInteractionResult {
            success: true,
            operation_id: String::from_str(env, "offline_enroll"),
            gas_used: 0,
            execution_time_ms: 50,
            user_message: String::from_str(
                env,
                "Enrollment queued for when connection is restored",
            ),
            next_actions: Vec::new(env),
            cached_data: Map::new(env),
        })
    }

    fn estimate_time(network_quality: &NetworkQuality) -> u32 {
        match network_quality {
            NetworkQuality::Excellent => 100,
            NetworkQuality::Good => 250,
            NetworkQuality::Fair => 500,
            NetworkQuality::Poor => 1000,
            NetworkQuality::Offline => 0,
        }
    }

    fn post_enrollment_actions(env: &Env) -> Vec<String> {
        let mut actions = Vec::new(env);
        actions.push_back(String::from_str(env, "Start first module"));
        actions.push_back(String::from_str(env, "View course materials"));
        actions.push_back(String::from_str(env, "Join course discussion"));
        actions
    }

    fn post_certificate_actions(env: &Env) -> Vec<String> {
        let mut actions = Vec::new(env);
        actions.push_back(String::from_str(env, "Share certificate"));
        actions.push_back(String::from_str(env, "Explore related courses"));
        actions.push_back(String::from_str(env, "Leave course review"));
        actions
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileInteractionResult {
    pub success: bool,
    pub operation_id: String,
    pub gas_used: u64,
    pub execution_time_ms: u32,
    pub user_message: String,
    pub next_actions: Vec<String>,
    pub cached_data: Map<String, String>,
}
