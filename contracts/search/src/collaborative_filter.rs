use soroban_sdk::{Address, Env, String, Vec};
use crate::types::*;

/// Collaborative Filter
/// Manages user similarity and collaborative filtering recommendations
/// Uses pre-computed similarity scores from off-chain ML
pub struct CollaborativeFilter;

impl CollaborativeFilter {
    /// Store user similarity scores from oracle
    pub fn store_similarity(
        env: &Env,
        user_a: Address,
        user_b: Address,
        score: SimilarityScore,
    ) {
        // Store bidirectionally for efficient lookup
        let key_ab = Self::similarity_key(env, &user_a, &user_b);
        let key_ba = Self::similarity_key(env, &user_b, &user_a);
        
        env.storage().persistent().set(&key_ab, &score);
        env.storage().persistent().set(&key_ba, &score);
    }
    
    /// Get similarity score between two users
    pub fn get_similarity(
        env: &Env,
        user_a: Address,
        user_b: Address,
    ) -> Option<SimilarityScore> {
        let key = Self::similarity_key(env, &user_a, &user_b);
        env.storage().persistent().get(&key)
    }
    
    /// Find similar users based on pre-computed scores
    pub fn find_similar_users(
        env: &Env,
        user: Address,
        min_score: u32,
        limit: u32,
    ) -> Vec<Address> {
        // In practice, this would query an indexed structure
        // For now, return empty - would be populated by oracle
        Vec::new(env)
    }
    
    /// Get collaborative recommendations based on similar users
    pub fn get_collaborative_recommendations(
        env: &Env,
        user: Address,
        limit: u32,
    ) -> Vec<Recommendation> {
        // Check cache first
        let cache_key = Self::collab_rec_cache_key(env, &user);
        if let Some(cached) = env.storage().persistent()
            .get::<DataKey, Vec<Recommendation>>(&cache_key) {
            return cached;
        }
        
        // Return empty - would be computed off-chain and submitted by oracle
        Vec::new(env)
    }
    
    /// Store collaborative recommendations from oracle
    pub fn store_collaborative_recommendations(
        env: &Env,
        user: Address,
        recommendations: Vec<Recommendation>,
        ttl_seconds: u64,
    ) {
        let cache_key = Self::collab_rec_cache_key(env, &user);
        env.storage().persistent().set(&cache_key, &recommendations);
        
        // Store expiry
        let expiry_key = Self::collab_rec_expiry_key(env, &user);
        let expiry_time = env.ledger().timestamp() + ttl_seconds;
        env.storage().persistent().set(&expiry_key, &expiry_time);
        
        // Emit event
        env.events().publish(
            (soroban_sdk::symbol_short!("collab_r"),),
            (user, recommendations.len())
        );
    }
    
    /// Calculate collaborative filtering score for a content item
    pub fn calculate_collaborative_score(
        env: &Env,
        user: Address,
        content_id: String,
    ) -> u32 {
        // Get similar users
        let similar_users = Self::find_similar_users(env, user.clone(), 50, 10);
        
        let mut total_score = 0u32;
        let mut count = 0u32;
        
        // Check if similar users interacted with this content
        for i in 0..similar_users.len() {
            if let Some(similar_user) = similar_users.get(i) {
                // Get similarity score
                if let Some(sim_score) = Self::get_similarity(env, user.clone(), similar_user.clone()) {
                    // Check if similar user engaged with content
                    if let Some(interaction_score) = Self::get_user_content_interaction(
                        env,
                        similar_user,
                        content_id.clone(),
                    ) {
                        // Weight by similarity
                        let weighted_score = (interaction_score * sim_score.similarity) / 100;
                        total_score += weighted_score;
                        count += 1;
                    }
                }
            }
        }
        
        if count > 0 {
            total_score / count
        } else {
            0
        }
    }
    
    /// Get user's interaction score with content
    fn get_user_content_interaction(
        env: &Env,
        user: Address,
        content_id: String,
    ) -> Option<u32> {
        // Get user interactions
        let key = DataKey::UserInteractions(user.clone());
        if let Some(interactions) = env.storage().persistent()
            .get::<DataKey, Vec<UserInteraction>>(&key) {
            
            // Find interaction with this content
            for i in 0..interactions.len() {
                if let Some(interaction) = interactions.get(i) {
                    if interaction.content_id == content_id {
                        return Some(Self::interaction_score(&interaction));
                    }
                }
            }
        }
        None
    }
    
