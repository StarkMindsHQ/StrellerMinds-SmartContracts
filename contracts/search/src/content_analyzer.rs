use soroban_sdk::{Env, String, Vec};
use crate::types::*;

/// Content Analyzer
/// Manages content analysis results from off-chain NLP/ML services
/// Provides automatic tagging and metadata enrichment
pub struct ContentAnalyzer;

impl ContentAnalyzer {
    /// Store content analysis from oracle
    pub fn store_analysis(
        env: &Env,
        content_id: String,
        analysis: ContentAnalysis,
    ) {
        let key = DataKey::ContentAnalysis(content_id.clone());
        env.storage().persistent().set(&key, &analysis);
        
        // Index by tags for searchability
        Self::index_by_tags(env, &content_id, &analysis.auto_generated_tags);
        
        // Index by skills
        Self::index_by_skills(env, &content_id, &analysis.identified_skills);
        
        // Emit event
        env.events().publish(
            (soroban_sdk::symbol_short!("analyzed"),),
            (content_id, analysis.quality_score)
        );
    }
    
    /// Get content analysis
    pub fn get_analysis(
        env: &Env,
        content_id: String,
    ) -> Option<ContentAnalysis> {
        let key = DataKey::ContentAnalysis(content_id);
        env.storage().persistent().get(&key)
    }
    
    /// Index content by tags for tag-based search
    fn index_by_tags(env: &Env, content_id: &String, tags: &Vec<String>) {
        for i in 0..tags.len() {
            if let Some(tag) = tags.get(i) {
                // Get existing content IDs for this tag
                let tag_key = Self::tag_index_key(env, &tag);
                let mut content_ids = env.storage().persistent()
                    .get::<String, Vec<String>>(&tag_key)
                    .unwrap_or_else(|| Vec::new(env));
                
                // Add this content ID if not already present
                if !content_ids.contains(content_id) {
                    content_ids.push_back(content_id.clone());
                    env.storage().persistent().set(&tag_key, &content_ids);
                }
            }
        }
    }
    
    /// Index content by skills
    fn index_by_skills(env: &Env, content_id: &String, skills: &Vec<Skill>) {
        for i in 0..skills.len() {
            if let Some(skill) = skills.get(i) {
                // Get existing content IDs for this skill
                let skill_key = Self::skill_index_key(env, &skill.skill_name);
                let mut content_ids = env.storage().persistent()
                    .get::<String, Vec<String>>(&skill_key)
                    .unwrap_or_else(|| Vec::new(env));
                
                // Add this content ID if not already present
                if !content_ids.contains(content_id) {
                    content_ids.push_back(content_id.clone());
                    env.storage().persistent().set(&skill_key, &content_ids);
                }
            }
        }
    }
    
    /// Search content by tag
    pub fn find_by_tag(
        env: &Env,
        tag: String,
    ) -> Vec<String> {
        let tag_key = Self::tag_index_key(env, &tag);
        env.storage().persistent()
            .get::<String, Vec<String>>(&tag_key)
            .unwrap_or_else(|| Vec::new(env))
    }
    
    /// Search content by skill
    pub fn find_by_skill(
        env: &Env,
        skill_name: String,
    ) -> Vec<String> {
        let skill_key = Self::skill_index_key(env, &skill_name);
        env.storage().persistent()
            .get::<String, Vec<String>>(&skill_key)
            .unwrap_or_else(|| Vec::new(env))
    }
    
    /// Get all auto-generated tags for content
    pub fn get_tags(
        env: &Env,
        content_id: String,
    ) -> Vec<String> {
        if let Some(analysis) = Self::get_analysis(env, content_id) {
            analysis.auto_generated_tags
        } else {
            Vec::new(env)
        }
    }
    
    /// Get identified skills for content
    pub fn get_skills(
        env: &Env,
        content_id: String,
    ) -> Vec<Skill> {
        if let Some(analysis) = Self::get_analysis(env, content_id) {
            analysis.identified_skills
        } else {
            Vec::new(env)
        }
    }
    
    /// Get extracted topics for content
    pub fn get_topics(
        env: &Env,
        content_id: String,
    ) -> Vec<Topic> {
        if let Some(analysis) = Self::get_analysis(env, content_id) {
            analysis.extracted_topics
        } else {
            Vec::new(env)
        }
    }
    
