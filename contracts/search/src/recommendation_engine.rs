use crate::types::*;
use soroban_sdk::{Address, Env, Map, String, Vec};

/// Recommendation Engine
/// Generates personalized course recommendations based on user learning history and preferences
pub struct RecommendationEngine;

impl RecommendationEngine {
    /// Generate personalized recommendations for a user
    pub fn generate_recommendations(env: &Env, user: Address, limit: u32) -> Vec<Recommendation> {
        let mut recommendations = Vec::new(env);

        // Check for cached recommendations first
        if let Some(cached) = Self::get_cached_recommendations(env, &user) {
            // Return cached if not expired
            let current_time = env.ledger().timestamp();
            let mut valid_recs = Vec::new(env);

            for i in 0..cached.len() {
                if let Some(rec) = cached.get(i) {
                    if rec.expires_at > current_time {
                        valid_recs.push_back(rec);
                        if valid_recs.len() >= limit {
                            break;
                        }
                    }
                }
            }

            if !valid_recs.is_empty() {
                return valid_recs;
            }
        }

        // No valid cache, return empty (oracle will compute)
        recommendations
    }

    /// Store recommendation scores from oracle
    pub fn store_recommendations(env: &Env, user: Address, recommendations: Vec<Recommendation>) {
        let key = DataKey::RecommendationScores(user);
        env.storage().persistent().set(&key, &recommendations);
    }

    /// Get cached recommendations
    fn get_cached_recommendations(env: &Env, user: &Address) -> Option<Vec<Recommendation>> {
        let key = DataKey::RecommendationScores(user.clone());
        env.storage().persistent().get(&key)
    }

    /// Update user profile from learning activity
    pub fn update_user_profile(env: &Env, user: Address, course_id: String, completed: bool) {
        let key = DataKey::UserProfile(user.clone());
        let mut profile = env
            .storage()
            .persistent()
            .get::<DataKey, UserProfile>(&key)
            .unwrap_or(UserProfile {
                user_address: user.clone(),
                completed_courses: Vec::new(env),
                skill_levels: Map::new(env),
                interaction_counts: Map::new(env),
                preference_scores: Vec::new(env),
                last_updated: env.ledger().timestamp(),
            });

        // Update completed courses
        if completed && !profile.completed_courses.contains(&course_id) {
            profile.completed_courses.push_back(course_id.clone());
        }

        // Update timestamp
        profile.last_updated = env.ledger().timestamp();

        // Store updated profile
        env.storage().persistent().set(&key, &profile);

        // Emit event for off-chain processing
        env.events().publish(
            (soroban_sdk::symbol_short!("profile"),),
            (user, course_id, completed),
        );
    }

    /// Get user learning profile
    pub fn get_user_profile(env: &Env, user: Address) -> Option<UserProfile> {
        let key = DataKey::UserProfile(user);
        env.storage().persistent().get(&key)
    }

    /// Calculate recommendation score for content (on-chain heuristic)
    pub fn calculate_recommendation_score(env: &Env, user: &Address, content_id: &String) -> u32 {
        let mut score = 500u32; // Base score

        // Get user profile
        if let Some(profile) = Self::get_user_profile(env, user.clone()) {
            // Boost if related to completed courses
            let completion_boost = Self::calculate_completion_boost(&profile, content_id);
            score += completion_boost;

            // Boost based on skill gaps
            let skill_gap_boost = Self::calculate_skill_gap_boost(&profile, content_id);
            score += skill_gap_boost;

            // Category preference boost
            let category_boost = Self::calculate_category_boost(&profile, content_id);
            score += category_boost;
        }

        score.min(1000) // Cap at 1000
    }

    /// Calculate boost based on completed courses
    fn calculate_completion_boost(profile: &UserProfile, content_id: &String) -> u32 {
        // Check if this is a natural progression
        // Simple heuristic: return fixed boost if user has completions
        if profile.completed_courses.len() > 0 {
            100
        } else {
            50 // Beginner boost
        }
    }

    /// Calculate boost based on skill gaps
    fn calculate_skill_gap_boost(profile: &UserProfile, content_id: &String) -> u32 {
        // Analyze which skills user needs
        // Return boost if content fills gap
        150 // Simplified for now
    }

    /// Calculate boost based on category preferences
    fn calculate_category_boost(profile: &UserProfile, content_id: &String) -> u32 {
        // Check interaction history with categories
        100 // Simplified for now
    }

    /// Find users with similar learning patterns (for collaborative filtering)
    pub fn find_similar_users(env: &Env, user: Address, limit: u32) -> Vec<Address> {
        // This would be computed off-chain and stored
        // Return empty for now - oracle will provide
        Vec::new(env)
    }

    /// Predict course completion likelihood
    pub fn predict_completion_likelihood(env: &Env, user: Address, course_id: String) -> u32 {
        // Get user profile
        if let Some(profile) = Self::get_user_profile(env, user) {
            // Calculate likelihood based on profile
            let base_likelihood = 50u32; // 50%

            // Boost if user has high completion rate
            let completion_rate = if profile.completed_courses.len() > 0 {
                // Would calculate actual rate
                70
            } else {
                40
            };

            // Adjust based on course difficulty vs user skill level
            let difficulty_match = 60; // Simplified

            // Combine factors
            let likelihood = (base_likelihood + completion_rate + difficulty_match) / 3;
            likelihood.min(100)
        } else {
            50 // Default 50% for new users
        }
    }

    /// Get trending recommendations (popular among similar users)
    pub fn get_trending_recommendations(
        env: &Env,
        user: Option<Address>,
        limit: u32,
    ) -> Vec<Recommendation> {
        // Would return trending courses based on community activity
        // This data comes from off-chain analytics
        Vec::new(env)
    }

    /// Get "continue learning" recommendations (incomplete courses)
    pub fn get_continue_learning_recommendations(env: &Env, user: Address) -> Vec<Recommendation> {
        // Find courses user started but didn't complete
        // Return as recommendations
        Vec::new(env)
    }
}