    /// Calculate score from interaction type
    fn interaction_score(interaction: &UserInteraction) -> u32 {
        match interaction.interaction_type {
            InteractionType::View => 20,
            InteractionType::Click => 40,
            InteractionType::Enroll => 60,
            InteractionType::Complete => 100,
            InteractionType::Share => 80,
            InteractionType::Like => 50,
            InteractionType::Bookmark => 70,
            InteractionType::Rate => 50,
            InteractionType::Save => 70,
        }
    }
    
    /// Record user interaction for collaborative filtering
    pub fn record_interaction(
        env: &Env,
        user: Address,
        interaction: UserInteraction,
    ) {
        // Get existing interactions
        let key = DataKey::UserInteractions(user.clone());
        let mut interactions = env.storage().persistent()
            .get::<DataKey, Vec<UserInteraction>>(&key)
            .unwrap_or_else(|| Vec::new(env));
        
        // Add new interaction
        interactions.push_back(interaction.clone());
        
        // Keep only recent interactions (last 100)
        if interactions.len() > 100 {
            // Remove oldest
            interactions.remove(0);
        }
        
        env.storage().persistent().set(&key, &interactions);
        
        // Emit event for off-chain processing
        env.events().publish(
            (soroban_sdk::symbol_short!("interact"),),
            (user, interaction.content_id)
        );
    }
    
    /// Get user's interaction history
    pub fn get_user_interactions(
        env: &Env,
        user: Address,
    ) -> Vec<UserInteraction> {
        let key = DataKey::UserInteractions(user);
        env.storage().persistent()
            .get::<DataKey, Vec<UserInteraction>>(&key)
            .unwrap_or_else(|| Vec::new(env))
    }
    
    /// Find content popular among similar users
    pub fn find_trending_among_similar(
        env: &Env,
        user: Address,
        limit: u32,
    ) -> Vec<String> {
        let mut trending = Vec::new(env);
        
        // Get similar users
        let similar_users = Self::find_similar_users(env, user.clone(), 60, 20);
        
        // This would aggregate interactions from similar users
        // For now, return empty - would be computed off-chain
        
        trending
    }
    
    /// Calculate user-to-user similarity based on interactions
    pub fn calculate_user_similarity(
        env: &Env,
        user_a: Address,
        user_b: Address,
    ) -> u32 {
        // Get interaction histories
        let interactions_a = Self::get_user_interactions(env, user_a);
        let interactions_b = Self::get_user_interactions(env, user_b);
        
        let mut overlap = 0u32;
        let mut overlap_weight = 0u32;
        
        // Find common content interactions
        for i in 0..interactions_a.len() {
            if let Some(int_a) = interactions_a.get(i) {
                for j in 0..interactions_b.len() {
                    if let Some(int_b) = interactions_b.get(j) {
                        if int_a.content_id == int_b.content_id {
                            overlap += 1;
                            // Weight by interaction strength
                            let weight_a = Self::interaction_score(&int_a);
                            let weight_b = Self::interaction_score(&int_b);
                            overlap_weight += (weight_a + weight_b) / 2;
                        }
                    }
                }
            }
        }
        
        // Calculate Jaccard similarity
        let total_unique = interactions_a.len() + interactions_b.len() - overlap;
        let jaccard = if total_unique > 0 {
            (overlap * 100) / total_unique
        } else {
            0
        };
        
        // Combine with weighted score
        let weighted_avg = if overlap > 0 {
            overlap_weight / overlap
        } else {
            0
        };
        
        // Final similarity (50% Jaccard, 50% weighted)
        (jaccard + weighted_avg) / 2
    }
    
    /// Generate similarity storage key
    fn similarity_key(env: &Env, user_a: &Address, user_b: &Address) -> DataKey {
        DataKey::SimilarityScores(String::from_str(env, "sim"))
    }
    
    /// Generate collaborative rec cache key
    fn collab_rec_cache_key(env: &Env, user: &Address) -> DataKey {
        DataKey::RecommendationScores(user.clone())
    }
    
    /// Generate collaborative rec expiry key
    fn collab_rec_expiry_key(env: &Env, user: &Address) -> DataKey {
        DataKey::RecommendationScores(user.clone())
    }
}
