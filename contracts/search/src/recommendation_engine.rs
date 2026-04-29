use crate::content_analyzer::ContentAnalyzer;
use crate::types::*;
use soroban_sdk::{Address, Env, Map, String, Vec};

/// Recommendation Engine
/// Generates personalized course recommendations based on user learning history and preferences
pub struct RecommendationEngine;

impl RecommendationEngine {
    /// Generate personalized recommendations for a user
    pub fn generate_recommendations(env: &Env, user: Address, limit: u32) -> Vec<Recommendation> {
        let mut recommendations = Vec::new(env);
        let current_time = env.ledger().timestamp();

        // Check for cached oracle recommendations first. Oracle scores can use
        // heavier ML models, while the on-chain fallback below keeps the
        // feature useful and auditable when no fresh oracle output exists.
        if let Some(cached) = Self::get_cached_recommendations(env, &user) {
            let mut valid_recs = Vec::new(env);

            for i in 0..cached.len() {
                if let Some(rec) = cached.get(i) {
                    if rec.expires_at > current_time && rec.confidence >= 15 {
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

        let Some(profile) = Self::get_user_profile(env, user.clone()) else {
            return Self::get_beginner_recommendations(env, limit);
        };

        let catalog = ContentAnalyzer::get_content_catalog(env);
        for i in 0..catalog.len() {
            if let Some(content_id) = catalog.get(i) {
                if profile.completed_courses.contains(&content_id) {
                    continue;
                }

                if let Some(analysis) = ContentAnalyzer::get_analysis(env, content_id.clone()) {
                    let score = Self::calculate_content_score(&profile, &analysis);
                    let confidence = Self::calculate_confidence(&profile, &analysis);
                    let reason = Self::recommendation_reason(env, &profile, &analysis);

                    let rec = Recommendation {
                        content_id: content_id.clone(),
                        content_type: String::from_str(env, "course"),
                        score,
                        reason,
                        confidence,
                        computed_at: current_time,
                        expires_at: current_time + 86_400,
                    };

                    Self::insert_ranked(env, &mut recommendations, rec, limit);
                }
            }
        }

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
        let mut profile =
            env.storage().persistent().get::<DataKey, UserProfile>(&key).unwrap_or(UserProfile {
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

        Self::apply_learning_signal(env, &mut profile, &course_id, completed);

        // Update timestamp
        profile.last_updated = env.ledger().timestamp();

        // Store updated profile
        env.storage().persistent().set(&key, &profile);

        // Emit event for off-chain processing
        env.events()
            .publish((soroban_sdk::symbol_short!("profile"),), (user, course_id, completed));
    }

    /// Get user learning profile
    pub fn get_user_profile(env: &Env, user: Address) -> Option<UserProfile> {
        let key = DataKey::UserProfile(user);
        env.storage().persistent().get(&key)
    }

    /// Calculate recommendation score for content (on-chain heuristic)
    pub fn calculate_recommendation_score(env: &Env, user: &Address, content_id: &String) -> u32 {
        if let Some(profile) = Self::get_user_profile(env, user.clone()) {
            if let Some(analysis) = ContentAnalyzer::get_analysis(env, content_id.clone()) {
                return Self::calculate_content_score(&profile, &analysis);
            }
        }

        500
    }

    fn calculate_content_score(profile: &UserProfile, analysis: &ContentAnalysis) -> u32 {
        let skill_gap_score = Self::calculate_skill_gap_boost(profile, &analysis.identified_skills);
        let category_score = Self::calculate_category_boost(profile, &analysis.extracted_topics);
        let difficulty_score = Self::calculate_difficulty_fit(profile, analysis.difficulty_score);
        let quality_score = analysis.quality_score.saturating_mul(2);

        ((skill_gap_score * 35)
            + (category_score * 20)
            + (difficulty_score * 25)
            + (quality_score * 20))
            / 100
    }

    fn calculate_confidence(profile: &UserProfile, analysis: &ContentAnalysis) -> u32 {
        let history_depth = (profile.completed_courses.len() * 8).min(35);
        let skill_depth = (profile.skill_levels.len() * 5).min(25);
        let content_depth = if analysis.identified_skills.is_empty() { 15 } else { 30 };

        (30 + history_depth + skill_depth + content_depth).min(100)
    }

    fn recommendation_reason(
        env: &Env,
        profile: &UserProfile,
        analysis: &ContentAnalysis,
    ) -> String {
        let gap_score = Self::calculate_skill_gap_boost(profile, &analysis.identified_skills);
        let difficulty_score = Self::calculate_difficulty_fit(profile, analysis.difficulty_score);

        if gap_score >= 700 {
            String::from_str(env, "fills-skill-gap")
        } else if difficulty_score >= 750 {
            String::from_str(env, "matches-current-level")
        } else if analysis.quality_score >= 85 {
            String::from_str(env, "high-quality-course")
        } else {
            String::from_str(env, "personalized-fit")
        }
    }

    /// Calculate boost based on skill gaps
    fn calculate_skill_gap_boost(profile: &UserProfile, skills: &Vec<Skill>) -> u32 {
        if skills.is_empty() {
            return 450;
        }

        let mut total = 0u32;
        let mut weight = 0u32;

        for i in 0..skills.len() {
            if let Some(skill) = skills.get(i) {
                let current_level =
                    profile.skill_levels.get(skill.skill_name.clone()).unwrap_or(0);
                let gap = skill.required_level.saturating_sub(current_level);
                let progress_room = 100u32.saturating_sub(current_level);
                let importance = skill.importance.max(1);
                total += (gap + progress_room / 2).min(100) * importance;
                weight += importance;
            }
        }

        if weight == 0 {
            450
        } else {
            total
                .checked_mul(10)
                .and_then(|value| value.checked_div(weight))
                .unwrap_or(450)
                .min(1000)
        }
    }

    /// Calculate boost based on category preferences
    fn calculate_category_boost(profile: &UserProfile, topics: &Vec<Topic>) -> u32 {
        if topics.is_empty() {
            return 500;
        }

        let mut best = 0u32;
        for i in 0..topics.len() {
            if let Some(topic) = topics.get(i) {
                let interactions =
                    profile.interaction_counts.get(topic.category.clone()).unwrap_or(0);
                let score =
                    (500 + interactions.saturating_mul(75) + topic.relevance_score / 4).min(1000);
                if score > best {
                    best = score;
                }
            }
        }

        best
    }

    fn calculate_difficulty_fit(profile: &UserProfile, difficulty_score: u32) -> u32 {
        let mut total_level = 0u32;
        let mut skill_count = 0u32;

        let keys = profile.skill_levels.keys();
        for i in 0..keys.len() {
            if let Some(skill) = keys.get(i) {
                total_level += profile.skill_levels.get(skill).unwrap_or(0);
                skill_count += 1;
            }
        }

        let avg_level = if skill_count > 0 {
            total_level.checked_div(skill_count).unwrap_or(50)
        } else if profile.completed_courses.is_empty() {
            25
        } else {
            50
        };

        let target = (avg_level + 15).min(100);
        let diff = target.abs_diff(difficulty_score.min(100));
        1000u32.saturating_sub(diff.saturating_mul(12)).max(100)
    }

    fn apply_learning_signal(
        env: &Env,
        profile: &mut UserProfile,
        course_id: &String,
        completed: bool,
    ) {
        if let Some(analysis) = ContentAnalyzer::get_analysis(env, course_id.clone()) {
            let skill_delta = if completed { 18 } else { 6 };
            for i in 0..analysis.identified_skills.len() {
                if let Some(skill) = analysis.identified_skills.get(i) {
                    let current = profile.skill_levels.get(skill.skill_name.clone()).unwrap_or(0);
                    let gain = (skill_delta * skill.importance.max(1))
                        .checked_div(100)
                        .unwrap_or(1)
                        .max(1);
                    let next = (current + gain).min(100).max(skill.required_level.min(100) / 2);
                    profile.skill_levels.set(skill.skill_name, next);
                }
            }

            for i in 0..analysis.extracted_topics.len() {
                if let Some(topic) = analysis.extracted_topics.get(i) {
                    let current =
                        profile.interaction_counts.get(topic.category.clone()).unwrap_or(0);
                    profile.interaction_counts.set(topic.category, current + 1);
                }
            }
        }
    }

    fn get_beginner_recommendations(env: &Env, limit: u32) -> Vec<Recommendation> {
        let mut recommendations = Vec::new(env);
        let now = env.ledger().timestamp();
        let catalog = ContentAnalyzer::get_content_catalog(env);

        for i in 0..catalog.len() {
            if let Some(content_id) = catalog.get(i) {
                if let Some(analysis) = ContentAnalyzer::get_analysis(env, content_id.clone()) {
                    let difficulty_fit =
                        1000u32.saturating_sub(analysis.difficulty_score.saturating_mul(8));
                    let score = ((difficulty_fit * 50) + (analysis.quality_score * 20 * 50)) / 100;
                    let rec = Recommendation {
                        content_id,
                        content_type: String::from_str(env, "course"),
                        score: score.min(1000),
                        reason: String::from_str(env, "beginner-friendly"),
                        confidence: 45,
                        computed_at: now,
                        expires_at: now + 86_400,
                    };
                    Self::insert_ranked(env, &mut recommendations, rec, limit);
                }
            }
        }

        recommendations
    }

    fn insert_ranked(
        _env: &Env,
        recommendations: &mut Vec<Recommendation>,
        rec: Recommendation,
        limit: u32,
    ) {
        let mut insert_at = recommendations.len();

        for i in 0..recommendations.len() {
            if let Some(existing) = recommendations.get(i) {
                if rec.score > existing.score {
                    insert_at = i;
                    break;
                }
            }
        }

        recommendations.insert(insert_at, rec);
        if recommendations.len() > limit {
            recommendations.remove(limit);
        }
    }

    /// Find users with similar learning patterns (for collaborative filtering)
    pub fn find_similar_users(env: &Env, _user: Address, _limit: u32) -> Vec<Address> {
        // This would be computed off-chain and stored
        // Return empty for now - oracle will provide
        Vec::new(env)
    }

    /// Predict course completion likelihood
    pub fn predict_completion_likelihood(env: &Env, user: Address, course_id: String) -> u32 {
        if let Some(profile) = Self::get_user_profile(env, user) {
            let history_score = if profile.completed_courses.is_empty() { 45 } else { 70 };

            if let Some(analysis) = ContentAnalyzer::get_analysis(env, course_id) {
                let fit = Self::calculate_difficulty_fit(&profile, analysis.difficulty_score) / 10;
                let confidence = Self::calculate_confidence(&profile, &analysis);
                ((history_score + fit + confidence) / 3).min(100)
            } else {
                history_score
            }
        } else {
            50 // Default 50% for new users
        }
    }

    /// Get trending recommendations (popular among similar users)
    pub fn get_trending_recommendations(
        env: &Env,
        _user: Option<Address>,
        _limit: u32,
    ) -> Vec<Recommendation> {
        // Would return trending courses based on community activity
        // This data comes from off-chain analytics
        Vec::new(env)
    }

    /// Get "continue learning" recommendations (incomplete courses)
    pub fn get_continue_learning_recommendations(env: &Env, _user: Address) -> Vec<Recommendation> {
        // Find courses user started but didn't complete
        // Return as recommendations
        Vec::new(env)
    }
}