    /// Get content quality score
    pub fn get_quality_score(
        env: &Env,
        content_id: String,
    ) -> u32 {
        if let Some(analysis) = Self::get_analysis(env, content_id) {
            analysis.quality_score
        } else {
            0
        }
    }
    
    /// Get content difficulty score
    pub fn get_difficulty_score(
        env: &Env,
        content_id: String,
    ) -> u32 {
        if let Some(analysis) = Self::get_analysis(env, content_id) {
            analysis.difficulty_score
        } else {
            50  // Default to medium difficulty
        }
    }
    
    /// Estimate if content matches user skill level
    pub fn matches_skill_level(
        env: &Env,
        content_id: String,
        user_skill_level: u32,  // 0-100
    ) -> bool {
        let content_difficulty = Self::get_difficulty_score(env, content_id);
        
        // Allow some range (±20 points)
        let lower_bound = if user_skill_level >= 20 {
            user_skill_level - 20
        } else {
            0
        };
        let upper_bound = (user_skill_level + 20).min(100);
        
        content_difficulty >= lower_bound && content_difficulty <= upper_bound
    }
    
    /// Find content with similar topics
    pub fn find_similar_by_topics(
        env: &Env,
        content_id: String,
        limit: u32,
    ) -> Vec<String> {
        let mut similar = Vec::new(env);
        
        // Get topics for this content
        let topics = Self::get_topics(env, content_id.clone());
        
        // Find other content with overlapping topics
        // This would be more efficient with proper indexing
        // For now, return empty (would be computed off-chain)
        
        similar
    }
    
    /// Calculate content similarity score
    pub fn calculate_content_similarity(
        env: &Env,
        content_a: String,
        content_b: String,
    ) -> u32 {
        // Get analyses for both
        let analysis_a = Self::get_analysis(env, content_a);
        let analysis_b = Self::get_analysis(env, content_b);
        
        if analysis_a.is_none() || analysis_b.is_none() {
            return 0;
        }
        
        let a = analysis_a.unwrap();
        let b = analysis_b.unwrap();
        
        let mut similarity = 0u32;
        
        // Compare tags
        let tag_similarity = Self::calculate_tag_similarity(&a.auto_generated_tags, &b.auto_generated_tags);
        similarity += tag_similarity;
        
        // Compare topics
        let topic_similarity = Self::calculate_topic_similarity(&a.extracted_topics, &b.extracted_topics);
        similarity += topic_similarity;
        
        // Compare difficulty
        let difficulty_diff = if a.difficulty_score > b.difficulty_score {
            a.difficulty_score - b.difficulty_score
        } else {
            b.difficulty_score - a.difficulty_score
        };
        let difficulty_similarity = 100 - difficulty_diff;
        similarity += difficulty_similarity;
        
        // Average the scores
        similarity / 3
    }
    
    /// Calculate tag overlap similarity
    fn calculate_tag_similarity(tags_a: &Vec<String>, tags_b: &Vec<String>) -> u32 {
        let mut overlap = 0u32;
        
        for i in 0..tags_a.len() {
            if let Some(tag_a) = tags_a.get(i) {
                if tags_b.contains(&tag_a) {
                    overlap += 1;
                }
            }
        }
        
        // Normalize to 0-100 scale
        let total_unique = tags_a.len() + tags_b.len() - overlap;
        if total_unique > 0 {
            (overlap * 100) / total_unique
        } else {
            0
        }
    }
    
    /// Calculate topic overlap similarity
    fn calculate_topic_similarity(topics_a: &Vec<Topic>, topics_b: &Vec<Topic>) -> u32 {
        let mut overlap = 0u32;
        
        for i in 0..topics_a.len() {
            if let Some(topic_a) = topics_a.get(i) {
                for j in 0..topics_b.len() {
                    if let Some(topic_b) = topics_b.get(j) {
                        if topic_a.name == topic_b.name {
                            // Weight by relevance scores
                            let avg_relevance = (topic_a.relevance_score + topic_b.relevance_score) / 2;
                            overlap += avg_relevance / 10;  // Scale down
                        }
                    }
                }
            }
        }
        
        overlap.min(100)  // Cap at 100
    }
    
    /// Generate tag index storage key
    fn tag_index_key(env: &Env, tag: &String) -> String {
        String::from_str(env, "tag_idx")
    }
    
    /// Generate skill index storage key
    fn skill_index_key(env: &Env, skill: &String) -> String {
        String::from_str(env, "skill_idx")
    }
}
